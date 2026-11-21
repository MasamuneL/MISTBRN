//! Allomancy for Horizon: the native adapter between the game and `allomancy-core`.
//!
//! The core decides; this crate observes the game, builds trusted snapshots and applies effects.
//! Nothing here is OS-specific: the same source builds for every host the pinned SDK supports.
#![deny(unsafe_code)]

mod bridge;
mod save;
mod scenario;

use mod_api::{
    self, Authority as HostAuthority, Hooks, PlayerCloneEvent, PlayerDiedEvent,
    PlayerLoggedInEvent, PlayerLoggedOutEvent, ServerHost, ServerTickEvent, ServerTickPhase,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) const MOD_VERSION: &str = "0.2.0";

struct Allomancy {
    session: Mutex<bridge::Session>,
    scenario: Mutex<scenario::Runner>,
}

impl Default for Allomancy {
    fn default() -> Self {
        eprintln!("MISTBRN event=loaded version={MOD_VERSION}");
        Self {
            session: Mutex::new(bridge::Session::default()),
            scenario: Mutex::new(scenario::Runner::from_env()),
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Allomancy {
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        // Integrated (single player / LAN) and dedicated servers are both authoritative.
        if !matches!(
            event.context.authority,
            HostAuthority::IntegratedServer | HostAuthority::DedicatedServer
        ) || !matches!(
            event.phase,
            ServerTickPhase::ServerTickEnd {
                world_running: true
            }
        ) {
            return;
        }
        // A panic in an earlier tick must not silently stop every later one: keep the data.
        let mut session = self.session();
        session.tick(host, event.context.epoch.get(), event.context.tick.get());
        let mut runner = self.scenario.lock().unwrap_or_else(PoisonError::into_inner);
        runner.tick(host, &mut session);
    }

    fn player_logged_in(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
        if let Some(player) = host.player(event.player) {
            self.session().ensure_loaded(host, player);
        }
    }

    fn player_logged_out(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedOutEvent<'_>) {
        self.session().logged_out(host, event.player);
    }

    fn player_died(&self, host: &mut dyn ServerHost, event: &mut PlayerDiedEvent<'_>) {
        self.session().died(host, event.player);
    }

    fn player_clone(&self, host: &mut dyn ServerHost, event: &mut PlayerCloneEvent<'_>) {
        if event.was_death {
            self.session().respawned_after_death(host, event.player);
        }
    }
}

impl Allomancy {
    fn session(&self) -> MutexGuard<'_, bridge::Session> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
