//! Canonical player storage through the real Host inline NeoForgeData path.
//! The bundle N0 guard MUST run before world adoption: Host cannot distinguish
//! a lost player file from a genuinely new player's absent namespace.
use allomancy_core::{
    BurnState, CreditPreparation, CreditReceipt, MAX_SAVE_BYTES, Metal, PlayerId, ReserveRules,
    SaveError, SavedPlayer, Tick, TransactionId, WorldId, commit_credit, consume_reserves,
    decode_save, encode_save, load_world_identity, on_death, on_reconnect, prepare_credit,
    update_player,
};
use horizon_client::mod_api::{Compound, Host, ItemStack, NativeInventoryKind, Tag};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Mutex,
};

const NAMESPACE: &str = "allomancy:state";
const STORAGE_FAULT: &str = "allomancy:storage_fault";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageError {
    Disabled,
    Identity,
    Unavailable,
    Root,
    PlayerIdentity,
    Health,
    ModsRoot,
    Quarantined,
    Namespace,
    FutureTransport(i32),
    Payload,
    Codec(SaveError),
    Busy,
    Fault,
    Conflict,
    StaleConnection,
    Slot,
    HostRejected,
    RollbackFailed,
    TestOnly,
}
impl From<SaveError> for StorageError {
    fn from(error: SaveError) -> Self {
        Self::Codec(error)
    }
}

pub fn player_id(uuid: u128) -> PlayerId {
    PlayerId::new(format!("{uuid:032x}"))
}

/// Explicit prototype rules, never a grant or a production balance decision.
pub fn initial_rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [1; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    }
}

pub fn encode_namespace(saved: &SavedPlayer) -> Result<Tag, StorageError> {
    Ok(Tag::compound([
        ("format_version", Tag::Int(1)),
        ("payload", Tag::ByteArray(encode_save(saved)?)),
    ]))
}

pub fn decode_namespace(
    tag: &Tag,
    world: &WorldId,
    player: &PlayerId,
) -> Result<SavedPlayer, StorageError> {
    let Tag::Compound(fields) = tag else {
        return Err(StorageError::Namespace);
    };
    // Probe transport version before imposing the current layout.
    match tag.get("format_version") {
        Some(Tag::Int(1)) => {}
        Some(Tag::Int(version)) if *version > 1 => {
            return Err(StorageError::FutureTransport(*version));
        }
        _ => return Err(StorageError::Namespace),
    }
    if fields.len() != 2 {
        return Err(StorageError::Namespace);
    }
    let Some(Tag::ByteArray(bytes)) = tag.get("payload") else {
        return Err(StorageError::Payload);
    };
    if bytes.len() > MAX_SAVE_BYTES {
        return Err(StorageError::Codec(SaveError::InputTooLarge));
    }
    Ok(decode_save(bytes, world, player)?)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPlayer {
    /// None retains true absence, distinct from a present empty Compound.
    pub mods: Option<Tag>,
    pub namespace: Option<Tag>,
    pub dead: bool,
}

/// Validate RAW Host snapshot before using a normalized persistent-data getter.
pub fn inspect_player(raw: &Tag, uuid: u128) -> Result<RawPlayer, StorageError> {
    let player = inspect_player_structure(raw, uuid)?;
    reject_quarantine(player.mods.as_ref())?;
    Ok(player)
}

// Structural reads retain quarantined Tags for exact marker confirmation.
fn inspect_player_structure(raw: &Tag, uuid: u128) -> Result<RawPlayer, StorageError> {
    if !matches!(raw, Tag::Compound(_)) {
        return Err(StorageError::Root);
    }
    let Some(Tag::IntArray(words)) = raw.get("UUID") else {
        return Err(StorageError::PlayerIdentity);
    };
    if words.len() != 4
        || words
            .iter()
            .fold(0u128, |id, word| (id << 32) | u128::from(*word as u32))
            != uuid
    {
        return Err(StorageError::PlayerIdentity);
    }
    let Some(Tag::Float(bits)) = raw.get("Health") else {
        return Err(StorageError::Health);
    };
    let health = f32::from_bits(*bits);
    if !health.is_finite() {
        return Err(StorageError::Health);
    }
    let mods = raw.get("NeoForgeData");
    if mods.is_some_and(|tag| !matches!(tag, Tag::Compound(_))) {
        return Err(StorageError::ModsRoot);
    }
    Ok(RawPlayer {
        mods: mods.cloned(),
        namespace: mods.and_then(|tag| tag.get(NAMESPACE)).cloned(),
        dead: health <= 0.0,
    })
}

fn reject_quarantine(mods: Option<&Tag>) -> Result<(), StorageError> {
    if mods.is_some_and(|tag| tag.get(STORAGE_FAULT).is_some()) {
        return Err(StorageError::Quarantined);
    }
    Ok(())
}

fn merge(mods: Option<&Tag>, namespace: Tag) -> Result<Tag, StorageError> {
    let mut map = match mods {
        None => Compound::new(),
        Some(Tag::Compound(map)) => map.clone(),
        _ => return Err(StorageError::ModsRoot),
    };
    map.insert(NAMESPACE.into(), namespace);
    Ok(Tag::Compound(map))
}

fn raw_player(host: &dyn Host, uuid: u128) -> Result<RawPlayer, StorageError> {
    inspect_player(
        &host
            .player_compound(uuid)
            .ok_or(StorageError::Unavailable)?,
        uuid,
    )
}

fn live_slot(host: &dyn Host, uuid: u128, slot: usize) -> Result<Option<ItemStack>, StorageError> {
    if slot >= 36 {
        return Err(StorageError::Slot);
    }
    if !host
        .server_players()
        .iter()
        .any(|p| p.uuid == uuid && p.health.is_finite() && p.health > 0.0)
    {
        return Err(StorageError::Unavailable);
    }
    // Public API411 exposes player_stacks, not internal inventory.online.
    // None rejects offline/dead/acting players without taking the file path.
    host.player_stacks(uuid)
        .ok_or(StorageError::Unavailable)?
        .get(slot)
        .cloned()
        .ok_or(StorageError::Slot)
}

#[derive(Clone, Debug)]
pub struct NativeTestConfig {
    pub world_root: PathBuf,
    pub world_id: WorldId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreditFailpoint {
    None,
    AfterInventory,
    AfterInline,
    RollbackInventory,
}

/// Server-resolved intent, never deserialized from client packets.
pub struct CreditIntent {
    pub player: u128,
    pub generation: u64,
    pub transaction: TransactionId,
    pub metal: Metal,
    pub amount_milli: i64,
    pub slot: usize,
    pub expected: Option<ItemStack>,
    pub replacement: Option<ItemStack>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerStatus {
    Ready,
    Busy,
    Fault,
}

struct Entry {
    saved: SavedPlayer,
    namespace: Tag,
    generation: u64,
    anchor: Option<u64>,
    pending_death: bool,
    death_applied: bool,
    status: PlayerStatus,
    // Retain exact evidence if rollback cannot be verified; no automatic retry.
    fault_preimage: Option<(RawPlayer, Option<ItemStack>, Option<ItemStack>)>,
}

#[derive(Default)]
struct Session {
    epoch: Option<u64>,
    root: Option<PathBuf>,
    world: Option<WorldId>,
    generation: u64,
    players: BTreeMap<u128, Entry>,
    pending_deaths: BTreeSet<u128>,
    blocked: BTreeSet<u128>,
    world_fault: bool,
}

#[derive(Default)]
pub struct SaveBackend {
    session: Mutex<Session>,
    test: Option<NativeTestConfig>,
}

impl SaveBackend {
    pub fn for_native_test(config: NativeTestConfig) -> Self {
        Self {
            session: Mutex::new(Session::default()),
            test: Some(config),
        }
    }

    fn world(&self, host: &dyn Host) -> Result<WorldId, StorageError> {
        let root = host.world_dir().ok_or(StorageError::Unavailable)?;
        let loaded = load_world_identity(&root);
        let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
        if session.root.as_ref() != Some(&root) {
            session.players.clear();
            session.pending_deaths.clear();
            session.blocked.clear();
            session.world = None;
            session.world_fault = false;
            session.root = Some(root);
        }
        if session.world_fault {
            return Err(StorageError::Identity);
        }
        let loaded = match loaded {
            Ok(loaded) => loaded,
            Err(_) => {
                session.world_fault = true;
                return Err(StorageError::Identity);
            }
        };
        if session.world.is_none() {
            if let Some(identity) = &loaded {
                eprintln!(
                    "ALLOMANCY_SAVE event=world_identity source={:?}",
                    identity.source
                );
            }
        }
        let world = loaded.map(|identity| identity.world_id);
        if session.world.is_some() && session.world != world {
            session.world_fault = true;
            return Err(StorageError::Identity);
        }
        session.world = world.clone();
        world.ok_or(StorageError::Disabled)
    }

    fn test_world(&self, host: &dyn Host) -> Result<WorldId, StorageError> {
        let config = self.test.as_ref().ok_or(StorageError::TestOnly)?;
        let actual = host
            .world_dir()
            .ok_or(StorageError::TestOnly)?
            .canonicalize()
            .map_err(|_| StorageError::TestOnly)?;
        let expected = config
            .world_root
            .canonicalize()
            .map_err(|_| StorageError::TestOnly)?;
        if actual != expected {
            return Err(StorageError::TestOnly);
        }
        let world = self.world(host)?;
        if world != config.world_id {
            return Err(StorageError::Identity);
        }
        Ok(world)
    }

    pub fn generation(&self, uuid: u128) -> Result<u64, StorageError> {
        let session = self.session.lock().map_err(|_| StorageError::Fault)?;
        session
            .players
            .get(&uuid)
            .map(|entry| entry.generation)
            .ok_or(StorageError::Unavailable)
    }

    pub fn status(&self, uuid: u128) -> Result<PlayerStatus, StorageError> {
        let session = self.session.lock().map_err(|_| StorageError::Fault)?;
        if session.blocked.contains(&uuid) {
            return Ok(PlayerStatus::Fault);
        }
        session
            .players
            .get(&uuid)
            .map(|entry| entry.status)
            .ok_or(StorageError::Unavailable)
    }

    fn fault(&self, uuid: u128, error: &StorageError) {
        if let Ok(mut session) = self.session.lock() {
            session.blocked.insert(uuid);
            if let Some(entry) = session.players.get_mut(&uuid) {
                entry.status = PlayerStatus::Fault;
            }
        }
        eprintln!("ALLOMANCY_SAVE event=fault reason={error:?}");
    }

    pub(crate) fn login(&self, host: &mut dyn Host, uuid: u128) {
        eprintln!("ALLOMANCY_SAVE event=login");
        let result = (|| {
            let world = self.world(host)?;
            // Check even an already loaded Entry before the early return.
            let raw = raw_player(host, uuid)?;
            {
                let session = self.session.lock().map_err(|_| StorageError::Fault)?;
                if session.blocked.contains(&uuid) {
                    return Err(StorageError::Fault);
                }
                if session.players.contains_key(&uuid) {
                    return Ok(());
                }
            }
            let mut saved = match &raw.namespace {
                Some(namespace) => decode_namespace(namespace, &world, &player_id(uuid))?,
                None => SavedPlayer::new(player_id(uuid), world, initial_rules())?,
            };
            let pending = self
                .session
                .lock()
                .map_err(|_| StorageError::Fault)?
                .pending_deaths
                .contains(&uuid);
            if raw.dead || pending {
                on_death(&mut saved)?;
            }
            on_reconnect(&mut saved)?;
            let namespace = encode_namespace(&saved)?;
            let merged = merge(raw.mods.as_ref(), namespace.clone())?;
            // The inline setter doesn't drain callbacks; no Session lock crosses Host.
            if !host.set_player_persistent_data(uuid, merged.clone()) {
                return Err(StorageError::HostRejected);
            }
            if raw_player(host, uuid)?.mods != Some(merged) {
                return Err(StorageError::Conflict);
            }
            let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
            session.generation = session
                .generation
                .checked_add(1)
                .ok_or(StorageError::StaleConnection)?;
            let generation = session.generation;
            session.pending_deaths.remove(&uuid);
            session.players.insert(
                uuid,
                Entry {
                    saved,
                    namespace,
                    generation,
                    anchor: None,
                    pending_death: false,
                    death_applied: raw.dead || pending,
                    status: PlayerStatus::Ready,
                    fault_preimage: None,
                },
            );
            eprintln!("ALLOMANCY_SAVE event=ready accepted_in_memory=true");
            Ok::<_, StorageError>(())
        })();
        if let Err(error) = result {
            if error != StorageError::Disabled {
                self.fault(uuid, &error);
            }
        }
    }

    pub(crate) fn queue_death(&self, uuid: u128) {
        if let Ok(mut session) = self.session.lock() {
            if session
                .players
                .get(&uuid)
                .is_some_and(|entry| entry.death_applied)
            {
                return;
            }
            session.pending_deaths.insert(uuid);
            if let Some(entry) = session.players.get_mut(&uuid) {
                entry.pending_death = true;
            }
        }
        eprintln!("ALLOMANCY_SAVE event=death_queued");
    }

    /// A complete inline candidate is accepted before replacing installed core.
    fn mutate<F>(
        &self,
        host: &mut dyn Host,
        uuid: u128,
        acting: bool,
        change: F,
    ) -> Result<(), StorageError>
    where
        F: FnOnce(&mut SavedPlayer) -> Result<(), SaveError>,
    {
        let (before, namespace, generation) = {
            let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
            if session.world_fault {
                return Err(StorageError::Identity);
            }
            let entry = session
                .players
                .get_mut(&uuid)
                .ok_or(StorageError::Unavailable)?;
            match entry.status {
                PlayerStatus::Ready => {}
                PlayerStatus::Busy => return Err(StorageError::Busy),
                PlayerStatus::Fault => return Err(StorageError::Fault),
            }
            entry.status = PlayerStatus::Busy;
            (
                entry.saved.clone(),
                entry.namespace.clone(),
                entry.generation,
            )
        };
        let result = (|| {
            let mods = if acting {
                // Already loaded/validated players only; never initialize via getter.
                let mods = host
                    .player_persistent_data(uuid)
                    .ok_or(StorageError::Unavailable)?;
                reject_quarantine(Some(&mods))?;
                if !matches!(mods, Tag::Compound(_)) || mods.get(NAMESPACE) != Some(&namespace) {
                    return Err(StorageError::Conflict);
                }
                Some(mods)
            } else {
                let raw = raw_player(host, uuid)?;
                if raw.namespace.as_ref() != Some(&namespace) {
                    return Err(StorageError::Conflict);
                }
                raw.mods
            };
            let mut candidate = before.clone();
            change(&mut candidate)?;
            let next_namespace = encode_namespace(&candidate)?;
            let merged = merge(mods.as_ref(), next_namespace.clone())?;
            if !host.set_player_persistent_data(uuid, merged.clone()) {
                return Err(StorageError::HostRejected);
            }
            let observed = if acting {
                let observed = host.player_persistent_data(uuid);
                reject_quarantine(observed.as_ref())?;
                observed
            } else {
                raw_player(host, uuid)?.mods
            };
            if observed != Some(merged) {
                return Err(StorageError::Conflict);
            }
            let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
            let entry = session
                .players
                .get_mut(&uuid)
                .ok_or(StorageError::Unavailable)?;
            if entry.status != PlayerStatus::Busy
                || entry.generation != generation
                || entry.saved != before
            {
                return Err(StorageError::Conflict);
            }
            entry.saved = candidate;
            entry.namespace = next_namespace;
            entry.status = PlayerStatus::Ready;
            Ok(())
        })();
        if let Err(error) = &result {
            self.fault(uuid, error);
        }
        result
    }

    fn drain_death(
        &self,
        host: &mut dyn Host,
        uuid: u128,
        acting: bool,
    ) -> Result<(), StorageError> {
        let pending = {
            let session = self.session.lock().map_err(|_| StorageError::Fault)?;
            session.pending_deaths.contains(&uuid)
                || session.players.get(&uuid).is_some_and(|e| e.pending_death)
        };
        if !pending {
            return Ok(());
        }
        self.mutate(host, uuid, acting, on_death)?;
        let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
        session.pending_deaths.remove(&uuid);
        if let Some(entry) = session.players.get_mut(&uuid) {
            entry.pending_death = false;
            entry.death_applied = true;
            entry.anchor = None;
        }
        eprintln!("ALLOMANCY_SAVE event=death_applied accepted_in_memory=true");
        Ok(())
    }

    pub(crate) fn logging_out(&self, host: &mut dyn Host, uuid: u128) {
        eprintln!("ALLOMANCY_SAVE event=logging_out");
        // Defend dead-before-logout even if the notification was missed.
        match raw_player(host, uuid) {
            Ok(raw) if raw.dead => self.queue_death(uuid),
            Ok(_) => {}
            Err(error) => {
                self.fault(uuid, &error);
                return;
            }
        }
        if let Err(error) = self.drain_death(host, uuid, false) {
            self.fault(uuid, &error);
        }
    }

    pub(crate) fn logged_out(&self, uuid: u128) {
        if let Ok(mut session) = self.session.lock() {
            // Retain unresolved deaths/fault evidence until session/world ends.
            if session
                .players
                .get(&uuid)
                .is_some_and(|e| e.status == PlayerStatus::Ready && !e.pending_death)
            {
                session.players.remove(&uuid);
            }
        }
        eprintln!("ALLOMANCY_SAVE event=logged_out");
    }

    pub(crate) fn respawned(&self, host: &mut dyn Host, uuid: u128) {
        eprintln!("ALLOMANCY_SAVE event=respawn");
        self.queue_death(uuid);
        if let Err(error) = self.drain_death(host, uuid, true) {
            self.fault(uuid, &error);
        }
        if let Ok(mut session) = self.session.lock() {
            if let Some(entry) = session.players.get_mut(&uuid) {
                entry.death_applied = false;
                entry.anchor = None;
            }
        }
    }

    pub(crate) fn changed_dimension(&self, uuid: u128) {
        if let Ok(mut session) = self.session.lock() {
            // Cancel slot intentions without changing persistent WorldId/ledger.
            session.generation = session.generation.saturating_add(1);
            let generation = session.generation;
            if let Some(entry) = session.players.get_mut(&uuid) {
                entry.generation = generation;
                entry.anchor = None;
            }
        }
        eprintln!("ALLOMANCY_SAVE event=dimension_changed");
    }

    pub(crate) fn tick(&self, host: &mut dyn Host, epoch: u64, tick: u64) {
        {
            let Ok(mut session) = self.session.lock() else {
                return;
            };
            if session.epoch.is_some_and(|old| old != epoch) {
                session.players.clear();
                session.pending_deaths.clear();
                session.blocked.clear();
                session.root = None;
                session.world = None;
                session.world_fault = false;
            }
            session.epoch = Some(epoch);
        }
        if let Err(error) = self.world(host) {
            if error != StorageError::Disabled {
                eprintln!("ALLOMANCY_SAVE event=world_fault reason={error:?}");
            }
            return;
        }
        let mut players = host.server_players();
        players.sort_by_key(|p| p.uuid);
        for player in players {
            if self
                .session
                .lock()
                .is_ok_and(|s| s.blocked.contains(&player.uuid))
            {
                continue;
            }
            let loaded = self
                .session
                .lock()
                .is_ok_and(|s| s.players.contains_key(&player.uuid));
            if !loaded {
                self.login(host, player.uuid);
            }
            if player.health <= 0.0 {
                self.queue_death(player.uuid);
            }
            if let Err(error) = self.drain_death(host, player.uuid, false) {
                self.fault(player.uuid, &error);
                continue;
            }
            let consume = {
                let Ok(mut session) = self.session.lock() else {
                    return;
                };
                let Some(entry) = session.players.get_mut(&player.uuid) else {
                    continue;
                };
                if entry.status != PlayerStatus::Ready || entry.pending_death {
                    continue;
                }
                let previous = entry.anchor.replace(tick);
                // Idle: no SavedPlayer clone, encode, revision increment or Host write.
                if !entry
                    .saved
                    .state()
                    .metals
                    .iter()
                    .any(|metal| metal.burn != BurnState::Off)
                {
                    continue;
                }
                previous
                    .and_then(|old| tick.checked_sub(old))
                    .filter(|elapsed| *elapsed > 0)
            };
            if let Some(elapsed) = consume {
                let result = self.mutate(host, player.uuid, false, |saved| {
                    let rules = saved.rules().clone();
                    update_player(saved, |state, accounting| {
                        consume_reserves(state, accounting, &rules, elapsed)?;
                        state.last_tick = Some(Tick(tick));
                        Ok(())
                    })
                });
                if let Err(error) = result {
                    self.fault(player.uuid, &error);
                }
            }
        }
    }

    /// Dedicated-test seed; no inventory grant. Controller prepares A's slot.
    pub fn seed_for_native_test(
        &self,
        host: &mut dyn Host,
        uuid: u128,
        saved: SavedPlayer,
    ) -> Result<(), StorageError> {
        let world = self.test_world(host)?;
        decode_save(&encode_save(&saved)?, &world, &player_id(uuid))?;
        self.mutate(host, uuid, false, |current| {
            *current = saved;
            Ok(())
        })
    }

    pub fn native_test_state(&self, uuid: u128) -> Option<(SavedPlayer, u64)> {
        self.test.as_ref()?;
        let session = self.session.lock().ok()?;
        let entry = session.players.get(&uuid)?;
        (entry.status == PlayerStatus::Ready).then(|| (entry.saved.clone(), entry.generation))
    }

    pub(crate) fn run_native_test_driver(
        &self,
        host: &mut dyn Host,
        driver: fn(&SaveBackend, &mut dyn Host),
    ) {
        if self.test_world(host).is_ok() {
            driver(self, host);
        }
    }

    pub fn credit_for_native_test(
        &self,
        host: &mut dyn Host,
        intent: CreditIntent,
        failpoint: CreditFailpoint,
    ) -> Result<CreditReceipt, StorageError> {
        self.test_world(host)?;
        let uuid = intent.player;
        let (before, namespace, prepared) = {
            let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
            let entry = session
                .players
                .get_mut(&uuid)
                .ok_or(StorageError::Unavailable)?;
            if entry.status != PlayerStatus::Ready {
                return Err(if entry.status == PlayerStatus::Busy {
                    StorageError::Busy
                } else {
                    StorageError::Fault
                });
            }
            if entry.generation != intent.generation || entry.pending_death {
                return Err(StorageError::StaleConnection);
            }
            let prepared = prepare_credit(
                &entry.saved,
                intent.transaction.clone(),
                intent.metal,
                intent.amount_milli,
            )?;
            let before = entry.saved.clone();
            let namespace = entry.namespace.clone();
            entry.status = PlayerStatus::Busy;
            (before, namespace, prepared)
        };
        let result = self.apply_credit(host, &intent, failpoint, &before, &namespace, prepared);
        if let Ok(mut session) = self.session.lock() {
            if let Some(entry) = session.players.get_mut(&uuid) {
                if entry.status == PlayerStatus::Busy {
                    entry.status = PlayerStatus::Ready;
                }
            }
        }
        result
    }

    fn apply_credit(
        &self,
        host: &mut dyn Host,
        intent: &CreditIntent,
        failpoint: CreditFailpoint,
        before: &SavedPlayer,
        namespace: &Tag,
        preparation: CreditPreparation,
    ) -> Result<CreditReceipt, StorageError> {
        let uuid = intent.player;
        let raw = match raw_player(host, uuid) {
            Ok(raw) => raw,
            Err(error) => {
                self.fault(uuid, &error);
                return Err(error);
            }
        };
        if raw.dead || raw.namespace.as_ref() != Some(namespace) {
            self.fault(uuid, &StorageError::Conflict);
            return Err(StorageError::Conflict);
        }
        let prepared = match preparation {
            CreditPreparation::Replay(receipt) => {
                eprintln!("ALLOMANCY_SAVE event=credit_replay");
                return Ok(receipt);
            }
            CreditPreparation::Pending(prepared) => prepared,
        };
        let next_namespace = encode_namespace(prepared.candidate())?;
        let merged = merge(raw.mods.as_ref(), next_namespace.clone())?;
        if live_slot(host, uuid, intent.slot)? != intent.expected {
            return Err(StorageError::Slot);
        }
        {
            let session = self.session.lock().map_err(|_| StorageError::Fault)?;
            let entry = session
                .players
                .get(&uuid)
                .ok_or(StorageError::Unavailable)?;
            if entry.status != PlayerStatus::Busy
                || entry.generation != intent.generation
                || entry.pending_death
                || !prepared.is_current(&entry.saved)
            {
                return Err(StorageError::StaleConnection);
            }
        }
        // Last live check before helper. Arbitrary foreign mods disconnecting
        // inside SaveToFile lie outside the compatibility scope of this API.
        if live_slot(host, uuid, intent.slot)? != intent.expected {
            return Err(StorageError::Slot);
        }
        let old = match host.set_inventory_slot(
            uuid,
            NativeInventoryKind::Player,
            intent.slot,
            intent.expected.as_ref(),
            intent.replacement.clone(),
        ) {
            Ok(old) => old,
            Err(_) => {
                if raw_player(host, uuid).is_ok_and(|now| now == raw)
                    && live_slot(host, uuid, intent.slot).is_ok_and(|slot| slot == intent.expected)
                {
                    return Err(StorageError::HostRejected);
                }
                self.fault(uuid, &StorageError::Conflict);
                return Err(StorageError::Conflict);
            }
        };
        let mut inline_applied = false;
        let applied = (|| {
            if old != intent.expected
                || raw_player(host, uuid)? != raw
                || live_slot(host, uuid, intent.slot)? != intent.replacement
            {
                return Err(StorageError::Conflict);
            }
            if matches!(
                failpoint,
                CreditFailpoint::AfterInventory | CreditFailpoint::RollbackInventory
            ) {
                return Err(StorageError::HostRejected);
            }
            if !host.set_player_persistent_data(uuid, merged.clone()) {
                return Err(StorageError::HostRejected);
            }
            inline_applied = true;
            if failpoint == CreditFailpoint::AfterInline {
                return Err(StorageError::HostRejected);
            }
            if raw_player(host, uuid)?.mods != Some(merged.clone())
                || live_slot(host, uuid, intent.slot)? != intent.replacement
            {
                return Err(StorageError::Conflict);
            }
            let mut session = self.session.lock().map_err(|_| StorageError::Fault)?;
            let entry = session
                .players
                .get_mut(&uuid)
                .ok_or(StorageError::Unavailable)?;
            if entry.status != PlayerStatus::Busy
                || entry.generation != intent.generation
                || entry.pending_death
                || entry.saved != *before
            {
                return Err(StorageError::StaleConnection);
            }
            let receipt = commit_credit(&mut entry.saved, prepared)?;
            entry.namespace = next_namespace;
            entry.status = PlayerStatus::Ready;
            eprintln!(
                "ALLOMANCY_SAVE event=credit_accepted revision={} durable=false",
                receipt.committed_revision
            );
            Ok(receipt)
        })();
        match applied {
            Ok(receipt) => Ok(receipt),
            Err(error) => {
                if self
                    .rollback(host, intent, &raw, &merged, inline_applied, failpoint)
                    .is_err()
                {
                    if let Ok(mut session) = self.session.lock() {
                        if let Some(entry) = session.players.get_mut(&uuid) {
                            entry.fault_preimage =
                                Some((raw, intent.expected.clone(), intent.replacement.clone()));
                        }
                    }
                    self.fault(uuid, &StorageError::RollbackFailed);
                    eprintln!("ALLOMANCY_SAVE event=rollback_unverified");
                    match self.quarantine(host, uuid) {
                        Ok(()) => eprintln!(
                            "ALLOMANCY_SAVE event=quarantine_confirmed accepted_in_memory=true durable=false"
                        ),
                        Err(error) => eprintln!(
                            "ALLOMANCY_SAVE event=quarantine_unconfirmed reason={error:?} durable=false"
                        ),
                    }
                    Err(StorageError::RollbackFailed)
                } else {
                    eprintln!("ALLOMANCY_SAVE event=rollback_verified");
                    Err(error)
                }
            }
        }
    }

    // Only the current valid raw Compound may be marked; never restore a stale
    // root or trust the normalized getter to repair missing/invalid data.
    fn quarantine(&self, host: &mut dyn Host, uuid: u128) -> Result<(), StorageError> {
        let current = inspect_player_structure(
            &host
                .player_compound(uuid)
                .ok_or(StorageError::Unavailable)?,
            uuid,
        )?;
        let already_marked = current
            .mods
            .as_ref()
            .is_some_and(|tag| tag.get(STORAGE_FAULT).is_some());
        let Some(Tag::Compound(mut mods)) = current.mods else {
            return Err(StorageError::ModsRoot);
        };
        if !already_marked {
            mods.insert(
                STORAGE_FAULT.into(),
                Tag::compound([
                    ("format_version", Tag::Int(1)),
                    ("reason", Tag::String("rollback_unverified".into())),
                ]),
            );
        }
        let expected = Tag::Compound(mods);
        if !already_marked && !host.set_player_persistent_data(uuid, expected.clone()) {
            return Err(StorageError::HostRejected);
        }
        let observed = inspect_player_structure(
            &host
                .player_compound(uuid)
                .ok_or(StorageError::Unavailable)?,
            uuid,
        )?;
        if observed.mods != Some(expected) {
            return Err(StorageError::Conflict);
        }
        Ok(())
    }

    fn rollback(
        &self,
        host: &mut dyn Host,
        intent: &CreditIntent,
        raw: &RawPlayer,
        merged: &Tag,
        inline_applied: bool,
        failpoint: CreditFailpoint,
    ) -> Result<(), StorageError> {
        let uuid = intent.player;
        if failpoint == CreditFailpoint::RollbackInventory {
            return Err(StorageError::RollbackFailed);
        }
        if live_slot(host, uuid, intent.slot)? != intent.replacement {
            return Err(StorageError::RollbackFailed);
        }
        let current = raw_player(host, uuid)?;
        let expected_mods = if inline_applied {
            Some(merged)
        } else {
            raw.mods.as_ref()
        };
        if current.mods.as_ref() != expected_mods {
            return Err(StorageError::RollbackFailed);
        }
        let restored = host
            .set_inventory_slot(
                uuid,
                NativeInventoryKind::Player,
                intent.slot,
                intent.replacement.as_ref(),
                intent.expected.clone(),
            )
            .map_err(|_| StorageError::RollbackFailed)?;
        if restored != intent.replacement {
            return Err(StorageError::RollbackFailed);
        }
        // Re-check after reentrant inverse CAS; don't overwrite foreign changes.
        if raw_player(host, uuid)?.mods.as_ref() != expected_mods {
            return Err(StorageError::RollbackFailed);
        }
        if inline_applied {
            let old_mods = raw
                .mods
                .clone()
                .unwrap_or_else(|| Tag::Compound(Compound::new()));
            if !host.set_player_persistent_data(uuid, old_mods) {
                return Err(StorageError::RollbackFailed);
            }
        }
        if raw_player(host, uuid)? != *raw || live_slot(host, uuid, intent.slot)? != intent.expected
        {
            return Err(StorageError::RollbackFailed);
        }
        Ok(())
    }
}
