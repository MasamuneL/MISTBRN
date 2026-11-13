//! Disposable P0 integration probe; use only in a disposable world.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use horizon_client::mod_api::{
    AttachmentCodec, AttachmentError, AttachmentType, Holder, Hooks, Host, KeyEvent,
    NativeScreenKind, PlayerLoggedInEvent, ServerTickEvent, Tag, TickEvent,
};

static CLIENT_TICKS: AtomicUsize = AtomicUsize::new(0);
static SERVER_TICKS: AtomicUsize = AtomicUsize::new(0);
static LOGIN_LOGS: AtomicUsize = AtomicUsize::new(0);
static PUSHES: AtomicUsize = AtomicUsize::new(0);
static HELD: AtomicBool = AtomicBool::new(false);

static COUNTER: AttachmentType<i32> = AttachmentType {
    namespace: "allomancy_probe", name: "login_count", default: || 0,
    codec: Some(AttachmentCodec {
        encode: |n| Ok([("count".into(), Tag::Int(*n))].into()),
        decode: |c| match c.get(&"count".into()) {
            Some(Tag::Int(n)) => Ok(*n),
            _ => Err(AttachmentError::Decode),
        },
    }),
    copy_on_death: false, sync: false,
};

pub fn hooks() -> Box<dyn Hooks> {
    eprintln!("ALLOMANCY_PROBE event=loaded version=0.0.1");
    Box::new(Probe)
}
struct Probe;
impl Hooks for Probe {
    fn tick(&self, host: &mut dyn Host, _: &mut TickEvent) {
        if host.window() && CLIENT_TICKS.fetch_add(1, Ordering::Relaxed) < 3 {
            eprintln!("ALLOMANCY_PROBE event=client_tick");
        }
    }
    fn server_tick(&self, _: &mut dyn Host, _: &mut ServerTickEvent<'_>) {
        if SERVER_TICKS.fetch_add(1, Ordering::Relaxed) < 3 {
            eprintln!("ALLOMANCY_PROBE event=server_tick");
        }
    }
    fn key(&self, host: &mut dyn Host, event: &mut KeyEvent<'_>) {
        if event.key != host.mod_key("allomancy_probe", "impulse", "key.keyboard.z") { return; }
        if !event.pressed { HELD.store(false, Ordering::Relaxed); return; }
        if HELD.swap(true, Ordering::Relaxed) { return; }
        if !host.client_state().is_some_and(|s| s.focused && s.logged_in && s.screen == NativeScreenKind::Playing) { return; }
        // A process-wide cap bounds this probe's added impulses, not total velocity.
        if PUSHES.fetch_add(1, Ordering::Relaxed) < 3 {
            host.push_player([0.0, 0.05, 0.0]);
            eprintln!("ALLOMANCY_PROBE event=impulse");
        }
    }
    fn player_logged_in(&self, host: &mut dyn Host, event: &mut PlayerLoggedInEvent<'_>) {
        let result = COUNTER.get(host, Holder::Player(event.player)).and_then(|old| {
            COUNTER.set(host, Holder::Player(event.player), old.saturating_add(1))?;
            Ok(old)
        });
        if LOGIN_LOGS.fetch_add(1, Ordering::Relaxed) < 10 {
            match result {
                Ok(old) => eprintln!("ALLOMANCY_PROBE event=login previous_count={old}"),
                Err(error) => eprintln!("ALLOMANCY_PROBE event=attachment_error error={error:?}"),
            }
        }
    }
}
