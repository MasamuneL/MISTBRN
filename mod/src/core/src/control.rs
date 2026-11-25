//! Player controls (P7a): the client's key requests and the server's HUD snapshot.
//!
//! Both travel as fixed-size, versioned byte strings with their own strict codecs (no serde).
//! A request names what the player pressed, never a metal or a state: the server resolves it
//! against its own `PlayerState` and the result still goes through `apply_command`.
//!
//! ```text
//! control = version:u8 (1) | op:u8 | arg:u8                       exactly 3 bytes
//!   op 1 cycle pair (arg 0) · 2 toggle side (arg 0 first, 1 second)
//!   op 3 toggle flare (arg 0) · 4 all off (arg 0)
//!   op 5 select pair (arg 0 iron/steel, 1 tin/pewter, 2 zinc/brass, 3 copper/bronze)
//!   op 6 buy node (arg node code << 2 | rank 1..=3) · 7 reveal hint (arg research << 2 | 1..=3)
//!   op 8 conclude research (arg research code, 1..=255; P7b)
//! hud     = version:u8 (1) | pair:u8 | 8 × (unlocked:u8 | burn:u8 | reserve:u32 LE
//!           | capacity:u32 LE), in `Metal::ALL` order             exactly 82 bytes
//! progress = version:u8 (2) | 4 × pair_xp:u32 LE | rank:u8 per catalog node
//!           | (evidence:u8 | hints:u8 | concluded:u8) per catalog research, in catalog order
//!           | milestones:u8 (claimed bits, P6)
//! ```
use crate::{
    BurnState, CATALOG, Command, Metal, Pair, PlayerState, Progress, ResearchState, ReserveRules,
    milestone_mask,
};

pub const CONTROL_VERSION: u8 = 1;
pub const CONTROL_LEN: usize = 3;
pub const HUD_VERSION: u8 = 1;
pub const HUD_LEN: usize = 2 + 10 * Metal::ALL.len();

/// The first or second metal of the selected pair (Z or X).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    First,
    Second,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlRequest {
    /// V: the next pair, through all four.
    CyclePair,
    /// Z / X: light that metal of the selected pair, or put it out if it burns.
    Toggle(Side),
    /// R: Burning ↔ Flared for the metal of the selected pair that burns.
    ToggleFlare,
    /// G: every metal off.
    Off,
    /// The pair wheel (hold V): that pair, locked or not, as `Command::SelectPair` allows.
    SelectPair(Pair),
    /// The notebook: buy `rank` of node `code` (P5). Not a `Command`: it is a save transaction.
    Purchase { code: u8, rank: u8 },
    /// The notebook: reveal hint `index` of research `research` (P5).
    RevealHint { research: u8, index: u8 },
    /// The book's Conclude button: conclude research `research`, looking at a research table
    /// (P7b). The server checks the table, the distance and readiness.
    Conclude { research: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlError {
    Length(usize),
    Version(u8),
    Op(u8),
    Arg { op: u8, arg: u8 },
}

impl ControlError {
    /// One word for the log (`rejected:<word>`).
    pub fn reason(self) -> &'static str {
        match self {
            Self::Length(_) => "length",
            Self::Version(_) => "version",
            Self::Op(_) => "op",
            Self::Arg { .. } => "arg",
        }
    }
}

pub fn encode_control(request: ControlRequest) -> [u8; CONTROL_LEN] {
    let (op, arg) = match request {
        ControlRequest::CyclePair => (1, 0),
        ControlRequest::Toggle(Side::First) => (2, 0),
        ControlRequest::Toggle(Side::Second) => (2, 1),
        ControlRequest::ToggleFlare => (3, 0),
        ControlRequest::Off => (4, 0),
        ControlRequest::SelectPair(pair) => (5, pair_code(pair)),
        ControlRequest::Purchase { code, rank } => (6, code << 2 | (rank & 3)),
        ControlRequest::RevealHint { research, index } => (7, research << 2 | (index & 3)),
        ControlRequest::Conclude { research } => (8, research),
    };
    [CONTROL_VERSION, op, arg]
}

pub fn decode_control(bytes: &[u8]) -> Result<ControlRequest, ControlError> {
    let &[version, op, arg] = bytes else {
        return Err(ControlError::Length(bytes.len()));
    };
    if version != CONTROL_VERSION {
        return Err(ControlError::Version(version));
    }
    match (op, arg) {
        (1, 0) => Ok(ControlRequest::CyclePair),
        (2, 0) => Ok(ControlRequest::Toggle(Side::First)),
        (2, 1) => Ok(ControlRequest::Toggle(Side::Second)),
        (3, 0) => Ok(ControlRequest::ToggleFlare),
        (4, 0) => Ok(ControlRequest::Off),
        (5, 0..=3) => Ok(ControlRequest::SelectPair(PAIRS[usize::from(arg)])),
        // Code 0 and index 0 are never valid; whether a code exists is the rules' answer.
        (6, _) if arg >> 2 != 0 && arg & 3 != 0 => Ok(ControlRequest::Purchase {
            code: arg >> 2,
            rank: arg & 3,
        }),
        (7, _) if arg >> 2 != 0 && arg & 3 != 0 => Ok(ControlRequest::RevealHint {
            research: arg >> 2,
            index: arg & 3,
        }),
        (8, 1..) => Ok(ControlRequest::Conclude { research: arg }),
        (1..=8, _) => Err(ControlError::Arg { op, arg }),
        _ => Err(ControlError::Op(op)),
    }
}

/// The four pairs in wire and wheel order (op 5's argument is the index).
pub const PAIRS: [Pair; 4] = [
    Pair::IronSteel,
    Pair::TinPewter,
    Pair::ZincBrass,
    Pair::CopperBronze,
];

fn pair_code(pair: Pair) -> u8 {
    match pair {
        Pair::IronSteel => 0,
        Pair::TinPewter => 1,
        Pair::ZincBrass => 2,
        Pair::CopperBronze => 3,
    }
}

pub fn pair_metals(pair: Pair) -> [Metal; 2] {
    match pair {
        Pair::IronSteel => [Metal::Iron, Metal::Steel],
        Pair::TinPewter => [Metal::Tin, Metal::Pewter],
        Pair::ZincBrass => [Metal::Zinc, Metal::Brass],
        Pair::CopperBronze => [Metal::Copper, Metal::Bronze],
    }
}

pub fn next_pair(pair: Pair) -> Pair {
    match pair {
        Pair::IronSteel => Pair::TinPewter,
        Pair::TinPewter => Pair::ZincBrass,
        Pair::ZincBrass => Pair::CopperBronze,
        Pair::CopperBronze => Pair::IronSteel,
    }
}

/// The command a request asks for, from the server's own state. `None`: nothing to do (R
/// with no metal of the selected pair burning) or not a command (the notebook's requests).
pub fn resolve_control(state: &PlayerState, request: ControlRequest) -> Option<Command> {
    let metals = pair_metals(state.selected_pair);
    let burn = |metal: Metal| state.metals[metal.index()].burn;
    match request {
        ControlRequest::CyclePair => Some(Command::SelectPair {
            pair: next_pair(state.selected_pair),
        }),
        ControlRequest::Toggle(side) => {
            let metal = metals[side as usize];
            Some(if burn(metal) == BurnState::Off {
                Command::Activate { metal }
            } else {
                Command::Extinguish { metal: Some(metal) }
            })
        }
        ControlRequest::ToggleFlare => metals.into_iter().find_map(|metal| match burn(metal) {
            BurnState::Off => None,
            BurnState::Burning => Some(Command::Flare { metal }),
            BurnState::Flared => Some(Command::Activate { metal }),
        }),
        ControlRequest::Off => Some(Command::Extinguish { metal: None }),
        ControlRequest::SelectPair(pair) => Some(Command::SelectPair { pair }),
        ControlRequest::Purchase { .. }
        | ControlRequest::RevealHint { .. }
        | ControlRequest::Conclude { .. } => None,
    }
}

/// Per-player request budget: `CAPACITY` at once, one more every `REFILL_TICKS` server ticks.
/// A player's own key handling sends at most one request per key press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestBudget {
    tokens: u8,
    last_tick: u64,
}

impl RequestBudget {
    pub const CAPACITY: u8 = 8;
    pub const REFILL_TICKS: u64 = 2;

    pub fn new(tick: u64) -> Self {
        Self {
            tokens: Self::CAPACITY,
            last_tick: tick,
        }
    }

    /// Takes one token at `tick`; false when none is left. A tick earlier than the last one
    /// (a new world) restarts the refill clock there without adding tokens.
    pub fn allow(&mut self, tick: u64) -> bool {
        if tick < self.last_tick {
            self.last_tick = tick;
        }
        let gained = (tick - self.last_tick) / Self::REFILL_TICKS;
        if gained > 0 {
            let tokens = u64::from(self.tokens).saturating_add(gained);
            self.tokens = u8::try_from(tokens.min(u64::from(Self::CAPACITY))).unwrap_or(0);
            self.last_tick += gained * Self::REFILL_TICKS;
        }
        if self.tokens == 0 {
            return false;
        }
        self.tokens -= 1;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudMetal {
    pub unlocked: bool,
    pub burn: BurnState,
    pub reserve_milli: u32,
    pub capacity_milli: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HudSnapshot {
    pub pair: Pair,
    pub metals: [HudMetal; 8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudError {
    Length(usize),
    Version(u8),
    Pair(u8),
    Flag(u8),
    Burn(u8),
    Reserve,
}

impl HudSnapshot {
    /// What the HUD shows of `state`. Amounts past `u32` (never with valid rules) saturate.
    pub fn from_state(state: &PlayerState, rules: &ReserveRules) -> Self {
        let metals = Metal::ALL.map(|metal| {
            let entry = &state.metals[metal.index()];
            let capacity = rules.capacity(metal).unwrap_or(0);
            let capacity_milli = u32::try_from(capacity).unwrap_or(u32::MAX);
            HudMetal {
                unlocked: entry.unlocked,
                burn: entry.burn,
                reserve_milli: u32::try_from(entry.reserve_milli)
                    .unwrap_or(u32::MAX)
                    .min(capacity_milli),
                capacity_milli,
            }
        });
        Self {
            pair: state.selected_pair,
            metals,
        }
    }
}

pub fn encode_hud(hud: &HudSnapshot) -> [u8; HUD_LEN] {
    let mut out = [0u8; HUD_LEN];
    out[0] = HUD_VERSION;
    out[1] = crate::persistence::pair_code(hud.pair);
    for (i, metal) in hud.metals.iter().enumerate() {
        let at = 2 + 10 * i;
        out[at] = u8::from(metal.unlocked);
        out[at + 1] = crate::persistence::burn_code(metal.burn);
        out[at + 2..at + 6].copy_from_slice(&metal.reserve_milli.to_le_bytes());
        out[at + 6..at + 10].copy_from_slice(&metal.capacity_milli.to_le_bytes());
    }
    out
}

pub fn decode_hud(bytes: &[u8]) -> Result<HudSnapshot, HudError> {
    if bytes.len() != HUD_LEN {
        return Err(HudError::Length(bytes.len()));
    }
    if bytes[0] != HUD_VERSION {
        return Err(HudError::Version(bytes[0]));
    }
    let pair = match bytes[1] {
        0 => Pair::IronSteel,
        1 => Pair::TinPewter,
        2 => Pair::ZincBrass,
        3 => Pair::CopperBronze,
        other => return Err(HudError::Pair(other)),
    };
    let word = |from: usize| {
        u32::from_le_bytes([
            bytes[from],
            bytes[from + 1],
            bytes[from + 2],
            bytes[from + 3],
        ])
    };
    let mut metals = [HudMetal {
        unlocked: false,
        burn: BurnState::Off,
        reserve_milli: 0,
        capacity_milli: 0,
    }; 8];
    for (i, metal) in metals.iter_mut().enumerate() {
        let at = 2 + 10 * i;
        metal.unlocked = match bytes[at] {
            0 => false,
            1 => true,
            other => return Err(HudError::Flag(other)),
        };
        metal.burn = match bytes[at + 1] {
            0 => BurnState::Off,
            1 => BurnState::Burning,
            2 => BurnState::Flared,
            other => return Err(HudError::Burn(other)),
        };
        metal.reserve_milli = word(at + 2);
        metal.capacity_milli = word(at + 6);
        if metal.reserve_milli > metal.capacity_milli {
            return Err(HudError::Reserve);
        }
    }
    Ok(HudSnapshot { pair, metals })
}

pub const PROGRESS_VERSION: u8 = 2;
pub const PROGRESS_LEN: usize = 1 + 4 * 4 + CATALOG.nodes.len() + 3 * CATALOG.research.len() + 1;

/// What the notebook shows: the player's progress as the server holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgressSnapshot {
    pub progress: Progress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressSnapshotError {
    Length(usize),
    Version(u8),
    Rank,
    Research,
    Milestones,
}

pub fn encode_progress_snapshot(progress: &Progress) -> [u8; PROGRESS_LEN] {
    let mut out = [0u8; PROGRESS_LEN];
    out[0] = PROGRESS_VERSION;
    for (i, pair) in PAIRS.iter().enumerate() {
        out[1 + 4 * i..5 + 4 * i].copy_from_slice(&progress.pair_xp(*pair).to_le_bytes());
    }
    let mut at = 17;
    for def in CATALOG.nodes {
        out[at] = progress.rank(def.code);
        at += 1;
    }
    for def in CATALOG.research {
        let entry = progress.research(def.code);
        out[at] = entry.evidence;
        out[at + 1] = entry.hints;
        out[at + 2] = u8::from(entry.concluded);
        at += 3;
    }
    out[at] = progress.milestones();
    out
}

/// Decodes a snapshot. The server is trusted, but nothing out of the catalog's range is drawn.
pub fn decode_progress_snapshot(bytes: &[u8]) -> Result<ProgressSnapshot, ProgressSnapshotError> {
    if bytes.len() != PROGRESS_LEN {
        return Err(ProgressSnapshotError::Length(bytes.len()));
    }
    if bytes[0] != PROGRESS_VERSION {
        return Err(ProgressSnapshotError::Version(bytes[0]));
    }
    let mut xp = [0u32; 4];
    for (i, slot) in xp.iter_mut().enumerate() {
        let at = 1 + 4 * i;
        *slot = u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    }
    let mut at = 17;
    let mut ranks = Vec::new();
    for def in CATALOG.nodes {
        if usize::from(bytes[at]) > def.ranks.len() {
            return Err(ProgressSnapshotError::Rank);
        }
        if bytes[at] > 0 {
            ranks.push((def.code, bytes[at]));
        }
        at += 1;
    }
    let mut research = Vec::new();
    for def in CATALOG.research {
        let (evidence, hints, concluded) = (bytes[at], bytes[at + 1], bytes[at + 2]);
        if evidence & !def.evidence != 0 || hints > def.hints || concluded > 1 {
            return Err(ProgressSnapshotError::Research);
        }
        research.push((
            def.code,
            ResearchState {
                evidence,
                hints,
                concluded: concluded == 1,
            },
        ));
        at += 3;
    }
    let milestones = bytes[at];
    if milestones & !milestone_mask(&CATALOG) != 0 {
        return Err(ProgressSnapshotError::Milestones);
    }
    Ok(ProgressSnapshot {
        progress: Progress::from_parts(xp, ranks, research, milestones),
    })
}
