//! The book (P7b), client side: trees, diary, milestones and reserves on one screen of the
//! mod's own, opened with J. It draws what the server last sent (`mistbrn:progress`,
//! `mistbrn:hud`) and asks for changes with `mistbrn:control` requests; the server decides.
//!
//! Everything is laid out in a local space of `PANEL_W` × `PANEL_H` (the panel's top left at
//! 0, 0; the tabs above it), drawn at the scale of the `book_scale` setting through
//! `paint::Painter`. `targets` is the one list of what can be pointed at or focused: clicks,
//! keyboard focus, tooltips and the scenarios' log all read it.
//!
//! The trees tab is one radial map (P8, `map`): the general nodes in the centre, a sector per
//! pair around them, dragged with the mouse, zoomed with the wheel and walked with the keys.
//! Only the catalog's nodes are real; the other places are silhouettes, «still to discover»,
//! and a locked pair's sector is a silhouette with the milestone that opens it.
mod draw;
mod map;

pub(crate) use draw::{Look, draw};
pub(crate) use map::View;

use allomancy_core::{
    CATALOG, ControlRequest, Metal, NodeStatus, Pair, Progress, node_def, node_status,
    research_def, research_ready,
};
use map::{SECTOR_ORDER, map_layout, nearest_towards, sector_view};

pub(crate) const PANEL_W: f32 = 344.0;
pub(crate) const PANEL_H: f32 = 200.0;
/// Room above the panel for the tabs, in local pixels.
const TABS_H: f32 = 14.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tab {
    Tree,
    Diary,
    /// The milestones claimed and still to find (P6).
    Milestones,
    /// The eight reserves (P7b).
    Reserves,
}

pub(crate) const TABS: [Tab; 4] = [Tab::Tree, Tab::Diary, Tab::Milestones, Tab::Reserves];

/// Something that can be pointed at, focused or clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Tab(Tab),
    Node(u16),
    /// A metal's own power, owned with its pair (not a catalog node).
    Innate(Metal),
    /// A silhouette: a place of the map with nothing in the catalog yet.
    Hidden,
    /// A place of a locked pair's sector: it tells which milestone opens the pair.
    Locked(Pair),
    Research(u16),
    /// Show revealed hint number n (1-based).
    HintPage(u8),
    RevealHint,
    Conclude,
}

impl Target {
    /// Whether the keyboard focus stops on it (silhouettes and the innate power only explain).
    fn focusable(self) -> bool {
        !matches!(self, Target::Innate(_) | Target::Hidden | Target::Locked(_))
    }

    /// The name scenarios click it by (`tab.diary`, `node.17`, `innate.tin`, ...).
    pub(crate) fn name(self) -> String {
        match self {
            Target::Tab(tab) => format!("tab.{}", tab_key(tab)),
            Target::Node(code) => format!("node.{code}"),
            Target::Innate(metal) => format!("innate.{}", crate::bridge::metal_name(metal)),
            Target::Hidden => "hidden".into(),
            Target::Locked(pair) => format!("locked.{}", crate::controls::pair_name(pair)),
            Target::Research(code) => format!("research.{code}"),
            Target::HintPage(index) => format!("page.{index}"),
            Target::RevealHint => "reveal".into(),
            Target::Conclude => "conclude".into(),
        }
    }
}

pub(crate) fn tab_key(tab: Tab) -> &'static str {
    match tab {
        Tab::Tree => "tree",
        Tab::Diary => "diary",
        Tab::Milestones => "milestones",
        Tab::Reserves => "reserves",
    }
}

/// The tab buttons (local).
pub(crate) fn tab_rect(tab: Tab) -> [f32; 4] {
    let i = TABS.iter().position(|t| *t == tab).unwrap_or(0) as f32;
    [6.0 + 64.0 * i, -TABS_H + 1.0, 60.0, 14.0]
}

/// The map's window (local, under its header) and the diary list's.
pub(crate) const CANVAS: [f32; 4] = [6.0, 24.0, PANEL_W - 12.0, PANEL_H - 30.0];
pub(crate) const LIST: [f32; 4] = [6.0, 8.0, 112.0, 186.0];
/// A row of the diary list: the name in up to two lines.
pub(crate) const LIST_ROW: f32 = 30.0;
/// The diary's detail column starts here.
pub(crate) const DETAIL_X: f32 = 126.0;
pub(crate) const DETAIL_W: f32 = PANEL_W - 6.0 - DETAIL_X;
pub(crate) const HINT_BUTTON: [f32; 4] = [DETAIL_X + DETAIL_W - 96.0, 84.0, 96.0, 12.0];
const HINT_PAGE: [f32; 2] = [DETAIL_X + 54.0, 84.0];
pub(crate) const CONCLUDE_BUTTON: [f32; 4] = [DETAIL_X, PANEL_H - 22.0, DETAIL_W, 14.0];

pub(crate) fn hint_page_rect(index: u8) -> [f32; 4] {
    let x = HINT_PAGE[0] + 14.0 * f32::from(index.max(1) - 1);
    [x, HINT_PAGE[1], 12.0, 12.0]
}

/// How far the diary list can scroll.
pub(crate) fn max_list_scroll() -> f32 {
    (CATALOG.research.len() as f32 * LIST_ROW - LIST[3]).max(0.0)
}

/// A pressed left button over the map: where, and whether it moved far enough to be a drag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Press {
    pub(crate) at: [f32; 2],
    pub(crate) target: Option<Target>,
    pub(crate) dragged: bool,
}

/// How far (GUI pixels, whatever the book's scale) a press may move and still be a click.
const CLICK_SLOP: f32 = 3.0;

/// A distance in the book's local pixels as GUI pixels, at the last frame's scale.
fn gui_distance(book: &Book, from: [f32; 2], to: [f32; 2]) -> f32 {
    let scale = book.frame.map_or(1.0, |(_, scale)| scale);
    ((to[0] - from[0]).powi(2) + (to[1] - from[1]).powi(2)).sqrt() * scale
}

/// Every place of the map as the player sees it: a locked pair's places are its silhouette.
fn shown_places(unlocked: &dyn Fn(Pair) -> bool) -> Vec<(Target, [f32; 2])> {
    map_layout()
        .0
        .into_iter()
        .map(|place| {
            let target = match (place.sector, place.target) {
                (Some(pair), _) if !unlocked(pair) => Target::Locked(pair),
                // A bridge shows once both of the pairs it joins are unlocked (P8).
                (None, Target::Node(code)) => match allomancy_core::node_def(code, &CATALOG) {
                    Some(def) => match def.pool {
                        allomancy_core::Pool::Bridge(a, b) if !unlocked(a) || !unlocked(b) => {
                            Target::Hidden
                        }
                        _ => place.target,
                    },
                    None => place.target,
                },
                _ => place.target,
            };
            (target, place.at)
        })
        .collect()
}

/// The open book's state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Book {
    pub(crate) tab: Tab,
    /// The map's view (P8).
    pub(crate) view: View,
    /// The left button held over the map, until it is let go.
    pub(crate) press: Option<Press>,
    /// Which pairs are unlocked, as the last frame drew them (the HUD's answer).
    pub(crate) unlocked: [bool; 4],
    /// The diary list's scroll, in local pixels.
    pub(crate) list_scroll: f32,
    pub(crate) research: u16,
    /// The revealed hint shown in the diary (`None`: the newest).
    pub(crate) hint_page: Option<u8>,
    pub(crate) focus: Option<Target>,
    /// The panel's top left and scale on screen, from the last frame.
    pub(crate) frame: Option<([f32; 2], f32)>,
    /// The targets line last logged (`event=book_view`), to log only what changed.
    pub(crate) logged: Option<String>,
    /// The last Conclude was not taken by the server: the diary says why it may have been.
    pub(crate) refused: bool,
    /// The focus moved by keyboard since the pointer last moved: its tooltip wins.
    pub(crate) keyboard: bool,
    /// The pointer (screen GUI pixels) at the last frame.
    pub(crate) last_pointer: Option<[f32; 2]>,
}

impl Book {
    /// A book opened on the map, around `selected`'s sector.
    pub(crate) fn new(selected: Pair) -> Self {
        Self {
            tab: Tab::Tree,
            view: View::on(selected),
            press: None,
            unlocked: [true, false, false, false],
            list_scroll: 0.0,
            research: CATALOG.research.first().map_or(0, |def| def.code),
            hint_page: None,
            focus: None,
            frame: None,
            logged: None,
            refused: false,
            keyboard: false,
            last_pointer: None,
        }
    }

    pub(crate) fn local(&self, pointer: [f32; 2]) -> Option<[f32; 2]> {
        let ([x, y], scale) = self.frame?;
        Some([(pointer[0] - x) / scale, (pointer[1] - y) / scale])
    }

    /// Whether `pair` is unlocked, as last drawn.
    pub(crate) fn pair_open(&self, pair: Pair) -> bool {
        self.unlocked[pair_slot(pair)]
    }
}

fn pair_slot(pair: Pair) -> usize {
    match pair {
        Pair::IronSteel => 0,
        Pair::TinPewter => 1,
        Pair::ZincBrass => 2,
        Pair::CopperBronze => 3,
    }
}

/// Where the panel goes on a GUI of `size`, at the `wanted` scale or smaller so it fits.
pub(crate) fn frame(size: [f32; 2], wanted: f32) -> ([f32; 2], f32) {
    let fit = ((size[0] - 8.0) / PANEL_W).min((size[1] - 8.0) / (PANEL_H + TABS_H));
    let scale = wanted.min(fit).max(0.25);
    let origin = [
        ((size[0] - PANEL_W * scale) / 2.0).floor(),
        ((size[1] - (PANEL_H - TABS_H) * scale) / 2.0).floor(),
    ];
    (origin, scale)
}

pub(crate) fn inside(point: [f32; 2], rect: [f32; 4]) -> bool {
    point[0] >= rect[0]
        && point[1] >= rect[1]
        && point[0] < rect[0] + rect[2]
        && point[1] < rect[1] + rect[3]
}

/// Whether `rect` shows whole inside the window `clip`: a part cut by the clip is neither
/// clicked, focused nor given a tooltip (Codex review, round 1).
fn shows(rect: [f32; 4], clip: [f32; 4]) -> bool {
    rect[0] >= clip[0]
        && rect[1] >= clip[1]
        && rect[0] + rect[2] <= clip[0] + clip[2]
        && rect[1] + rect[3] <= clip[1] + clip[3]
}

/// Every target of the book as it stands, in focus order, with its local rectangle and
/// whether it shows (a map node out of the window does not, but keeps its place in the focus
/// order).
pub(crate) fn targets(book: &Book, progress: &Progress) -> Vec<(Target, [f32; 4], bool)> {
    let mut out: Vec<(Target, [f32; 4], bool)> = TABS
        .iter()
        .map(|&t| (Target::Tab(t), tab_rect(t), true))
        .collect();
    match book.tab {
        Tab::Tree => {
            for (target, at) in shown_places(&|pair| book.pair_open(pair)) {
                let rect = book.view.node_rect(at);
                out.push((target, rect, shows(rect, CANVAS)));
            }
        }
        Tab::Diary => {
            for (i, def) in CATALOG.research.iter().enumerate() {
                let rect = [
                    LIST[0],
                    LIST[1] + LIST_ROW * i as f32 - book.list_scroll,
                    LIST[2],
                    LIST_ROW - 2.0,
                ];
                out.push((Target::Research(def.code), rect, shows(rect, LIST)));
            }
            if let Some(def) = research_def(book.research, &CATALOG) {
                let entry = progress.research(def.code);
                for index in 1..=entry.hints {
                    out.push((Target::HintPage(index), hint_page_rect(index), true));
                }
                if entry.hints < def.hints {
                    out.push((Target::RevealHint, HINT_BUTTON, true));
                }
                if !entry.concluded && research_ready(progress, def) {
                    out.push((Target::Conclude, CONCLUDE_BUTTON, true));
                }
            }
        }
        Tab::Milestones | Tab::Reserves => {}
    }
    out
}

/// The target under the local point `at`, if any.
pub(crate) fn target_at(book: &Book, progress: &Progress, at: [f32; 2]) -> Option<Target> {
    targets(book, progress)
        .into_iter()
        .find(|(_, rect, shown)| *shown && inside(at, *rect))
        .map(|(target, ..)| target)
}

/// What happens when `target` is clicked or activated: the book changes, and maybe a request
/// goes to the server (a node that can be bought, the next hint, a conclusion).
pub(crate) fn activate(
    book: &mut Book,
    target: Target,
    progress: &Progress,
    unlocked: impl Fn(Pair) -> bool,
) -> Option<ControlRequest> {
    if target.focusable() {
        book.focus = Some(target);
    }
    match target {
        Target::Tab(tab) => {
            book.tab = tab;
            // A press held over the map does not survive leaving it.
            book.press = None;
            None
        }
        Target::Research(code) => {
            if book.research != code {
                book.research = code;
                book.hint_page = None;
            }
            None
        }
        Target::HintPage(index) => {
            book.hint_page = Some(index);
            None
        }
        Target::Innate(_) | Target::Hidden | Target::Locked(_) => None,
        Target::Node(code) => {
            let def = node_def(code, &CATALOG)?;
            (node_status(progress, def, unlocked, &CATALOG) == NodeStatus::Available).then(|| {
                // Codes stop at 127 (`MAX_CODE`); op 9 carries the ones above 63.
                ControlRequest::Purchase {
                    code: u8::try_from(code).unwrap_or(0),
                    rank: progress.rank(code) + 1,
                }
            })
        }
        Target::RevealHint => {
            let def = research_def(book.research, &CATALOG)?;
            let shown = progress.research(def.code).hints;
            // Show the hint about to be revealed.
            book.hint_page = None;
            (shown < def.hints).then(|| ControlRequest::RevealHint {
                research: u8::try_from(def.code).unwrap_or(0),
                index: shown + 1,
            })
        }
        Target::Conclude => {
            let def = research_def(book.research, &CATALOG)?;
            let entry = progress.research(def.code);
            (!entry.concluded && research_ready(progress, def)).then(|| ControlRequest::Conclude {
                research: u8::try_from(def.code).unwrap_or(0),
            })
        }
    }
}

/// The wheel by `lines` notches (positive: up, as the game counts) over the local point `at`
/// (`None`: the tab's own window): over the map it zooms round the pointer (up: in), over the
/// diary list it scrolls. True when the window is there.
pub(crate) fn scroll(book: &mut Book, at: Option<[f32; 2]>, lines: f32) -> bool {
    const NOTCH: f32 = 16.0;
    const ZOOM_STEP: f32 = 1.25;
    let window = match book.tab {
        Tab::Tree => CANVAS,
        Tab::Diary => LIST,
        _ => return false,
    };
    if at.is_some_and(|at| !inside(at, window)) {
        return false;
    }
    if book.tab == Tab::Tree {
        book.view
            .zoom_at(ZOOM_STEP.powf(lines.clamp(-4.0, 4.0)), at);
    } else {
        book.list_scroll = (book.list_scroll - lines * NOTCH).clamp(0.0, max_list_scroll());
    }
    true
}

/// The left button went down at the local point `at` over the map: remembered until it is let
/// go (a click) or moves (a drag). False when `at` is not over the map.
pub(crate) fn press(book: &mut Book, progress: &Progress, at: [f32; 2]) -> bool {
    if book.tab != Tab::Tree || !inside(at, CANVAS) {
        return false;
    }
    book.press = Some(Press {
        at,
        target: target_at(book, progress, at),
        dragged: false,
    });
    true
}

/// The held button moved by `delta` local pixels: past the slop, the map follows it.
pub(crate) fn drag(book: &mut Book, at: [f32; 2], delta: [f32; 2]) -> bool {
    let Some(start) = book.press.map(|p| p.at) else {
        return false;
    };
    let moved = gui_distance(book, start, at);
    let Some(press) = book.press.as_mut() else {
        return false;
    };
    if !press.dragged {
        if moved < CLICK_SLOP {
            return true;
        }
        press.dragged = true;
        // The first step of a drag also takes what moved below the slop.
        let start = [
            at[0] - press.at[0] - delta[0],
            at[1] - press.at[1] - delta[1],
        ];
        book.view.drag(start);
    }
    book.view.drag(delta);
    true
}

/// The button let go at the local point `at`: a press that did not drag is a click on what it
/// went down on, if the map still shows and that is still under the pointer (a tab changed by
/// keyboard meanwhile, or a pointer that left, clicks nothing: Codex review, round 1).
pub(crate) fn release(
    book: &mut Book,
    progress: &Progress,
    at: [f32; 2],
    unlocked: impl Fn(Pair) -> bool,
) -> (bool, Option<ControlRequest>) {
    let Some(press) = book.press.take() else {
        return (false, None);
    };
    let moved = gui_distance(book, press.at, at);
    // A release past the slop is a drag even if no drag event said so (Codex review, round 2).
    let still = book.tab == Tab::Tree
        && moved < CLICK_SLOP
        && target_at(book, progress, at) == press.target;
    let request = match press.target {
        Some(target) if !press.dragged && still => activate(book, target, progress, unlocked),
        _ => None,
    };
    (true, request)
}

/// A key the book reads while open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Next,
    Previous,
    Activate,
    PageUp,
    PageDown,
    /// An arrow: 0 up, 1 right, 2 down, 3 left. On the map it walks to the nearest node that
    /// way; elsewhere up/left go back and down/right forward.
    Arrow(u8),
    ZoomIn,
    ZoomOut,
    /// Home: the map's centre.
    Home,
}

/// Vanilla's key names (`screen_key_pressed_pre`) to the book's keys. Shift+Tab goes back.
pub(crate) fn key(name: &str, shift: bool) -> Option<Key> {
    Some(match name {
        "key.keyboard.tab" if shift => Key::Previous,
        "key.keyboard.tab" => Key::Next,
        "key.keyboard.up" => Key::Arrow(0),
        "key.keyboard.right" => Key::Arrow(1),
        "key.keyboard.down" => Key::Arrow(2),
        "key.keyboard.left" => Key::Arrow(3),
        "key.keyboard.enter" | "key.keyboard.keypad.enter" | "key.keyboard.space" => Key::Activate,
        "key.keyboard.page.up" => Key::PageUp,
        "key.keyboard.page.down" => Key::PageDown,
        "key.keyboard.equal" | "key.keyboard.keypad.add" => Key::ZoomIn,
        "key.keyboard.minus" | "key.keyboard.keypad.subtract" => Key::ZoomOut,
        "key.keyboard.home" => Key::Home,
        _ => return None,
    })
}

/// Moves the focus to the next or previous focusable target (wrapping round) and moves the
/// map or scrolls the list so it shows.
pub(crate) fn move_focus(book: &mut Book, progress: &Progress, forward: bool) {
    let order: Vec<(Target, [f32; 4])> = targets(book, progress)
        .into_iter()
        .filter(|(t, ..)| t.focusable())
        .map(|(t, rect, _)| (t, rect))
        .collect();
    if order.is_empty() {
        return;
    }
    let now = book
        .focus
        .and_then(|f| order.iter().position(|(t, _)| *t == f));
    let next = match (now, forward) {
        (None, true) => 0,
        (None, false) => order.len() - 1,
        (Some(i), true) => (i + 1) % order.len(),
        (Some(i), false) => (i + order.len() - 1) % order.len(),
    };
    let (target, rect) = order[next];
    focus_on(book, target, rect);
}

/// Gives `target` (at local `rect`) the focus and brings it into view.
fn focus_on(book: &mut Book, target: Target, rect: [f32; 4]) {
    book.focus = Some(target);
    book.keyboard = true;
    match target {
        Target::Node(_) if !shows(rect, CANVAS) => {
            let centre = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
            let at = book.view.to_map(centre);
            book.view.centre(at);
        }
        Target::Research(_) => {
            // `rect` is already scrolled: move by how far it sticks out.
            let top = LIST[1] + 2.0;
            let bottom = LIST[1] + LIST[3] - 2.0;
            let value = &mut book.list_scroll;
            if rect[1] < top {
                *value -= top - rect[1];
            } else if rect[1] + rect[3] > bottom {
                *value += rect[1] + rect[3] - bottom;
            }
            *value = value.clamp(0.0, max_list_scroll());
        }
        _ => {}
    }
}

/// An arrow on the map: the focus walks to the nearest focusable node that way (from the
/// focused node, or from the view's centre). Elsewhere, up/left go back and down/right on.
pub(crate) fn arrow(book: &mut Book, progress: &Progress, direction: u8) {
    if book.tab != Tab::Tree {
        move_focus(book, progress, matches!(direction, 1 | 2));
        return;
    }
    let dir = match direction {
        0 => [0.0, -1.0],
        1 => [1.0, 0.0],
        2 => [0.0, 1.0],
        _ => [-1.0, 0.0],
    };
    let nodes: Vec<(Target, [f32; 2])> = shown_places(&|pair| book.pair_open(pair))
        .into_iter()
        .filter(|(target, _)| matches!(target, Target::Node(_)))
        .collect();
    let from = book
        .focus
        .and_then(|f| nodes.iter().find(|(t, _)| *t == f).map(|(_, at)| *at))
        .unwrap_or(book.view.pan);
    if let Some(target) = nearest_towards(from, dir, nodes.iter().copied()) {
        let at = nodes
            .iter()
            .find(|(t, _)| *t == target)
            .map_or(from, |n| n.1);
        let rect = book.view.node_rect(at);
        focus_on(book, target, rect);
    }
}

/// Page Down / Page Up on the map: the view goes to the next / previous sector (the centre
/// counts as one, before iron and steel); in the diary, the list scrolls.
pub(crate) fn page(book: &mut Book, forward: bool) {
    if book.tab != Tab::Tree {
        scroll(book, None, if forward { -3.0 } else { 3.0 });
        return;
    }
    let stops: Vec<[f32; 2]> = std::iter::once([0.0, 0.0])
        .chain(SECTOR_ORDER.iter().map(|&pair| sector_view(pair)))
        .collect();
    let distance =
        |p: [f32; 2]| (p[0] - book.view.pan[0]).powi(2) + (p[1] - book.view.pan[1]).powi(2);
    let now = (0..stops.len())
        .min_by(|&a, &b| distance(stops[a]).total_cmp(&distance(stops[b])))
        .unwrap_or(0);
    let next = if forward {
        (now + 1) % stops.len()
    } else {
        (now + stops.len() - 1) % stops.len()
    };
    book.view.centre(stops[next]);
}

/// The milestone that unlocks `pair`, if one does.
pub(crate) fn unlocking_milestone(pair: Pair) -> Option<&'static allomancy_core::MilestoneDef> {
    CATALOG.milestones.iter().find(|m| m.unlocks == Some(pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    use allomancy_core::{NODE_ACCEL, NODE_COIN_JUMP, NODE_ECONOMY, NODE_REACH};

    // Catches the Tab order skipping a node, or a focused node left out of view.
    #[test]
    fn focus_wraps_and_brings_the_focused_node_into_view() {
        let progress = Progress::default();
        let mut book = Book::new(Pair::IronSteel);
        let focusable = targets(&book, &progress)
            .iter()
            .filter(|(t, ..)| t.focusable())
            .count();
        // 4 tabs, the 6 general nodes, reach, accel and coin jump (the bridge to tin and
        // pewter stays hidden while they are locked).
        assert_eq!(focusable, 13);
        for _ in 0..focusable {
            move_focus(&mut book, &progress, true);
            let (_, rect, shown) = targets(&book, &progress)
                .into_iter()
                .find(|(t, ..)| Some(*t) == book.focus)
                .unwrap();
            assert!(shown, "{:?} at {rect:?}", book.focus);
        }
        assert_eq!(book.focus, Some(Target::Node(NODE_COIN_JUMP)));
        move_focus(&mut book, &progress, true);
        assert_eq!(book.focus, Some(Target::Tab(Tab::Tree)));
        move_focus(&mut book, &progress, false);
        assert_eq!(book.focus, Some(Target::Node(NODE_COIN_JUMP)));
    }

    // Catches a locked sector's places being bought, focused or named as nodes.
    #[test]
    fn a_locked_sector_is_a_silhouette_naming_its_pair() {
        let book = Book::new(Pair::IronSteel);
        let all = targets(&book, &Progress::default());
        let locked = |pair| {
            all.iter()
                .filter(|(t, ..)| *t == Target::Locked(pair))
                .count()
        };
        assert_eq!(locked(Pair::TinPewter), 19);
        assert_eq!(locked(Pair::ZincBrass), 19);
        assert_eq!(locked(Pair::CopperBronze), 19);
        assert_eq!(locked(Pair::IronSteel), 0);
        assert!(!Target::Locked(Pair::TinPewter).focusable());
        assert_eq!(Target::Locked(Pair::TinPewter).name(), "locked.tin_pewter");
        assert_eq!(Target::Innate(Metal::Pewter).name(), "innate.pewter");
    }

    // Catches a drag that also clicks, or a click lost to a tiny jitter.
    #[test]
    fn a_press_is_a_click_unless_it_drags() {
        let progress = Progress::default();
        let mut book = Book::new(Pair::IronSteel);
        let (_, rect, _) = targets(&book, &progress)
            .into_iter()
            .find(|(t, ..)| *t == Target::Node(NODE_REACH))
            .unwrap();
        let at = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
        // 100 iron and steel XP (one point), as the server's snapshot would carry it.
        let mut bytes = allomancy_core::encode_progress_snapshot(&progress);
        bytes[1..5].copy_from_slice(&100u32.to_le_bytes());
        let progress_with_points = allomancy_core::decode_progress_snapshot(&bytes)
            .unwrap()
            .progress;
        // A jitter below the slop is still a click.
        assert!(press(&mut book, &progress_with_points, at));
        assert!(drag(&mut book, [at[0] + 1.0, at[1]], [1.0, 0.0]));
        let (ours, request) = release(
            &mut book,
            &progress_with_points,
            [at[0] + 1.0, at[1]],
            |p| p == Pair::IronSteel,
        );
        assert!(ours);
        assert_eq!(
            request,
            Some(ControlRequest::Purchase {
                code: NODE_REACH as u8,
                rank: 1
            })
        );
        // A drag moves the map and buys nothing.
        let pan = book.view.pan;
        assert!(press(&mut book, &progress_with_points, at));
        assert!(drag(&mut book, [at[0] + 2.0, at[1]], [2.0, 0.0]));
        assert!(drag(&mut book, [at[0] + 10.0, at[1]], [8.0, 0.0]));
        assert_eq!(
            release(
                &mut book,
                &progress_with_points,
                [at[0] + 10.0, at[1]],
                |_| true
            ),
            (true, None)
        );
        assert!((book.view.pan[0] - (pan[0] - 10.0)).abs() < 1e-3);
        // A tab changed by keyboard while the button is held cancels the click.
        let (_, rect, _) = targets(&book, &progress)
            .into_iter()
            .find(|(t, ..)| *t == Target::Node(NODE_REACH))
            .unwrap();
        let at = [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0];
        assert!(press(&mut book, &progress_with_points, at));
        activate(
            &mut book,
            Target::Tab(Tab::Diary),
            &progress_with_points,
            |_| true,
        );
        assert_eq!(
            release(&mut book, &progress_with_points, at, |_| true),
            (false, None)
        );
        // A pointer that moved off the node (out of the slop by a held still press) buys nothing.
        book.tab = Tab::Tree;
        assert!(press(&mut book, &progress_with_points, at));
        assert_eq!(
            release(
                &mut book,
                &progress_with_points,
                [CANVAS[0] + 1.0, CANVAS[1] + 1.0],
                |_| true
            ),
            (true, None)
        );
        // A release 4 pixels away with no drag event between is a drag: no click (Codex r2).
        assert!(press(&mut book, &progress_with_points, at));
        assert_eq!(
            release(
                &mut book,
                &progress_with_points,
                [at[0] + 4.0, at[1]],
                |_| true
            ),
            (true, None)
        );
        // At 125 % the slop is 3 GUI pixels: 2.5 local pixels (3.1 GUI) already drag (Codex r3).
        book.frame = Some(([0.0, 0.0], 1.25));
        assert!(press(&mut book, &progress_with_points, at));
        assert_eq!(
            release(
                &mut book,
                &progress_with_points,
                [at[0] + 2.5, at[1]],
                |_| true
            ),
            (true, None)
        );
        book.frame = None;
        // Outside the map, a press is not ours.
        assert!(!press(&mut book, &progress, [1.0, 1.0]));
    }

    #[test]
    fn the_wheel_zooms_the_map_and_scrolls_the_diary() {
        let mut book = Book::new(Pair::IronSteel);
        assert!(!scroll(&mut book, Some([0.0, 0.0]), -1.0));
        let inside_map = [CANVAS[0] + 10.0, CANVAS[1] + 10.0];
        assert!(scroll(&mut book, Some(inside_map), 1.0));
        assert!((book.view.zoom - 1.25).abs() < 1e-4);
        // One event moves at most 4 notches; enough of them stop at the bottom.
        for _ in 0..3 {
            assert!(scroll(&mut book, None, -100.0));
        }
        assert_eq!(book.view.zoom, map::ZOOM_MIN);
        book.tab = Tab::Diary;
        assert!(scroll(&mut book, None, -1.0));
        assert_eq!(book.list_scroll, max_list_scroll().min(16.0));
    }

    #[test]
    fn arrows_walk_the_map_and_pages_visit_the_sectors() {
        let progress = Progress::default();
        let mut book = Book::new(Pair::IronSteel);
        book.focus = Some(Target::Node(NODE_REACH));
        // From reach, down-right leads inwards; up from accel leads on along the branch.
        arrow(&mut book, &progress, 1);
        assert!(book.focus.is_some());
        book.view.centre([0.0, 0.0]);
        book.focus = None;
        arrow(&mut book, &progress, 0);
        assert_eq!(book.focus, Some(Target::Node(NODE_ECONOMY)));
        page(&mut book, true);
        assert_eq!(book.view.pan, sector_view(Pair::IronSteel));
        page(&mut book, false);
        assert_eq!(book.view.pan, [0.0, 0.0]);
        page(&mut book, false);
        assert_eq!(book.view.pan, sector_view(Pair::ZincBrass));
        let _ = NODE_ACCEL;
    }

    #[test]
    fn the_frame_fits_small_guis_and_keeps_the_wanted_scale_on_big_ones() {
        let (_, scale) = frame([480.0, 270.0], 1.25);
        assert!(scale <= 1.25 && (PANEL_H + TABS_H) * scale <= 262.5);
        let (origin, scale) = frame([960.0, 540.0], 1.25);
        assert_eq!(scale, 1.25);
        assert!(origin[1] - TABS_H * scale >= 0.0);
        assert_eq!(frame([960.0, 540.0], 0.75).1, 0.75);
    }

    #[test]
    fn keys_map_as_the_plan_says() {
        assert_eq!(key("key.keyboard.tab", false), Some(Key::Next));
        assert_eq!(key("key.keyboard.tab", true), Some(Key::Previous));
        assert_eq!(key("key.keyboard.page.down", false), Some(Key::PageDown));
        assert_eq!(key("key.keyboard.down", false), Some(Key::Arrow(2)));
        assert_eq!(key("key.keyboard.equal", false), Some(Key::ZoomIn));
        assert_eq!(key("key.keyboard.home", false), Some(Key::Home));
        assert_eq!(key("key.keyboard.j", false), None);
    }
}
