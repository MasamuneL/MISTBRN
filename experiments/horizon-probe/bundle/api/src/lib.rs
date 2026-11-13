//! Disposable P0 integration probe; use only in a disposable world.
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use horizon_client::mod_api::{
    AttachmentCodec, AttachmentError, AttachmentType, Holder, Hooks, Host, KeyEvent,
    NativeScreenKind, PlayerLoggedInEvent, ServerTickEvent, Tag, TickEvent, HudEvent,
    NetFromClientEvent, NetFromServerEvent, UseItemEvent, UseItemOnEvent,
    LivingDeathEvent, LootDropsEvent, Flow,
};

static CLIENT_TICKS: AtomicUsize = AtomicUsize::new(0);
static SERVER_TICKS: AtomicUsize = AtomicUsize::new(0);
static LOGIN_LOGS: AtomicUsize = AtomicUsize::new(0);
static PUSHES: AtomicUsize = AtomicUsize::new(0);
static HELD: AtomicBool = AtomicBool::new(false);
static EVENTS: AtomicUsize = AtomicUsize::new(0);
static CLIENT_MESSAGES: AtomicUsize = AtomicUsize::new(0);
static SERVER_MESSAGES: AtomicUsize = AtomicUsize::new(0);
const CHANNEL: &str = "allomancy_probe:probe";
const PING: &[u8] = &[1, 1];
const ACK: &[u8] = &[1, 2];

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
fn update_inline_storage(host: &mut dyn Host, player: u128) -> Result<(i32, bool), &'static str> {
    let mut data = host.player_persistent_data(player).ok_or("unavailable")?;
    let Tag::Compound(root) = &mut data else { return Err("root_type"); };
    let key = "allomancy_probe".into();
    let namespace = root.entry(key).or_insert_with(|| Tag::Compound([
        ("schema".into(), Tag::Int(1)),
        ("login_count".into(), Tag::Int(0)),
    ].into()));
    let Tag::Compound(fields) = namespace else { return Err("namespace_type"); };
    if !matches!(fields.get(&"schema".into()), Some(Tag::Int(1))) { return Err("schema"); }
    let previous = match fields.get(&"login_count".into()) {
        Some(Tag::Int(n)) if *n >= 0 => *n,
        _ => return Err("counter"),
    };
    fields.insert("login_count".into(), Tag::Int(previous.saturating_add(1)));
    Ok((previous, host.set_player_persistent_data(player, data)))
}
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
            host.push_player([0.0, 0.30, 0.0]);
            eprintln!("ALLOMANCY_PROBE event=impulse");
        }
    }
    fn hud(&self, host: &mut dyn Host, event: &mut HudEvent<'_>) {
        host.fill(event.batch, [8.0, 8.0, 10.0, 10.0], 0xFF00_FFFF);
        host.fill(event.batch, [10.0, 10.0, 6.0, 6.0], 0xFF00_0000);
    }
    fn net_from_server(&self, host: &mut dyn Host, event: &mut NetFromServerEvent<'_>) {
        if event.channel != CHANNEL || CLIENT_MESSAGES.fetch_add(1, Ordering::Relaxed) >= 10 { return; }
        let valid = event.data == PING;
        let sent = valid && host.net_send_to_server(CHANNEL, ACK).is_ok();
        eprintln!("ALLOMANCY_PROBE event=ping valid={valid} ack_sent={sent}");
    }
    fn net_from_client(&self, _: &mut dyn Host, event: &mut NetFromClientEvent<'_>) {
        if event.channel != CHANNEL || SERVER_MESSAGES.fetch_add(1, Ordering::Relaxed) >= 10 { return; }
        eprintln!("ALLOMANCY_PROBE event=ack valid={}", event.data == ACK);
    }
    fn use_item(&self, _: &mut dyn Host, event: &mut UseItemEvent<'_>) -> Flow {
        if EVENTS.fetch_add(1, Ordering::Relaxed) < 10 { eprintln!("ALLOMANCY_PROBE event=use_item count={}", event.stack.count); }
        Flow::Continue
    }
    fn use_item_on(&self, _: &mut dyn Host, event: &mut UseItemOnEvent<'_>) -> Flow {
        if EVENTS.fetch_add(1, Ordering::Relaxed) < 10 { eprintln!("ALLOMANCY_PROBE event=use_item_on count={}", event.stack.count); }
        Flow::Continue
    }
    fn living_death(&self, _: &mut dyn Host, _: &mut LivingDeathEvent<'_>) -> Flow {
        if EVENTS.fetch_add(1, Ordering::Relaxed) < 10 { eprintln!("ALLOMANCY_PROBE event=living_death"); }
        Flow::Continue
    }
    fn loot_drops(&self, _: &mut dyn Host, event: &mut LootDropsEvent<'_>) {
        if EVENTS.fetch_add(1, Ordering::Relaxed) < 10 { eprintln!("ALLOMANCY_PROBE event=loot_drops count={}", event.drops.len()); }
    }
    fn player_logged_in(&self, host: &mut dyn Host, event: &mut PlayerLoggedInEvent<'_>) {
        let inline = update_inline_storage(host, event.player);
        if LOGIN_LOGS.load(Ordering::Relaxed) < 10 {
            match inline {
                Ok((previous, write_ok)) => eprintln!("ALLOMANCY_PROBE event=inline_storage previous_count={previous} write_ok={write_ok}"),
                Err(reason) => eprintln!("ALLOMANCY_PROBE event=inline_storage_rejected reason={reason}"),
            }
            let sent = host.net_send_to_player(event.player, CHANNEL, PING).is_ok();
            let sound = host.play_sound_to(event.player, "minecraft:entity.experience_orb.pickup", "player", None, 1.0, 1.0);
            let inventory = host.player_stacks(event.player).map(|stacks| (stacks.len(), stacks.iter().flatten().count()));
            eprintln!("ALLOMANCY_PROBE event=login_capabilities ping_sent={sent} sound_queued={sound} inventory_slots_occupied={inventory:?}");
        }
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
