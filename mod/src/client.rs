//! Keys and the reserve HUD (P7a), client side.
//!
//! Five rebindable keys (Options > Controls > Masamune's MISTBRN): V the pair wheel (hold,
//! point, let go; a tap is the next pair), Z / X toggle
//! the pair's first / second metal, R toggle flaring, G all off. A key sends one
//! `mistbrn:control` request per press, and only while playing with nothing over the world
//! and the window focused: the chat, the inventory and every other screen block it. Holding a
//! key does not repeat it. The server decides; the HUD only draws what the server last sent.
use crate::controls::{CONTROL_CHANNEL, pair_name};
use crate::hud;
use crate::notebook::{self, Tab};
use crate::{bridge::metal_name, scenario};
use allomancy_core::{
    BurnState, ControlRequest, HudSnapshot, Metal, Pair, Progress, Side, decode_hud,
    decode_progress_snapshot, encode_control, pair_metals,
};
use mod_api::{
    ClientHost, DrawContext, GuiLayer, GuiLayerPlacement, IPayloadContext, IPayloadHandler,
    KeyConflictContext, KeyRegistration, PayloadBytes, PayloadHandleError, PayloadId, Registrar,
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
/// J opens the notebook (P5); with the notebook open, J or Esc closes it. B was taken by
/// Horizon's emote wheel (p5-1 smoke run).
const NOTEBOOK_KEY: (&str, &str) = ("notebook", "key.keyboard.j");
/// Test runs only (`MISTBRN_TEST_SCENARIO=p4`): sends malformed requests and a burst (K7).
const FUZZ_KEY: (&str, &str) = ("test_fuzz", "key.keyboard.k");
/// Clicks taken per key per tick at most; the rest wait (a held key auto-repeats).
const CLICKS_PER_TICK: u32 = 64;

/// The key that opens the pair wheel (`cycle_pair`, V by default).
const WHEEL_KEY: &str = "cycle_pair";
/// A release this soon after opening, with the cursor in the centre, is a tap: next pair.
const TAP_TICKS: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WheelAction {
    Select(Pair),
    Cycle,
    Close,
}

/// The open pair wheel: when it opened, the slot under the cursor, what to do on the next tick.
struct Wheel {
    opened: u64,
    hovered: Option<Pair>,
    action: Option<WheelAction>,
    /// The GUI size of the last frame (clicks pick their slot against it); logged once, on
    /// the first frame, for scenarios that click its slots.
    size: Option<[f32; 2]>,
}

/// The open notebook screen: the tab shown, the GUI size of the last frame, and a request or a
/// close chosen in a screen hook (which cannot send), applied on the next tick.
struct Notebook {
    tab: Tab,
    /// The revealed hint shown in the diary (`None`: the newest).
    hint_page: Option<u8>,
    size: Option<[f32; 2]>,
    request: Option<ControlRequest>,
    close: bool,
}

struct ClientState {
    was_down: [bool; KEYS.len() + 2],
    hud: Option<HudSnapshot>,
    ticks: u64,
    wheel: Option<Wheel>,
    /// Set when the wheel closes while V is still physically held: V must be let go before it
    /// opens the wheel again (its auto-repeat would otherwise reopen it at once).
    wheel_latched: bool,
    /// The player's progress as the server last sent it (`mistbrn:progress`).
    progress: Option<Progress>,
    notebook: Option<Notebook>,
}

static STATE: Mutex<ClientState> = Mutex::new(ClientState {
    was_down: [false; KEYS.len() + 2],
    hud: None,
    ticks: 0,
    wheel: None,
    wheel_latched: false,
    progress: None,
    notebook: None,
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

struct ProgressInbox;

impl IPayloadHandler for ProgressInbox {
    fn handle(
        &self,
        payload: PayloadBytes,
        _: &mut dyn IPayloadContext,
    ) -> Result<(), PayloadHandleError> {
        match decode_progress_snapshot(&payload.0) {
            Ok(snapshot) => {
                let progress = snapshot.progress;
                let ranks: Vec<String> = progress
                    .nodes()
                    .iter()
                    .map(|(code, rank)| format!("{code}:{rank}"))
                    .collect();
                let research: Vec<String> = progress
                    .all_research()
                    .iter()
                    .map(|(code, r)| {
                        format!(
                            "{code}:e{}h{}c{}",
                            r.evidence,
                            r.hints,
                            u8::from(r.concluded)
                        )
                    })
                    .collect();
                eprintln!(
                    "MISTBRN progress xp_iron_steel={} nodes={} research={}",
                    progress.pair_xp(Pair::IronSteel),
                    ranks.join(","),
                    research.join(",")
                );
                state().progress = Some(progress);
            }
            Err(error) => eprintln!("MISTBRN event=progress_rejected error={error:?}"),
        }
        Ok(())
    }
}

pub(crate) fn progress_handler() -> Arc<dyn IPayloadHandler> {
    Arc::new(ProgressInbox)
}

/// Opens the notebook screen (the notebook item, or B). Nothing if another screen is open.
pub(crate) fn open_notebook(host: &mut dyn ClientHost) {
    let mut state = state();
    if state.wheel.is_some() || state.notebook.is_some() {
        return;
    }
    if host.mod_screen_open(MOD_ID) {
        eprintln!("MISTBRN event=notebook result=open");
        state.notebook = Some(Notebook {
            tab: Tab::Tree,
            hint_page: None,
            size: None,
            request: None,
            close: false,
        });
    } else {
        eprintln!("MISTBRN event=notebook result=no_screen");
    }
}

pub(crate) fn register_keys(registrar: &mut dyn Registrar) {
    let fuzz = scenario::commands_enabled().then_some(FUZZ_KEY);
    let keys = KEYS.iter().map(|(id, key, _)| (*id, *key));
    for (id, key) in keys.chain(Some(NOTEBOOK_KEY)).chain(fuzz) {
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
    state.ticks += 1;
    let mut sends = Vec::new();
    wheel_tick(host, &mut state, &mut sends);
    notebook_tick(host, &mut state, &mut sends);
    if state.wheel_latched && !wheel_key_down(host) {
        state.wheel_latched = false;
    }
    for (index, (id, _, request)) in KEYS.iter().enumerate() {
        if pressed(host, index, id, &mut state.was_down) {
            if !allowed {
                eprintln!("MISTBRN event=key key={id} result=blocked");
            } else if *id == WHEEL_KEY {
                // V opens the wheel; the choice is sent when it closes. Nothing is sent if the
                // wheel cannot open (another screen won) or V was not let go since it closed.
                if state.wheel_latched {
                    eprintln!("MISTBRN event=key key={id} result=held");
                } else if host.mod_screen_open(MOD_ID) {
                    eprintln!("MISTBRN event=key key={id} result=wheel");
                    state.wheel = Some(Wheel {
                        opened: state.ticks,
                        hovered: None,
                        action: None,
                        size: None,
                    });
                } else {
                    eprintln!("MISTBRN event=key key={id} result=no_wheel");
                }
            } else {
                eprintln!("MISTBRN event=key key={id}");
                sends.push(encode_control(*request).to_vec());
            }
        }
    }
    let notebook_pressed = pressed(host, KEYS.len() + 1, NOTEBOOK_KEY.0, &mut state.was_down);
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
    if notebook_pressed {
        if allowed {
            eprintln!("MISTBRN event=key key={}", NOTEBOOK_KEY.0);
            open_notebook(host);
        } else {
            eprintln!("MISTBRN event=key key={} result=blocked", NOTEBOOK_KEY.0);
        }
    }
}

/// Sends the notebook's chosen request, closes it when asked, and forgets a notebook that
/// something else closed (Esc, another screen).
fn notebook_tick(host: &mut dyn ClientHost, state: &mut ClientState, sends: &mut Vec<Vec<u8>>) {
    let Some(book) = state.notebook.as_mut() else {
        return;
    };
    if host.mod_screen() != Some(MOD_ID) {
        state.notebook = None;
        eprintln!("MISTBRN event=notebook result=closed");
        return;
    }
    if let Some(request) = book.request.take()
        && host.window_focused()
    {
        eprintln!("MISTBRN event=notebook result=request:{request:?}");
        sends.push(encode_control(request).to_vec());
    }
    if book.close {
        state.notebook = None;
        host.mod_screen_close(MOD_ID);
        eprintln!("MISTBRN event=notebook result=closed");
    }
}

/// Whether a pair is unlocked, from the HUD the server last sent.
fn unlocked_in(hud: Option<HudSnapshot>) -> impl Fn(Pair) -> bool {
    move |pair| {
        hud.is_some_and(|hud| {
            pair_metals(pair)
                .iter()
                .all(|metal| hud.metals[metal.index()].unlocked)
        })
    }
}

/// Applies the wheel's choice (made in a screen hook, which cannot close the screen) and
/// forgets a wheel that something else closed (Escape, another screen).
fn wheel_tick(host: &mut dyn ClientHost, state: &mut ClientState, sends: &mut Vec<Vec<u8>>) {
    let Some(wheel) = state.wheel.as_mut() else {
        return;
    };
    if let Some(action) = wheel.action.take() {
        state.wheel = None;
        state.wheel_latched = wheel_key_down(host);
        // A choice made in a hook counts only if the wheel is still ours and the window still
        // has focus on this tick: Escape, another screen or a lost focus in between cancel it.
        if !wheel_open(host) || !host.window_focused() {
            eprintln!("MISTBRN event=wheel result=cancelled");
            return;
        }
        host.mod_screen_close(MOD_ID);
        let request = match action {
            WheelAction::Select(pair) if state.hud.map(|hud| hud.pair) != Some(pair) => {
                Some(ControlRequest::SelectPair(pair))
            }
            WheelAction::Select(_) | WheelAction::Close => None,
            WheelAction::Cycle => Some(ControlRequest::CyclePair),
        };
        match request {
            Some(request) => {
                eprintln!("MISTBRN event=wheel result={}", wheel_word(action));
                sends.push(encode_control(request).to_vec());
            }
            None => eprintln!("MISTBRN event=wheel result=unchanged"),
        }
    } else if !wheel_open(host) && state.ticks > wheel.opened + 1 {
        state.wheel = None;
        state.wheel_latched = wheel_key_down(host);
        eprintln!("MISTBRN event=wheel result=closed");
    } else if wheel_open(host) && !wheel_key_down(host) {
        // V is up: the screen's release event may not come (a key let go as the screen opened,
        // a release the window never saw), so the key's own state decides, once a tick.
        let ticks = state.ticks;
        wheel.action = Some(release_action(wheel, host.cursor(), ticks));
    }
}

/// The physical key bound to the wheel is held (with a screen open, mappings read as up).
fn wheel_key_down(host: &dyn ClientHost) -> bool {
    host.mod_key(MOD_ID, WHEEL_KEY)
        .is_some_and(|(key, _)| host.is_key_down(&key))
}

/// What letting V go does: the slot under the cursor; in the centre, a quick tap is the next
/// pair (as before the wheel) and a long hold changes nothing.
fn release_action(wheel: &Wheel, cursor: [f32; 2], ticks: u64) -> WheelAction {
    // Where the pointer is now, not where the last frame saw it.
    let hovered = match wheel.size {
        Some(size) => hud::hovered(size, cursor),
        None => wheel.hovered,
    };
    match hovered {
        Some(pair) => WheelAction::Select(pair),
        None if ticks.saturating_sub(wheel.opened) <= TAP_TICKS => WheelAction::Cycle,
        None => WheelAction::Close,
    }
}

fn wheel_word(action: WheelAction) -> String {
    match action {
        WheelAction::Select(pair) => format!("select:{}", pair_name(pair)),
        WheelAction::Cycle => "cycle".into(),
        WheelAction::Close => "close".into(),
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
    state.wheel = None;
    state.wheel_latched = false;
    state.was_down = [false; KEYS.len() + 2];
    state.progress = None;
    state.notebook = None;
}

/// The reserve layer: the selected pair beside the hotbar (`hud::draw_reserves`).
pub(crate) fn draw(_: &dyn ClientHost, layer: &str, draw: &mut dyn DrawContext) {
    if layer != HUD_LAYER {
        return;
    }
    let state = state();
    // The wheel shows the reserves itself; the bars would sit under its hint.
    if let (Some(hud), None) = (state.hud, state.wheel.as_ref()) {
        hud::draw_reserves(draw, &hud);
    }
}

fn wheel_open(host: &dyn ClientHost) -> bool {
    host.mod_screen() == Some(MOD_ID)
}

/// Over our own screen (the wheel or the notebook): track what is under the cursor, draw it.
pub(crate) fn screen_draw(host: &dyn ClientHost, pointer: [f32; 2], draw: &mut dyn DrawContext) {
    if !wheel_open(host) {
        return;
    }
    {
        let mut state = state();
        if let Some(book) = state.notebook.as_mut() {
            let size = draw.size();
            if book.size.is_none() {
                // Node centres in GUI pixels, for scenarios that click them.
                let nodes: Vec<String> = allomancy_core::CATALOG
                    .nodes
                    .iter()
                    .filter_map(|def| {
                        notebook::node_centre(size, def.code)
                            .map(|[x, y]| format!("{}@{x},{y}", def.code))
                    })
                    .collect();
                eprintln!(
                    "MISTBRN event=notebook_open gui={}x{} nodes={} hint_button={}",
                    size[0],
                    size[1],
                    nodes.join(";"),
                    notebook::hint_button_centre(size)
                        .map_or_else(String::new, |[x, y]| format!("{x},{y}")),
                );
                let [dx, dy] = notebook::tab_centre(size, Tab::Diary);
                eprintln!("MISTBRN event=notebook_tabs diary={dx},{dy}");
            }
            book.size = Some(size);
            let (tab, page) = (book.tab, book.hint_page);
            let progress = state.progress.clone();
            let unlocked = unlocked_in(state.hud);
            drop(state);
            notebook::draw(host, draw, tab, progress.as_ref(), &unlocked, pointer, page);
            return;
        }
    }
    let size = draw.size();
    let hovered = hud::hovered(size, pointer);
    let hud = {
        let mut state = state();
        if let Some(wheel) = state.wheel.as_mut() {
            wheel.hovered = hovered;
            if wheel.size.is_none() {
                eprintln!("MISTBRN event=wheel_open gui={}x{}", size[0], size[1]);
            }
            wheel.size = Some(size);
        }
        state.hud
    };
    hud::draw_wheel(host, draw, hud.as_ref(), hovered);
}

/// V let go over the wheel (`release_action`). True when the release was ours.
pub(crate) fn key_released(host: &dyn ClientHost, key: &str) -> bool {
    {
        // Over the notebook, releases are not ours: J closes it on its press
        // (`notebook_key_pressed`), so letting go of the J that opened it changes nothing.
        if state().notebook.is_some() {
            return false;
        }
    }
    if !wheel_open(host)
        || host
            .mod_key(MOD_ID, WHEEL_KEY)
            .map(|(bound, _)| bound)
            .as_deref()
            != Some(key)
    {
        return false;
    }
    let mut state = state();
    let ticks = state.ticks;
    if let Some(wheel) = state.wheel.as_mut() {
        let action = release_action(wheel, host.cursor(), ticks);
        wheel.action.get_or_insert(action);
    }
    true
}

/// A left click over the wheel chooses the slot under the pointer. True when it was ours.
pub(crate) fn mouse_pressed(host: &dyn ClientHost, pointer: [f32; 2], button: i32) -> bool {
    if !wheel_open(host) || button != 0 {
        return false;
    }
    let mut state = state();
    let unlocked = unlocked_in(state.hud);
    let progress = state.progress.clone();
    if let Some(book) = state.notebook.as_mut() {
        let Some(size) = book.size else {
            return true;
        };
        let revealed = progress.as_ref().map_or(0, |p| {
            allomancy_core::CATALOG
                .research
                .first()
                .map_or(0, |def| p.research(def.code).hints)
        });
        match notebook::target(size, pointer, book.tab, revealed) {
            Some(notebook::Target::Tab(tab)) => book.tab = tab,
            Some(notebook::Target::HintPage(index)) => book.hint_page = Some(index),
            Some(target) => {
                if target == notebook::Target::RevealHint {
                    // Show the hint about to be revealed.
                    book.hint_page = None;
                }
                if let Some(request) = progress
                    .as_ref()
                    .and_then(|p| notebook::request(target, p, &unlocked))
                {
                    book.request.get_or_insert(request);
                }
            }
            None => {}
        }
        eprintln!(
            "MISTBRN event=notebook_click at={},{}",
            pointer[0], pointer[1]
        );
        return true;
    }
    if let Some(wheel) = state.wheel.as_mut()
        && let Some(pair) = wheel.size.and_then(|size| hud::hovered(size, pointer))
    {
        wheel.hovered = Some(pair);
        wheel.action.get_or_insert(WheelAction::Select(pair));
    }
    true
}

/// A fresh press of the notebook key over the open notebook closes it (not its release: the
/// release of the press that opened it would close it at once). True when the press was ours.
pub(crate) fn notebook_key_pressed(
    host: &dyn ClientHost,
    key: &str,
    modifiers: u32,
    repeat: bool,
) -> bool {
    // Shift, Control, Alt, Super: a rebinding to Ctrl+J is not closed by J alone.
    const MODIFIERS: u32 = 0xF;
    if !wheel_open(host) {
        return false;
    }
    let mut state = state();
    let Some(book) = state.notebook.as_mut() else {
        return false;
    };
    let Some((bound, bound_modifiers)) = host.mod_key(MOD_ID, NOTEBOOK_KEY.0) else {
        return false;
    };
    if bound != key || bound_modifiers.bits & MODIFIERS != modifiers & MODIFIERS {
        return false;
    }
    if !repeat {
        book.close = true;
    }
    true
}
