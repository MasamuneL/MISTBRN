//! One player's canonical save: state, reserve accounting, the rules that explain it, and the
//! transaction counters that make every credit apply exactly once.
//!
//! Binary format, schema 4 (all integers little-endian):
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
//!
//! schema 2 appends the player's progress (P5) to the schema 1 payload:
//!   | 4 × pair_xp:u32                                             in `Pair` order
//!   | n_nodes:u8 | n × (code:u16 | rank:u8)                       strictly increasing codes
//!   | n_research:u8 | n × (code:u16 | evidence:u8 | hints:u8 | concluded:u8)   increasing
//!   | 3 × (window_start:u64 | window_count:u8)                    practice categories
//!
//! schema 3 appends the claimed milestones (P6) to the schema 2 payload:
//!   | milestones:u8                                               one bit per milestone
//!
//! schema 4 (P8) stores the practice windows of the newer categories (3 and up) between the
//! schema 2 windows and the milestones, so the milestones still end the payload:
//!   | ... 3 × window | n_extra:u8 | n_extra × (window_start:u64 | window_count:u8) | milestones:u8
//! n_extra is at most `PRACTICE_CATEGORIES − LEGACY_WINDOWS`; windows not stored are unopened.
//! ```
//!
//! Schemas 1, 2 and 3 still decode: they are migrated to an empty progress (1), no milestones
//! (2) or no newer windows (3), everything else identical, and the next write stores schema 4.
//! Any other schema is refused before its payload is read.
//!
//! `last_tick` is not saved: server ticks restart with every world open, so a loaded player
//! starts with `None`. Decoding never repairs or defaults anything; any defect is an error and the
//! caller must not overwrite the bytes it could not read.
use crate::{
    BurnState, CATALOG, EventKind, LEGACY_WINDOWS, Metal, NodeEffect, Observed,
    PRACTICE_CATEGORIES, Pair, PlayerId, PlayerState, Progress, ProgressError, ResearchState,
    ReserveAccounting, ReserveError, ReserveRules, VerifiedEvent, WorldId, apply_claim,
    apply_conclusion, apply_evidence, apply_hint, apply_practice, check_purchase,
    clear_reserves_on_death, credit_reserve, milestone_mask, practice_def, research_def,
    validate_progress, validate_reserves,
};
use std::collections::BTreeMap;

pub const SAVE_SCHEMA: u16 = 4;
/// The oldest schema still read (and migrated).
pub const OLDEST_SCHEMA: u16 = 1;
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
    /// Every metal of the pair is already unlocked: nothing to unlock, nothing to consume.
    AlreadyUnlocked,
    /// A basic-metals vial with no unlocked metal below its capacity: nothing to credit.
    NothingToCredit,
    /// Invalid saved progress, or a progress request the rules refuse.
    Progress(ProgressError),
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
    progress: Progress,
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
            progress: Progress::default(),
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
    /// Experience, nodes, research and practice windows (P5).
    pub fn progress(&self) -> &Progress {
        &self.progress
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
        .map_err(SaveError::InvalidReserves)?;
    validate_progress(&saved.progress, &saved.state, &saved.rules, &CATALOG)
        .map_err(SaveError::Progress)
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

/// Prepares `change` as transaction `last_transaction + 1` on a copy of the save. Nothing
/// changes until `commit_credit`; a refused or invalid change prepares nothing.
fn prepare<F>(saved: &SavedPlayer, change_fn: F) -> Result<PreparedCredit, SaveError>
where
    F: FnOnce(&mut SavedPlayer) -> Result<(), SaveError>,
{
    validate_save(saved)?;
    let seq = saved
        .last_transaction
        .checked_add(1)
        .ok_or(SaveError::CounterOverflow)?;
    let mut candidate = saved.clone();
    change(&mut candidate, |next| {
        change_fn(next)?;
        next.last_transaction = seq;
        Ok(())
    })?;
    Ok(PreparedCredit {
        seq: TransactionSeq(seq),
        base_revision: saved.revision,
        candidate,
    })
}

/// Prepares one credit as transaction `last_transaction + 1`. Nothing changes until commit.
pub fn prepare_credit(
    saved: &SavedPlayer,
    metal: Metal,
    amount_milli: i64,
) -> Result<PreparedCredit, SaveError> {
    prepare(saved, |next| {
        credit_reserve(
            &mut next.state,
            &next.accounting,
            &next.rules,
            metal,
            amount_milli,
        )
        .map_err(SaveError::InvalidReserves)
    })
}

fn unlock_pair(state: &mut PlayerState, pair: Pair) {
    unlock_metals(state, pair);
    // The pair just learned becomes the selected one: its keys act on it at once.
    state.selected_pair = pair;
}

fn unlock_metals(state: &mut PlayerState, pair: Pair) {
    for metal in pair.metals() {
        state.metals[metal.index()].unlocked = true;
    }
}

fn pair_unlocked(state: &PlayerState, pair: Pair) -> bool {
    pair.metals()
        .iter()
        .all(|metal| state.metals[metal.index()].unlocked)
}

/// Dosing's extra share of a vial of `amount_milli` (P8), at most `room`. Computed wide, so
/// no amount the core accepts loses any of it (Codex review, round 1).
fn dosing_bonus(amount_milli: i64, progress: &Progress, room: u64) -> i64 {
    let dosing = u128::from(progress.effect(NodeEffect::DosingMilli, &CATALOG));
    let amount = u128::try_from(amount_milli).unwrap_or(0);
    let bonus = (amount * dosing / 1_000).min(u128::from(room));
    // At most half of an i64 amount (dosing ≤ 500): it fits.
    i64::try_from(bonus).unwrap_or(i64::MAX)
}

/// A drink's credit (P5 tutorial): the vial's reserve, and the first vial of a tutorial pair
/// (`CATALOG.tutorial_pairs`) drunk while that pair is locked also unlocks and selects it, in
/// the same transaction. Returns whether it unlocked.
pub fn prepare_vial(
    saved: &SavedPlayer,
    metal: Metal,
    amount_milli: i64,
) -> Result<(PreparedCredit, bool), SaveError> {
    let pair = metal.pair();
    let unlocks = CATALOG.tutorial_pairs.contains(&pair) && !pair_unlocked(&saved.state, pair);
    let prepared = prepare(saved, |next| {
        credit_reserve(
            &mut next.state,
            &next.accounting,
            &next.rules,
            metal,
            amount_milli,
        )
        .map_err(SaveError::InvalidReserves)?;
        // Dosing (P8): a share more, only as far as the capacity has room after the vial.
        let room = next
            .rules
            .capacity(metal)
            .map_err(SaveError::InvalidReserves)?
            .saturating_sub(next.state.metals[metal.index()].reserve_milli);
        let bonus = dosing_bonus(amount_milli, &next.progress, room);
        if bonus > 0 {
            credit_reserve(&mut next.state, &next.accounting, &next.rules, metal, bonus)
                .map_err(SaveError::InvalidReserves)?;
        }
        if unlocks {
            unlock_pair(&mut next.state, pair);
        }
        Ok(())
    })?;
    Ok((prepared, unlocks))
}

/// A basic-metals vial's credit (P3c): `amount_milli` to every unlocked metal, each up to its
/// capacity, in one transaction; it unlocks nothing. Dosing (P8) adds its share to each. Refused with `NothingToCredit` (the vial
/// is kept) only when no metal would receive anything. Returns what each metal receives, in
/// `Metal::ALL` order.
pub fn prepare_basic_vial(
    saved: &SavedPlayer,
    amount_milli: i64,
) -> Result<(PreparedCredit, [u64; 8]), SaveError> {
    validate_save(saved)?;
    let amount = u64::try_from(amount_milli)
        .map_err(|_| SaveError::InvalidReserves(crate::ReserveError::NegativeCredit))?;
    let mut credited = [0u64; 8];
    for metal in Metal::ALL {
        let value = &saved.state.metals[metal.index()];
        if value.unlocked {
            let capacity = saved
                .rules
                .capacity(metal)
                .map_err(SaveError::InvalidReserves)?;
            let room = capacity.saturating_sub(value.reserve_milli);
            let bonus = dosing_bonus(amount_milli, &saved.progress, room).unsigned_abs();
            credited[metal.index()] = amount.saturating_add(bonus).min(room);
        }
    }
    if credited.iter().all(|&c| c == 0) {
        return Err(SaveError::NothingToCredit);
    }
    let prepared = prepare(saved, |next| {
        for metal in Metal::ALL {
            // The vial's part (at most `amount`, an i64) and dosing's (at most `dosing_bonus`,
            // an i64) credited apart: their sum may not fit an i64 (Codex review, round 2).
            let credit = credited[metal.index()];
            let base = credit.min(amount);
            for part in [base, credit - base] {
                if part > 0 {
                    let part = i64::try_from(part).map_err(|_| {
                        SaveError::InvalidReserves(ReserveError::ArithmeticOverflow(metal))
                    })?;
                    credit_reserve(&mut next.state, &next.accounting, &next.rules, metal, part)
                        .map_err(SaveError::InvalidReserves)?;
                }
            }
        }
        Ok(())
    })?;
    Ok((prepared, credited))
}

/// Prepares unlocking both metals of `pair` as transaction `last_transaction + 1`, for an
/// unlock item consumed in the same host transaction (the V1 primer: no recipe since P5, but
/// existing ones still work). The pair becomes the selected one; reserves and burns are
/// untouched. A pair already fully unlocked is `AlreadyUnlocked`, so the caller keeps the
/// item. Committed with `commit_credit`, under the same exactly-once rules.
pub fn prepare_unlock(saved: &SavedPlayer, pair: Pair) -> Result<PreparedCredit, SaveError> {
    validate_save(saved)?;
    if pair_unlocked(&saved.state, pair) {
        return Err(SaveError::AlreadyUnlocked);
    }
    prepare(saved, |next| {
        unlock_pair(&mut next.state, pair);
        Ok(())
    })
}

/// Prepares buying `rank` of node `code`. The request names the rank it wants: once that rank
/// is owned, the same request is `AlreadyOwned`, never a second rank. A node whose effect lives
/// in `ReserveRules` (economy, capacity) changes the rules in the same transaction.
pub fn prepare_purchase(
    saved: &SavedPlayer,
    code: u16,
    rank: u8,
) -> Result<PreparedCredit, SaveError> {
    validate_save(saved)?;
    check_purchase(&saved.progress, &saved.state, code, rank, &CATALOG)
        .map_err(SaveError::Progress)?;
    prepare(saved, |next| {
        next.progress.nodes.insert(code, rank);
        next.rules.reduction_milli = next
            .progress
            .effect(crate::NodeEffect::ReductionMilli, &CATALOG);
        next.rules.capacity_bonus_milli = next
            .progress
            .effect(crate::NodeEffect::CapacityBonusMilli, &CATALOG);
        Ok(())
    })
}

/// Prepares concluding research `code` (at the table): once, with enough evidence; grants its
/// pair experience and, through the discovery, its knowledge points.
pub fn prepare_conclude(saved: &SavedPlayer, code: u16) -> Result<PreparedCredit, SaveError> {
    validate_save(saved)?;
    prepare(saved, |next| {
        apply_conclusion(&mut next.progress, &next.state, code, &CATALOG)
            .map_err(SaveError::Progress)
    })
}

/// Prepares claiming milestone `bit` (P6) as transaction `last_transaction + 1`: once per
/// player; a second claim is `AlreadyClaimed` and prepares nothing. The milestone's pair is
/// unlocked in the same transaction but not selected (the player may be using another), and
/// its knowledge follows from the claim. Returns the pair it unlocks, if any.
pub fn prepare_claim(
    saved: &SavedPlayer,
    bit: u8,
) -> Result<(PreparedCredit, Option<Pair>), SaveError> {
    validate_save(saved)?;
    let mut unlocks = None;
    let prepared = prepare(saved, |next| {
        let def = apply_claim(&mut next.progress, bit, &CATALOG).map_err(SaveError::Progress)?;
        if let Some(pair) = def.unlocks {
            unlock_metals(&mut next.state, pair);
            unlocks = Some(pair);
        }
        Ok(())
    })?;
    Ok((prepared, unlocks))
}

/// Prepares revealing hint `index` (1-based) of research `code`: only the next one, never
/// one already shown. Hints cost nothing and change nothing else.
pub fn prepare_hint(
    saved: &SavedPlayer,
    code: u16,
    index: u8,
) -> Result<PreparedCredit, SaveError> {
    validate_save(saved)?;
    prepare(saved, |next| {
        apply_hint(&mut next.progress, code, index, &CATALOG).map_err(SaveError::Progress)
    })
}

/// Records one server-observed event: practice experience (with its window's falloff at
/// `game_time`) or research evidence. Only an event of this player and world counts, and
/// only through `VerifiedEvent`, which client bytes cannot build. A change that adds nothing
/// leaves the save untouched (no revision).
pub fn observe(
    saved: &mut SavedPlayer,
    event: &VerifiedEvent,
    game_time: u64,
) -> Result<Observed, SaveError> {
    if event.actor() != &saved.state.player_id || event.world_id() != &saved.state.world_id {
        return Err(SaveError::Progress(ProgressError::WrongActor));
    }
    let mut next = saved.progress.clone();
    let observed = match event.kind() {
        EventKind::PracticeAction { category } => {
            let def = practice_def(*category, &CATALOG)
                .ok_or(SaveError::Progress(ProgressError::UnknownCategory))?;
            if !pair_unlocked(&saved.state, def.pair) {
                return Err(SaveError::Progress(ProgressError::PairLocked));
            }
            apply_practice(&mut next, def, game_time, &CATALOG)
        }
        EventKind::ResearchEvidence { research, evidence } => {
            let def = research_def(*research, &CATALOG)
                .ok_or(SaveError::Progress(ProgressError::UnknownResearch))?;
            if !pair_unlocked(&saved.state, def.pair) {
                return Err(SaveError::Progress(ProgressError::PairLocked));
            }
            apply_evidence(&mut next, def, *evidence).map_err(SaveError::Progress)?
        }
        _ => return Ok(Observed::Nothing),
    };
    if observed != Observed::Nothing {
        change(saved, |s| {
            s.progress = next;
            Ok(())
        })?;
    }
    Ok(observed)
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

fn encode_progress(out: &mut Vec<u8>, progress: &Progress) -> Result<(), SaveError> {
    for xp in progress.pair_xp {
        out.extend_from_slice(&xp.to_le_bytes());
    }
    let nodes = u8::try_from(progress.nodes.len()).map_err(|_| SaveError::TooLarge)?;
    out.push(nodes);
    for (code, rank) in &progress.nodes {
        out.extend_from_slice(&code.to_le_bytes());
        out.push(*rank);
    }
    let research = u8::try_from(progress.research.len()).map_err(|_| SaveError::TooLarge)?;
    out.push(research);
    for (code, entry) in &progress.research {
        out.extend_from_slice(&code.to_le_bytes());
        out.push(entry.evidence);
        out.push(entry.hints);
        out.push(u8::from(entry.concluded));
    }
    for (i, window) in progress.windows.iter().enumerate() {
        if i == LEGACY_WINDOWS {
            out.push((PRACTICE_CATEGORIES - LEGACY_WINDOWS) as u8);
        }
        out.extend_from_slice(&window.start.to_le_bytes());
        out.push(window.count);
    }
    out.push(progress.milestones);
    Ok(())
}

/// Reads schema 2's progress. Codes must strictly increase (sorted, no duplicates): one byte
/// string, one progress. Meaning (known codes, ranks, evidence) is checked by `validate_save`.
fn decode_progress(r: &mut Reader<'_>) -> Result<Progress, SaveError> {
    let mut progress = Progress::default();
    for xp in &mut progress.pair_xp {
        *xp = r.u32()?;
    }
    let mut nodes = BTreeMap::new();
    let mut last = None;
    for _ in 0..r.u8()? {
        let code = r.u16()?;
        let rank = r.u8()?;
        if last.is_some_and(|prev| code <= prev) {
            return Err(SaveError::InvalidEnum);
        }
        last = Some(code);
        nodes.insert(code, rank);
    }
    let mut research = BTreeMap::new();
    let mut last = None;
    for _ in 0..r.u8()? {
        let code = r.u16()?;
        let evidence = r.u8()?;
        let hints = r.u8()?;
        let concluded = match r.u8()? {
            0 => false,
            1 => true,
            _ => return Err(SaveError::InvalidEnum),
        };
        if last.is_some_and(|prev| code <= prev) {
            return Err(SaveError::InvalidEnum);
        }
        last = Some(code);
        research.insert(
            code,
            ResearchState {
                evidence,
                hints,
                concluded,
            },
        );
    }
    progress.nodes = nodes;
    progress.research = research;
    for window in &mut progress.windows[..LEGACY_WINDOWS] {
        window.start = r.u64()?;
        window.count = r.u8()?;
    }
    Ok(progress)
}

/// Reads schema 4's newer practice windows (categories 3 and up) into `progress`.
fn decode_extra_windows(r: &mut Reader<'_>, progress: &mut Progress) -> Result<(), SaveError> {
    let extra = usize::from(r.u8()?);
    if extra > PRACTICE_CATEGORIES - LEGACY_WINDOWS {
        return Err(SaveError::InvalidEnum);
    }
    for window in &mut progress.windows[LEGACY_WINDOWS..LEGACY_WINDOWS + extra] {
        window.start = r.u64()?;
        window.count = r.u8()?;
    }
    Ok(())
}

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
    encode_progress(&mut payload, &saved.progress)?;

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
    fn u32(&mut self) -> Result<u32, SaveError> {
        Ok(le_u32(self.take(4)?))
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
    if !(OLDEST_SCHEMA..=SAVE_SCHEMA).contains(&schema) {
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
    // Schema 1 → 4: nothing learned yet; every schema 1 field is kept as it was.
    let mut progress = if schema >= 2 {
        decode_progress(&mut r)?
    } else {
        Progress::default()
    };
    // Schema 3 → 4: no window of the newer practice categories opened yet.
    if schema >= 4 {
        decode_extra_windows(&mut r, &mut progress)?;
    }
    // Schema 2 → 3: no milestone claimed yet.
    if schema >= 3 {
        let milestones = r.u8()?;
        if milestones & !milestone_mask(&CATALOG) != 0 {
            return Err(SaveError::InvalidEnum);
        }
        progress.milestones = milestones;
    }
    if !r.bytes.is_empty() {
        return Err(SaveError::LengthMismatch);
    }
    if schema == 1 {
        // The save must be valid as schema 1 wrote it before anything is normalized.
        validate_reserves(&state, &accounting, &rules).map_err(SaveError::InvalidReserves)?;
        // Since P5 the reduction and the capacity bonus are what the general nodes give, and a
        // schema 1 save owns none. No shipped adapter wrote anything but 0 there; a save that
        // did keeps its fuel: the bonus is dropped only while every reserve still fits the
        // base capacity, otherwise the save stays invalid and the adapter blocks it unwritten.
        rules.reduction_milli = 0;
        let fits = Metal::ALL.iter().all(|metal| {
            state.metals[metal.index()].reserve_milli <= rules.base_capacity_milli[metal.index()]
        });
        if fits {
            rules.capacity_bonus_milli = 0;
        }
    }
    let saved = SavedPlayer {
        state,
        accounting,
        rules,
        progress,
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
