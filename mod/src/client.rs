//! Keys and the reserve HUD (P7a), client side.
//!
//! Five rebindable keys (Options > Controls > Masamune's MISTBRN): V next pair, Z / X toggle
//! the pair's first / second metal, R toggle flaring, G all off. A key sends one
//! `mistbrn:control` request per press, and only while playing with nothing over the world
//! and the window focused: the chat, the inventory and every other screen block it. Holding a
//! key does not repeat it. The server decides; the HUD only draws what the server last sent.
use crate::controls::{CONTROL_CHANNEL, pair_name};
use crate::{bridge::metal_name, scenario};
use allomancy_core::{
    BurnState, ControlRequest, HudSnapshot, Metal, Pair, Side, decode_hud, encode_control,
    pair_metals,
};
use mod_api::{
    ClientHost, DrawContext, GuiLayer, GuiLayerPlacement, IPayloadContext, IPayloadHandler,
    KeyConflictContext, KeyRegistration, PayloadBytes, PayloadHandleError, PayloadId, Registrar,
    Text,
};
use std::sync::{Arc, Mutex, PoisonError};

const MOD_ID: &str = "mistbrn";
pub(crate) const HUD_LAYER: &str = "mistbrn:reserves";

/// Read in this order each tick. V comes last: a metal key pressed in the same tick as V
/// acts on the pair the player saw when pressing it.
const KEYS: [(&str, &str, ControlRequest); 5] = [
    (
        "toggle_first",
        "key.keyboard.z",
        ControlRequest::Toggle(Side::First),
    ),
    (
        "toggle_second",
        "key.keyboard.x",
        ControlRequest::Toggle(Side::Second),
    ),
    ("flare", "key.keyboard.r", ControlRequest::ToggleFlare),
    ("off", "key.keyboard.g", ControlRequest::Off),
    ("cycle_pair", "key.keyboard.v", ControlRequest::CyclePair),
];
/// Test runs only (`MISTBRN_TEST_SCENARIO=p4`): sends malformed requests and a burst (K7).
const FUZZ_KEY: (&str, &str) = ("test_fuzz", "key.keyboard.k");
/// Clicks taken per key per tick at most; the rest wait (a held key auto-repeats).
const CLICKS_PER_TICK: u32 = 64;

struct ClientState {
    was_down: [bool; KEYS.len() + 1],
    hud: Option<HudSnapshot>,
}

static STATE: Mutex<ClientState> = Mutex::new(ClientState {
    was_down: [false; KEYS.len() + 1],
    hud: None,
});

fn state() -> std::sync::MutexGuard<'static, ClientState> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

struct HudInbox;

impl IPayloadHandler for HudInbox {
    fn handle(
        &self,
        payload: PayloadBytes,
        _: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        match decode_hud(&payload.0) {
            Ok(hud) => {
                let [a, b] = pair_metals(hud.pair);
                let show = |metal: Metal| {
                    let entry = hud.metals[metal.index()];
                    format!(
                        "{}={}/{}",
                        metal_name(metal),
                        entry.reserve_milli,
                        burn_name(entry.burn)
                    )
                };
                eprintln!(
                    "MISTBRN hud pair={} {} {}",
                    pair_name(hud.pair),
                    show(a),
                    show(b)
                );
                state().hud = Some(hud);
            }
            // The server is trusted, but a bad snapshot is dropped, not drawn.
            Err(error) => eprintln!("MISTBRN event=hud_rejected error={error:?}"),
        }
        Ok(())
    }
}

fn burn_name(burn: BurnState) -> &'static str {
    match burn {
        BurnState::Off => "off",
        BurnState::Burning => "burning",
        BurnState::Flared => "flared",
    }
}

pub(crate) fn hud_handler() -> Arc<dyn IPayloadHandler> {
    Arc::new(HudInbox)
}

pub(crate) fn register_keys(registrar: &mut dyn Registrar) {
    let fuzz = scenario::commands_enabled().then_some(FUZZ_KEY);
    for (id, key) in KEYS.iter().map(|(id, key, _)| (*id, *key)).chain(fuzz) {
        registrar.register_key_mapping(
            KeyRegistration::new(id, key)
                .label(&format!("key.mistbrn.{id}"))
                .category("key.category.mistbrn.allomancy")
                .context(KeyConflictContext::InGame),
        );
    }
}

pub(crate) fn gui_layers(layers: &mut Vec<GuiLayer>) {
    layers.push(GuiLayer {
        id: HUD_LAYER.into(),
        placement: GuiLayerPlacement::AboveAll,
    });
}

/// True once for each new press: clicks were taken and the key was not already held last tick
/// (auto-repeat adds clicks while it stays down).
fn pressed(host: &mut dyn ClientHost, index: usize, id: &str, was_down: &mut [bool]) -> bool {
    let mut clicks = 0;
    while clicks < CLICKS_PER_TICK && host.consume_click(MOD_ID, id) {
        clicks += 1;
    }
    let fresh = clicks > 0 && !was_down[index];
    was_down[index] = host.is_down(MOD_ID, id);
    fresh
}

fn send(host: &mut dyn ClientHost, bytes: Vec<u8>) {
    let channel = PayloadId(CONTROL_CHANNEL.into());
    if let Err(error) = host.send_payload_to_server(&channel, PayloadBytes(bytes)) {
        eprintln!("MISTBRN event=key_send_failed error={error:?}");
    }
}

pub(crate) fn tick(host: &mut dyn ClientHost) {
    let allowed = host.playing() && host.window_focused();
    let mut state = state();
    let mut sends = Vec::new();
    for (index, (id, _, request)) in KEYS.iter().enumerate() {
        if pressed(host, index, id, &mut state.was_down) {
            if allowed {
                eprintln!("MISTBRN event=key key={id}");
                sends.push(encode_control(*request).to_vec());
            } else {
                eprintln!("MISTBRN event=key key={id} result=blocked");
            }
        }
    }
    if scenario::commands_enabled()
        && pressed(host, KEYS.len(), FUZZ_KEY.0, &mut state.was_down)
        && allowed
    {
        eprintln!("MISTBRN event=key key={}", FUZZ_KEY.0);
        sends.extend(fuzz_requests());
    }
    drop(state);
    for bytes in sends {
        send(host, bytes);
    }
}

/// K7: one of each malformed kind (length, version, op, arg), then a burst of valid offs.
fn fuzz_requests() -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![
        vec![],
        vec![1, 4],
        vec![1, 4, 0, 0],
        vec![2, 4, 0],
        vec![1, 9, 0],
        vec![1, 2, 2],
    ];
    out.extend(std::iter::repeat_n(
        encode_control(ControlRequest::Off).to_vec(),
        20,
    ));
    out
}

pub(crate) fn logged_out() {
    let mut state = state();
    state.hud = None;
    state.was_down = [false; KEYS.len() + 1];
}

fn tr(host: &dyn ClientHost, key: &str) -> String {
    host.text_string(&Text::translate(key, Vec::new()))
        .unwrap_or_else(|_| key.into())
}

const PANEL: u32 = 0x9000_0000;
const TEXT: u32 = 0xFFE0_E0E0;
const DIM: u32 = 0xFF90_9090;
const BAR_BACK: u32 = 0xFF30_3030;
const BAR_BURNING: u32 = 0xFFC8_A050;
const BAR_FLARED: u32 = 0xFFFF_7030;
const BAR_OFF: u32 = 0xFF80_8080;
const BAR_W: f32 = 60.0;
const LINE: f32 = 10.0;
/// Wide enough for a pair's two rows (name, bar and state).
const PANEL_MIN_W: f32 = 150.0;

/// The reserve panel, bottom left: the selected pair with a bar per metal and its state, then
/// the other unlocked metals in one line. Nothing while no metal is unlocked.
pub(crate) fn draw(host: &dyn ClientHost, layer: &str, draw: &mut dyn DrawContext) {
    if layer != HUD_LAYER {
        return;
    }
    let Some(hud) = state().hud else {
        return;
    };
    if !hud.metals.iter().any(|metal| metal.unlocked) {
        return;
    }
    let [_, height] = draw.size();
    let metals = pair_metals(hud.pair);
    let others: Vec<String> = Metal::ALL
        .into_iter()
        .filter(|metal| !metals.contains(metal) && hud.metals[metal.index()].unlocked)
        .map(|metal| {
            let entry = hud.metals[metal.index()];
            format!(
                "{} {}%",
                tr(host, &format!("hud.mistbrn.metal.{}", metal_name(metal))),
                percent(entry.reserve_milli, entry.capacity_milli)
            )
        })
        .collect();
    let lines = if others.is_empty() { 3.0 } else { 4.0 };
    let (x, top) = (4.0, height - 4.0 - lines * LINE - 4.0);
    let others_line = others.join("  ");
    let width = PANEL_MIN_W.max(draw.text_width(&others_line) + 4.0);
    draw.fill([x - 2.0, top - 2.0, width, lines * LINE + 4.0], PANEL);
    let mut title = tr(host, &format!("hud.mistbrn.pair.{}", pair_name(hud.pair)));
    if hud.pair != Pair::IronSteel {
        title = format!("{title} · {}", tr(host, "hud.mistbrn.unavailable"));
    }
    draw.text(&title, [x, top], TEXT, true);
    for (row, metal) in metals.into_iter().enumerate() {
        let y = top + LINE * (row as f32 + 1.0);
        let entry = hud.metals[metal.index()];
        let name = tr(host, &format!("hud.mistbrn.metal.{}", metal_name(metal)));
        draw.text(&name, [x, y], if entry.unlocked { TEXT } else { DIM }, true);
        let bar_x = x + 44.0;
        if !entry.unlocked {
            draw.text(&tr(host, "hud.mistbrn.locked"), [bar_x, y], DIM, true);
            continue;
        }
        draw.fill([bar_x, y + 2.0, BAR_W, 4.0], BAR_BACK);
        let filled = BAR_W * percent(entry.reserve_milli, entry.capacity_milli) as f32 / 100.0;
        let color = match entry.burn {
            BurnState::Off => BAR_OFF,
            BurnState::Burning => BAR_BURNING,
            BurnState::Flared => BAR_FLARED,
        };
        draw.fill([bar_x, y + 2.0, filled, 4.0], color);
        let label = match entry.burn {
            BurnState::Off => None,
            BurnState::Burning => Some("hud.mistbrn.burning"),
            BurnState::Flared => Some("hud.mistbrn.flared"),
        };
        if let Some(label) = label {
            draw.text(&tr(host, label), [bar_x + BAR_W + 4.0, y], color, true);
        }
    }
    if !others.is_empty() {
        draw.text(&others_line, [x, top + LINE * 3.0], DIM, true);
    }
}

fn percent(reserve: u32, capacity: u32) -> u32 {
    if capacity == 0 {
        return 0;
    }
    u32::try_from(u64::from(reserve) * 100 / u64::from(capacity)).unwrap_or(100)
}
