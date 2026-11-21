//! Server-side session: loads each online player's save, runs the core every server tick,
//! persists changes and applies credits transactionally with the inventory.
use crate::save::{self, Blocked, Loaded};
use allomancy_core::{
    Authority, BurnState, InputContext, Metal, PlayerId, ReserveRules, SaveError, SavedPlayer,
    Tick, TransactionSeq, Vec3, WorldId, WorldSnapshot, advance_tick, commit_credit,
    consume_reserves, on_death, on_reconnect, prepare_credit, update_player,
};
use mod_api::{Host, HostTransaction, ItemKey, PlayerRef, ServerHost, Text};
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
}

fn tell(host: &mut dyn ServerHost, player: PlayerRef, line: &str) {
    let _ = player.tell(host, Text::literal(line.to_string()));
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
        if let Slot::Blocked(blocked) = &slot {
            eprintln!("MISTBRN event=save_blocked reason={}", blocked.reason());
            tell(
                host,
                player,
                "Alomancia: datos guardados ilegibles; no se modificarán.",
            );
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
            // Stale or dead player: keep it dirty and retry on a later tick.
            Err(error) => eprintln!("MISTBRN event=save_write_failed error={error}"),
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
        if self.change(uuid, true, on_death).is_ok() {
            eprintln!("MISTBRN event=death_cleared");
            if let Some(player) = host.player(uuid) {
                self.flush(host, player);
            }
        }
    }

    /// Stops all writes for this player for the rest of the session.
    pub(crate) fn block(&mut self, uuid: u128, reason: Blocked) {
        eprintln!("MISTBRN event=save_blocked reason={}", reason.reason());
        self.players.insert(uuid, Slot::Blocked(reason));
    }

    pub(crate) fn logged_out(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        if let Some(player) = host.player(uuid) {
            self.flush(host, player);
        }
        self.players.remove(&uuid);
    }

    /// Credits `amount_milli` of `metal` for one item taken from `slot`, as one transaction:
    /// inventory and save change together in the game, then the core commits. `fail_inside`
    /// injects a failure after both writes (tests).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn credit_from_slot(
        &mut self,
        host: &mut dyn ServerHost,
        player: PlayerRef,
        slot: u32,
        item: ItemKey,
        metal: Metal,
        amount_milli: i64,
        fail_inside: bool,
    ) -> Result<TransactionSeq, String> {
        let uuid = player.uuid();
        let Some(Slot::Active {
            saved,
            dirty,
            urgent,
            since_write,
        }) = self.players.get_mut(&uuid)
        else {
            return Err("not_active".into());
        };
        let prepared = prepare_credit(saved, metal, amount_milli).map_err(|e| format!("{e:?}"))?;
        host.transaction(|host, _tx| {
            let read: &dyn Host = host;
            let stack = player
                .inventory_slots(read)
                .and_then(|slots| slots.into_iter().find(|(s, _)| *s == slot))
                .map(|(_, stack)| stack)
                .filter(|stack| stack.item() == item)
                .ok_or("item_not_in_slot")?;
            player
                .set_inventory_slot(host, slot, stack.shrink(1))
                .map_err(|e| format!("set_slot:{e:?}"))?;
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
                Ok(seq)
            }
            Err(error) => {
                // The game already holds the candidate: writing the older in-memory state would
                // erase the credit (docs/persistence.md). Stop writing this player instead.
                self.block(uuid, Blocked::Decode(error));
                Err(format!("commit_after_host:{error:?}"))
            }
        }
    }

    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, epoch: u64, tick: u64) {
        if self.epoch != Some(epoch) {
            // A new world open: in-memory state never crosses world opens.
            self.epoch = Some(epoch);
            self.players.clear();
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
