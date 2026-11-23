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
const LINE: f32 = 11.0;
/// Wide enough for a pair's two rows (gem, name, framed bar, flame and state).
const PANEL_MIN_W: f32 = 166.0;
/// Half of vanilla's boss bar, centred at the top.
const BOSS_BAR_HALF: f32 = 91.0;
/// Padding inside the panel frame.
const PAD: f32 = 6.0;
/// Where a row's bar frame starts, from the row's left edge (gem 6 + gap + name column).
const BAR_COLUMN: f32 = 46.0;

/// The HUD sprite sheet (`assets/mistbrn/textures/gui/hud.png`, 128x32), generated by
/// `tools/gen-hud.py`: regions are `[x, y, w, h]` in the sheet's pixels.
const SHEET: &str = "mistbrn:gui/hud";
/// 16x16 nine-slice panel with 5 px corners.
const SPRITE_PANEL: [f32; 2] = [0.0, 0.0];
const PANEL_CORNER: f32 = 5.0;
const PANEL_SIZE: f32 = 16.0;
/// 64x8 bar frame whose 60x4 track starts at (2, 2).
const SPRITE_FRAME: [f32; 4] = [16.0, 0.0, 64.0, 8.0];
const SPRITE_FILL_BURNING: [f32; 2] = [16.0, 8.0];
const SPRITE_FILL_FLARED: [f32; 2] = [16.0, 12.0];
const SPRITE_FILL_OFF: [f32; 2] = [16.0, 16.0];
const SPRITE_GEM: [f32; 4] = [80.0, 0.0, 5.0, 6.0];
const SPRITE_LOCK: [f32; 4] = [88.0, 0.0, 7.0, 8.0];
const SPRITE_FLAME: [f32; 4] = [96.0, 0.0, 7.0, 8.0];
const SPRITE_FLAME_FLARED: [f32; 4] = [104.0, 0.0, 7.0, 8.0];
/// 6x6 metal gems in `Metal::ALL` order, 8 px apart.
const SPRITE_METAL_Y: f32 = 24.0;

/// The reserve panel, top left (clear of the chat, the hotbar and the bars above it, and
/// narrower than the space left of a boss bar): the selected pair with a framed bar per metal
/// and its state, then the other unlocked metals in one line. Nothing while no metal is
/// unlocked. Drawn from the sprite sheet; flat rectangles if the sheet is missing (a resource
/// pack that removes it).
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
    let [gui_width, _] = draw.size();
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
    let others_line = others.join("  ");
    let mut title = tr(host, &format!("hud.mistbrn.pair.{}", pair_name(hud.pair)));
    if hud.pair != Pair::IronSteel {
        title = format!("{title} · {}", tr(host, "hud.mistbrn.unavailable"));
    }
    let width = PANEL_MIN_W
        .max(draw.text_width(&others_line) + 2.0 * PAD)
        .max(draw.text_width(&title) + 2.0 * PAD)
        .min((gui_width / 2.0 - BOSS_BAR_HALF - 6.0).max(PANEL_MIN_W));
    let panel = [2.0, 2.0, width, lines * LINE + 2.0 * PAD - 2.0];
    let textured = nine_slice(draw, panel);
    if !textured {
        draw.fill(panel, PANEL);
    }
    let (x, top) = (panel[0] + PAD, panel[1] + PAD);
    draw.text(&title, [x, top], BRASS_TEXT, true);
    for (row, metal) in metals.into_iter().enumerate() {
        let y = top + LINE * (row as f32 + 1.0);
        let entry = hud.metals[metal.index()];
        let gem = [8.0 * metal.index() as f32, SPRITE_METAL_Y, 6.0, 6.0];
        let name_x = if textured && draw.sprite(SHEET, gem, [x, y + 1.0, 6.0, 6.0], None) {
            x + 8.0
        } else {
            x
        };
        let name = tr(host, &format!("hud.mistbrn.metal.{}", metal_name(metal)));
        draw.text(
            &name,
            [name_x, y],
            if entry.unlocked { TEXT } else { DIM },
            true,
        );
        let bar_x = x + BAR_COLUMN;
        if !entry.unlocked {
            let lock = [bar_x, y, SPRITE_LOCK[2], SPRITE_LOCK[3]];
            let text_x = if textured && draw.sprite(SHEET, SPRITE_LOCK, lock, None) {
                bar_x + 10.0
            } else {
                bar_x
            };
            draw.text(&tr(host, "hud.mistbrn.locked"), [text_x, y], DIM, true);
            continue;
        }
        let share = percent(entry.reserve_milli, entry.capacity_milli) as f32 / 100.0;
        let color = match entry.burn {
            BurnState::Off => BAR_OFF,
            BurnState::Burning => BAR_BURNING,
            BurnState::Flared => BAR_FLARED,
        };
        let end = if textured {
            framed_bar(draw, [bar_x, y], share, entry.burn)
        } else {
            draw.fill([bar_x, y + 2.0, BAR_W, 4.0], BAR_BACK);
            draw.fill([bar_x, y + 2.0, BAR_W * share, 4.0], color);
            bar_x + BAR_W
        };
        let (label, flame) = match entry.burn {
            BurnState::Off => continue,
            BurnState::Burning => ("hud.mistbrn.burning", SPRITE_FLAME),
            BurnState::Flared => ("hud.mistbrn.flared", SPRITE_FLAME_FLARED),
        };
        let mut label_x = end + 3.0;
        if textured && draw.sprite(SHEET, flame, [label_x, y, flame[2], flame[3]], None) {
            label_x += flame[2] + 2.0;
        }
        draw.text(&tr(host, label), [label_x, y], color, true);
    }
    if !others.is_empty() {
        draw.text(&others_line, [x, top + LINE * 3.0], DIM, true);
    }
}

/// Title colour: the brass of the frame's studs.
const BRASS_TEXT: u32 = 0xFFF0_D48A;

/// The panel frame as a nine-slice of `SPRITE_PANEL`; false (nothing drawn) without the sheet.
fn nine_slice(draw: &mut dyn DrawContext, [x, y, w, h]: [f32; 4]) -> bool {
    let [sx, sy] = SPRITE_PANEL;
    let (c, mid) = (PANEL_CORNER, PANEL_SIZE - 2.0 * PANEL_CORNER);
    // Source and destination spans per axis: start corner, stretched middle, end corner.
    let cols = [
        (0.0, c, x, c),
        (c, mid, x + c, w - 2.0 * c),
        (c + mid, c, x + w - c, c),
    ];
    let rows = [
        (0.0, c, y, c),
        (c, mid, y + c, h - 2.0 * c),
        (c + mid, c, y + h - c, c),
    ];
    for (src_y, src_h, dst_y, dst_h) in rows {
        for (src_x, src_w, dst_x, dst_w) in cols {
            if !draw.sprite(
                SHEET,
                [sx + src_x, sy + src_y, src_w, src_h],
                [dst_x, dst_y, dst_w, dst_h],
                None,
            ) {
                return false;
            }
        }
    }
    true
}

/// A framed bar at `at` (64x8 GUI px) filled to `share`, with the gem on the fill's end.
/// Returns where the frame ends.
fn framed_bar(draw: &mut dyn DrawContext, [x, y]: [f32; 2], share: f32, burn: BurnState) -> f32 {
    let [_, _, fw, fh] = SPRITE_FRAME;
    draw.sprite(SHEET, SPRITE_FRAME, [x, y, fw, fh], None);
    let [sx, sy] = match burn {
        BurnState::Off => SPRITE_FILL_OFF,
        BurnState::Burning => SPRITE_FILL_BURNING,
        BurnState::Flared => SPRITE_FILL_FLARED,
    };
    let filled = (BAR_W * share).clamp(0.0, BAR_W);
    if filled > 0.0 {
        // Take the fill's left part rather than squeezing the whole strip: no stretched sheen.
        draw.sprite(
            SHEET,
            [sx, sy, filled, 4.0],
            [x + 2.0, y + 2.0, filled, 4.0],
            None,
        );
        let [gx, gy, gw, gh] = SPRITE_GEM;
        let tip = (x + 2.0 + filled - gw / 2.0).clamp(x, x + fw - gw);
        draw.sprite(SHEET, [gx, gy, gw, gh], [tip, y + 1.0, gw, gh], None);
    }
    x + fw
}

fn percent(reserve: u32, capacity: u32) -> u32 {
    if capacity == 0 {
        return 0;
    }
    u32::try_from(u64::from(reserve) * 100 / u64::from(capacity)).unwrap_or(100)
}
