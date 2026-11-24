//! Server-side session: loads each online player's save, runs the core every server tick,
//! persists changes and applies credits transactionally with the inventory.
use crate::content::{Drink, Finished, Study};
use crate::controls::pair_name;
use crate::save::{self, Blocked, Loaded};
use allomancy_core::{
    Authority, BurnState, CATALOG, EMPTY_VIAL, EventId, EventKind, InputContext, Metal, Observed,
    PlayerId, PreparedCredit, ReserveRules, SaveError, SavedPlayer, Tick, TransactionSeq,
    VIAL_MILLI, Vec3, VerifiedEvent, WorldId, WorldSnapshot, advance_tick, commit_credit,
    consume_reserves, milestone_def, node_def, observe, on_death, on_reconnect, prepare_claim,
    prepare_conclude, prepare_hint, prepare_purchase, prepare_unlock, prepare_vial, research_def,
    update_player,
};
use mod_api::{
    GameMode, Host, HostTransaction, Identifier, ItemStack, PlayerRef, ServerHost, Text,
};
use std::collections::BTreeMap;

/// Per-player isolation between worlds comes from the game's per-world player files, so the
/// core sees one constant world identity (see docs/contracts.md).
pub(crate) const WORLD_ID: &str = "local";
/// Dirty state is written at most this many ticks late (and at once on exhaustion, death and
/// every transaction). A cut loses at most this much consumption, in the player's favour.
const WRITE_EVERY_TICKS: u32 = 20;
const TICK_REPORTS: u8 = 3;

/// Trial values (docs/reserves.md); P3 moves them to data.
pub(crate) fn default_rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [10; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    }
}

pub(crate) fn player_id(uuid: u128) -> PlayerId {
    PlayerId::new(format!("{uuid:032x}"))
}

pub(crate) fn world_id() -> WorldId {
    WorldId::new(WORLD_ID)
}

// One slot per online player: the size difference is irrelevant next to boxing churn.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Slot {
    Active {
        saved: SavedPlayer,
        /// Changed since the last successful write.
        dirty: bool,
        /// Write on the next tick regardless of the interval.
        urgent: bool,
        since_write: u32,
    },
    Blocked(Blocked),
}

#[derive(Default)]
pub(crate) struct Session {
    epoch: Option<u64>,
    players: BTreeMap<u128, Slot>,
    reports: u8,
    /// Finished uses of our items (vials and unlock items), applied once the hand is written.
    uses: Vec<Finished>,
}

/// Chat line from the mod's translations (`assets/mistbrn/lang`), never a raw error code.
pub(crate) fn tell(host: &mut dyn ServerHost, player: PlayerRef, key: &str) {
    let _ = player.tell(host, Text::translate(key, Vec::new()));
}

fn slot_stack(host: &dyn Host, player: PlayerRef, slot: u32) -> Option<ItemStack> {
    player
        .inventory_slots(host)?
        .into_iter()
        .find(|(s, _)| *s == slot)
        .map(|(_, stack)| stack)
}

/// What a refused drink tells the player. `item_not_in_slot` is silent: the vial moved before
/// the next tick, or a duplicate of an already applied drink was refused.
pub(crate) fn refusal_message(error: &str) -> Option<&'static str> {
    if error == "item_not_in_slot" {
        None
    } else if error.contains("CapacityExceeded") {
        Some("message.mistbrn.vial.reserve_full")
    } else {
        Some("message.mistbrn.vial.failed")
    }
}

/// Why an unlock item was not used. Only `Failed` changed nothing for a reason worth reporting
/// as a failure; the item stays in every case.
pub(crate) enum Refusal {
    AlreadyUnlocked,
    NotInSlot,
    Failed(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyUnlocked => f.write_str("AlreadyUnlocked"),
            Self::NotInSlot => f.write_str("item_not_in_slot"),
            Self::Failed(reason) => f.write_str(reason),
        }
    }
}

impl Session {
    pub(crate) fn slot(&self, uuid: u128) -> Option<&Slot> {
        self.players.get(&uuid)
    }

    pub(crate) fn saved(&self, uuid: u128) -> Option<&SavedPlayer> {
        match self.players.get(&uuid)? {
            Slot::Active { saved, .. } => Some(saved),
            Slot::Blocked(_) => None,
        }
    }

    /// Loads the player's save once per session. Absent → fresh save written at once;
    /// found → reconnect (burns off) and written; unreadable → blocked, never written.
    pub(crate) fn ensure_loaded(&mut self, host: &mut dyn ServerHost, player: PlayerRef) {
        let uuid = player.uuid();
        if self.players.contains_key(&uuid) {
            return;
        }
        let id = player_id(uuid);
        let world = world_id();
        let slot = match save::read(host, player, &world, &id) {
            Err(save::Unavailable) => return, // retry next tick
            Ok(Loaded::Absent) => match SavedPlayer::new(id, world, default_rules()) {
                Ok(saved) => {
                    eprintln!("MISTBRN event=save_loaded state=fresh");
                    active(saved)
                }
                Err(error) => Slot::Blocked(Blocked::Decode(error)),
            },
            Ok(Loaded::Found(mut saved)) => {
                let _ = on_reconnect(&mut saved);
                eprintln!(
                    "MISTBRN event=save_loaded state=found revision={} last_transaction={}",
                    saved.revision(),
                    saved.last_transaction()
                );
                active(saved)
            }
            Ok(Loaded::Blocked(blocked)) => Slot::Blocked(blocked),
        };
        if let Slot::Blocked(blocked) = slot {
            self.block(host, uuid, blocked);
            return;
        }
        self.players.insert(uuid, slot);
        self.flush(host, player);
    }

    /// Writes a dirty, active player's save if due (or `force`).
    fn flush_if(&mut self, host: &mut dyn ServerHost, player: PlayerRef, force: bool) {
        let Some(Slot::Active {
            saved,
            dirty,
            urgent,
            since_write,
        }) = self.players.get_mut(&player.uuid())
        else {
            return;
        };
        if !*dirty || !(force || *urgent || *since_write >= WRITE_EVERY_TICKS) {
            return;
        }
        match host.transaction(|host, _tx| save::write(host, player, saved)) {
            Ok(()) => {
                *dirty = false;
                *urgent = false;
                *since_write = 0;
            }
            // Stale or dead player: keep it dirty and retry after a full interval.
            Err(error) => {
                *urgent = false;
                *since_write = 0;
                eprintln!("MISTBRN event=save_write_failed error={error}");
            }
        }
    }

    pub(crate) fn flush(&mut self, host: &mut dyn ServerHost, player: PlayerRef) {
        self.flush_if(host, player, true);
    }

    /// Trusted change of an active player's save, marked for writing.
    pub(crate) fn change<F>(&mut self, uuid: u128, urgent_write: bool, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut SavedPlayer) -> Result<(), SaveError>,
    {
        match self.players.get_mut(&uuid) {
            Some(Slot::Active {
                saved,
                dirty,
                urgent,
                ..
            }) => {
                f(saved).map_err(|error| format!("{error:?}"))?;
                *dirty = true;
                *urgent |= urgent_write;
                Ok(())
            }
            Some(Slot::Blocked(blocked)) => Err(format!("blocked:{}", blocked.reason())),
            None => Err("not_loaded".into()),
        }
    }

    /// Death: reserves cleared in memory at once and written as soon as the player can be
    /// written (now if alive enough, else after the respawn).
    pub(crate) fn died(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        self.uses.retain(|queued| queued.player().uuid() != uuid);
        if self.change(uuid, true, on_death).is_ok() {
            eprintln!("MISTBRN event=death_cleared");
            if let Some(player) = host.player(uuid) {
                self.flush(host, player);
            }
        }
    }

    /// Respawn after a death (`player_clone` with `was_death`). The dying player usually cannot be
    /// written (`Dead`), and a player who leaves from the death screen takes no write with them,
    /// so the death is applied here, on the new player, from whatever save it carried over
    /// (`PlayerPersisted` survives the respawn). `on_death` is idempotent in state.
    pub(crate) fn respawned_after_death(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        let Some(player) = host.player(uuid) else {
            return;
        };
        self.ensure_loaded(host, player);
        if self.change(uuid, true, on_death).is_ok() {
            eprintln!("MISTBRN event=respawn_death_applied");
            self.flush(host, player);
        }
    }

    /// Stops all writes for this player for the rest of the session and tells them.
    pub(crate) fn block(&mut self, host: &mut dyn ServerHost, uuid: u128, reason: Blocked) {
        eprintln!("MISTBRN event=save_blocked reason={}", reason.reason());
        if let Some(player) = host.player(uuid) {
            tell(host, player, "message.mistbrn.save_blocked");
        }
        self.players.insert(uuid, Slot::Blocked(reason));
    }

    pub(crate) fn logged_out(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        self.uses.retain(|queued| queued.player().uuid() != uuid);
        if let Some(player) = host.player(uuid) {
            self.flush(host, player);
        }
        self.players.remove(&uuid);
    }

    /// Queues one finished use (vial or unlock item). The finish hook can be heard more than
    /// once for one use (on a server it may fire for every player of the world): one use per
    /// player and slot per tick.
    pub(crate) fn queue_use(&mut self, finished: Finished) {
        if !self.uses.iter().any(|queued| {
            queued.player().uuid() == finished.player().uuid() && queued.slot() == finished.slot()
        }) {
            self.uses.push(finished);
        }
    }

    pub(crate) fn finish_uses(&mut self, host: &mut dyn ServerHost, fail_inside: bool) {
        for finished in std::mem::take(&mut self.uses) {
            let Some(player) = host.player(finished.player().uuid()) else {
                continue;
            };
            if player.entity() != finished.player().entity()
                || !matches!(
                    player.game_mode(host),
                    Some(GameMode::Survival | GameMode::Adventure)
                )
            {
                continue;
            }
            let drink = match finished {
                Finished::Drink(drink) => drink,
                Finished::Study(study) => {
                    self.finish_study(host, player, study, fail_inside);
                    continue;
                }
            };
            let metal = metal_name(drink.metal);
            match self.credit_from_slot(host, drink, VIAL_MILLI, fail_inside) {
                Ok((seq, unlocked)) => {
                    eprintln!(
                        "MISTBRN event=vial_credit metal={metal} seq={} unlocked={unlocked}",
                        seq.0
                    );
                    if unlocked {
                        // The tutorial: the first iron or steel vial teaches the pair.
                        eprintln!(
                            "MISTBRN event=unlock pair=iron_steel seq={} via=vial",
                            seq.0
                        );
                        tell(host, player, "message.mistbrn.tutorial.unlocked");
                    }
                }
                Err(error) => {
                    eprintln!("MISTBRN event=vial_refused metal={metal} reason={error}");
                    if let Some(key) = refusal_message(&error) {
                        tell(host, player, key);
                    }
                }
            }
        }
    }

    fn finish_study(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        study: Study,
        fail_inside: bool,
    ) {
        let pair = pair_name(study.pair);
        match self.unlock_from_slot(host, study, fail_inside) {
            Ok(seq) => {
                eprintln!("MISTBRN event=unlock pair={pair} seq={}", seq.0);
                tell(host, player, "message.mistbrn.primer.unlocked");
            }
            Err(refusal) => {
                eprintln!("MISTBRN event=unlock_refused pair={pair} reason={refusal}");
                match refusal {
                    Refusal::AlreadyUnlocked => tell(host, player, "message.mistbrn.primer.known"),
                    // The item moved: nothing was used, nothing to say.
                    Refusal::NotInSlot => {}
                    Refusal::Failed(_) => tell(host, player, "message.mistbrn.primer.failed"),
                }
            }
        }
    }

    /// Consume one unlock item and persist its pair's unlock together (provisional V1 unlock,
    /// replaced by P5/P6 progression). Same exactly-once path as a vial credit; a pair already
    /// unlocked keeps the item. `fail_inside` injects a failure after all writes (tests).
    pub(crate) fn unlock_from_slot(
        &mut self,
        host: &mut dyn ServerHost,
        study: Study,
        fail_inside: bool,
    ) -> Result<TransactionSeq, Refusal> {
        let Study {
            player,
            slot,
            stack: expected,
            pair,
        } = study;
        let uuid = player.uuid();
        if host.transaction_current().is_some() {
            return Err(Refusal::Failed("nested_transaction".into()));
        }
        if slot_stack(host, player, slot).as_ref() != Some(&expected) {
            return Err(Refusal::NotInSlot);
        }
        let Some(Slot::Active {
            saved,
            dirty,
            urgent,
            since_write,
        }) = self.players.get_mut(&uuid)
        else {
            return Err(Refusal::Failed("not_active".into()));
        };
        let prepared = prepare_unlock(saved, pair).map_err(|error| match error {
            SaveError::AlreadyUnlocked => Refusal::AlreadyUnlocked,
            other => Refusal::Failed(format!("{other:?}")),
        })?;
        host.transaction(|host, _tx| {
            let stack = slot_stack(host, player, slot)
                .filter(|stack| *stack == expected)
                .ok_or(Refusal::NotInSlot)?;
            player
                .set_inventory_slot(host, slot, stack.shrink(1))
                .map_err(|e| Refusal::Failed(format!("set_slot:{e:?}")))?;
            save::write(host, player, prepared.candidate()).map_err(Refusal::Failed)?;
            if fail_inside {
                return Err(Refusal::Failed("injected_failure".into()));
            }
            Ok(())
        })?;
        match commit_credit(saved, prepared) {
            Ok(seq) => {
                *dirty = false;
                *urgent = false;
                *since_write = 0;
                Ok(seq)
            }
            Err(error) => {
                self.block(host, uuid, Blocked::Decode(error));
                Err(Refusal::Failed(format!("commit_after_host:{error:?}")))
            }
        }
    }

    /// Exchange one filled vial for its empty bottle and persist the credit together; the first
    /// iron or steel vial also unlocks the pair (P5 tutorial, `prepare_vial`). Returns the
    /// transaction and whether it unlocked. `fail_inside` injects a failure after all writes.
    pub(crate) fn credit_from_slot(
        &mut self,
        host: &mut dyn ServerHost,
        drink: Drink,
        amount_milli: i64,
        fail_inside: bool,
    ) -> Result<(TransactionSeq, bool), String> {
        let Drink {
            player,
            slot,
            stack: expected,
            metal,
        } = drink;
        let uuid = player.uuid();
        // Only an outermost transaction makes its writes final; inside another one, an outer
        // abort could undo the game's writes after the core committed.
        if host.transaction_current().is_some() {
            return Err("nested_transaction".into());
        }
        // A vial that moved is refused silently before any rule is checked (a full reserve
        // must not be reported for a drink that no longer exists); re-checked in the transaction.
        if slot_stack(host, player, slot).as_ref() != Some(&expected) {
            return Err("item_not_in_slot".into());
        }
        let Some(Slot::Active {
            saved,
            dirty,
            urgent,
            since_write,
        }) = self.players.get_mut(&uuid)
        else {
            return Err("not_active".into());
        };
        let (prepared, unlocked) =
            prepare_vial(saved, metal, amount_milli).map_err(|e| format!("{e:?}"))?;
        let empty = Identifier::parse(EMPTY_VIAL)
            .ok()
            .and_then(|id| host.item_key(&id))
            .and_then(|key| ItemStack::new(key, 1))
            .ok_or("empty_vial_unavailable")?;
        host.transaction(|host, _tx| {
            let stack = slot_stack(host, player, slot)
                .filter(|stack| *stack == expected)
                .ok_or("item_not_in_slot")?;
            let remaining = stack.shrink(1);
            let replacement = remaining.clone().or_else(|| Some(empty.clone()));
            player
                .set_inventory_slot(host, slot, replacement)
                .map_err(|e| format!("set_slot:{e:?}"))?;
            if remaining.is_some() {
                player
                    .give_stack(host, empty)
                    .map_err(|e| format!("give_empty_vial:{e:?}"))?;
            }
            save::write(host, player, prepared.candidate())?;
            if fail_inside {
                return Err("injected_failure".to_string());
            }
            Ok(())
        })?;
        match commit_credit(saved, prepared) {
            Ok(seq) => {
                // The candidate is already in the game; nothing left to write.
                *dirty = false;
                *urgent = false;
                *since_write = 0;
                Ok((seq, unlocked))
            }
            Err(error) => {
                // The game already holds the candidate: writing the older in-memory state would
                // erase the credit (docs/persistence.md). Stop writing this player instead.
                self.block(host, uuid, Blocked::Decode(error));
                Err(format!("commit_after_host:{error:?}"))
            }
        }
    }

    /// Writes a prepared save-only change (a purchase, a hint, a conclusion) in its own host
    /// transaction and commits it once that transaction committed, as a vial credit does.
    fn commit_save_only(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        prepare: impl FnOnce(&SavedPlayer) -> Result<PreparedCredit, SaveError>,
    ) -> Result<TransactionSeq, String> {
        let uuid = player.uuid();
        if host.transaction_current().is_some() {
            return Err("nested_transaction".into());
        }
        let (saved, dirty, urgent, since_write) = match self.players.get_mut(&uuid) {
            Some(Slot::Active {
                saved,
                dirty,
                urgent,
                since_write,
            }) => (saved, dirty, urgent, since_write),
            // A blocked save is never written this session: nothing to retry.
            Some(Slot::Blocked(_)) => return Err("blocked".into()),
            None => return Err("not_active".into()),
        };
        let prepared = prepare(saved).map_err(|error| match error {
            SaveError::Progress(error) => format!("{error:?}"),
            other => format!("{other:?}"),
        })?;
        host.transaction(|host, _tx| save::write(host, player, prepared.candidate()))?;
        match commit_credit(saved, prepared) {
            Ok(seq) => {
                *dirty = false;
                *urgent = false;
                *since_write = 0;
                Ok(seq)
            }
            Err(error) => {
                self.block(host, uuid, Blocked::Decode(error));
                Err(format!("commit_after_host:{error:?}"))
            }
        }
    }

    /// Buys `rank` of node `code` (the notebook, op 6). Logged with its result.
    pub(crate) fn purchase(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        code: u16,
        rank: u8,
    ) -> Result<TransactionSeq, String> {
        let node = node_def(code, &CATALOG).map_or("unknown", |def| def.key);
        let result =
            self.commit_save_only(host, player, |saved| prepare_purchase(saved, code, rank));
        match &result {
            Ok(seq) => {
                eprintln!(
                    "MISTBRN event=purchase node={node} rank={rank} result=ok seq={}",
                    seq.0
                );
                tell(host, player, "message.mistbrn.notebook.bought");
            }
            Err(error) => {
                eprintln!("MISTBRN event=purchase node={node} rank={rank} result={error}")
            }
        }
        result
    }

    /// Reveals hint `index` of research `code` (the notebook, op 7).
    pub(crate) fn reveal_hint(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        code: u16,
        index: u8,
    ) -> Result<TransactionSeq, String> {
        let research = research_def(code, &CATALOG).map_or("unknown", |def| def.key);
        let result = self.commit_save_only(host, player, |saved| prepare_hint(saved, code, index));
        match &result {
            Ok(seq) => eprintln!(
                "MISTBRN event=hint_revealed research={research} index={index} result=ok seq={}",
                seq.0
            ),
            Err(error) => eprintln!(
                "MISTBRN event=hint_revealed research={research} index={index} result={error}"
            ),
        }
        result
    }

    /// Claims milestone `bit` for `player` (P6) in its own host transaction: its pair is
    /// unlocked (not selected) and the player told. A second claim is refused by the core and
    /// changes nothing; both outcomes are logged.
    pub(crate) fn claim(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        bit: u8,
    ) -> Result<TransactionSeq, String> {
        let key = milestone_def(bit, &CATALOG).map_or("unknown", |def| def.key);
        let mut unlocks = None;
        let result = self.commit_save_only(host, player, |saved| {
            let (prepared, pair) = prepare_claim(saved, bit)?;
            unlocks = pair;
            Ok(prepared)
        });
        match &result {
            Ok(seq) => {
                eprintln!(
                    "MISTBRN event=milestone key={key} result=ok seq={} unlocks={}",
                    seq.0,
                    unlocks.map_or("none", pair_name)
                );
                tell(host, player, &format!("message.mistbrn.milestone.{key}"));
            }
            Err(error) => eprintln!("MISTBRN event=milestone key={key} result={error}"),
        }
        result
    }

    /// Concludes every research that is ready (the notebook used on a crafting table). Tells
    /// the player the outcome; returns how many were concluded.
    pub(crate) fn conclude(&mut self, host: &mut dyn ServerHost, player: PlayerRef) -> usize {
        let mut concluded = 0;
        let mut last_refusal = None;
        for def in CATALOG.research {
            match self.commit_save_only(host, player, |saved| prepare_conclude(saved, def.code)) {
                Ok(seq) => {
                    concluded += 1;
                    eprintln!(
                        "MISTBRN event=conclude research={} result=ok seq={}",
                        def.key, seq.0
                    );
                    tell(host, player, "message.mistbrn.research.discovered");
                }
                Err(error) => {
                    eprintln!("MISTBRN event=conclude research={} result={error}", def.key);
                    last_refusal = Some(error);
                }
            }
        }
        if concluded == 0 {
            let key = match last_refusal.as_deref() {
                Some("AlreadyConcluded") => "message.mistbrn.research.known",
                Some("PairLocked") => "message.mistbrn.research.locked",
                _ => "message.mistbrn.research.not_ready",
            };
            tell(host, player, key);
        }
        concluded
    }

    /// Records one server-observed event (practice or evidence, P5) for `player` at the
    /// level's game time. The save is marked for writing; nothing is told unless evidence makes
    /// a research ready.
    pub(crate) fn observe_event(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        kind: EventKind,
    ) {
        let uuid = player.uuid();
        let read: &dyn Host = host;
        let (Some([x, y, z]), Some(game_time)) = (
            player.entity().position(read),
            player
                .level(read)
                .and_then(|level| level.game_time(read))
                .and_then(|t| u64::try_from(t).ok()),
        ) else {
            return;
        };
        let Some(saved) = self.saved(uuid) else {
            return;
        };
        let snapshot = WorldSnapshot {
            tick: saved.state().last_tick.unwrap_or(Tick(0)),
            world_id: world_id(),
            actor: player_id(uuid),
            position: Vec3 { x, y, z },
            targets: Vec::new(),
            authority: Authority::Server,
            input: InputContext {
                gameplay_allowed: true,
            },
        };
        let label = match &kind {
            EventKind::PracticeAction { category } => format!("practice:{category}"),
            EventKind::ResearchEvidence { research, evidence } => {
                format!("evidence:{research}:{evidence}")
            }
            _ => "other".into(),
        };
        let Ok(event) =
            VerifiedEvent::from_server_observation(EventId::new(label), kind, &snapshot)
        else {
            return;
        };
        // Tried on a copy first: an observation that adds nothing (a spent window, evidence
        // already seen) leaves the save clean. One that does is written on this tick's flush:
        // a death followed by leaving from the death screen takes no later write.
        let mut probe = saved.clone();
        let observed = observe(&mut probe, &event, game_time);
        let result = match &observed {
            Ok(Observed::Nothing) => Ok(()),
            Ok(_) => self.change(uuid, true, |saved| {
                *saved = probe;
                Ok(())
            }),
            Err(error) => Err(format!("{error:?}")),
        };
        let observed = observed.unwrap_or(Observed::Nothing);
        if result.is_ok() && observed != Observed::Nothing {
            // Now, not at the tick's end: the player may die before it and leave from the
            // death screen, where no write reaches a dead player.
            self.flush(host, player);
        }
        // Evidence is announced only once it is written (a failed write retries later).
        let written = matches!(
            self.players.get(&uuid),
            Some(Slot::Active { dirty: false, .. })
        );
        match (result, observed) {
            (Err(error), _) => eprintln!("MISTBRN event=observe_refused error={error}"),
            (
                Ok(()),
                Observed::Practice {
                    category,
                    xp,
                    count,
                },
            ) => {
                let total = self
                    .saved(uuid)
                    .map_or(0, |s| s.progress().pair_xp(allomancy_core::Pair::IronSteel));
                let key = allomancy_core::practice_def(category, &CATALOG).map_or("?", |d| d.key);
                eprintln!(
                    "MISTBRN event=practice category={key} xp={xp} count={count} total_xp={total} game_time={game_time}"
                );
            }
            (
                Ok(()),
                Observed::Evidence {
                    research,
                    bit,
                    ready,
                },
            ) => {
                let key = research_def(research, &CATALOG).map_or("?", |d| d.key);
                eprintln!("MISTBRN event=evidence research={key} bit={bit} ready={ready}");
                if !written {
                    return;
                }
                let message = if ready {
                    "message.mistbrn.research.ready"
                } else {
                    "message.mistbrn.research.evidence"
                };
                tell(host, player, message);
            }
            (Ok(()), Observed::Nothing) => {}
        }
    }

    /// Test runs only (`/mistbrn-test xp`): `amount / 10` anchor practice actions, each in its
    /// own window far past the world's clock, so a later real action opens a fresh window.
    pub(crate) fn test_xp(&mut self, host: &mut dyn ServerHost, player: PlayerRef, amount: u32) {
        let uuid = player.uuid();
        let window = CATALOG.window_ticks;
        let mut granted = 0;
        let result = self.change(uuid, true, |saved| {
            for i in 0..u64::from(amount / 10) {
                let snapshot = WorldSnapshot {
                    tick: Tick(0),
                    world_id: world_id(),
                    actor: player_id(uuid),
                    position: Vec3::default(),
                    targets: Vec::new(),
                    authority: Authority::Server,
                    input: InputContext {
                        gameplay_allowed: true,
                    },
                };
                let event = VerifiedEvent::from_server_observation(
                    EventId::new("test_xp"),
                    EventKind::PracticeAction {
                        category: allomancy_core::PRACTICE_ANCHOR_MOVE,
                    },
                    &snapshot,
                )
                .map_err(|_| SaveError::InvalidIdentity)?;
                if let Observed::Practice { xp, .. } =
                    observe(saved, &event, (1 << 40) + i * window)?
                {
                    granted += xp;
                }
            }
            Ok(())
        });
        eprintln!(
            "MISTBRN event=test_xp granted={granted} result={}",
            result.err().unwrap_or_else(|| "ok".into())
        );
        self.flush(host, player);
    }

    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, epoch: u64, tick: u64) {
        if self.epoch != Some(epoch) {
            // A new world open: in-memory state never crosses world opens. The first epoch of
            // the process keeps players loaded at login, which belong to this world.
            if self.epoch.is_some() {
                self.players.clear();
                self.uses.clear();
            }
            self.epoch = Some(epoch);
            self.reports = 0;
        }
        let mut online = host.players().server_players();
        online.sort_by_key(|player| player.uuid());
        self.players
            .retain(|uuid, _| online.iter().any(|p| p.uuid() == *uuid));

        for player in online {
            self.ensure_loaded(host, player);
            let read: &dyn Host = host;
            let position = player.entity().position(read);
            let uuid = player.uuid();
            if self.reports < TICK_REPORTS && self.players.contains_key(&uuid) {
                let state = match self.players.get(&uuid) {
                    Some(Slot::Active { .. }) => "active",
                    _ => "blocked",
                };
                eprintln!("MISTBRN event=player_tick state={state}");
                self.reports += 1;
            }
            let (Some([x, y, z]), Some(Slot::Active { saved, .. })) =
                (position, self.players.get(&uuid))
            else {
                continue;
            };
            let snapshot = WorldSnapshot {
                tick: Tick(tick),
                world_id: world_id(),
                actor: player_id(uuid),
                position: Vec3 { x, y, z },
                targets: Vec::new(),
                authority: Authority::Server,
                input: InputContext {
                    gameplay_allowed: false,
                },
            };
            let burning = saved
                .state()
                .metals
                .iter()
                .any(|metal| metal.burn != BurnState::Off);
            if burning {
                let rules = saved.rules().clone();
                let mut exhausted = Vec::new();
                let result = self.change(uuid, false, |saved| {
                    update_player(saved, |state, accounting| {
                        advance_tick(state, &snapshot);
                        exhausted = consume_reserves(state, accounting, &rules, 1)?;
                        Ok(())
                    })
                });
                if let Err(error) = result {
                    eprintln!("MISTBRN event=consume_failed error={error}");
                }
                for metal in &exhausted {
                    eprintln!("MISTBRN event=exhausted metal={}", metal_name(*metal));
                }
                if !exhausted.is_empty() {
                    let _ = self.change(uuid, true, |_| Ok(()));
                }
            }
            if let Some(Slot::Active { since_write, .. }) = self.players.get_mut(&uuid) {
                *since_write = since_write.saturating_add(1);
            }
            self.flush_if(host, player, false);
        }
    }
}

fn active(saved: SavedPlayer) -> Slot {
    Slot::Active {
        saved,
        dirty: true,
        urgent: true,
        since_write: 0,
    }
}

pub(crate) fn metal_name(metal: Metal) -> &'static str {
    match metal {
        Metal::Iron => "iron",
        Metal::Steel => "steel",
        Metal::Tin => "tin",
        Metal::Pewter => "pewter",
        Metal::Zinc => "zinc",
        Metal::Brass => "brass",
        Metal::Copper => "copper",
        Metal::Bronze => "bronze",
    }
}

#[cfg(test)]
mod tests {
    use super::refusal_message;

    #[test]
    fn refusals_use_translation_keys_and_moved_vials_stay_silent() {
        assert_eq!(refusal_message("item_not_in_slot"), None);
        assert_eq!(
            refusal_message("InvalidReserves(CapacityExceeded(Iron))"),
            Some("message.mistbrn.vial.reserve_full")
        );
        assert_eq!(
            refusal_message("set_slot:Refused"),
            Some("message.mistbrn.vial.failed")
        );
    }
}
