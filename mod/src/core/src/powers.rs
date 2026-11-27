//! The powers' rules (P8), pure: what each power does at the player's nodes, lit or flared.
//!
//! The adapter applies them (effects, attribute modifiers, damage and fall hooks); nothing here
//! is saved. Values are trial values, not balance.
use crate::{Catalog, NodeEffect, Progress};

/// Tin: the fog is this many times farther, burning and flared (before `tin_range`).
pub const FOG_BURNING: f32 = 1.5;
pub const FOG_FLARED: f32 = 2.0;
/// Tin's hidden night vision: given for this long, renewed when this much is left.
pub const NIGHT_VISION_TICKS: i32 = 400;
pub const NIGHT_VISION_RENEW: i32 = 200;
/// Flared tin in light this bright or more at the head blinds the player (tin's stun).
pub const STUN_LIGHT: i32 = 14;
pub const STUN_TICKS: u64 = 40;
/// A stun waits at least this long after the last one.
pub const STUN_COOLDOWN: u64 = 100;
/// Hearing: base range in blocks and how long a heard creature stays marked, in ticks.
pub const HEARING_RANGE: f64 = 16.0;
pub const HEARING_MEMORY: u64 = 40;
/// At most this many marked creatures are sent to the client.
pub const SENSES_MAX: usize = 32;
/// Pewter's drag: flared this long without a break, then put out or back to burning.
pub const DRAG_AFTER: u64 = 200;
pub const DRAG_MAX: u64 = 600;
/// Flared pewter tires the player this much a tick (food exhaustion).
pub const FLARE_EXHAUSTION: f32 = 0.05;
/// Reflexes' dodge: pewter spent, speed in blocks a tick, ticks without melee or projectile
/// damage, ticks between two dodges.
pub const DODGE_COST_MILLI: u64 = 500;
pub const DODGE_SPEED: f64 = 1.0;
pub const DODGE_IFRAMES: u64 = 8;
pub const DODGE_COOLDOWN: u64 = 40;
/// Falls that count: pewter's practice and the threshold research's evidence, in blocks.
pub const PRACTICE_FALL: f64 = 8.0;
pub const EVIDENCE_FALL: f64 = 12.0;

/// What tin and pewter do with a player's nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TinPewter {
    tin_range: u16,
    clarity_milli: u16,
    hearing: bool,
    hearing_range: u16,
    hearing_memory: u16,
    strength: u16,
    endurance: u16,
    jump: bool,
    jump_height: u16,
    cushion: u16,
    transition_milli: u16,
    steel_body: u16,
}

/// Pewter's body, as the adapter applies it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PewterBody {
    /// `minecraft:attack_damage`, added times the base.
    pub attack: f64,
    /// `minecraft:movement_speed`, added times the base.
    pub speed: f64,
    /// `minecraft:knockback_resistance`, added.
    pub knockback: f64,
    /// Damage taken is multiplied by this.
    pub damage_taken: f32,
    /// Fall damage is multiplied by this.
    pub fall: f32,
}

impl TinPewter {
    pub fn new(progress: &Progress, catalog: &Catalog) -> Self {
        let effect = |e| progress.effect(e, catalog);
        let owned = |code| progress.rank(code) > 0;
        Self {
            tin_range: effect(NodeEffect::TinRangePercent),
            clarity_milli: effect(NodeEffect::ClarityMilli).min(1_000),
            hearing: owned(crate::NODE_HEARING),
            hearing_range: effect(NodeEffect::HearingRangeBlocks),
            hearing_memory: effect(NodeEffect::HearingMemoryTicks),
            strength: effect(NodeEffect::PewterStrengthPercent),
            endurance: effect(NodeEffect::PewterEndurancePercent),
            jump: owned(crate::NODE_PEWTER_JUMP),
            jump_height: effect(NodeEffect::JumpPercent),
            cushion: effect(NodeEffect::CushionPercent),
            transition_milli: effect(NodeEffect::TransitionMilli).min(1_000),
            steel_body: effect(NodeEffect::SteelBodyPercent),
        }
    }

    /// How many times farther the fog is.
    pub fn fog(&self, flared: bool) -> f32 {
        let base = if flared { FOG_FLARED } else { FOG_BURNING };
        base * (1.0 + f32::from(self.tin_range) / 100.0)
    }

    /// What is left of a darkness or blindness of `ticks` with tin lit (0: none).
    pub fn clarity(&self, ticks: i32) -> i32 {
        let kept = 1_000 - i64::from(self.clarity_milli);
        (i64::from(ticks.max(0)) * kept / 1_000) as i32
    }

    pub fn has_clarity(&self) -> bool {
        self.clarity_milli > 0
    }

    /// `ticks` of an end-of-flare penalty after `transition`.
    pub fn after_transition(&self, ticks: u64) -> u64 {
        ticks * (1_000 - u64::from(self.transition_milli)) / 1_000
    }

    /// The stun's length with the nodes.
    pub fn stun(&self) -> u64 {
        self.after_transition(STUN_TICKS)
    }

    /// Hearing's range in blocks and memory in ticks, when the technique is owned.
    pub fn hearing(&self) -> Option<(f64, u64)> {
        self.hearing.then(|| {
            (
                HEARING_RANGE + f64::from(self.hearing_range),
                HEARING_MEMORY + u64::from(self.hearing_memory),
            )
        })
    }

    pub fn body(&self, flared: bool) -> PewterBody {
        let twice = if flared { 2.0 } else { 1.0 };
        let percent = |v: u16| f64::from(v) / 100.0;
        let taken = (0.15 * twice + percent(self.endurance)).min(0.9);
        let mut fall = 0.30 * twice;
        if self.jump {
            fall += 0.20 + percent(self.cushion);
        }
        PewterBody {
            attack: 0.20 * twice + percent(self.strength),
            speed: 0.10 * twice,
            knockback: 0.3 * twice,
            damage_taken: (1.0 - taken) as f32,
            fall: (1.0 - fall.min(0.9)) as f32,
        }
    }

    /// The jump power multiplier with pewter lit (1: no jump technique).
    pub fn jump(&self) -> f32 {
        if !self.jump {
            return 1.0;
        }
        1.0 + (25.0 + f32::from(self.jump_height)) / 100.0
    }

    /// The drag after flaring `flared_ticks` without a break (None: too short for one).
    pub fn drag(&self, flared_ticks: u64) -> Option<u64> {
        (flared_ticks >= DRAG_AFTER)
            .then(|| self.after_transition((flared_ticks / 2).min(DRAG_MAX)))
            .filter(|&ticks| ticks > 0)
    }

    /// Extra push off an anchor below the feet with pewter lit, in percent (the bridge).
    pub fn steel_body(&self) -> u16 {
        self.steel_body
    }
}

/// The dodge's velocity for op 10's direction (0 left, 1 right, 2 back, 3 forward, relative to
/// the look's yaw in degrees). Minecraft's yaw 0 looks to +z and 90 to −x.
pub fn dodge_velocity(yaw_degrees: f32, direction: u8) -> Option<[f64; 3]> {
    let (sin, cos) = f64::from(yaw_degrees).to_radians().sin_cos();
    let forward = [-sin, cos];
    let left = [cos, sin];
    let [x, z] = match direction {
        0 => left,
        1 => [-left[0], -left[1]],
        2 => [-forward[0], -forward[1]],
        3 => forward,
        _ => return None,
    };
    Some([x * DODGE_SPEED, 0.1, z * DODGE_SPEED])
}

/// Vanilla creatures that attack on sight (pewter's fight practice; zinc and brass reuse it).
const HOSTILE: [&str; 36] = [
    "blaze",
    "bogged",
    "breeze",
    "cave_spider",
    "creaking",
    "creeper",
    "drowned",
    "elder_guardian",
    "ender_dragon",
    "endermite",
    "evoker",
    "ghast",
    "guardian",
    "hoglin",
    "husk",
    "illusioner",
    "magma_cube",
    "parched",
    "phantom",
    "piglin_brute",
    "pillager",
    "ravager",
    "shulker",
    "silverfish",
    "skeleton",
    "slime",
    "spider",
    "stray",
    "vex",
    "vindicator",
    "warden",
    "witch",
    "wither",
    "wither_skeleton",
    "zoglin",
    "zombie",
];

/// Whether an entity type id (`minecraft:zombie`) is a hostile creature. Zombie villagers count.
pub fn is_hostile(kind: &str) -> bool {
    kind.strip_prefix("minecraft:")
        .is_some_and(|path| HOSTILE.contains(&path) || path == "zombie_villager")
}

/// `mistbrn:senses` (server to client): the version, a count, then each entity id (u32, little
/// endian). At most [`SENSES_MAX`] ids.
pub const SENSES_VERSION: u8 = 1;

pub fn encode_senses(ids: &[u32]) -> Vec<u8> {
    let ids = &ids[..ids.len().min(SENSES_MAX)];
    let mut bytes = vec![SENSES_VERSION, ids.len() as u8];
    for id in ids {
        bytes.extend_from_slice(&id.to_le_bytes());
    }
    bytes
}

/// The ids, or None for any malformed message (wrong version, count past the limit or not
/// matching the length).
pub fn decode_senses(bytes: &[u8]) -> Option<Vec<u32>> {
    let (&[version, count], rest) = bytes.split_first_chunk::<2>()?;
    let count = usize::from(count);
    if version != SENSES_VERSION || count > SENSES_MAX || rest.len() != count * 4 {
        return None;
    }
    Some(
        rest.as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn with(nodes: &[(u16, u8)]) -> TinPewter {
        let progress = Progress {
            nodes: nodes.iter().copied().collect(),
            ..Progress::default()
        };
        TinPewter::new(&progress, &CATALOG)
    }

    // Catches a fog, body or jump that ignores the flare or the nodes.
    #[test]
    fn values_follow_the_flare_and_the_nodes() {
        let none = with(&[]);
        assert_eq!(none.fog(false), 1.5);
        assert_eq!(none.fog(true), 2.0);
        assert_eq!(with(&[(NODE_TIN_RANGE, 2)]).fog(false), 2.25);
        let body = none.body(false);
        assert!((body.attack - 0.2).abs() < 1e-9 && (body.knockback - 0.3).abs() < 1e-9);
        assert!((body.damage_taken - 0.85).abs() < 1e-6 && (body.fall - 0.7).abs() < 1e-6);
        let flared = none.body(true);
        assert!((flared.damage_taken - 0.7).abs() < 1e-6 && (flared.fall - 0.4).abs() < 1e-6);
        let strong = with(&[(NODE_PEWTER_STRENGTH, 3), (NODE_PEWTER_ENDURANCE, 3)]).body(false);
        assert!((strong.attack - 0.5).abs() < 1e-9 && (strong.damage_taken - 0.7).abs() < 1e-6);
        assert_eq!(none.jump(), 1.0);
        let jumper = with(&[
            (NODE_PEWTER_STRENGTH, 1),
            (NODE_PEWTER_JUMP, 1),
            (NODE_JUMP_HEIGHT, 2),
            (NODE_JUMP_CUSHION, 2),
        ]);
        assert!((jumper.jump() - 1.45).abs() < 1e-6);
        // 0.3 + 0.2 + 0.2 off; flared 0.6 + 0.4 caps at 0.9.
        assert!((jumper.body(false).fall - 0.3).abs() < 1e-6);
        assert!((jumper.body(true).fall - 0.1).abs() < 1e-6);
        assert_eq!(none.hearing(), None);
        let hearer = with(&[
            (NODE_TIN_RANGE, 1),
            (NODE_HEARING, 1),
            (NODE_HEARING_RANGE, 2),
            (NODE_HEARING_MEMORY, 1),
        ]);
        assert_eq!(hearer.hearing(), Some((24.0, 60)));
    }

    // Catches clarity or transition that leave too much, or a drag after a short flare.
    #[test]
    fn penalties_shrink_with_the_nodes() {
        let none = with(&[]);
        assert!(!none.has_clarity());
        assert_eq!(none.clarity(260), 260);
        assert_eq!(with(&[(NODE_TIN_CLARITY, 1)]).clarity(300), 200);
        assert_eq!(with(&[(NODE_TIN_CLARITY, 3)]).clarity(300), 0);
        assert_eq!(none.drag(199), None);
        assert_eq!(none.drag(200), Some(100));
        assert_eq!(none.drag(5_000), Some(DRAG_MAX));
        assert_eq!(none.stun(), 40);
        let calm = with(&[
            (NODE_ECONOMY, 1),
            (NODE_FLARE_COST, 1),
            (NODE_TRANSITION, 2),
        ]);
        assert_eq!(calm.drag(400), Some(100));
        assert_eq!(calm.stun(), 20);
    }

    // Catches a senses message the client would misread, and hostiles taken for passives.
    #[test]
    fn senses_round_trip_and_hostiles() {
        let ids: Vec<u32> = (0..40).collect();
        let bytes = encode_senses(&ids);
        assert_eq!(decode_senses(&bytes), Some(ids[..SENSES_MAX].to_vec()));
        assert_eq!(decode_senses(&[1, 0]), Some(vec![]));
        assert_eq!(decode_senses(&[2, 0]), None);
        assert_eq!(decode_senses(&[1, 1, 0, 0, 0]), None);
        assert_eq!(decode_senses(&[1, 33]), None);
        assert_eq!(decode_senses(&[1]), None);
        assert!(is_hostile("minecraft:zombie") && is_hostile("minecraft:zombie_villager"));
        assert!(!is_hostile("minecraft:cow") && !is_hostile("mistbrn:zombie"));
    }

    // Catches a dodge that goes the wrong way for the look.
    #[test]
    fn the_dodge_goes_relative_to_the_look() {
        let close =
            |a: [f64; 3], x: f64, z: f64| (a[0] - x).abs() < 1e-9 && (a[2] - z).abs() < 1e-9;
        // Looking to +z (south), the left is +x (east).
        assert!(close(dodge_velocity(0.0, 0).unwrap(), 1.0, 0.0));
        assert!(close(dodge_velocity(0.0, 1).unwrap(), -1.0, 0.0));
        assert!(close(dodge_velocity(0.0, 2).unwrap(), 0.0, -1.0));
        assert!(close(dodge_velocity(90.0, 3).unwrap(), -1.0, 0.0));
        assert_eq!(dodge_velocity(0.0, 4), None);
    }
}
