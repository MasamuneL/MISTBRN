//! The book (P7b), client side: trees, diary, milestones and reserves on one screen of the
//! mod's own, opened with J. It draws what the server last sent (`mistbrn:progress`,
//! `mistbrn:hud`) and asks for changes with `mistbrn:control` requests; the server decides.
//!
//! Everything is laid out in a local space of `PANEL_W` × `PANEL_H` (the panel's top left at
//! 0, 0; the tabs above it), drawn at the scale of the `book_scale` setting through
//! `paint::Painter`. `targets` is the one list of what can be pointed at or focused: clicks,
//! keyboard focus, tooltips and the scenarios' log all read it.
//!
//! The trees are a visual template (P7b): every pair has the design's shape (two branches of
//! three techniques with two sub-nodes each, a combined technique below), but only the
//! catalog's nodes are real. The other places are silhouettes, «still to discover».
mod draw;

pub(crate) use draw::{Look, draw};

use allomancy_core::{
    CATALOG, ControlRequest, NODE_ACCEL, NODE_CAPACITY, NODE_COIN_JUMP, NODE_ECONOMY, NODE_REACH,
    NodeStatus, PAIRS, Pair, Progress, node_def, node_status, research_def, research_ready,
};

pub(crate) const PANEL_W: f32 = 344.0;
pub(crate) const PANEL_H: f32 = 200.0;
/// Room above the panel for the tabs, in local pixels.
const TABS_H: f32 = 14.0;
const NODE: f32 = 22.0;

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

/// The tree shown: a pair's (index into `PAIRS`) or the general one (`GENERAL`).
pub(crate) const GENERAL: u8 = 4;
const VIEWS: u8 = 5;

/// Something that can be pointed at, focused or clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Tab(Tab),
    View(u8),
    Node(u16),
    /// The pair's own power, owned with the pair (not a catalog node).
    Innate,
    /// A silhouette: a place of the template with nothing in the catalog yet.
    Hidden,
    Research(u16),
    /// Show revealed hint number n (1-based).
    HintPage(u8),
    RevealHint,
    Conclude,
}

impl Target {
    /// Whether the keyboard focus stops on it (silhouettes and the innate power only explain).
    fn focusable(self) -> bool {
        !matches!(self, Target::Innate | Target::Hidden)
    }

    /// The name scenarios click it by (`tab.diary`, `view.4`, `node.17`, ...).
    pub(crate) fn name(self) -> String {
        match self {
            Target::Tab(tab) => format!("tab.{}", tab_key(tab)),
            Target::View(view) => format!("view.{view}"),
            Target::Node(code) => format!("node.{code}"),
            Target::Innate => "innate".into(),
            Target::Hidden => "hidden".into(),
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

/// A place of a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Node(u16),
    Innate,
    Hidden,
}

impl Slot {
    fn target(self) -> Target {
        match self {
            Slot::Node(code) => Target::Node(code),
            Slot::Innate => Target::Innate,
            Slot::Hidden => Target::Hidden,
        }
    }
}

/// A technique and its two sub-nodes.
type Technique = (Slot, [Slot; 2]);
const UNKNOWN: Technique = (Slot::Hidden, [Slot::Hidden, Slot::Hidden]);

struct Template {
    branches: [[Technique; 3]; 2],
    combined: Slot,
    /// Translation keys of the branch titles.
    titles: [&'static str; 2],
}

const UNKNOWN_BRANCH: &str = "notebook.mistbrn.branch.unknown";

/// Iron and steel: the «Anchor» branch holds the catalog's nodes (P5); the rest is to come.
const IRON_STEEL: Template = Template {
    branches: [
        [
            (
                Slot::Innate,
                [Slot::Node(NODE_REACH), Slot::Node(NODE_ACCEL)],
            ),
            (Slot::Node(NODE_COIN_JUMP), [Slot::Hidden, Slot::Hidden]),
            UNKNOWN,
        ],
        [UNKNOWN, UNKNOWN, UNKNOWN],
    ],
    combined: Slot::Hidden,
    titles: ["notebook.mistbrn.branch.anchor", UNKNOWN_BRANCH],
};

/// The three pairs whose powers come later: the shape only.
const LATER: Template = Template {
    branches: [[UNKNOWN, UNKNOWN, UNKNOWN], [UNKNOWN, UNKNOWN, UNKNOWN]],
    combined: Slot::Hidden,
    titles: [UNKNOWN_BRANCH, UNKNOWN_BRANCH],
};

/// The general tree: economy and capacity (P5), then four to come (flaring cost,
/// simultaneous burning, transition, dosing).
const GENERAL_SLOTS: [Slot; 6] = [
    Slot::Node(NODE_ECONOMY),
    Slot::Node(NODE_CAPACITY),
    Slot::Hidden,
    Slot::Hidden,
    Slot::Hidden,
    Slot::Hidden,
];

/// The branch titles of tree `view` (none for the general tree).
pub(crate) fn branch_titles(view: u8) -> Option<[&'static str; 2]> {
    match view {
        0 => Some(IRON_STEEL.titles),
        GENERAL => None,
        _ => Some(LATER.titles),
    }
}

/// The tab buttons (local).
pub(crate) fn tab_rect(tab: Tab) -> [f32; 4] {
    let i = TABS.iter().position(|t| *t == tab).unwrap_or(0) as f32;
    [6.0 + 64.0 * i, -TABS_H + 1.0, 60.0, 14.0]
}

/// The tree selector's buttons.
pub(crate) fn view_rect(view: u8) -> [f32; 4] {
    [6.0, 8.0 + 20.0 * f32::from(view), 100.0, 18.0]
}

/// The tree canvas's window (local) and the diary list's.
pub(crate) const CANVAS: [f32; 4] = [112.0, 30.0, 226.0, 164.0];
pub(crate) const LIST: [f32; 4] = [6.0, 8.0, 112.0, 186.0];
/// A row of the diary list: the name in up to two lines.
pub(crate) const LIST_ROW: f32 = 30.0;
/// The diary's detail column starts here.
pub(crate) const DETAIL_X: f32 = 126.0;
pub(crate) const DETAIL_W: f32 = PANEL_W - 6.0 - DETAIL_X;
pub(crate) const HINT_BUTTON: [f32; 4] = [DETAIL_X + DETAIL_W - 96.0, 84.0, 96.0, 12.0];
const HINT_PAGE: [f32; 2] = [DETAIL_X + 54.0, 84.0];
pub(crate) const CONCLUDE_BUTTON: [f32; 4] = [DETAIL_X, PANEL_H - 22.0, DETAIL_W, 14.0];

/// Rows of the branch trees, in canvas pixels.
pub(crate) const ROW_TOP: f32 = 16.0;
const ROW_STEP: f32 = 56.0;
const SUB_DX: f32 = 27.0;
/// The branches' centres across the canvas.
pub(crate) const BRANCH_X: [f32; 2] = [56.0, 170.0];

pub(crate) fn hint_page_rect(index: u8) -> [f32; 4] {
    let x = HINT_PAGE[0] + 14.0 * f32::from(index.max(1) - 1);
    [x, HINT_PAGE[1], 12.0, 12.0]
}

/// One drawn place of a tree, in canvas pixels (before scrolling).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Place {
    pub(crate) target: Target,
    pub(crate) at: [f32; 2],
}

/// A line between two places' centres, and the place it leads to (lit once that is owned).
pub(crate) type Link = ([f32; 2], [f32; 2], Target);

fn centre(at: [f32; 2]) -> [f32; 2] {
    [at[0] + NODE / 2.0, at[1] + NODE / 2.0]
}

/// The places and links of tree `view`, and the canvas height they take.
pub(crate) fn tree_layout(view: u8) -> (Vec<Place>, Vec<Link>, f32) {
    let mut places = Vec::new();
    let mut links = Vec::new();
    if view == GENERAL {
        for (i, slot) in GENERAL_SLOTS.iter().enumerate() {
            let at = [
                38.0 + 75.0 * (i % 3) as f32 - NODE / 2.0,
                ROW_TOP + 60.0 * (i / 3) as f32,
            ];
            places.push(Place {
                target: slot.target(),
                at,
            });
        }
        return (places, links, ROW_TOP + 60.0 + NODE + 20.0);
    }
    let template = if view == 0 { &IRON_STEEL } else { &LATER };
    let combined_at = [
        (BRANCH_X[0] + BRANCH_X[1]) / 2.0 - NODE / 2.0,
        ROW_TOP + ROW_STEP * 3.0,
    ];
    for (b, branch) in template.branches.iter().enumerate() {
        let x = BRANCH_X[b] - NODE / 2.0;
        for (row, (technique, subs)) in branch.iter().enumerate() {
            let at = [x, ROW_TOP + ROW_STEP * row as f32];
            places.push(Place {
                target: technique.target(),
                at,
            });
            for (side, sub) in subs.iter().enumerate() {
                let dx = if side == 0 { -SUB_DX } else { SUB_DX };
                let sub_at = [at[0] + dx, at[1] + 6.0];
                places.push(Place {
                    target: sub.target(),
                    at: sub_at,
                });
                links.push((centre(at), centre(sub_at), sub.target()));
            }
            let (next, into) = match branch.get(row + 1) {
                Some((next, _)) => ([x, ROW_TOP + ROW_STEP * (row + 1) as f32], next.target()),
                None => (combined_at, template.combined.target()),
            };
            links.push((centre(at), centre(next), into));
        }
    }
    places.push(Place {
        target: template.combined.target(),
        at: combined_at,
    });
    (places, links, combined_at[1] + NODE + 14.0)
}

/// How far the tree canvas or the diary list can scroll.
pub(crate) fn max_scroll(tab: Tab, view: u8) -> f32 {
    match tab {
        Tab::Tree => (tree_layout(view).2 - CANVAS[3]).max(0.0),
        Tab::Diary => (CATALOG.research.len() as f32 * LIST_ROW - LIST[3]).max(0.0),
        _ => 0.0,
    }
}

/// The open book's state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Book {
    pub(crate) tab: Tab,
    pub(crate) view: u8,
    /// Tree canvas and diary list scroll, in local pixels.
    pub(crate) scroll: f32,
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
    pub(crate) fn new() -> Self {
        Self {
            tab: Tab::Tree,
            view: 0,
            scroll: 0.0,
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
    rect[1] >= clip[1] && rect[1] + rect[3] <= clip[1] + clip[3]
}

/// Every target of the book as it stands, in focus order, with its local rectangle and
/// whether it shows (a canvas node scrolled out of the window does not, but keeps its place
/// in the focus order).
pub(crate) fn targets(book: &Book, progress: &Progress) -> Vec<(Target, [f32; 4], bool)> {
    let mut out: Vec<(Target, [f32; 4], bool)> = TABS
        .iter()
        .map(|&t| (Target::Tab(t), tab_rect(t), true))
        .collect();
    match book.tab {
        Tab::Tree => {
            out.extend((0..VIEWS).map(|v| (Target::View(v), view_rect(v), true)));
            for place in tree_layout(book.view).0 {
                let rect = [
                    CANVAS[0] + place.at[0],
                    CANVAS[1] + place.at[1] - book.scroll,
                    NODE,
                    NODE,
                ];
                out.push((place.target, rect, shows(rect, CANVAS)));
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
            None
        }
        Target::View(view) => {
            if book.view != view {
                book.view = view;
                book.scroll = 0.0;
            }
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
        Target::Innate | Target::Hidden => None,
        Target::Node(code) => {
            let def = node_def(code, &CATALOG)?;
            (node_status(progress, def, unlocked, &CATALOG) == NodeStatus::Available).then(|| {
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

/// Scrolls the window under the local point `at` (`None`: the tab's own window) by `lines`
/// notches of the wheel (positive: up, as the game counts). True when the window is there.
pub(crate) fn scroll(book: &mut Book, at: Option<[f32; 2]>, lines: f32) -> bool {
    const NOTCH: f32 = 16.0;
    let max = max_scroll(book.tab, book.view);
    let (value, window) = match book.tab {
        Tab::Tree => (&mut book.scroll, CANVAS),
        Tab::Diary => (&mut book.list_scroll, LIST),
        _ => return false,
    };
    if at.is_some_and(|at| !inside(at, window)) {
        return false;
    }
    *value = (*value - lines * NOTCH).clamp(0.0, max);
    true
}

/// A key the book reads while open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Key {
    Next,
    Previous,
    Activate,
    PageUp,
    PageDown,
}

/// Vanilla's key names (`screen_key_pressed_pre`) to the book's keys. Shift+Tab goes back.
pub(crate) fn key(name: &str, shift: bool) -> Option<Key> {
    Some(match name {
        "key.keyboard.tab" if shift => Key::Previous,
        "key.keyboard.tab" | "key.keyboard.down" | "key.keyboard.right" => Key::Next,
        "key.keyboard.up" | "key.keyboard.left" => Key::Previous,
        "key.keyboard.enter" | "key.keyboard.keypad.enter" | "key.keyboard.space" => Key::Activate,
        "key.keyboard.page.up" => Key::PageUp,
        "key.keyboard.page.down" => Key::PageDown,
        _ => return None,
    })
}

/// Moves the focus to the next or previous focusable target (wrapping round) and scrolls the
/// canvas or list so it shows.
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
    book.focus = Some(target);
    book.keyboard = true;
    let max = max_scroll(book.tab, book.view);
    let (value, window) = match target {
        Target::Node(_) => (&mut book.scroll, CANVAS),
        Target::Research(_) => (&mut book.list_scroll, LIST),
        _ => return,
    };
    // `rect` is already scrolled: move by how far it sticks out.
    let top = window[1] + 2.0;
    let bottom = window[1] + window[3] - 2.0;
    if rect[1] < top {
        *value -= top - rect[1];
    } else if rect[1] + rect[3] > bottom {
        *value += rect[1] + rect[3] - bottom;
    }
    *value = value.clamp(0.0, max);
}

/// The milestone that unlocks `pair`, if one does.
pub(crate) fn unlocking_milestone(pair: Pair) -> Option<&'static allomancy_core::MilestoneDef> {
    CATALOG.milestones.iter().find(|m| m.unlocks == Some(pair))
}

/// The pair of tree `view` (`None` for the general tree).
pub(crate) fn view_pair(view: u8) -> Option<Pair> {
    PAIRS.get(usize::from(view)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_node_has_one_place_and_places_stay_on_the_canvas() {
        let mut seen = Vec::new();
        for view in 0..VIEWS {
            let (places, _, height) = tree_layout(view);
            for place in &places {
                assert!(
                    place.at[0] >= 0.0 && place.at[0] + NODE <= CANVAS[2],
                    "{place:?}"
                );
                assert!(
                    place.at[1] >= 0.0 && place.at[1] + NODE <= height,
                    "{place:?}"
                );
                if let Target::Node(code) = place.target {
                    seen.push(code);
                }
            }
            let expected = if view == GENERAL { 6 } else { 19 };
            assert_eq!(places.len(), expected, "view {view}");
        }
        seen.sort_unstable();
        let mut catalog: Vec<u16> = CATALOG.nodes.iter().map(|d| d.code).collect();
        catalog.sort_unstable();
        assert_eq!(seen, catalog);
    }

    #[test]
    fn focus_wraps_and_scrolls_the_canvas_to_the_focused_node() {
        let progress = Progress::default();
        let mut book = Book::new();
        let focusable = targets(&book, &progress)
            .iter()
            .filter(|(t, ..)| t.focusable())
            .count();
        // 4 tabs, 5 trees, reach, accel and coin jump.
        assert_eq!(focusable, 12);
        for _ in 0..focusable {
            move_focus(&mut book, &progress, true);
        }
        assert_eq!(book.focus, Some(Target::Node(NODE_COIN_JUMP)));
        move_focus(&mut book, &progress, true);
        assert_eq!(book.focus, Some(Target::Tab(Tab::Tree)));
        move_focus(&mut book, &progress, false);
        assert_eq!(book.focus, Some(Target::Node(NODE_COIN_JUMP)));
        assert!(book.scroll >= 0.0 && book.scroll <= max_scroll(Tab::Tree, 0));
    }

    #[test]
    fn scrolling_is_clamped_and_only_over_the_window() {
        let mut book = Book::new();
        let max = max_scroll(Tab::Tree, 0);
        assert!(
            max > 0.0,
            "the iron and steel tree is taller than its window"
        );
        assert!(!scroll(&mut book, Some([0.0, 0.0]), -1.0));
        assert!(scroll(
            &mut book,
            Some([CANVAS[0] + 1.0, CANVAS[1] + 1.0]),
            -100.0
        ));
        assert_eq!(book.scroll, max);
        assert!(scroll(&mut book, None, 100.0));
        assert_eq!(book.scroll, 0.0);
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
        assert_eq!(key("key.keyboard.j", false), None);
    }
}
