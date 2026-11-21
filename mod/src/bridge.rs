//! Server-side session: loads each online player's save, runs the core every server tick,
//! persists changes and applies credits transactionally with the inventory.
use crate::content::Drink;
use crate::save::{self, Blocked, Loaded};
use allomancy_core::{
    Authority, BurnState, EMPTY_VIAL, InputContext, Metal, PlayerId, ReserveRules, SaveError,
    SavedPlayer, Tick, TransactionSeq, VIAL_MILLI, Vec3, WorldId, WorldSnapshot, advance_tick,
    commit_credit, consume_reserves, on_death, on_reconnect, prepare_credit, update_player,
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
    drinks: Vec<Drink>,
}

/// Chat line from the mod's translations (`assets/mistbrn/lang`), never a raw error code.
fn tell(host: &mut dyn ServerHost, player: PlayerRef, key: &str) {
    let _ = player.tell(host, Text::translate(key, Vec::new()));
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
        self.drinks.retain(|drink| drink.player.uuid() != uuid);
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
        self.drinks.retain(|drink| drink.player.uuid() != uuid);
        if let Some(player) = host.player(uuid) {
            self.flush(host, player);
        }
        self.players.remove(&uuid);
    }

    /// Queues one finished drink. The finish hook can be heard more than once for one use (on a
    /// server it may fire for every player of the world): one drink per player and slot per tick.
    pub(crate) fn queue_drink(&mut self, drink: Drink) {
        if !self
            .drinks
            .iter()
            .any(|queued| queued.player.uuid() == drink.player.uuid() && queued.slot == drink.slot)
        {
            self.drinks.push(drink);
        }
    }

    pub(crate) fn finish_drinks(&mut self, host: &mut dyn ServerHost, fail_inside: bool) {
        for drink in std::mem::take(&mut self.drinks) {
            let Some(player) = host.player(drink.player.uuid()) else {
                continue;
            };
            if player.entity() != drink.player.entity()
                || !matches!(
                    player.game_mode(host),
                    Some(GameMode::Survival | GameMode::Adventure)
                )
            {
                continue;
            }
            let metal = metal_name(drink.metal);
            match self.credit_from_slot(host, drink, VIAL_MILLI, fail_inside) {
                Ok(seq) => eprintln!("MISTBRN event=vial_credit metal={metal} seq={}", seq.0),
                Err(error) => {
                    eprintln!("MISTBRN event=vial_refused metal={metal} reason={error}");
                    if let Some(key) = refusal_message(&error) {
                        tell(host, player, key);
                    }
                }
            }
        }
    }

    /// Exchange one filled vial for its empty bottle and persist the credit together.
    /// `fail_inside` injects a failure after all writes (tests).
    pub(crate) fn credit_from_slot(
        &mut self,
        host: &mut dyn ServerHost,
        drink: Drink,
        amount_milli: i64,
        fail_inside: bool,
    ) -> Result<TransactionSeq, String> {
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
        let empty = Identifier::parse(EMPTY_VIAL)
            .ok()
            .and_then(|id| host.item_key(&id))
            .and_then(|key| ItemStack::new(key, 1))
            .ok_or("empty_vial_unavailable")?;
        host.transaction(|host, _tx| {
            let read: &dyn Host = host;
            let stack = player
                .inventory_slots(read)
                .and_then(|slots| slots.into_iter().find(|(s, _)| *s == slot))
                .map(|(_, stack)| stack)
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
                Ok(seq)
            }
            Err(error) => {
                // The game already holds the candidate: writing the older in-memory state would
                // erase the credit (docs/persistence.md). Stop writing this player instead.
                self.block(host, uuid, Blocked::Decode(error));
                Err(format!("commit_after_host:{error:?}"))
            }
        }
    }

    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, epoch: u64, tick: u64) {
        if self.epoch != Some(epoch) {
            // A new world open: in-memory state never crosses world opens. The first epoch of
            // the process keeps players loaded at login, which belong to this world.
            if self.epoch.is_some() {
                self.players.clear();
                self.drinks.clear();
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
