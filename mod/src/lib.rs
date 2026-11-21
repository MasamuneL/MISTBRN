//! Allomancy for Horizon: the native adapter between the game and `allomancy-core`.
//!
//! The core decides; this crate observes the game, builds trusted snapshots and applies effects.
//! Nothing here is OS-specific: the same source builds for every host the pinned SDK supports.
#![deny(unsafe_code)]

mod bridge;
mod content;
mod save;
mod scenario;

use mod_api::{
    self, Authority as HostAuthority, BlockDropsEvent, ContainerLootFilledEvent, Flow, Hooks,
    HostSide, LevelHost, LivingEntityUseItemFinishEvent, PlayerCloneEvent, PlayerDiedEvent,
    PlayerLoggedInEvent, PlayerLoggedOutEvent, Registrar, RegistrationContext, ServerHost,
    ServerStoppedEvent, ServerTickEvent, ServerTickPhase,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) const MOD_VERSION: &str = "0.2.0";

struct Allomancy {
    session: Mutex<bridge::Session>,
    scenario: Mutex<scenario::Runner>,
    content: Mutex<content::Content>,
}

impl Default for Allomancy {
    fn default() -> Self {
        eprintln!("MISTBRN event=loaded version={MOD_VERSION}");
        Self {
            session: Mutex::new(bridge::Session::default()),
            scenario: Mutex::new(scenario::Runner::from_env()),
            content: Mutex::default(),
        }
    }
}

#[mod_api::horizon_mod]
impl Hooks for Allomancy {
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        self.content
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .register(registrar);
    }

    fn living_entity_use_item_finish(
        &self,
        side: HostSide<'_>,
        event: &mut LivingEntityUseItemFinishEvent<'_>,
    ) {
        let HostSide::Server(host) = side else { return };
        let drink = self
            .content
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .finish(host, event);
        if let Some(drink) = drink {
            // The finish hook will write its result back into the hand. Apply our
            // transactional inventory change after that write has finished.
            self.session().queue_drink(drink);
        }
    }

    fn block_drops(&self, level: &dyn LevelHost, event: &mut BlockDropsEvent<'_>) -> Flow {
        self.content
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .block_drops(level, event);
        Flow::Continue
    }

    fn container_loot_filled(&self, _: &dyn LevelHost, event: &mut ContainerLootFilledEvent<'_>) {
        self.content
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .chest_drops(event);
    }

    fn server_stopped(&self, _: &mut dyn ServerHost, _: &mut ServerStoppedEvent) {
        *self.session() = bridge::Session::default();
    }

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
        session.finish_drinks(host, runner.fail_vial());
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
