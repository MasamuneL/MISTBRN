//! P1 bridge only: authoritative in-memory state, without powers or persistence.
#![forbid(unsafe_code)]

use allomancy_core::{
    Authority, Command, EffectKind, EffectStage, InputContext, PlayerId, PlayerState, Tick, Vec3,
    WorldId, WorldSnapshot, advance_tick, apply_command,
};
use horizon_client::mod_api::{Hooks, Host, NativeAuthority, ServerTickEvent, ServerTickPhase};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

#[derive(Default)]
struct Session {
    epoch: Option<u64>,
    players: BTreeMap<PlayerId, PlayerState>,
    reports: u8,
}

#[derive(Default)]
struct Allomancy {
    session: Mutex<Session>,
}

pub fn hooks() -> Box<dyn Hooks> {
    eprintln!("ALLOMANCY_CORE event=loaded version=0.1.0");
    Box::new(Allomancy::default())
}

impl Hooks for Allomancy {
    fn server_tick(&self, host: &mut dyn Host, event: &mut ServerTickEvent<'_>) {
        if !matches!(event.context.authority, NativeAuthority::IntegratedServer)
            || !matches!(
                event.phase,
                ServerTickPhase::ServerTickEnd {
                    world_running: true
                }
            )
        {
            return;
        }

        let mut players = host.server_players();
        players.sort_by_key(|player| player.uuid);
        let active: BTreeSet<_> = players
            .iter()
            .map(|player| PlayerId::new(format!("{:032x}", player.uuid)))
            .collect();
        let Ok(mut session) = self.session.lock() else {
            return;
        };
        let epoch = event.context.epoch.get();
        if session.epoch != Some(epoch) {
            session.epoch = Some(epoch);
            session.players.clear();
            session.reports = 0;
        }
        session.players.retain(|id, _| active.contains(id));

        // Epoch is only a transient P1 scope; never persist it as a world identity.
        let world = WorldId::new(format!("horizon-session:{epoch}"));
        for player in players {
            let actor = PlayerId::new(format!("{:032x}", player.uuid));
            let snapshot = WorldSnapshot {
                tick: Tick(event.context.tick.get()),
                world_id: world.clone(),
                actor: actor.clone(),
                position: Vec3 {
                    x: player.position[0],
                    y: player.position[1],
                    z: player.position[2],
                },
                targets: Vec::new(),
                authority: Authority::Server,
                // No client input is attested here. Only a safety shutdown is requested.
                input: InputContext {
                    gameplay_allowed: false,
                },
            };
            let (accepted, tick_recorded) = {
                let state = session
                    .players
                    .entry(actor.clone())
                    .or_insert_with(|| PlayerState::new(actor, world.clone()));
                let before = state.clone();
                let tick_effects = advance_tick(state, &snapshot);
                let accepted = apply_command(state, Command::Extinguish { metal: None }, &snapshot)
                    .is_ok_and(|effects| {
                        effects.iter().all(|effect| {
                            effect.stage == EffectStage::AcceptedInMemory
                                && matches!(&effect.kind, EffectKind::Signal(_))
                        })
                    })
                    && tick_effects.is_empty();
                if !accepted {
                    *state = before;
                }
                (accepted, accepted && state.last_tick == Some(snapshot.tick))
            };
            if session.reports < 3 {
                eprintln!(
                    "ALLOMANCY_CORE event=player_tick accepted={accepted} tick_recorded={tick_recorded}"
                );
                session.reports += 1;
            }
        }
    }
}
