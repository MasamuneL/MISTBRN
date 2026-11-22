//! Iron pulls and steel pushes (P4): a pure step from positions and velocities to new ones.
//!
//! Units are the game's: blocks and blocks per tick. The adapter reads positions from the
//! server, calls [`step`] once per tick and applies the velocities it returns; the engine moves
//! both bodies with its own collision. Values are trial values, not balance.
use crate::Vec3;

/// Which way the force acts on the target, seen from the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Iron: the target and the player are drawn together.
    Pull,
    /// Steel: the target and the player are driven apart.
    Push,
}

/// How the target answers the force.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// A block: it does not move, so the whole change goes to the player.
    Anchor,
    /// A dropped item. One on the ground pushed downwards is held by the ground and acts as an
    /// anchor (the coin jump).
    Item { on_ground: bool },
}

/// Accelerations in blocks/tick², distances in blocks, speeds in blocks/tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForceRules {
    pub accel: f64,
    pub range: f64,
    pub player_cap: f64,
    pub item_cap: f64,
    /// Mass of a dropped item relative to the player's 1.
    pub item_mass: f64,
    /// Below this downward share of the direction an on-ground item is not an anchor.
    pub anchor_down: f64,
    /// Share of the vertical velocity removed each tick while the force lifts the player.
    /// Falloff makes a hover a spring; this damps it near critical (2·√(accel/range) ≈ 0.17),
    /// and absorbs the client's late velocity reports.
    pub lift_damping: f64,
}

impl ForceRules {
    pub const BURNING: Self = Self {
        accel: 0.12,
        range: 16.0,
        player_cap: 1.0,
        item_cap: 2.0,
        item_mass: 0.05,
        anchor_down: 0.5,
        lift_damping: 0.15,
    };
    pub const FLARED: Self = Self {
        accel: 0.24,
        range: 24.0,
        player_cap: 1.4,
        ..Self::BURNING
    };

    pub fn for_flare(flared: bool) -> Self {
        if flared { Self::FLARED } else { Self::BURNING }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepInput {
    /// The point the force acts on: the middle of the player's body.
    pub player: Vec3,
    pub player_velocity: Vec3,
    /// The block's centre or the item's position.
    pub target: Vec3,
    pub target_velocity: Vec3,
    pub body: Body,
    pub direction: Direction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepOutcome {
    pub distance: f64,
    /// Whether the target held still and the player took the change.
    pub anchored: bool,
    /// The change of the player's velocity (zero when the target moves instead).
    pub player_delta: Vec3,
    /// The player's new velocity, `None` when it is left alone.
    pub player_velocity: Option<Vec3>,
    /// The target's new velocity, `None` for an anchor.
    pub target_velocity: Option<Vec3>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    NonFinite,
    /// The target is beyond the range (the caller drops it).
    OutOfRange,
    /// The player and the target are at the same point: no direction.
    Coincident,
}

/// Vanilla's gravity and vertical air drag for a player (`LivingEntity.travel`).
pub const PLAYER_GRAVITY: f64 = 0.08;
pub const VERTICAL_DRAG: f64 = 0.98;

/// The velocity the client holds after the move it last reported. The server reads that move
/// (`getKnownMovement`), but the client then applied gravity and drag: setting the player's
/// velocity from the bare report every tick would cancel gravity (N3 hovered at the range's
/// edge). Horizontal friction is not modelled: under a force the player glides.
pub fn client_velocity(reported: Vec3) -> Vec3 {
    Vec3 {
        y: (reported.y - PLAYER_GRAVITY) * VERTICAL_DRAG,
        ..reported
    }
}

/// The acceleration at a distance: full at the player, falling linearly to zero at the range.
pub fn acceleration(rules: &ForceRules, distance: f64) -> f64 {
    if !(0.0..rules.range).contains(&distance) {
        return 0.0;
    }
    rules.accel * (1.0 - distance / rules.range)
}

/// One tick of force between the player and a target.
pub fn step(rules: &ForceRules, input: &StepInput) -> Result<StepOutcome, StepError> {
    if ![
        input.player,
        input.player_velocity,
        input.target,
        input.target_velocity,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return Err(StepError::NonFinite);
    }
    let offset = sub(input.target, input.player);
    let distance = norm(offset);
    if distance >= rules.range {
        return Err(StepError::OutOfRange);
    }
    if distance < 1e-6 {
        return Err(StepError::Coincident);
    }
    // `toward` points from the player to the target.
    let toward = scale(offset, 1.0 / distance);
    let a = acceleration(rules, distance);
    let sign = match input.direction {
        Direction::Pull => 1.0,
        Direction::Push => -1.0,
    };
    let anchored = match input.body {
        Body::Anchor => true,
        Body::Item { on_ground } => {
            on_ground && input.direction == Direction::Push && toward.y < -rules.anchor_down
        }
    };
    if anchored {
        let mut player_delta = scale(toward, sign * a);
        if player_delta.y > 0.0 {
            player_delta.y -= rules.lift_damping * input.player_velocity.y;
        }
        let velocity = capped(
            input.player_velocity,
            add(input.player_velocity, player_delta),
            rules.player_cap,
        );
        return Ok(StepOutcome {
            distance,
            anchored,
            player_delta,
            player_velocity: Some(velocity),
            target_velocity: None,
        });
    }
    // The item takes the change, scaled by its lightness. Pulled close, it slows so it settles
    // at the player to be picked up instead of flying through.
    let item_delta = scale(toward, -sign * a / rules.item_mass);
    let mut velocity = capped(
        input.target_velocity,
        add(input.target_velocity, item_delta),
        rules.item_cap,
    );
    if input.direction == Direction::Pull && distance < 1.5 {
        velocity = limit(velocity, distance * 0.5);
    }
    Ok(StepOutcome {
        distance,
        anchored,
        player_delta: Vec3::default(),
        player_velocity: None,
        target_velocity: Some(velocity),
    })
}

/// The damage a pushed coin deals to a living body it crosses, `None` below the hurting speed.
pub fn coin_damage(speed: f64) -> Option<f32> {
    (speed.is_finite() && speed > 0.5).then_some((2.0 + 4.0 * speed) as f32)
}

/// A force never makes a body faster than the cap; a body already faster (falling) keeps its
/// speed but is not sped up.
fn capped(before: Vec3, after: Vec3, cap: f64) -> Vec3 {
    let speed = norm(after);
    if speed <= cap || speed <= norm(before) {
        return after;
    }
    limit(after, cap.max(norm(before)))
}

fn limit(v: Vec3, max: f64) -> Vec3 {
    let speed = norm(v);
    if speed <= max || speed == 0.0 {
        v
    } else {
        scale(v, max / speed)
    }
}

pub fn norm(v: Vec3) -> f64 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

fn add(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.x + b.x,
        y: a.y + b.y,
        z: a.z + b.z,
    }
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3 {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

fn scale(v: Vec3, k: f64) -> Vec3 {
    Vec3 {
        x: v.x * k,
        y: v.y * k,
        z: v.z * k,
    }
}
