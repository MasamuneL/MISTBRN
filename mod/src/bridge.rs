//! Server-tick bridge: builds trusted snapshots for each online player and runs the core.
//!
//! P0b scope: in-memory state only. Persistence arrives in P2 (`save.rs`).
use allomancy_core::{
    Authority, Command, EffectKind, EffectStage, InputContext, PlayerId, PlayerState, Tick, Vec3,
    WorldId, WorldSnapshot, advance_tick, apply_command,
};
use mod_api::{Host, ServerHost};
use std::collections::{BTreeMap, BTreeSet};

/// Per-player isolation between worlds comes from the game's per-world player files, so the
/// core sees one constant world identity (see docs/contracts.md).
pub(crate) const WORLD_ID: &str = "local";
const TICK_REPORTS: u8 = 3;

#[derive(Default)]
pub(crate) struct Session {
    epoch: Option<u64>,
    players: BTreeMap<PlayerId, PlayerState>,
    reports: u8,
}

pub(crate) fn player_id(uuid: u128) -> PlayerId {
    PlayerId::new(format!("{uuid:032x}"))
}

impl Session {
    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, epoch: u64, tick: u64) {
        if self.epoch != Some(epoch) {
            // A new world open: in-memory state never crosses world opens.
            self.epoch = Some(epoch);
            self.players.clear();
            self.reports = 0;
        }
        let mut online = host.players().server_players();
        online.sort_by_key(|player| player.uuid());
        let active: BTreeSet<_> = online.iter().map(|p| player_id(p.uuid())).collect();
        self.players.retain(|id, _| active.contains(id));

        let world = WorldId::new(WORLD_ID);
        for player in online {
            let read: &dyn Host = host;
            let Some([x, y, z]) = player.entity().position(read) else {
                continue;
            };
            let actor = player_id(player.uuid());
            let snapshot = WorldSnapshot {
                tick: Tick(tick),
                world_id: world.clone(),
                actor: actor.clone(),
                position: Vec3 { x, y, z },
                targets: Vec::new(),
                authority: Authority::Server,
                // No client input is attested in P0b; only the safety shutdown is requested.
                input: InputContext {
                    gameplay_allowed: false,
                },
            };
            let state = self
                .players
                .entry(actor.clone())
                .or_insert_with(|| PlayerState::new(actor, world.clone()));
            let before = state.clone();
            let tick_effects = advance_tick(state, &snapshot);
            let accepted = tick_effects.is_empty()
                && apply_command(state, Command::Extinguish { metal: None }, &snapshot).is_ok_and(
                    |effects| {
                        effects.iter().all(|effect| {
                            effect.stage == EffectStage::AcceptedInMemory
                                && matches!(effect.kind, EffectKind::Signal(_))
                        })
                    },
                );
            if !accepted {
                *state = before;
            }
            let recorded = accepted && state.last_tick == Some(snapshot.tick);
            if self.reports < TICK_REPORTS {
                eprintln!("MISTBRN event=player_tick accepted={accepted} tick_recorded={recorded}");
                self.reports += 1;
            }
        }
    }
}
