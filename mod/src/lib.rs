//! Allomancy for Horizon: the native adapter between the game and `allomancy-core`.
//!
//! The core decides; this crate observes the game, builds trusted snapshots and applies effects.
//! Nothing here is OS-specific: the same source builds for every host the pinned SDK supports.
#![deny(unsafe_code)]

mod bridge;
mod client;
mod content;
mod controls;
mod hud;
mod notebook;
mod p4;
mod save;
mod scenario;

use mod_api::command::{CommandArgumentValue, CommandSource};
use mod_api::{
    self, Authority as HostAuthority, BlockDropsEvent, ClientHost, ClientPlayerLoggingOutEvent,
    ClientTickPostEvent, ContainerLootFilledEvent, EntityTeleportEvent, Flow, GuiLayerDrawEvent,
    Hooks, HostSide, InteractionResult, ItemPickedUpPostEvent, LevelHost, LevelTickPostEvent,
    LivingEntityUseItemFinishEvent, PlayerChangedDimensionEvent, PlayerCloneEvent,
    PlayerContainerCloseEvent, PlayerContainerOpenEvent, PlayerDiedEvent, PlayerLoggedInEvent,
    PlayerLoggedOutEvent, RegisterCommandsEvent, RegisterGuiLayersEvent, Registrar,
    RegistrationContext, RightClickBlockEvent, RightClickItemEvent, RunModCommandEvent,
    ScreenKeyPressedPreEvent, ScreenKeyReleasedPreEvent, ScreenMouseButtonPressedPreEvent,
    ScreenRenderPostEvent, ServerHost, ServerStoppedEvent, ServerTickEvent, ServerTickPhase,
};
use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) const MOD_VERSION: &str = "0.2.0";

struct Allomancy {
    session: Mutex<bridge::Session>,
    scenario: Mutex<scenario::Runner>,
    content: Mutex<content::Content>,
    powers: Mutex<p4::Powers>,
    controls: Mutex<controls::Controls>,
}

impl Default for Allomancy {
    fn default() -> Self {
        eprintln!("MISTBRN event=loaded version={MOD_VERSION}");
        Self {
            session: Mutex::new(bridge::Session::default()),
            scenario: Mutex::new(scenario::Runner::from_env()),
            content: Mutex::default(),
            powers: Mutex::default(),
            controls: Mutex::default(),
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
        controls::register(registrar, client::hud_handler(), client::progress_handler());
        client::register_keys(registrar);
    }

    fn living_entity_use_item_finish(
        &self,
        side: HostSide<'_>,
        event: &mut LivingEntityUseItemFinishEvent<'_>,
    ) {
        let HostSide::Server(host) = side else { return };
        let finished = self
            .content
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .finish(host, event);
        if let Some(finished) = finished {
            // The finish hook will write its result back into the hand. Apply our
            // transactional inventory change after that write has finished.
            self.session().queue_use(finished);
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
        *self.powers() = p4::Powers::default();
        let mut controls = self.controls();
        controls.purge_all();
        *controls = controls::Controls::default();
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
        session.finish_uses(host, runner.fail_use());
        runner.tick(host, &mut session);
        let tick = event.context.tick.get();
        let mut powers = self.powers();
        let mut controls = self.controls();
        // Key requests first, so a metal put out this tick takes no force.
        controls.apply(host, &mut session, &mut powers, tick);
        powers.apply_prepared(host, &session, tick);
        // Practice and evidence seen this tick (P5), after the forces that caused them.
        for (uuid, kind) in powers.take_observations() {
            if let Some(player) = host.player(uuid) {
                session.observe_event(host, player, kind);
            }
        }
        controls.sync_hud(host, &session, tick);
        controls.sync_progress(host, &session, tick);
    }

    fn level_tick_post(&self, side: HostSide<'_>, event: &mut LevelTickPostEvent) {
        // Iron and steel act per level, while it is the level in place (see `p4::Powers`).
        let HostSide::Server(host) = side else { return };
        let hits = self
            .powers()
            .tick_level(host, &mut self.session(), event.level);
        // A lethal hit runs the death hooks, which take the session and power locks again.
        let shooters = p4::deal_hits(host, hits);
        for shooter in shooters {
            let player = host
                .players()
                .server_players()
                .into_iter()
                .find(|player| player.entity() == shooter);
            if let Some(player) = player {
                self.session()
                    .observe_event(host, player, p4::coin_hit_practice());
            }
        }
    }

    fn item_picked_up_post(&self, _: &mut dyn ServerHost, event: &mut ItemPickedUpPostEvent<'_>) {
        self.powers().picked_up(event.player, event.item.entity());
    }

    /// The notebook used on a crafting table concludes the research that is ready (P5).
    fn right_click_block(&self, side: HostSide<'_>, event: &mut RightClickBlockEvent<'_>) -> Flow {
        let HostSide::Server(host) = side else {
            return Flow::Continue;
        };
        let click = event.click;
        let read: &dyn mod_api::Host = host;
        // The main hand only, as the plan says (an off-hand notebook leaves the table alone).
        if click.hand != mod_api::Hand::Main
            || !click
                .stack
                .is_some_and(|stack| content::is_notebook(read, stack))
        {
            return Flow::Continue;
        }
        let Some(player) = host.player(click.player) else {
            return Flow::Continue;
        };
        let read: &dyn mod_api::Host = host;
        let table = player
            .level(read)
            .and_then(|level| level.block(read, click.pos))
            .is_some_and(|block| block.id(read) == "minecraft:crafting_table");
        let playing = matches!(
            player.game_mode(read),
            Some(mod_api::GameMode::Survival | mod_api::GameMode::Adventure)
        );
        if !table || !playing {
            return Flow::Continue;
        }
        self.session().conclude(host, player);
        *event.cancellation_result = InteractionResult::Success;
        Flow::Cancel
    }

    /// The notebook in the hand opens the notebook screen (client side; B does too).
    fn right_click_item(&self, side: HostSide<'_>, event: &mut RightClickItemEvent<'_>) -> Flow {
        let HostSide::Client(host) = side else {
            return Flow::Continue;
        };
        if !content::is_notebook(host, event.click.stack) {
            return Flow::Continue;
        }
        client::open_notebook(host);
        *event.cancellation_result = InteractionResult::Success;
        Flow::Cancel
    }

    fn register_commands(&self, _: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        // Test commands exist only in test runs (P10 checks a normal world has none).
        if scenario::commands_enabled() {
            event.commands.extend(
                p4::TEST_COMMANDS
                    .iter()
                    .map(|usage| mod_api::Command::op(usage, 2)),
            );
        }
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let command = event.command;
        if !scenario::commands_enabled()
            || command.literals.first().map(String::as_str) != Some("mistbrn-test")
        {
            return Flow::Continue;
        }
        let CommandSource::Player { player, .. } = event.source else {
            return Flow::Cancel;
        };
        let word = |name: &str| match command.argument(name) {
            Some(CommandArgumentValue::String(value)) => Some(value.as_str()),
            _ => None,
        };
        let milli = match command.argument("milli") {
            Some(CommandArgumentValue::Integer(value)) => Some(*value),
            _ => None,
        };
        if command.literals.get(1).map(String::as_str) == Some("flood") {
            if let Some(CommandArgumentValue::Integer(count)) = command.argument("count") {
                controls::flood(player.uuid(), u32::try_from(*count).unwrap_or(0));
            }
            return Flow::Cancel;
        }
        if command.literals.get(1).map(String::as_str) == Some("inject") {
            if let Some(CommandArgumentValue::String(op)) = command.argument("op") {
                controls::inject(player.uuid(), op);
            }
            return Flow::Cancel;
        }
        let integer = |name: &str| match command.argument(name) {
            Some(CommandArgumentValue::Integer(value)) => u32::try_from(*value).ok(),
            _ => None,
        };
        match command.literals.get(1).map(String::as_str) {
            Some("xp") => {
                self.session()
                    .test_xp(host, *player, integer("amount").unwrap_or(0));
                return Flow::Cancel;
            }
            Some("evidence") => {
                let bit = integer("bit")
                    .and_then(|b| u8::try_from(b).ok())
                    .unwrap_or(0);
                let kind = anchor_evidence(bit);
                let mut session = self.session();
                session.observe_event(host, *player, kind);
                session.flush(host, *player);
                return Flow::Cancel;
            }
            _ => {}
        }
        let test = p4::TestCommand {
            literals: &command.literals,
            metal: word("metal"),
            mode: word("mode"),
            milli,
        };
        p4::run_test_command(
            host,
            &mut self.session(),
            &mut self.powers(),
            *player,
            &test,
        );
        Flow::Cancel
    }

    fn entity_teleport(&self, host: &mut dyn ServerHost, event: &mut EntityTeleportEvent<'_>) {
        self.powers()
            .teleported(host, event.entity, event.from, event.to);
    }

    fn player_changed_dimension(
        &self,
        _: &mut dyn ServerHost,
        event: &mut PlayerChangedDimensionEvent<'_>,
    ) {
        self.powers()
            .changed_dimension(&mut self.session(), event.player);
        let mut controls = self.controls();
        controls.purge(event.player);
        controls.resend(event.player);
    }

    fn player_container_open(
        &self,
        _: &mut dyn ServerHost,
        event: &mut PlayerContainerOpenEvent<'_>,
    ) {
        self.controls().menu_opened(event.player);
    }

    fn player_container_close(
        &self,
        _: &mut dyn ServerHost,
        event: &mut PlayerContainerCloseEvent<'_>,
    ) {
        self.controls().menu_closed(event.player);
    }

    fn player_logged_in(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
        if let Some(player) = host.player(event.player) {
            self.session().ensure_loaded(host, player);
        }
        self.controls().resend(event.player);
    }

    fn player_logged_out(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedOutEvent<'_>) {
        self.session().logged_out(host, event.player);
        self.controls().logged_out(event.player);
    }

    fn player_died(&self, host: &mut dyn ServerHost, event: &mut PlayerDiedEvent<'_>) {
        self.controls().purge(event.player);
        self.powers().died(event.player);
        self.session().died(host, event.player);
    }

    fn player_clone(&self, host: &mut dyn ServerHost, event: &mut PlayerCloneEvent<'_>) {
        if event.was_death {
            self.session().respawned_after_death(host, event.player);
        }
        self.controls().resend(event.player);
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _: &mut ClientTickPostEvent) {
        client::tick(host);
    }

    fn register_gui_layers(&self, _: &RegistrationContext, event: &mut RegisterGuiLayersEvent<'_>) {
        client::gui_layers(event.layers);
    }

    fn gui_layer_draw(&self, host: &dyn ClientHost, event: &mut GuiLayerDrawEvent<'_>) {
        client::draw(host, event.layer, event.draw);
    }

    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        client::screen_draw(host, event.screen.mouse, event.draw);
    }

    fn screen_key_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        if client::notebook_key_pressed(host, event.key, event.repeat) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    fn screen_key_released_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenKeyReleasedPreEvent<'_>,
    ) -> Flow {
        if client::key_released(host, event.key) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        if client::mouse_pressed(host, event.screen.mouse, event.button) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    fn client_player_logging_out(
        &self,
        _: &mut dyn ClientHost,
        _: &mut ClientPlayerLoggingOutEvent,
    ) {
        client::logged_out();
    }
}

/// The anchor research's evidence `bit`, as the server would observe it (test runs).
fn anchor_evidence(bit: u8) -> allomancy_core::EventKind {
    allomancy_core::EventKind::ResearchEvidence {
        research: allomancy_core::RESEARCH_ANCHOR,
        evidence: bit,
    }
}

impl Allomancy {
    fn powers(&self) -> MutexGuard<'_, p4::Powers> {
        self.powers.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn controls(&self) -> MutexGuard<'_, controls::Controls> {
        self.controls.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn session(&self) -> MutexGuard<'_, bridge::Session> {
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
