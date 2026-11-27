//! The skill map (P8): one radial canvas instead of a tree per pair.
//!
//! The centre holds the general nodes (knowledge points); four sectors around it hold the
//! pairs, placed as the allomantic table places them: physical above, mental below, external
//! on the left, internal on the right. Each sector has its pulling metal's branch and its
//! pushing metal's branch (three techniques, the first the metal's own power, each with two
//! sub-nodes) and the combined technique at its far end. Bridge nodes sit on the borders of
//! neighbouring sectors. Only the catalog's nodes are real; the other places are silhouettes.
//!
//! Places live in map units, the canvas centre showing the view's `pan` at `zoom` local
//! pixels per unit. Nothing here reaches the server: positions are the book's alone.
use super::{CANVAS, Target};
use allomancy_core::{NODE_ACCEL, NODE_CAPACITY, NODE_COIN_JUMP, NODE_DOSING, NODE_ECONOMY};
use allomancy_core::{
    NODE_BRASS_DURATION, NODE_BRASS_RANGE, NODE_COURAGE, NODE_COURAGE_DURATION, NODE_COURAGE_LEVEL,
    NODE_OUTER_REACH, NODE_RETREAT, NODE_RETREAT_DISTANCE, NODE_RETREAT_SPEED, NODE_SELECTIVE,
    NODE_ZINC_DURATION, NODE_ZINC_RANGE,
};
use allomancy_core::{
    NODE_FLARE_COST, NODE_HEARING, NODE_HEARING_MEMORY, NODE_HEARING_RANGE, NODE_JUMP_CUSHION,
    NODE_JUMP_HEIGHT, NODE_PEWTER_ENDURANCE, NODE_PEWTER_JUMP, NODE_PEWTER_STRENGTH, NODE_REACH,
    NODE_REFLEXES, NODE_SIMULTANEOUS, NODE_STEEL_BODY, NODE_TIN_CLARITY, NODE_TIN_RANGE,
    NODE_TRANSITION, Pair,
};

/// A node's side in map units (the frame sprite's 22 pixels at 100 %).
pub(crate) const NODE_U: f32 = 22.0;
pub(crate) const ZOOM_MIN: f32 = 0.5;
pub(crate) const ZOOM_MAX: f32 = 2.0;
/// How far the view's centre may go from the map's (past the combined techniques).
pub(crate) const PAN_LIMIT: f32 = 340.0;
/// Radii of the general ring, the three techniques, the sub-nodes' offset and the combined.
const GENERAL_R: f32 = 48.0;
const TECHNIQUE_R: [f32; 3] = [110.0, 180.0, 250.0];
const SUB_DR: f32 = 30.0;
const SUB_DEG: f32 = 12.0;
const BRANCH_DEG: f32 = 20.0;
const COMBINED_R: f32 = 330.0;
const BRIDGE_R: f32 = 180.0;
/// Where a sector's name goes, between its two branches.
pub(crate) const LABEL_R: f32 = 215.0;

/// A place of the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    Node(u16),
    /// The branch metal's own power: owned with the pair, not a catalog node.
    Innate,
    Hidden,
}

/// A technique and its two sub-nodes.
type Technique = (Slot, [Slot; 2]);
const UNKNOWN: Technique = (Slot::Hidden, [Slot::Hidden, Slot::Hidden]);
const INNATE: Technique = (Slot::Innate, [Slot::Hidden, Slot::Hidden]);

/// A sector: the pulling branch, the pushing branch, the combined technique.
struct Sector {
    pair: Pair,
    /// The sector's middle, in degrees (screen: 0 to the right, 90 down).
    bisector: f32,
    branches: [[Technique; 3]; 2],
    combined: Slot,
}

/// Iron and steel: the P5 nodes keep their codes; reach and acceleration hang from the pull
/// (they apply to both metals), the coin jump is steel's second technique. Every pair's first
/// metal (`Pair::metals`) is its pulling one.
const SECTORS: [Sector; 4] = [
    Sector {
        pair: Pair::IronSteel,
        bisector: -135.0,
        branches: [
            [
                (
                    Slot::Innate,
                    [Slot::Node(NODE_REACH), Slot::Node(NODE_ACCEL)],
                ),
                UNKNOWN,
                UNKNOWN,
            ],
            [
                INNATE,
                (Slot::Node(NODE_COIN_JUMP), [Slot::Hidden; 2]),
                UNKNOWN,
            ],
        ],
        combined: Slot::Hidden,
    },
    // Tin and pewter (P8): each innate power with its two sub-nodes, the second techniques
    // (hearing, pewter's jump) with theirs, and Reflexes at the far end.
    Sector {
        pair: Pair::TinPewter,
        bisector: -45.0,
        branches: [
            [
                (
                    Slot::Innate,
                    [Slot::Node(NODE_TIN_RANGE), Slot::Node(NODE_TIN_CLARITY)],
                ),
                (
                    Slot::Node(NODE_HEARING),
                    [
                        Slot::Node(NODE_HEARING_RANGE),
                        Slot::Node(NODE_HEARING_MEMORY),
                    ],
                ),
                UNKNOWN,
            ],
            [
                (
                    Slot::Innate,
                    [
                        Slot::Node(NODE_PEWTER_STRENGTH),
                        Slot::Node(NODE_PEWTER_ENDURANCE),
                    ],
                ),
                (
                    Slot::Node(NODE_PEWTER_JUMP),
                    [Slot::Node(NODE_JUMP_HEIGHT), Slot::Node(NODE_JUMP_CUSHION)],
                ),
                UNKNOWN,
            ],
        ],
        combined: Slot::Node(NODE_REFLEXES),
    },
    Sector {
        pair: Pair::CopperBronze,
        bisector: 45.0,
        branches: [[INNATE, UNKNOWN, UNKNOWN], [INNATE, UNKNOWN, UNKNOWN]],
        combined: Slot::Hidden,
    },
    // Zinc and brass (P8): each innate power with its two sub-nodes, courage and retreat with
    // theirs, and selective influence at the far end.
    Sector {
        pair: Pair::ZincBrass,
        bisector: 135.0,
        branches: [
            [
                (
                    Slot::Innate,
                    [Slot::Node(NODE_ZINC_RANGE), Slot::Node(NODE_ZINC_DURATION)],
                ),
                (
                    Slot::Node(NODE_COURAGE),
                    [
                        Slot::Node(NODE_COURAGE_LEVEL),
                        Slot::Node(NODE_COURAGE_DURATION),
                    ],
                ),
                UNKNOWN,
            ],
            [
                (
                    Slot::Innate,
                    [
                        Slot::Node(NODE_BRASS_RANGE),
                        Slot::Node(NODE_BRASS_DURATION),
                    ],
                ),
                (
                    Slot::Node(NODE_RETREAT),
                    [
                        Slot::Node(NODE_RETREAT_DISTANCE),
                        Slot::Node(NODE_RETREAT_SPEED),
                    ],
                ),
                UNKNOWN,
            ],
        ],
        combined: Slot::Node(NODE_SELECTIVE),
    },
];

/// The general ring, clockwise from the top: consumption, flare cost, simultaneous burning,
/// capacity, dosing, transition.
const GENERAL_SLOTS: [Slot; 6] = [
    Slot::Node(NODE_ECONOMY),
    Slot::Node(NODE_FLARE_COST),
    Slot::Node(NODE_SIMULTANEOUS),
    Slot::Node(NODE_CAPACITY),
    Slot::Node(NODE_DOSING),
    Slot::Node(NODE_TRANSITION),
];

/// The bridges, on the borders between neighbouring sectors: (border angle, the two sectors'
/// indices in `SECTORS`, the node). The others' codes arrive with their pairs' branches.
const BRIDGES: [(f32, usize, usize, Slot); 4] = [
    (-90.0, 0, 1, Slot::Node(NODE_STEEL_BODY)),
    (0.0, 1, 2, Slot::Hidden),
    (90.0, 2, 3, Slot::Hidden),
    (180.0, 3, 0, Slot::Node(NODE_OUTER_REACH)),
];

/// One drawn place: its target, its centre in map units, and the sector it belongs to
/// (`None`: the general ring or a bridge).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Place {
    pub(crate) target: Target,
    pub(crate) at: [f32; 2],
    pub(crate) sector: Option<Pair>,
}

/// A line between two places' centres (map units) and the place it leads to (lit once owned).
pub(crate) type Link = ([f32; 2], [f32; 2], Target);

pub(crate) fn polar(radius: f32, degrees: f32) -> [f32; 2] {
    let (sin, cos) = degrees.to_radians().sin_cos();
    [radius * cos, radius * sin]
}

/// The middle of `pair`'s sector, in degrees.
pub(crate) fn bisector(pair: Pair) -> f32 {
    SECTORS
        .iter()
        .find(|s| s.pair == pair)
        .map_or(0.0, |s| s.bisector)
}

/// The sectors in the order Page Down visits them, clockwise from the top left.
pub(crate) const SECTOR_ORDER: [Pair; 4] = [
    Pair::IronSteel,
    Pair::TinPewter,
    Pair::CopperBronze,
    Pair::ZincBrass,
];

/// Where the view goes to show `pair`'s sector: around its first techniques.
pub(crate) fn sector_view(pair: Pair) -> [f32; 2] {
    polar(130.0, bisector(pair))
}

fn target(slot: Slot, pair: Pair, branch: usize) -> Target {
    match slot {
        Slot::Node(code) => Target::Node(code),
        Slot::Innate => Target::Innate(pair.metals()[branch]),
        Slot::Hidden => Target::Hidden,
    }
}

/// Every place and link of the map.
pub(crate) fn map_layout() -> (Vec<Place>, Vec<Link>) {
    let mut places = Vec::new();
    let mut links = Vec::new();
    for (i, slot) in GENERAL_SLOTS.iter().enumerate() {
        places.push(Place {
            target: target(*slot, Pair::IronSteel, 0),
            at: polar(GENERAL_R, -90.0 + 60.0 * i as f32),
            sector: None,
        });
    }
    // The second technique of each sector's branches, for the bridges.
    let mut seconds = [[[0.0f32; 2]; 2]; 4];
    for (s, sector) in SECTORS.iter().enumerate() {
        let combined = polar(COMBINED_R, sector.bisector);
        for (b, branch) in sector.branches.iter().enumerate() {
            let angle = sector.bisector + if b == 0 { -BRANCH_DEG } else { BRANCH_DEG };
            for (row, (technique, subs)) in branch.iter().enumerate() {
                let at = polar(TECHNIQUE_R[row], angle);
                if row == 1 {
                    seconds[s][b] = at;
                }
                places.push(Place {
                    target: target(*technique, sector.pair, b),
                    at,
                    sector: Some(sector.pair),
                });
                for (side, sub) in subs.iter().enumerate() {
                    let offset = if side == 0 { -SUB_DEG } else { SUB_DEG };
                    let sub_at = polar(TECHNIQUE_R[row] + SUB_DR, angle + offset);
                    let sub_target = target(*sub, sector.pair, b);
                    places.push(Place {
                        target: sub_target,
                        at: sub_at,
                        sector: Some(sector.pair),
                    });
                    links.push((at, sub_at, sub_target));
                }
                let (next, into) = match branch.get(row + 1) {
                    Some((next, _)) => (
                        polar(TECHNIQUE_R[row + 1], angle),
                        target(*next, sector.pair, b),
                    ),
                    None => (combined, target(sector.combined, sector.pair, b)),
                };
                links.push((at, next, into));
            }
        }
        places.push(Place {
            target: target(sector.combined, sector.pair, 0),
            at: combined,
            sector: Some(sector.pair),
        });
    }
    for &(angle, a, b, slot) in &BRIDGES {
        let at = polar(BRIDGE_R, angle);
        let bridge = target(slot, Pair::IronSteel, 0);
        places.push(Place {
            target: bridge,
            at,
            sector: None,
        });
        // A bridge joins the facing branches: each sector's second technique nearer the border.
        let near = |s: usize| {
            let distance = |p: [f32; 2]| (p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2);
            let [first, second] = seconds[s];
            if distance(first) <= distance(second) {
                first
            } else {
                second
            }
        };
        // A catalog bridge's lines come from the nodes it requires (they are its path).
        let required: Vec<[f32; 2]> = match slot {
            Slot::Node(code) => allomancy_core::node_def(code, &allomancy_core::CATALOG)
                .map(|def| def.requires)
                .unwrap_or_default()
                .iter()
                .filter_map(|requirement| match *requirement {
                    allomancy_core::Requirement::Node { code, .. } => places
                        .iter()
                        .find(|p| p.target == Target::Node(code))
                        .map(|p| p.at),
                    allomancy_core::Requirement::Discovery { .. } => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        if required.len() == 2 {
            for from in required {
                links.push((from, at, bridge));
            }
        } else {
            links.push((near(a), at, bridge));
            links.push((near(b), at, bridge));
        }
    }
    (places, links)
}

/// The view of the map: the map point at the canvas centre and the local pixels per unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    pub(crate) pan: [f32; 2],
    pub(crate) zoom: f32,
}

impl View {
    /// Opens on `pair`'s sector at 100 %.
    pub(crate) fn on(pair: Pair) -> Self {
        Self {
            pan: sector_view(pair),
            zoom: 1.0,
        }
    }

    fn centre_local() -> [f32; 2] {
        [CANVAS[0] + CANVAS[2] / 2.0, CANVAS[1] + CANVAS[3] / 2.0]
    }

    /// A map point to the book's local pixels.
    pub(crate) fn to_local(self, [x, y]: [f32; 2]) -> [f32; 2] {
        let [cx, cy] = Self::centre_local();
        [
            cx + (x - self.pan[0]) * self.zoom,
            cy + (y - self.pan[1]) * self.zoom,
        ]
    }

    /// A local point back to map units.
    pub(crate) fn to_map(self, [x, y]: [f32; 2]) -> [f32; 2] {
        let [cx, cy] = Self::centre_local();
        [
            self.pan[0] + (x - cx) / self.zoom,
            self.pan[1] + (y - cy) / self.zoom,
        ]
    }

    /// A node's local rectangle.
    pub(crate) fn node_rect(self, at: [f32; 2]) -> [f32; 4] {
        let [x, y] = self.to_local(at);
        let side = NODE_U * self.zoom;
        [x - side / 2.0, y - side / 2.0, side, side]
    }

    /// Moves the view by `delta` local pixels (a drag: the map follows the pointer).
    pub(crate) fn drag(&mut self, delta: [f32; 2]) {
        self.pan = clamp_pan([
            self.pan[0] - delta[0] / self.zoom,
            self.pan[1] - delta[1] / self.zoom,
        ]);
    }

    /// Zooms by `factor`, keeping the map point under the local point `at` (the canvas centre
    /// when `None`) where it is.
    pub(crate) fn zoom_at(&mut self, factor: f32, at: Option<[f32; 2]>) {
        let at = at.unwrap_or_else(Self::centre_local);
        let before = self.to_map(at);
        self.zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        let after = self.to_map(at);
        self.pan = clamp_pan([
            self.pan[0] + before[0] - after[0],
            self.pan[1] + before[1] - after[1],
        ]);
    }

    /// Centres the view on the map point `at`.
    pub(crate) fn centre(&mut self, at: [f32; 2]) {
        self.pan = clamp_pan(at);
    }
}

fn clamp_pan([x, y]: [f32; 2]) -> [f32; 2] {
    let length = (x * x + y * y).sqrt();
    if length <= PAN_LIMIT {
        return [x, y];
    }
    [x / length * PAN_LIMIT, y / length * PAN_LIMIT]
}

/// The place nearest `from` in direction `dir` (a unit vector, screen y down), within 30° of
/// it, among `candidates`. Nearer along the direction wins; a sideways one counts as farther.
pub(crate) fn nearest_towards(
    from: [f32; 2],
    dir: [f32; 2],
    candidates: impl Iterator<Item = (Target, [f32; 2])>,
) -> Option<Target> {
    let cos_limit = 30f32.to_radians().cos();
    candidates
        .filter_map(|(target, at)| {
            let d = [at[0] - from[0], at[1] - from[1]];
            let length = (d[0] * d[0] + d[1] * d[1]).sqrt();
            let along = (d[0] * dir[0] + d[1] * dir[1]) / length.max(f32::EPSILON);
            (length > 1.0 && along >= cos_limit).then_some((target, length / along))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(target, _)| target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use allomancy_core::{CATALOG, Pool, node_def};

    // Catches two places drawn over each other, or a place past the reachable view.
    #[test]
    fn places_never_overlap_and_stay_within_reach() {
        let (places, _) = map_layout();
        for (i, a) in places.iter().enumerate() {
            let r = (a.at[0].powi(2) + a.at[1].powi(2)).sqrt();
            assert!(r <= PAN_LIMIT, "{a:?}");
            for b in &places[i + 1..] {
                let d = ((a.at[0] - b.at[0]).powi(2) + (a.at[1] - b.at[1]).powi(2)).sqrt();
                assert!(d >= NODE_U + 6.0, "{a:?} and {b:?} are {d} apart");
            }
        }
    }

    // Catches a catalog node with no place (it could never be bought) or with two.
    #[test]
    fn every_catalog_node_has_exactly_one_place() {
        let (places, _) = map_layout();
        let mut seen: Vec<u16> = places
            .iter()
            .filter_map(|p| match p.target {
                Target::Node(code) => Some(code),
                _ => None,
            })
            .collect();
        seen.sort_unstable();
        let mut catalog: Vec<u16> = CATALOG.nodes.iter().map(|d| d.code).collect();
        catalog.sort_unstable();
        assert_eq!(seen, catalog);
        // 6 general + 4 sectors × (2 branches × 3 × 3 + combined) + 4 bridges.
        assert_eq!(places.len(), 6 + 4 * 19 + 4);
    }

    // Catches a node drawn in another sector than the one whose points it costs.
    #[test]
    fn pair_nodes_sit_in_their_own_sector() {
        let (places, _) = map_layout();
        for place in &places {
            if let Target::Node(code) = place.target {
                let def = node_def(code, &CATALOG).unwrap();
                let expected = match def.pool {
                    Pool::Pair(pair) => Some(pair),
                    _ => None,
                };
                assert_eq!(place.sector, expected, "node {code}");
            }
        }
    }

    #[test]
    fn the_view_maps_both_ways_and_zooms_round_the_pointer() {
        let mut view = View::on(Pair::IronSteel);
        let point = [-80.0, -60.0];
        let local = view.to_local(point);
        let back = view.to_map(local);
        assert!((back[0] - point[0]).abs() < 1e-3 && (back[1] - point[1]).abs() < 1e-3);
        view.zoom_at(2.0, Some(local));
        assert_eq!(view.zoom, 2.0);
        let still = view.to_local(point);
        assert!((still[0] - local[0]).abs() < 1e-3 && (still[1] - local[1]).abs() < 1e-3);
        view.zoom_at(10.0, None);
        assert_eq!(view.zoom, ZOOM_MAX);
        view.zoom_at(0.01, None);
        assert_eq!(view.zoom, ZOOM_MIN);
        view.drag([-10_000.0, 0.0]);
        let r = (view.pan[0].powi(2) + view.pan[1].powi(2)).sqrt();
        assert!((r - PAN_LIMIT).abs() < 1e-2);
    }

    #[test]
    fn an_arrow_finds_the_nearest_place_in_its_direction() {
        let candidates = [
            (Target::Node(1), [10.0, 0.0]),
            (Target::Node(2), [30.0, 1.0]),
            (Target::Node(3), [0.0, -20.0]),
            (Target::Node(16), [-5.0, 40.0]),
        ];
        let pick = |dir| nearest_towards([0.0, 0.0], dir, candidates.iter().copied());
        assert_eq!(pick([1.0, 0.0]), Some(Target::Node(1)));
        assert_eq!(pick([0.0, -1.0]), Some(Target::Node(3)));
        assert_eq!(pick([0.0, 1.0]), Some(Target::Node(16)));
        assert_eq!(pick([-1.0, 0.0]), None);
    }
}
