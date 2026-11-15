//! Server storage bridge. Gameplay remains closed pending the native P2 gates.
#![forbid(unsafe_code)]
#[cfg(feature = "native-test")]
mod native_test;
pub mod save;
use horizon_client::mod_api::{
    Hooks, Host, NativeAuthority, PlayerChangedDimensionEvent, PlayerDiedEvent,
    PlayerLoggedInEvent, PlayerLoggedOutEvent, PlayerLoggingOutEvent, PlayerRespawnedEvent,
    PlayerSaveToFileEvent, ServerTickEvent, ServerTickPhase,
};
use std::sync::Arc;
struct Allomancy {
    backend: Arc<save::SaveBackend>,
    driver: Option<fn(&save::SaveBackend, &mut dyn Host)>,
}
pub fn hooks() -> Box<dyn Hooks> {
    #[cfg(feature = "native-test")]
    {
        return native_test::make_hooks();
    }
    #[cfg(not(feature = "native-test"))]
    hooks_without_native_test()
}
pub fn hooks_without_native_test() -> Box<dyn Hooks> {
    eprintln!("ALLOMANCY_SAVE event=loaded gameplay=false");
    Box::new(Allomancy {
        backend: Arc::new(save::SaveBackend::default()),
        driver: None,
    })
}
/// Explicit controller for a dedicated prepared test world; no automatic grants.
pub fn hooks_with_native_test(
    config: save::NativeTestConfig,
    driver: fn(&save::SaveBackend, &mut dyn Host),
) -> (Box<dyn Hooks>, Arc<save::SaveBackend>) {
    let backend = Arc::new(save::SaveBackend::for_native_test(config));
    (
        Box::new(Allomancy {
            backend: backend.clone(),
            driver: Some(driver),
        }),
        backend,
    )
}
impl Hooks for Allomancy {
    #[cfg(feature = "native-test")]
    fn world_lifecycle(
        &self,
        _host: &mut dyn Host,
        event: &mut horizon_client::mod_api::WorldLifecycleEvent<'_>,
    ) {
        eprintln!(
            "ALLOMANCY_NATIVE_TEST event=world_lifecycle phase={:?} epoch={} error_present={}",
            event.phase,
            event.context.epoch.get(),
            !event.error_text.is_empty()
        );
    }
    fn server_tick(&self, host: &mut dyn Host, event: &mut ServerTickEvent<'_>) {
        if matches!(event.context.authority, NativeAuthority::IntegratedServer)
            && matches!(
                event.phase,
                ServerTickPhase::ServerTickEnd {
                    world_running: true
                }
            )
        {
            self.backend
                .tick(host, event.context.epoch.get(), event.context.tick.get());
            if let Some(driver) = self.driver {
                self.backend.run_native_test_driver(host, driver);
            }
        }
    }
    fn player_logged_in(&self, host: &mut dyn Host, event: &mut PlayerLoggedInEvent<'_>) {
        self.backend.login(host, event.player);
    }
    fn player_died(&self, _host: &mut dyn Host, event: &mut PlayerDiedEvent<'_>) {
        self.backend.queue_death(event.player);
    }
    fn player_logging_out(&self, host: &mut dyn Host, event: &mut PlayerLoggingOutEvent<'_>) {
        self.backend.logging_out(host, event.player);
    }
    fn player_logged_out(&self, _host: &mut dyn Host, event: &mut PlayerLoggedOutEvent<'_>) {
        self.backend.logged_out(event.player);
    }
    fn player_respawned(&self, host: &mut dyn Host, event: &mut PlayerRespawnedEvent<'_>) {
        self.backend.respawned(host, event.player);
    }
    fn player_changed_dimension(
        &self,
        _host: &mut dyn Host,
        event: &mut PlayerChangedDimensionEvent<'_>,
    ) {
        self.backend.changed_dimension(event.player);
    }
    fn player_save_to_file(&self, _host: &mut dyn Host, _event: &mut PlayerSaveToFileEvent<'_>) {
        // Reentrant during inventory's FIFO drain: no Host, lock, write or commit.
        eprintln!("ALLOMANCY_SAVE event=save_observed durable=false");
    }
}
