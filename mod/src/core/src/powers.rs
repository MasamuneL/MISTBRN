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

/// Zinc and brass: base range of the look (blocks), how long an effect lasts after the metal is
/// put out, and the flare's range multiplier.
pub const ZB_RANGE: f64 = 16.0;
pub const ZB_AFTER: u64 = 100;
pub const ZB_FLARE_RANGE: f64 = 1.5;
/// After an effect ends, the creature ignores the same metal this long (bosses: longer, and the
/// effect itself is cut short).
pub const ZB_IMMUNITY: u64 = 200;
pub const BOSS_EFFECT: u64 = 40;
pub const BOSS_IMMUNITY: u64 = 600;
/// Courage's strength and speed, in ticks, before `courage_duration`.
pub const COURAGE_TICKS: u64 = 200;
/// Retreat: blocks away from the player and navigation speed, before the nodes.
pub const RETREAT_BLOCKS: f64 = 8.0;
pub const RETREAT_SPEED: f64 = 1.0;
/// Selective influence soothes the compatible creatures this close to the player.
pub const SELECTIVE_RADIUS: f64 = 8.0;
/// Practice: a provoked creature this close; a soothed chaser soothed this long.
pub const PROVOKE_NEAR: f64 = 3.0;
pub const SOOTHE_PRACTICE: u64 = 100;
/// The emotion research's third evidence: provoked and soothed within this many ticks.
pub const EMOTION_PAIR_TICKS: u64 = 600;

/// Hostiles whose target the pin cannot write (brain-backed in 26.3) or that ignore it (the
/// dragon flies by its phase): zinc and brass leave them alone.
const NOT_STEERABLE: [&str; 7] = [
    "breeze",
    "creaking",
    "ender_dragon",
    "hoglin",
    "piglin_brute",
    "warden",
    "zoglin",
];
/// Neutral creatures with goal-based targets (zinc's provoked-neutral evidence).
const NEUTRAL: [&str; 11] = [
    "bee",
    "dolphin",
    "enderman",
    "fox",
    "iron_golem",
    "llama",
    "panda",
    "polar_bear",
    "trader_llama",
    "wolf",
    "zombified_piglin",
];

pub fn is_neutral(kind: &str) -> bool {
    kind.strip_prefix("minecraft:")
        .is_some_and(|path| NEUTRAL.contains(&path))
}

/// Whether zinc and brass act on this entity type. Never a player.
pub fn is_compatible(kind: &str) -> bool {
    let steerable = kind
        .strip_prefix("minecraft:")
        .is_some_and(|path| !NOT_STEERABLE.contains(&path));
    steerable && (is_hostile(kind) || is_neutral(kind))
}

/// Bosses zinc and brass touch only briefly.
pub fn is_boss(kind: &str) -> bool {
    matches!(kind, "minecraft:elder_guardian" | "minecraft:wither")
}

/// What zinc and brass do with a player's nodes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ZincBrass {
    zinc_range: u16,
    zinc_after: u16,
    brass_range: u16,
    brass_after: u16,
    outer: u16,
    courage: bool,
    courage_level: u16,
    courage_ticks: u16,
    retreat: bool,
    retreat_blocks: u16,
    retreat_percent: u16,
    selective: bool,
}

impl ZincBrass {
    pub fn new(progress: &Progress, catalog: &Catalog) -> Self {
        let effect = |e| progress.effect(e, catalog);
        let owned = |code| progress.rank(code) > 0;
        Self {
            zinc_range: effect(NodeEffect::ZincRangeBlocks),
            zinc_after: effect(NodeEffect::ZincAfterTicks),
            brass_range: effect(NodeEffect::BrassRangeBlocks),
            brass_after: effect(NodeEffect::BrassAfterTicks),
            outer: effect(NodeEffect::OuterReachBlocks),
            courage: owned(crate::NODE_COURAGE),
            courage_level: effect(NodeEffect::CourageLevel),
            courage_ticks: effect(NodeEffect::CourageTicks),
            retreat: owned(crate::NODE_RETREAT),
            retreat_blocks: effect(NodeEffect::RetreatBlocks),
            retreat_percent: effect(NodeEffect::RetreatPercent),
            selective: owned(crate::NODE_SELECTIVE),
        }
    }

    /// The look's range in blocks, for zinc (`true`) or brass.
    pub fn range(&self, zinc: bool, flared: bool) -> f64 {
        let extra = if zinc {
            self.zinc_range
        } else {
            self.brass_range
        };
        let range = ZB_RANGE + f64::from(extra) + f64::from(self.outer);
        if flared {
            range * ZB_FLARE_RANGE
        } else {
            range
        }
    }

    /// Ticks an effect lasts once the metal is put out.
    pub fn after(&self, zinc: bool) -> u64 {
        ZB_AFTER
            + u64::from(if zinc {
                self.zinc_after
            } else {
                self.brass_after
            })
    }

    /// Courage's effect amplifier (0 = level I) and ticks, when the technique is owned.
    pub fn courage(&self) -> Option<(u8, u64)> {
        self.courage.then(|| {
            (
                self.courage_level.min(2) as u8,
                COURAGE_TICKS + u64::from(self.courage_ticks),
            )
        })
    }

    /// Retreat's distance in blocks and navigation speed, when the technique is owned.
    pub fn retreat(&self) -> Option<(f64, f64)> {
        self.retreat.then(|| {
            (
                RETREAT_BLOCKS + f64::from(self.retreat_blocks),
                RETREAT_SPEED * (100.0 + f64::from(self.retreat_percent)) / 100.0,
            )
        })
    }

    pub fn selective(&self) -> bool {
        self.selective
    }
}

/// One creature under zinc or brass: since when, and when the metal let go of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Influence {
    pub started: u64,
    /// The tick the player put the metal out or looked away to another creature.
    pub released: Option<u64>,
    pub boss: bool,
}

impl Influence {
    /// The tick the effect ends (None: it lasts while the metal holds it).
    pub fn ends(&self, after: u64) -> Option<u64> {
        let held = self.released.map(|at| at + after);
        if self.boss {
            let cut = self.started + BOSS_EFFECT;
            Some(held.map_or(cut, |end| end.min(cut)))
        } else {
            held
        }
    }

    pub fn active(&self, after: u64, tick: u64) -> bool {
        self.ends(after).is_none_or(|end| tick < end)
    }

    /// Until when the creature ignores the same metal, once the effect has ended.
    pub fn immune_until(&self, after: u64) -> Option<u64> {
        let immunity = if self.boss {
            BOSS_IMMUNITY
        } else {
            ZB_IMMUNITY
        };
        self.ends(after).map(|end| end + immunity)
    }
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

    // Catches zinc/brass reaching players, brain mobs or the dragon, or values ignoring nodes.
    #[test]
    fn zinc_brass_values_and_kinds() {
        let progress = |nodes: &[(u16, u8)]| Progress {
            nodes: nodes.iter().copied().collect(),
            ..Progress::default()
        };
        let none = ZincBrass::new(&progress(&[]), &CATALOG);
        assert_eq!(none.range(true, false), 16.0);
        assert_eq!(none.range(false, true), 24.0);
        assert_eq!(none.after(true), 100);
        assert_eq!(
            (none.courage(), none.retreat(), none.selective()),
            (None, None, false)
        );
        let all = ZincBrass::new(
            &progress(&[
                (NODE_ZINC_RANGE, 2),
                (NODE_ZINC_DURATION, 1),
                (NODE_BRASS_RANGE, 1),
                (NODE_COURAGE, 1),
                (NODE_COURAGE_LEVEL, 1),
                (NODE_COURAGE_DURATION, 2),
                (NODE_RETREAT, 1),
                (NODE_RETREAT_DISTANCE, 1),
                (NODE_RETREAT_SPEED, 2),
                (NODE_OUTER_REACH, 1),
            ]),
            &CATALOG,
        );
        assert_eq!(all.range(true, false), 26.0);
        assert_eq!(all.range(false, false), 22.0);
        assert_eq!(all.after(true), 160);
        assert_eq!(all.after(false), 100);
        assert_eq!(all.courage(), Some((1, 400)));
        assert_eq!(all.retreat(), Some((12.0, 1.4)));
        // The bridge also lengthens iron and steel's reach.
        let base = crate::physics::ForceRules::BURNING;
        let reach = |nodes: &[(u16, u8)]| tuned_force(&base, &progress(nodes), &CATALOG).range;
        assert_eq!(
            reach(&[(NODE_REACH, 1), (NODE_OUTER_REACH, 1)]),
            base.range + 4.0
        );
        for kind in [
            "minecraft:zombie",
            "minecraft:husk",
            "minecraft:zombified_piglin",
            "minecraft:wolf",
        ] {
            assert!(is_compatible(kind), "{kind}");
        }
        for kind in [
            "minecraft:player",
            "minecraft:warden",
            "minecraft:piglin",
            "minecraft:hoglin",
            "minecraft:ender_dragon",
            "minecraft:cow",
            "minecraft:villager",
        ] {
            assert!(!is_compatible(kind), "{kind}");
        }
        assert!(is_neutral("minecraft:zombified_piglin") && !is_neutral("minecraft:zombie"));
        assert!(is_boss("minecraft:wither") && is_compatible("minecraft:elder_guardian"));
    }

    // Catches an effect that never ends, a boss held past its cut or immunity starting early.
    #[test]
    fn influence_ends_and_immunity() {
        let held = Influence {
            started: 10,
            released: None,
            boss: false,
        };
        assert!(held.active(100, 10_000));
        assert_eq!(held.immune_until(100), None);
        let let_go = Influence {
            released: Some(50),
            ..held
        };
        assert!(let_go.active(100, 149) && !let_go.active(100, 150));
        assert_eq!(let_go.immune_until(100), Some(350));
        let boss = Influence {
            started: 10,
            released: None,
            boss: true,
        };
        assert!(boss.active(100, 49) && !boss.active(100, 50));
        assert_eq!(boss.immune_until(100), Some(650));
        let boss_early = Influence {
            released: Some(20),
            ..boss
        };
        assert_eq!(boss_early.ends(5), Some(25));
    }
}
