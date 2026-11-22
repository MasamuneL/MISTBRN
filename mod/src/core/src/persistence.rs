//! One player's canonical save: state, reserve accounting, the rules that explain it, and the
//! transaction counters that make every credit apply exactly once.
//!
//! Binary format, schema 1 (all integers little-endian):
//!
//! ```text
//! "MBRN" | schema:u16 | payload_len:u32 | payload | crc32:u32 (IEEE, over everything before it)
//! payload = player_id:str | world_id:str | selected_pair:u8
//!         | 8 × (unlocked:u8 | reserve_milli:u64 | burn:u8)      in Metal::ALL order
//!         | 8 × remainder:u16
//!         | 8 × consumption_milli_per_tick:u64 | 8 × base_capacity_milli:u64
//!         | reduction_milli:u16 | capacity_bonus_milli:u16
//!         | revision:u64 | last_transaction:u64
//! str     = len:u16 | UTF-8 bytes
//! ```
//!
//! `last_tick` is not saved: server ticks restart with every world open, so a loaded player
//! starts with `None`. Decoding never repairs or defaults anything; any defect is an error and the
//! caller must not overwrite the bytes it could not read.
use crate::{
    BurnState, Metal, Pair, PlayerId, PlayerState, ReserveAccounting, ReserveError, ReserveRules,
    WorldId, clear_reserves_on_death, credit_reserve, validate_reserves,
};

pub const SAVE_SCHEMA: u16 = 1;
pub const MAX_SAVE_BYTES: usize = 64 * 1024;
pub const MAX_ID_BYTES: usize = 256;
const MAGIC: [u8; 4] = *b"MBRN";
const HEADER_BYTES: usize = 4 + 2 + 4;
const CRC_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveError {
    TooLarge,
    Truncated,
    BadMagic,
    UnsupportedVersion(u16),
    LengthMismatch,
    ChecksumMismatch,
    TrailingBytes,
    InvalidEnum,
    InvalidIdentity,
    WorldMismatch,
    PlayerMismatch,
    InvalidCounters,
    InvalidReserves(ReserveError),
    Duplicate,
    StalePreparation,
    CounterOverflow,
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SaveError {}

/// The sequence number of one committed transaction for one player: 1, 2, 3, ...
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionSeq(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedPlayer {
    state: PlayerState,
    accounting: ReserveAccounting,
    rules: ReserveRules,
    revision: u64,
    last_transaction: u64,
}

impl SavedPlayer {
    /// A fresh save, for a player whose save is genuinely absent (never after a decode error).
    pub fn new(player: PlayerId, world: WorldId, rules: ReserveRules) -> Result<Self, SaveError> {
        let saved = Self {
            state: PlayerState::new(player, world),
            accounting: ReserveAccounting::default(),
            rules,
            revision: 0,
            last_transaction: 0,
        };
        validate_save(&saved)?;
        Ok(saved)
    }
    pub fn state(&self) -> &PlayerState {
        &self.state
    }
    pub fn accounting(&self) -> &ReserveAccounting {
        &self.accounting
    }
    pub fn rules(&self) -> &ReserveRules {
        &self.rules
    }
    /// Increases by one with every accepted change; never reset.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// The sequence of the last committed transaction; 0 before the first.
    pub fn last_transaction(&self) -> u64 {
        self.last_transaction
    }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_BYTES && !id.chars().any(char::is_control)
}

pub fn validate_save(saved: &SavedPlayer) -> Result<(), SaveError> {
    if !valid_id(saved.state.player_id.as_str()) || !valid_id(saved.state.world_id.as_str()) {
        return Err(SaveError::InvalidIdentity);
    }
    if saved.last_transaction > saved.revision {
        return Err(SaveError::InvalidCounters);
    }
    validate_reserves(&saved.state, &saved.accounting, &saved.rules)
        .map_err(SaveError::InvalidReserves)
}

/// Applies `change` to copies, validates the result, then replaces the save and bumps the
/// revision. A failing change or validation leaves the save untouched.
fn change<F>(saved: &mut SavedPlayer, change: F) -> Result<(), SaveError>
where
    F: FnOnce(&mut SavedPlayer) -> Result<(), SaveError>,
{
    let mut next = saved.clone();
    change(&mut next)?;
    next.revision = saved
        .revision
        .checked_add(1)
        .ok_or(SaveError::CounterOverflow)?;
    validate_save(&next)?;
    *saved = next;
    Ok(())
}

/// Trusted in-place change of state and accounting (ticks, burns, unlocks), validated as a whole.
pub fn update_player<F>(saved: &mut SavedPlayer, update: F) -> Result<(), SaveError>
where
    F: FnOnce(&mut PlayerState, &mut ReserveAccounting) -> Result<(), ReserveError>,
{
    change(saved, |next| {
        update(&mut next.state, &mut next.accounting).map_err(SaveError::InvalidReserves)
    })
}

/// Replaces the rules only if the existing reserves stay valid under them.
pub fn replace_rules(saved: &mut SavedPlayer, rules: ReserveRules) -> Result<(), SaveError> {
    change(saved, |next| {
        next.rules = rules;
        Ok(())
    })
}

/// Death: reserves, remainders and burns cleared; unlocks and counters kept.
pub fn on_death(saved: &mut SavedPlayer) -> Result<(), SaveError> {
    change(saved, |next| {
        clear_reserves_on_death(&mut next.state, &mut next.accounting);
        Ok(())
    })
}

/// Reconnect: every burn stops; reserves and remainders stay.
pub fn on_reconnect(saved: &mut SavedPlayer) -> Result<(), SaveError> {
    change(saved, |next| {
        for metal in &mut next.state.metals {
            metal.burn = BurnState::Off;
        }
        Ok(())
    })
}

/// A credit computed against one exact revision; committing it is the only way to apply it.
#[derive(Debug, PartialEq, Eq)]
pub struct PreparedCredit {
    seq: TransactionSeq,
    base_revision: u64,
    candidate: SavedPlayer,
}

impl PreparedCredit {
    pub fn seq(&self) -> TransactionSeq {
        self.seq
    }
    /// The save as it will be once committed: encode this, write it with the inventory change in
    /// one host transaction, and commit only after that transaction commits.
    pub fn candidate(&self) -> &SavedPlayer {
        &self.candidate
    }
}

/// Prepares one credit as transaction `last_transaction + 1`. Nothing changes until commit.
pub fn prepare_credit(
    saved: &SavedPlayer,
    metal: Metal,
    amount_milli: i64,
) -> Result<PreparedCredit, SaveError> {
    validate_save(saved)?;
    let seq = saved
        .last_transaction
        .checked_add(1)
        .ok_or(SaveError::CounterOverflow)?;
    let mut candidate = saved.clone();
    change(&mut candidate, |next| {
        credit_reserve(
            &mut next.state,
            &next.accounting,
            &next.rules,
            metal,
            amount_milli,
        )
        .map_err(SaveError::InvalidReserves)?;
        next.last_transaction = seq;
        Ok(())
    })?;
    Ok(PreparedCredit {
        seq: TransactionSeq(seq),
        base_revision: saved.revision,
        candidate,
    })
}

/// Commits a prepared credit exactly once. A preparation made against another revision is
/// stale; one whose sequence is already committed is a duplicate. Either leaves the save as is.
pub fn commit_credit(
    saved: &mut SavedPlayer,
    prepared: PreparedCredit,
) -> Result<TransactionSeq, SaveError> {
    if prepared.seq.0 <= saved.last_transaction {
        return Err(SaveError::Duplicate);
    }
    if prepared.base_revision != saved.revision
        || prepared.candidate.state.player_id != saved.state.player_id
        || prepared.candidate.state.world_id != saved.state.world_id
    {
        return Err(SaveError::StalePreparation);
    }
    *saved = prepared.candidate;
    Ok(prepared.seq)
}

// ---- codec ----

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

pub(crate) fn burn_code(burn: BurnState) -> u8 {
    match burn {
        BurnState::Off => 0,
        BurnState::Burning => 1,
        BurnState::Flared => 2,
    }
}

pub(crate) fn pair_code(pair: Pair) -> u8 {
    match pair {
        Pair::IronSteel => 0,
        Pair::TinPewter => 1,
        Pair::ZincBrass => 2,
        Pair::CopperBronze => 3,
    }
}

fn put_str(out: &mut Vec<u8>, text: &str) -> Result<(), SaveError> {
    let len = u16::try_from(text.len()).map_err(|_| SaveError::InvalidIdentity)?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    Ok(())
}

pub fn encode_save(saved: &SavedPlayer) -> Result<Vec<u8>, SaveError> {
    validate_save(saved)?;
    let mut payload = Vec::with_capacity(256);
    put_str(&mut payload, saved.state.player_id.as_str())?;
    put_str(&mut payload, saved.state.world_id.as_str())?;
    payload.push(pair_code(saved.state.selected_pair));
    for metal in &saved.state.metals {
        payload.push(u8::from(metal.unlocked));
        payload.extend_from_slice(&metal.reserve_milli.to_le_bytes());
        payload.push(burn_code(metal.burn));
    }
    for remainder in saved.accounting.remainder {
        payload.extend_from_slice(&remainder.to_le_bytes());
    }
    for value in saved.rules.consumption_milli_per_tick {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    for value in saved.rules.base_capacity_milli {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&saved.rules.reduction_milli.to_le_bytes());
    payload.extend_from_slice(&saved.rules.capacity_bonus_milli.to_le_bytes());
    payload.extend_from_slice(&saved.revision.to_le_bytes());
    payload.extend_from_slice(&saved.last_transaction.to_le_bytes());

    let total = HEADER_BYTES + payload.len() + CRC_BYTES;
    if total > MAX_SAVE_BYTES {
        return Err(SaveError::TooLarge);
    }
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&SAVE_SCHEMA.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);
    let crc = crc32(&out);
    out.extend_from_slice(&crc.to_le_bytes());
    Ok(out)
}

/// Reads the schema of an encoded save without validating the rest (for diagnostics).
pub fn save_schema(bytes: &[u8]) -> Option<u16> {
    (bytes.len() >= 6 && bytes[..4] == MAGIC).then(|| u16::from_le_bytes([bytes[4], bytes[5]]))
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], SaveError> {
        if self.bytes.len() < n {
            return Err(SaveError::Truncated);
        }
        let (head, tail) = self.bytes.split_at(n);
        self.bytes = tail;
        Ok(head)
    }
    fn u8(&mut self) -> Result<u8, SaveError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, SaveError> {
        let raw = self.take(2)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }
    fn u64(&mut self) -> Result<u64, SaveError> {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(self.take(8)?);
        Ok(u64::from_le_bytes(raw))
    }
    fn str(&mut self) -> Result<String, SaveError> {
        let len = usize::from(self.u16()?);
        let raw = self.take(len)?;
        String::from_utf8(raw.to_vec()).map_err(|_| SaveError::InvalidIdentity)
    }
}

fn le_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Decodes and fully validates a save for exactly `expected_world`/`expected_player`.
pub fn decode_save(
    bytes: &[u8],
    expected_world: &WorldId,
    expected_player: &PlayerId,
) -> Result<SavedPlayer, SaveError> {
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(SaveError::TooLarge);
    }
    if bytes.len() < MAGIC.len() {
        return Err(SaveError::Truncated);
    }
    if bytes[..4] != MAGIC {
        return Err(SaveError::BadMagic);
    }
    if bytes.len() < HEADER_BYTES {
        return Err(SaveError::Truncated);
    }
    let schema = u16::from_le_bytes([bytes[4], bytes[5]]);
    if schema != SAVE_SCHEMA {
        return Err(SaveError::UnsupportedVersion(schema));
    }
    let payload_len = le_u32(&bytes[6..10]) as usize;
    let expected_total = HEADER_BYTES
        .checked_add(payload_len)
        .and_then(|n| n.checked_add(CRC_BYTES))
        .ok_or(SaveError::LengthMismatch)?;
    if bytes.len() < expected_total {
        return Err(SaveError::Truncated);
    }
    if bytes.len() > expected_total {
        return Err(SaveError::TrailingBytes);
    }
    let body = &bytes[..expected_total - CRC_BYTES];
    if crc32(body) != le_u32(&bytes[expected_total - CRC_BYTES..]) {
        return Err(SaveError::ChecksumMismatch);
    }

    let mut r = Reader {
        bytes: &body[HEADER_BYTES..],
    };
    let player = PlayerId::new(r.str()?);
    let world = WorldId::new(r.str()?);
    let selected_pair = match r.u8()? {
        0 => Pair::IronSteel,
        1 => Pair::TinPewter,
        2 => Pair::ZincBrass,
        3 => Pair::CopperBronze,
        _ => return Err(SaveError::InvalidEnum),
    };
    let mut state = PlayerState::new(player, world);
    state.selected_pair = selected_pair;
    for metal in &mut state.metals {
        metal.unlocked = match r.u8()? {
            0 => false,
            1 => true,
            _ => return Err(SaveError::InvalidEnum),
        };
        metal.reserve_milli = r.u64()?;
        metal.burn = match r.u8()? {
            0 => BurnState::Off,
            1 => BurnState::Burning,
            2 => BurnState::Flared,
            _ => return Err(SaveError::InvalidEnum),
        };
    }
    let mut accounting = ReserveAccounting::default();
    for remainder in &mut accounting.remainder {
        *remainder = r.u16()?;
    }
    let mut rules = ReserveRules {
        consumption_milli_per_tick: [0; 8],
        base_capacity_milli: [0; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    };
    for value in &mut rules.consumption_milli_per_tick {
        *value = r.u64()?;
    }
    for value in &mut rules.base_capacity_milli {
        *value = r.u64()?;
    }
    rules.reduction_milli = r.u16()?;
    rules.capacity_bonus_milli = r.u16()?;
    let revision = r.u64()?;
    let last_transaction = r.u64()?;
    if !r.bytes.is_empty() {
        return Err(SaveError::LengthMismatch);
    }
    let saved = SavedPlayer {
        state,
        accounting,
        rules,
        revision,
        last_transaction,
    };
    validate_save(&saved)?;
    if &saved.state.world_id != expected_world {
        return Err(SaveError::WorldMismatch);
    }
    if &saved.state.player_id != expected_player {
        return Err(SaveError::PlayerMismatch);
    }
    Ok(saved)
}
