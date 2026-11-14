//! Validated snapshots and trusted credit preparation. Host application and durability
//! remain adapter responsibilities; no operation here authenticates client input.
use crate::{
    BurnState, Metal, PlayerId, PlayerState, ReserveAccounting, ReserveError, ReserveRules,
    TransactionId, WorldId, clear_reserves_on_death, credit_reserve, validate_reserves,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

pub const MAX_SAVE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_LEDGER_ENTRIES: usize = 4096;
const SCHEMA_VERSION: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    InputTooLarge,
    InvalidJson,
    UnsupportedVersion(u64),
    InvalidSchema,
    InvalidIdentity,
    WorldMismatch,
    PlayerMismatch,
    InvalidReserves(ReserveError),
    InvalidLedger,
    NegativeCredit,
    TransactionConflict,
    LedgerFull,
    RevisionOverflow,
    StalePreparation,
    Io,
}
impl From<ReserveError> for SaveError {
    fn from(error: ReserveError) -> Self {
        Self::InvalidReserves(error)
    }
}
impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SaveError {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditPayload {
    pub metal: Metal,
    pub amount_milli: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreditReceipt {
    pub payload: CreditPayload,
    pub committed_revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerEntry {
    pub transaction: TransactionId,
    pub payload: CreditPayload,
    pub committed_revision: u64,
}
impl LedgerEntry {
    fn receipt(&self) -> CreditReceipt {
        CreditReceipt {
            payload: self.payload.clone(),
            committed_revision: self.committed_revision,
        }
    }
}

/// Deserialize alone is not validation; load through `decode_save`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedPlayer {
    state: PlayerState,
    accounting: ReserveAccounting,
    rules: ReserveRules,
    revision: u64,
    ledger: Vec<LedgerEntry>,
}
impl SavedPlayer {
    /// For a genuinely absent save, never for suppressing a decode error.
    pub fn new(player: PlayerId, world: WorldId, rules: ReserveRules) -> Result<Self, SaveError> {
        let saved = Self {
            state: PlayerState::new(player, world),
            accounting: ReserveAccounting::default(),
            rules,
            revision: 0,
            ledger: Vec::new(),
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
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn ledger(&self) -> &[LedgerEntry] {
        &self.ledger
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedCredit {
    before: SavedPlayer,
    candidate: SavedPlayer,
    receipt: CreditReceipt,
}
impl PreparedCredit {
    /// Includes the new balance, fractional accounting, revision and ledger.
    pub fn candidate(&self) -> &SavedPlayer {
        &self.candidate
    }
    pub fn receipt(&self) -> &CreditReceipt {
        &self.receipt
    }
    pub fn is_current(&self, saved: &SavedPlayer) -> bool {
        &self.before == saved
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreditPreparation {
    Replay(CreditReceipt),
    Pending(PreparedCredit),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    schema_version: u64,
    saved_player: T,
}
// Intentionally accepts other fields: do not interpret a future payload as v1.
#[derive(Deserialize)]
struct VersionProbe {
    schema_version: u64,
}

fn json_error(error: serde_json::Error) -> SaveError {
    if error.is_syntax() || error.is_eof() {
        SaveError::InvalidJson
    } else {
        SaveError::InvalidSchema
    }
}
fn check_version(bytes: &[u8], expected: u64) -> Result<(), SaveError> {
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(SaveError::InputTooLarge);
    }
    let probe: VersionProbe = serde_json::from_slice(bytes).map_err(json_error)?;
    if probe.schema_version != expected {
        return Err(SaveError::UnsupportedVersion(probe.schema_version));
    }
    Ok(())
}
fn validate_id(value: &str) -> Result<(), SaveError> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        Err(SaveError::InvalidIdentity)
    } else {
        Ok(())
    }
}
fn validate_expected(
    saved: &SavedPlayer,
    world: &WorldId,
    player: &PlayerId,
) -> Result<(), SaveError> {
    validate_id(world.as_str())?;
    validate_id(player.as_str())?;
    if &saved.state.world_id != world {
        return Err(SaveError::WorldMismatch);
    }
    if &saved.state.player_id != player {
        return Err(SaveError::PlayerMismatch);
    }
    Ok(())
}

fn require_object(value: &serde_json::Value) -> Result<(), SaveError> {
    if value.is_object() {
        Ok(())
    } else {
        Err(SaveError::InvalidSchema)
    }
}

// Serde structs also accept positional arrays. Enforce the JSON object contract
// without DTO mirrors; the subsequent typed decode still rejects duplicate keys.
fn validate_object_shapes(bytes: &[u8], fixture_v0: bool) -> Result<(), SaveError> {
    let document: serde_json::Value = serde_json::from_slice(bytes).map_err(json_error)?;
    require_object(&document)?;
    let saved = if fixture_v0 {
        &document
    } else {
        &document["saved_player"]
    };
    require_object(saved)?;
    require_object(&saved["state"])?;
    require_object(&saved["rules"])?;
    for metal in saved["state"]["metals"]
        .as_array()
        .ok_or(SaveError::InvalidSchema)?
    {
        require_object(metal)?;
    }
    if !fixture_v0 {
        require_object(&saved["accounting"])?;
        for entry in saved["ledger"].as_array().ok_or(SaveError::InvalidSchema)? {
            require_object(entry)?;
            require_object(&entry["payload"])?;
        }
    }
    Ok(())
}

pub fn validate_save(saved: &SavedPlayer) -> Result<(), SaveError> {
    validate_id(saved.state.world_id.as_str())?;
    validate_id(saved.state.player_id.as_str())?;
    validate_reserves(&saved.state, &saved.accounting, &saved.rules)?;
    if saved.ledger.len() > MAX_LEDGER_ENTRIES {
        return Err(SaveError::InvalidLedger);
    }
    let mut revisions = BTreeSet::new();
    let mut previous = None;
    for entry in &saved.ledger {
        validate_id(entry.transaction.as_str())?;
        if previous.is_some_and(|id| id >= &entry.transaction)
            || entry.payload.amount_milli < 0
            || entry.committed_revision == 0
            || entry.committed_revision > saved.revision
            || !revisions.insert(entry.committed_revision)
        {
            return Err(SaveError::InvalidLedger);
        }
        previous = Some(&entry.transaction);
    }
    Ok(())
}
pub fn encode_save(saved: &SavedPlayer) -> Result<Vec<u8>, SaveError> {
    validate_save(saved)?;
    let bytes = serde_json::to_vec(&Envelope {
        schema_version: SCHEMA_VERSION,
        saved_player: saved,
    })
    .map_err(json_error)?;
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(SaveError::InputTooLarge);
    }
    Ok(bytes)
}
pub fn decode_save(
    bytes: &[u8],
    expected_world: &WorldId,
    expected_player: &PlayerId,
) -> Result<SavedPlayer, SaveError> {
    check_version(bytes, SCHEMA_VERSION)?;
    validate_object_shapes(bytes, false)?;
    let envelope: Envelope<SavedPlayer> = serde_json::from_slice(bytes).map_err(json_error)?;
    validate_save(&envelope.saved_player)?;
    validate_expected(&envelope.saved_player, expected_world, expected_player)?;
    Ok(envelope.saved_player)
}

/// Trusted mutation on a clone; every success advances revision, including no-ops.
pub fn update_player<F>(saved: &mut SavedPlayer, update: F) -> Result<(), SaveError>
where
    F: FnOnce(&mut PlayerState, &mut ReserveAccounting) -> Result<(), ReserveError>,
{
    let mut candidate = saved.clone();
    candidate.revision = saved
        .revision
        .checked_add(1)
        .ok_or(SaveError::RevisionOverflow)?;
    update(&mut candidate.state, &mut candidate.accounting)?;
    validate_expected(&candidate, &saved.state.world_id, &saved.state.player_id)?;
    validate_save(&candidate)?;
    *saved = candidate;
    Ok(())
}
pub fn replace_rules(saved: &mut SavedPlayer, rules: ReserveRules) -> Result<(), SaveError> {
    let mut candidate = saved.clone();
    candidate.revision = saved
        .revision
        .checked_add(1)
        .ok_or(SaveError::RevisionOverflow)?;
    candidate.rules = rules;
    validate_save(&candidate)?;
    *saved = candidate;
    Ok(())
}
pub fn on_death(saved: &mut SavedPlayer) -> Result<(), SaveError> {
    update_player(saved, |state, accounting| {
        clear_reserves_on_death(state, accounting);
        state.last_tick = None;
        Ok(())
    })
}
pub fn on_reconnect(saved: &mut SavedPlayer) -> Result<(), SaveError> {
    update_player(saved, |state, _| {
        for metal in &mut state.metals {
            metal.burn = BurnState::Off;
        }
        state.last_tick = None;
        Ok(())
    })
}

/// Does not consume inventory; all inputs must be resolved by trusted server logic.
pub fn prepare_credit(
    saved: &SavedPlayer,
    transaction: TransactionId,
    metal: Metal,
    amount_milli: i64,
) -> Result<CreditPreparation, SaveError> {
    validate_save(saved)?;
    validate_id(transaction.as_str())?;
    if amount_milli < 0 {
        return Err(SaveError::NegativeCredit);
    }
    let payload = CreditPayload {
        metal,
        amount_milli,
    };
    let index = match saved
        .ledger
        .binary_search_by(|entry| entry.transaction.cmp(&transaction))
    {
        Ok(index) => {
            let entry = &saved.ledger[index];
            if entry.payload != payload {
                return Err(SaveError::TransactionConflict);
            }
            return Ok(CreditPreparation::Replay(entry.receipt()));
        }
        Err(index) => index,
    };
    if saved.ledger.len() == MAX_LEDGER_ENTRIES {
        return Err(SaveError::LedgerFull);
    }
    let mut candidate = saved.clone();
    candidate.revision = saved
        .revision
        .checked_add(1)
        .ok_or(SaveError::RevisionOverflow)?;
    credit_reserve(
        &mut candidate.state,
        &candidate.accounting,
        &candidate.rules,
        metal,
        amount_milli,
    )?;
    let receipt = CreditReceipt {
        payload: payload.clone(),
        committed_revision: candidate.revision,
    };
    // ponytail: retain every ID; at 4096 reject new credits instead of evicting replay protection.
    candidate.ledger.insert(
        index,
        LedgerEntry {
            transaction,
            payload,
            committed_revision: candidate.revision,
        },
    );
    validate_save(&candidate)?;
    Ok(CreditPreparation::Pending(PreparedCredit {
        before: saved.clone(),
        candidate,
        receipt,
    }))
}
/// Only after the Host accepts inventory and the entire candidate; not a durable receipt.
pub fn commit_credit(
    saved: &mut SavedPlayer,
    prepared: PreparedCredit,
) -> Result<CreditReceipt, SaveError> {
    if !prepared.is_current(saved) {
        return Err(SaveError::StalePreparation);
    }
    validate_save(&prepared.candidate)?;
    *saved = prepared.candidate;
    Ok(prepared.receipt)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureV0 {
    schema_version: u64,
    state: PlayerState,
    rules: ReserveRules,
}
/// Artificial, never-deployed fixture migration for offline tests/exports only.
pub fn migrate_fixture_v0(
    bytes: &[u8],
    world: &WorldId,
    player: &PlayerId,
) -> Result<SavedPlayer, SaveError> {
    check_version(bytes, 0)?;
    validate_object_shapes(bytes, true)?;
    let fixture: FixtureV0 = serde_json::from_slice(bytes).map_err(json_error)?;
    if fixture.schema_version != 0
        || fixture.state.last_tick.is_some()
        || fixture
            .state
            .metals
            .iter()
            .any(|metal| metal.reserve_milli != 0 || metal.burn != BurnState::Off)
    {
        return Err(SaveError::InvalidSchema);
    }
    let saved = SavedPlayer {
        state: fixture.state,
        accounting: ReserveAccounting::default(),
        rules: fixture.rules,
        revision: 0,
        ledger: Vec::new(),
    };
    validate_save(&saved)?;
    validate_expected(&saved, world, player)?;
    Ok(saved)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportSource {
    Primary,
    Backup,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveredExport {
    pub saved: SavedPlayer,
    pub source: ExportSource,
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, SaveError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(SaveError::Io),
    };
    let mut bytes = Vec::new();
    file.take(MAX_SAVE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| SaveError::Io)?;
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(SaveError::InputTooLarge);
    }
    Ok(Some(bytes))
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<(), SaveError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| SaveError::Io)?;
    let result = file.write_all(bytes).and_then(|()| file.flush());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(path);
        return Err(SaveError::Io);
    }
    Ok(())
}
/// Exclusive offline export only; same-directory rename without fsync.
/// Never use against Horizon live files or as an economic sidecar.
pub fn write_export_atomic(path: &Path, saved: &SavedPlayer) -> Result<(), SaveError> {
    let bytes = encode_save(saved)?;
    let backup = sibling(path, ".bak");
    let old_bytes = read_optional(path)?;
    let backup_bytes = read_optional(&backup)?;
    for previous in old_bytes.iter().chain(backup_bytes.iter()) {
        let old = decode_save(previous, &saved.state.world_id, &saved.state.player_id)?;
        if old.revision > saved.revision || (old.revision == saved.revision && old != *saved) {
            return Err(SaveError::StalePreparation);
        }
    }
    let temporary = sibling(path, ".tmp");
    let backup_temporary = sibling(path, ".bak.tmp");
    write_new(&temporary, &bytes)?;
    let result = (|| {
        if let Some(old) = old_bytes {
            write_new(&backup_temporary, &old)?;
            if fs::rename(&backup_temporary, &backup).is_err() {
                let _ = fs::remove_file(&backup_temporary);
                return Err(SaveError::Io);
            }
        }
        fs::rename(&temporary, path).map_err(|_| SaveError::Io)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
/// Explicit recovery result; no rewrite. Future schemas never fall back to old data.
pub fn read_export_with_backup(
    path: &Path,
    world: &WorldId,
    player: &PlayerId,
) -> Result<RecoveredExport, SaveError> {
    match read_optional(path) {
        Ok(Some(bytes)) => match decode_save(&bytes, world, player) {
            Ok(saved) => {
                return Ok(RecoveredExport {
                    saved,
                    source: ExportSource::Primary,
                });
            }
            Err(
                error @ (SaveError::UnsupportedVersion(_)
                | SaveError::WorldMismatch
                | SaveError::PlayerMismatch),
            ) => return Err(error),
            Err(_) => {}
        },
        Ok(None) | Err(SaveError::InputTooLarge) => {}
        Err(error) => return Err(error),
    }
    let bytes = read_optional(&sibling(path, ".bak"))?.ok_or(SaveError::Io)?;
    Ok(RecoveredExport {
        saved: decode_save(&bytes, world, player)?,
        source: ExportSource::Backup,
    })
}
