//! Progression (P5): practice experience, skill nodes and research, with the catalog as data.
//!
//! The catalog ([`CATALOG`]) names every node, research and practice category by a stable
//! code that is never reused: saves store codes, so a code's meaning is part of the format.
//! What a player owns lives in [`Progress`] inside the save; every change goes through
//! `persistence` (`observe`, `prepare_purchase`, `prepare_conclude`, `prepare_hint`), which
//! validates the whole save before replacing it.
//!
//! Points are never stored as a balance: a pool's earned points follow from its experience
//! (pair) or concluded research (knowledge), and its spent points from the ranks owned.
use crate::physics::ForceRules;
use crate::{Pair, PlayerState, ReserveRules};
use std::collections::BTreeMap;

/// Practice categories saved per player (one window each).
pub const PRACTICE_CATEGORIES: usize = 3;
/// The largest code a node or research may use: requests carry `code << 2 | index` in a byte.
pub const MAX_CODE: u16 = 63;

/// Where a node's cost is paid from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    /// Experience points of one pair; never transferred to another.
    Pair(Pair),
    /// General knowledge, granted only by concluded research.
    Knowledge,
}

/// What a node's current rank does. `Rank::value` is absolute for that rank, not a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeEffect {
    /// Extra range of iron and steel, in blocks.
    ReachBlocks,
    /// Extra acceleration of iron and steel, in percent.
    AccelPercent,
    /// Extra acceleration when pushing an anchor below the feet, in percent.
    PushBelowPercent,
    /// `ReserveRules::reduction_milli`.
    ReductionMilli,
    /// `ReserveRules::capacity_bonus_milli`.
    CapacityBonusMilli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    /// Another node at this rank or above.
    Node { code: u16, rank: u8 },
    /// A concluded research.
    Discovery { code: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rank {
    pub cost: u16,
    pub value: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeDef {
    pub code: u16,
    /// Stable name for logs and translation keys (`node.mistbrn.<key>`).
    pub key: &'static str,
    pub pool: Pool,
    pub effect: NodeEffect,
    pub ranks: &'static [Rank],
    pub requires: &'static [Requirement],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResearchDef {
    pub code: u16,
    pub key: &'static str,
    /// The pair whose experience the conclusion grants (and that must be unlocked).
    pub pair: Pair,
    /// Bits of the evidence that counts; any `needed` of them make it ready.
    pub evidence: u8,
    pub needed: u8,
    /// Voluntary hints, revealed one by one.
    pub hints: u8,
    pub pair_xp: u32,
    pub knowledge: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PracticeDef {
    pub category: u8,
    pub key: &'static str,
    pub pair: Pair,
    pub xp: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub nodes: &'static [NodeDef],
    pub research: &'static [ResearchDef],
    pub practice: &'static [PracticeDef],
    /// Length of a practice window, in game-time ticks.
    pub window_ticks: u64,
    /// Share of a category's experience for the 1st, 2nd, ... action in one window; 0 after.
    pub falloff_percent: &'static [u8],
    /// Cumulative pair experience that earns each point, then one more every `point_step`.
    pub point_thresholds: &'static [u32],
    pub point_step: u32,
    /// Pairs whose first vial unlocks them (the tutorial).
    pub tutorial_pairs: &'static [Pair],
}

pub const NODE_REACH: u16 = 1;
pub const NODE_ACCEL: u16 = 2;
pub const NODE_COIN_JUMP: u16 = 3;
pub const NODE_ECONOMY: u16 = 16;
pub const NODE_CAPACITY: u16 = 17;
pub const RESEARCH_ANCHOR: u16 = 1;
/// Evidence of the anchor principle: a loose metal item pulled into the hand.
pub const EVIDENCE_ITEM_PULLED: u8 = 1;
/// A pull towards a block anchor that moved the player.
pub const EVIDENCE_SELF_PULL: u8 = 2;
/// A push off an anchor below the feet that lifted the player.
pub const EVIDENCE_PUSH_LIFT: u8 = 4;
pub const PRACTICE_ANCHOR_MOVE: u8 = 0;
pub const PRACTICE_COIN_JUMP: u8 = 1;
pub const PRACTICE_COIN_HIT: u8 = 2;

const fn rank(cost: u16, value: u16) -> Rank {
    Rank { cost, value }
}

/// The first vertical slice (P5); trial values, not balance.
pub const CATALOG: Catalog = Catalog {
    nodes: &[
        NodeDef {
            code: NODE_REACH,
            key: "reach",
            pool: Pool::Pair(Pair::IronSteel),
            effect: NodeEffect::ReachBlocks,
            ranks: &[rank(1, 2), rank(2, 4), rank(3, 6)],
            requires: &[],
        },
        NodeDef {
            code: NODE_ACCEL,
            key: "accel",
            pool: Pool::Pair(Pair::IronSteel),
            effect: NodeEffect::AccelPercent,
            ranks: &[rank(1, 10), rank(2, 20), rank(3, 30)],
            requires: &[Requirement::Node {
                code: NODE_REACH,
                rank: 1,
            }],
        },
        NodeDef {
            code: NODE_COIN_JUMP,
            key: "coin_jump",
            pool: Pool::Pair(Pair::IronSteel),
            effect: NodeEffect::PushBelowPercent,
            ranks: &[rank(2, 25)],
            requires: &[Requirement::Discovery {
                code: RESEARCH_ANCHOR,
            }],
        },
        NodeDef {
            code: NODE_ECONOMY,
            key: "economy",
            pool: Pool::Knowledge,
            effect: NodeEffect::ReductionMilli,
            ranks: &[rank(1, 50), rank(2, 100)],
            requires: &[],
        },
        NodeDef {
            code: NODE_CAPACITY,
            key: "capacity",
            pool: Pool::Knowledge,
            effect: NodeEffect::CapacityBonusMilli,
            ranks: &[rank(1, 100), rank(2, 250)],
            requires: &[],
        },
    ],
    research: &[ResearchDef {
        code: RESEARCH_ANCHOR,
        key: "anchor_principle",
        pair: Pair::IronSteel,
        evidence: EVIDENCE_ITEM_PULLED | EVIDENCE_SELF_PULL | EVIDENCE_PUSH_LIFT,
        needed: 2,
        hints: 3,
        pair_xp: 200,
        knowledge: 3,
    }],
    practice: &[
        PracticeDef {
            category: PRACTICE_ANCHOR_MOVE,
            key: "anchor_move",
            pair: Pair::IronSteel,
            xp: 10,
        },
        PracticeDef {
            category: PRACTICE_COIN_JUMP,
            key: "coin_jump",
            pair: Pair::IronSteel,
            xp: 15,
        },
        PracticeDef {
            category: PRACTICE_COIN_HIT,
            key: "coin_hit",
            pair: Pair::IronSteel,
            xp: 10,
        },
    ],
    window_ticks: 1200,
    falloff_percent: &[100, 50, 25],
    point_thresholds: &[100, 250, 450, 700, 1000],
    point_step: 400,
    tutorial_pairs: &[Pair::IronSteel],
};

/// Why the catalog is unusable (a programming error, caught by tests).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogError {
    CodeOutOfRange(u16),
    DuplicateCode(u16),
    NoRanks(u16),
    TooManyRanks(u16),
    MissingRequirement(u16),
    RequirementRankOutOfRange(u16),
    Cycle(u16),
    EffectOutOfLimits(u16),
    BadResearch(u16),
    BadPractice(u8),
    BadThresholds,
}

/// One research's progress for a player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResearchState {
    pub evidence: u8,
    pub hints: u8,
    pub concluded: bool,
}

/// One practice category's window: when it opened (game time) and actions counted in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PracticeWindow {
    pub start: u64,
    pub count: u8,
}

/// Everything a player has learned. Kept on death and reconnect; changed only through the
/// functions of `persistence`, which validate it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub(crate) pair_xp: [u32; 4],
    pub(crate) nodes: BTreeMap<u16, u8>,
    pub(crate) research: BTreeMap<u16, ResearchState>,
    pub(crate) windows: [PracticeWindow; PRACTICE_CATEGORIES],
}

pub(crate) fn pair_index(pair: Pair) -> usize {
    match pair {
        Pair::IronSteel => 0,
        Pair::TinPewter => 1,
        Pair::ZincBrass => 2,
        Pair::CopperBronze => 3,
    }
}

impl Progress {
    /// A progress from its parts, for what the client is shown (never a save: saves come only
    /// from `decode_save`, which validates them). Zero ranks and default research are dropped.
    pub(crate) fn from_parts(
        pair_xp: [u32; 4],
        nodes: Vec<(u16, u8)>,
        research: Vec<(u16, ResearchState)>,
    ) -> Self {
        Self {
            pair_xp,
            nodes: nodes.into_iter().filter(|(_, rank)| *rank > 0).collect(),
            research: research
                .into_iter()
                .filter(|(_, entry)| *entry != ResearchState::default())
                .collect(),
            windows: [PracticeWindow::default(); PRACTICE_CATEGORIES],
        }
    }
    pub fn pair_xp(&self, pair: Pair) -> u32 {
        self.pair_xp[pair_index(pair)]
    }
    /// Owned rank of a node (0: not owned).
    pub fn rank(&self, code: u16) -> u8 {
        self.nodes.get(&code).copied().unwrap_or(0)
    }
    pub fn nodes(&self) -> &BTreeMap<u16, u8> {
        &self.nodes
    }
    pub fn research(&self, code: u16) -> ResearchState {
        self.research.get(&code).copied().unwrap_or_default()
    }
    pub fn all_research(&self) -> &BTreeMap<u16, ResearchState> {
        &self.research
    }
    pub fn windows(&self) -> &[PracticeWindow; PRACTICE_CATEGORIES] {
        &self.windows
    }
    pub fn discovered(&self, code: u16) -> bool {
        self.research(code).concluded
    }
    pub fn earned(&self, pool: Pool, catalog: &Catalog) -> u32 {
        match pool {
            Pool::Pair(pair) => points_for_xp(self.pair_xp(pair), catalog),
            Pool::Knowledge => catalog
                .research
                .iter()
                .filter(|def| self.discovered(def.code))
                .map(|def| u32::from(def.knowledge))
                .sum(),
        }
    }
    pub fn spent(&self, pool: Pool, catalog: &Catalog) -> u32 {
        catalog
            .nodes
            .iter()
            .filter(|def| def.pool == pool)
            .map(|def| {
                let owned = usize::from(self.rank(def.code)).min(def.ranks.len());
                def.ranks[..owned]
                    .iter()
                    .map(|r| u32::from(r.cost))
                    .sum::<u32>()
            })
            .sum()
    }
    /// Points that can still be spent from `pool` (never negative in a valid save).
    pub fn available(&self, pool: Pool, catalog: &Catalog) -> u32 {
        self.earned(pool, catalog)
            .saturating_sub(self.spent(pool, catalog))
    }
    /// The current value of an effect (0 when no node with it is owned).
    pub fn effect(&self, effect: NodeEffect, catalog: &Catalog) -> u16 {
        catalog
            .nodes
            .iter()
            .filter(|def| def.effect == effect)
            .filter_map(|def| {
                let owned = usize::from(self.rank(def.code));
                (owned > 0).then(|| def.ranks.get(owned - 1).map_or(0, |r| r.value))
            })
            .sum()
    }
}

/// Points earned by a pool with `xp` experience.
pub fn points_for_xp(xp: u32, catalog: &Catalog) -> u32 {
    let reached = catalog
        .point_thresholds
        .iter()
        .take_while(|&&t| t <= xp)
        .count() as u32;
    match catalog.point_thresholds.last() {
        Some(&last) if xp > last && catalog.point_step > 0 => {
            reached + (xp - last) / catalog.point_step
        }
        _ => reached,
    }
}

/// Experience still needed for the next point of a pool with `xp`.
pub fn xp_to_next_point(xp: u32, catalog: &Catalog) -> u32 {
    if let Some(&next) = catalog.point_thresholds.iter().find(|&&t| t > xp) {
        return next - xp;
    }
    let last = catalog.point_thresholds.last().copied().unwrap_or(0);
    let step = catalog.point_step.max(1);
    step - (xp - last) % step
}

pub fn node_def(code: u16, catalog: &Catalog) -> Option<&'static NodeDef> {
    catalog.nodes.iter().find(|def| def.code == code)
}

pub fn research_def(code: u16, catalog: &Catalog) -> Option<&'static ResearchDef> {
    catalog.research.iter().find(|def| def.code == code)
}

pub fn practice_def(category: u8, catalog: &Catalog) -> Option<&'static PracticeDef> {
    catalog.practice.iter().find(|def| def.category == category)
}

fn requirement_met(progress: &Progress, requirement: &Requirement) -> bool {
    match *requirement {
        Requirement::Node { code, rank } => progress.rank(code) >= rank,
        Requirement::Discovery { code } => progress.discovered(code),
    }
}

/// Every requirement of `def` is met by `progress`.
pub fn requirements_met(progress: &Progress, def: &NodeDef) -> bool {
    def.requires.iter().all(|r| requirement_met(progress, r))
}

pub fn validate_catalog(catalog: &Catalog) -> Result<(), CatalogError> {
    let mut codes = Vec::new();
    for def in catalog.nodes {
        if def.code == 0 || def.code > MAX_CODE {
            return Err(CatalogError::CodeOutOfRange(def.code));
        }
        if codes.contains(&def.code) {
            return Err(CatalogError::DuplicateCode(def.code));
        }
        codes.push(def.code);
        if def.ranks.is_empty() {
            return Err(CatalogError::NoRanks(def.code));
        }
        // A request carries the rank in two bits.
        if def.ranks.len() > 3 {
            return Err(CatalogError::TooManyRanks(def.code));
        }
        let max = def.ranks.iter().map(|r| r.value).max().unwrap_or(0);
        let limit = match def.effect {
            NodeEffect::ReachBlocks => 16,
            NodeEffect::AccelPercent | NodeEffect::PushBelowPercent => 100,
            NodeEffect::ReductionMilli => 200,
            NodeEffect::CapacityBonusMilli => 500,
        };
        if max > limit || def.ranks.windows(2).any(|w| w[1].value < w[0].value) {
            return Err(CatalogError::EffectOutOfLimits(def.code));
        }
    }
    // Effects that land in ReserveRules add up across nodes: their sum keeps the reserve limits.
    for (effect, limit) in [
        (NodeEffect::ReductionMilli, 200u32),
        (NodeEffect::CapacityBonusMilli, 500),
    ] {
        let total: u32 = catalog
            .nodes
            .iter()
            .filter(|d| d.effect == effect)
            .map(|d| u32::from(d.ranks.last().map_or(0, |r| r.value)))
            .sum();
        if total > limit {
            return Err(CatalogError::EffectOutOfLimits(0));
        }
    }
    let mut research_codes = Vec::new();
    for def in catalog.research {
        let valid = def.code != 0
            && def.code <= MAX_CODE
            && !research_codes.contains(&def.code)
            && def.evidence != 0
            && def.needed >= 1
            && u32::from(def.needed) <= def.evidence.count_ones()
            && def.hints <= 3;
        if !valid {
            return Err(CatalogError::BadResearch(def.code));
        }
        research_codes.push(def.code);
    }
    for def in catalog.nodes {
        for requirement in def.requires {
            match *requirement {
                Requirement::Node { code, rank } => {
                    let Some(other) = node_def(code, catalog) else {
                        return Err(CatalogError::MissingRequirement(def.code));
                    };
                    if rank == 0 || usize::from(rank) > other.ranks.len() {
                        return Err(CatalogError::RequirementRankOutOfRange(def.code));
                    }
                }
                Requirement::Discovery { code } => {
                    if research_def(code, catalog).is_none() {
                        return Err(CatalogError::MissingRequirement(def.code));
                    }
                }
            }
        }
        // Walk node requirements: reaching the start again is a cycle.
        let mut stack: Vec<u16> = vec![def.code];
        let mut seen: Vec<u16> = Vec::new();
        while let Some(code) = stack.pop() {
            let Some(node) = node_def(code, catalog) else {
                continue;
            };
            for requirement in node.requires {
                if let Requirement::Node { code: next, .. } = *requirement {
                    if next == def.code {
                        return Err(CatalogError::Cycle(def.code));
                    }
                    if !seen.contains(&next) {
                        seen.push(next);
                        stack.push(next);
                    }
                }
            }
        }
    }
    // Research has no requirements in this shape: it is observed from actions any player with
    // the pair unlocked can do, so no node can ever need the discovery it would itself open.
    let mut practice_seen = Vec::new();
    for def in catalog.practice {
        if usize::from(def.category) >= PRACTICE_CATEGORIES
            || practice_seen.contains(&def.category)
            || def.xp == 0
        {
            return Err(CatalogError::BadPractice(def.category));
        }
        practice_seen.push(def.category);
    }
    let increasing = catalog.point_thresholds.windows(2).all(|w| w[0] < w[1]);
    if catalog.point_thresholds.is_empty()
        || catalog.point_thresholds[0] == 0
        || !increasing
        || catalog.point_step == 0
        || catalog.window_ticks == 0
        || catalog.falloff_percent.iter().any(|&p| p > 100)
    {
        return Err(CatalogError::BadThresholds);
    }
    Ok(())
}

/// Why a progress value or a progress request is not valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressError {
    // Invalid saved progress (decode and every change check it).
    UnknownNode,
    RankOutOfRange,
    MissingRequirement,
    PairLocked,
    Overspent,
    UnknownResearch,
    EvidenceOutOfRange,
    HintsOutOfRange,
    ConcludedWithoutEvidence,
    ConcludedWithoutExperience,
    WindowOutOfRange,
    RulesMismatch,
    // Refused requests (preparations and observations).
    AlreadyOwned,
    RankSkipped,
    NotEnoughPoints,
    NotReady,
    AlreadyConcluded,
    HintKnown,
    HintSkipped,
    UnknownCategory,
    WrongActor,
}

fn pair_unlocked(state: &PlayerState, pair: Pair) -> bool {
    pair.metals()
        .iter()
        .all(|metal| state.metals[metal.index()].unlocked)
}

/// Checks a progress against the catalog, the player's unlocks and the rules it explains.
pub fn validate_progress(
    progress: &Progress,
    state: &PlayerState,
    rules: &ReserveRules,
    catalog: &Catalog,
) -> Result<(), ProgressError> {
    for (&code, &rank) in &progress.nodes {
        let def = node_def(code, catalog).ok_or(ProgressError::UnknownNode)?;
        if rank == 0 || usize::from(rank) > def.ranks.len() {
            return Err(ProgressError::RankOutOfRange);
        }
        if !requirements_met(progress, def) {
            return Err(ProgressError::MissingRequirement);
        }
        if let Pool::Pair(pair) = def.pool
            && !pair_unlocked(state, pair)
        {
            return Err(ProgressError::PairLocked);
        }
    }
    // A window counts at most its falloff's actions (`apply_practice` stops counting there),
    // and an unused window was never opened.
    if progress.windows.iter().any(|window| {
        usize::from(window.count) > catalog.falloff_percent.len()
            || (window.count == 0 && window.start != 0)
    }) {
        return Err(ProgressError::WindowOutOfRange);
    }
    for (&code, research) in &progress.research {
        let def = research_def(code, catalog).ok_or(ProgressError::UnknownResearch)?;
        // Evidence and conclusions are only observed with the pair unlocked, and a conclusion
        // brought its experience (hints alone need nothing).
        if (research.evidence != 0 || research.concluded) && !pair_unlocked(state, def.pair) {
            return Err(ProgressError::PairLocked);
        }
        if research.concluded && progress.pair_xp(def.pair) < def.pair_xp {
            return Err(ProgressError::ConcludedWithoutExperience);
        }
        if research.evidence & !def.evidence != 0 {
            return Err(ProgressError::EvidenceOutOfRange);
        }
        if research.hints > def.hints {
            return Err(ProgressError::HintsOutOfRange);
        }
        if research.concluded && research.evidence.count_ones() < u32::from(def.needed) {
            return Err(ProgressError::ConcludedWithoutEvidence);
        }
    }
    let pools = [
        Pool::Pair(Pair::IronSteel),
        Pool::Pair(Pair::TinPewter),
        Pool::Pair(Pair::ZincBrass),
        Pool::Pair(Pair::CopperBronze),
        Pool::Knowledge,
    ];
    if pools
        .iter()
        .any(|&pool| progress.spent(pool, catalog) > progress.earned(pool, catalog))
    {
        return Err(ProgressError::Overspent);
    }
    if rules.reduction_milli != progress.effect(NodeEffect::ReductionMilli, catalog)
        || rules.capacity_bonus_milli != progress.effect(NodeEffect::CapacityBonusMilli, catalog)
    {
        return Err(ProgressError::RulesMismatch);
    }
    Ok(())
}

/// Iron and steel's force rules for a player with `progress`: `base` (burning or flared) with
/// the reach, acceleration and push-below gain of the nodes owned.
pub fn tuned_force(base: &ForceRules, progress: &Progress, catalog: &Catalog) -> ForceRules {
    let reach = f64::from(progress.effect(NodeEffect::ReachBlocks, catalog));
    let accel = f64::from(progress.effect(NodeEffect::AccelPercent, catalog));
    let below = f64::from(progress.effect(NodeEffect::PushBelowPercent, catalog));
    ForceRules {
        range: base.range + reach,
        accel: base.accel * (100.0 + accel) / 100.0,
        push_below_gain: base.push_below_gain * (100.0 + below) / 100.0,
        ..*base
    }
}

/// What an observation did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observed {
    /// Practice experience gained by action number `count` of its window.
    Practice { category: u8, xp: u32, count: u8 },
    /// New evidence for a research; `ready` once there is enough to conclude.
    Evidence { research: u16, bit: u8, ready: bool },
    /// Nothing new: evidence already seen, research concluded, or a window already spent.
    Nothing,
}

/// The experience an action of `def` gives as action number `count + 1` in its window.
pub(crate) fn practice_xp(def: &PracticeDef, count: u8, catalog: &Catalog) -> u32 {
    let percent = catalog
        .falloff_percent
        .get(usize::from(count))
        .copied()
        .unwrap_or(0);
    def.xp * u32::from(percent) / 100
}

/// Applies one practice action at `game_time` (on a copy: the caller validates and commits).
pub(crate) fn apply_practice(
    progress: &mut Progress,
    def: &PracticeDef,
    game_time: u64,
    catalog: &Catalog,
) -> Observed {
    let window = &mut progress.windows[usize::from(def.category)];
    // A window opens with its first action. One that ended, or a game time before its start
    // (another save's clock), opens anew.
    if window.count == 0
        || game_time < window.start
        || game_time - window.start >= catalog.window_ticks
    {
        *window = PracticeWindow {
            start: game_time,
            count: 0,
        };
    }
    if usize::from(window.count) >= catalog.falloff_percent.len() {
        return Observed::Nothing;
    }
    let xp = practice_xp(def, window.count, catalog);
    window.count += 1;
    let count = window.count;
    let slot = &mut progress.pair_xp[pair_index(def.pair)];
    *slot = slot.saturating_add(xp);
    Observed::Practice {
        category: def.category,
        xp,
        count,
    }
}

pub(crate) fn apply_evidence(
    progress: &mut Progress,
    def: &ResearchDef,
    bit: u8,
) -> Result<Observed, ProgressError> {
    if bit.count_ones() != 1 || bit & def.evidence == 0 {
        return Err(ProgressError::EvidenceOutOfRange);
    }
    let entry = progress.research.entry(def.code).or_default();
    if entry.concluded || entry.evidence & bit != 0 {
        return Ok(Observed::Nothing);
    }
    entry.evidence |= bit;
    Ok(Observed::Evidence {
        research: def.code,
        bit,
        ready: entry.evidence.count_ones() >= u32::from(def.needed),
    })
}

pub(crate) fn check_purchase(
    progress: &Progress,
    state: &PlayerState,
    code: u16,
    rank: u8,
    catalog: &Catalog,
) -> Result<&'static NodeDef, ProgressError> {
    let def = node_def(code, catalog).ok_or(ProgressError::UnknownNode)?;
    if rank == 0 || usize::from(rank) > def.ranks.len() {
        return Err(ProgressError::RankOutOfRange);
    }
    let owned = progress.rank(code);
    if owned >= rank {
        return Err(ProgressError::AlreadyOwned);
    }
    if owned + 1 != rank {
        return Err(ProgressError::RankSkipped);
    }
    if let Pool::Pair(pair) = def.pool
        && !pair_unlocked(state, pair)
    {
        return Err(ProgressError::PairLocked);
    }
    if !requirements_met(progress, def) {
        return Err(ProgressError::MissingRequirement);
    }
    let cost = u32::from(def.ranks[usize::from(rank) - 1].cost);
    if progress.available(def.pool, catalog) < cost {
        return Err(ProgressError::NotEnoughPoints);
    }
    Ok(def)
}

pub(crate) fn apply_conclusion(
    progress: &mut Progress,
    state: &PlayerState,
    code: u16,
    catalog: &Catalog,
) -> Result<(), ProgressError> {
    let def = research_def(code, catalog).ok_or(ProgressError::UnknownResearch)?;
    if !pair_unlocked(state, def.pair) {
        return Err(ProgressError::PairLocked);
    }
    let entry = progress.research.entry(code).or_default();
    if entry.concluded {
        return Err(ProgressError::AlreadyConcluded);
    }
    if entry.evidence.count_ones() < u32::from(def.needed) {
        return Err(ProgressError::NotReady);
    }
    entry.concluded = true;
    let slot = &mut progress.pair_xp[pair_index(def.pair)];
    *slot = slot.saturating_add(def.pair_xp);
    Ok(())
}

pub(crate) fn apply_hint(
    progress: &mut Progress,
    code: u16,
    index: u8,
    catalog: &Catalog,
) -> Result<(), ProgressError> {
    let def = research_def(code, catalog).ok_or(ProgressError::UnknownResearch)?;
    if index == 0 || index > def.hints {
        return Err(ProgressError::HintsOutOfRange);
    }
    let entry = progress.research.entry(code).or_default();
    if index <= entry.hints {
        return Err(ProgressError::HintKnown);
    }
    if index != entry.hints + 1 {
        return Err(ProgressError::HintSkipped);
    }
    entry.hints = index;
    Ok(())
}

/// How the notebook shows a node to a player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// Its pair is locked or a requirement is missing.
    Locked,
    /// The next rank can be bought now.
    Available,
    /// The next rank is open but costs more than the points left.
    TooExpensive,
    /// Every rank is owned.
    Owned,
}

/// The status of `def` for a player with `progress` (`unlocked` answers whether a pair is).
pub fn node_status(
    progress: &Progress,
    def: &NodeDef,
    unlocked: impl Fn(Pair) -> bool,
    catalog: &Catalog,
) -> NodeStatus {
    let owned = usize::from(progress.rank(def.code));
    if owned >= def.ranks.len() {
        return NodeStatus::Owned;
    }
    let pair_ok = match def.pool {
        Pool::Pair(pair) => unlocked(pair),
        Pool::Knowledge => true,
    };
    if !pair_ok || !requirements_met(progress, def) {
        return NodeStatus::Locked;
    }
    if progress.available(def.pool, catalog) >= u32::from(def.ranks[owned].cost) {
        NodeStatus::Available
    } else {
        NodeStatus::TooExpensive
    }
}

/// Whether a research has enough evidence to be concluded (and is not yet).
pub fn research_ready(progress: &Progress, def: &ResearchDef) -> bool {
    let entry = progress.research(def.code);
    !entry.concluded && entry.evidence.count_ones() >= u32::from(def.needed)
}
