//! Iron pulls and steel pushes (P4), server side.
//!
//! Each tick, for a player burning iron or steel: keep or find a metal target along the view,
//! ask `allomancy_core::physics::step` for the new velocities and hand them to the engine,
//! which moves both bodies with its own collision. The core decides; this file observes and
//! applies. Log lines follow the contract in `docs/plan.md` (P4).
//!
//! Coin hits are only collected during the tick and dealt by [`deal_hits`] once the caller
//! has released its locks: a lethal hit fires death hooks that take those locks again.
use crate::bridge::{Session, metal_name, player_id, world_id};
use allomancy_core::physics::{self, Body, Direction, ForceRules, StepError, StepInput};
use allomancy_core::{
    Authority, BurnState, Command, Effect, InputContext, Metal, Tick, Vec3, WorldSnapshot,
    apply_command, credit_reserve, is_coin, is_metal, update_player,
};
use mod_api::{ClipBlock, ClipFluid, EntityRef, Host, LevelRef, PlayerRef, ServerHost};
use std::collections::{BTreeMap, HashSet};

/// The middle of a standing player's body, above the feet.
const BODY_MIDDLE: f64 = 0.9;
/// Entities are hit by a ray grown by this margin, as vanilla picks projectiles' targets.
const RAY_MARGIN: f64 = 0.3;
/// A coin hits bodies its move crosses, with this margin.
const COIN_MARGIN: f64 = 0.1;
/// Damage type of a coin's hit (`minecraft:thrown`, as a snowball's).
const COIN_DAMAGE: &str = "minecraft:thrown";
/// Slack before a block in front of a held target counts as an obstacle.
const SIGHT_SLACK: f64 = 0.05;

#[derive(Clone, Debug, PartialEq)]
enum Target {
    Block {
        level: LevelRef,
        pos: [i32; 3],
        id: String,
    },
    Item {
        level: LevelRef,
        entity: EntityRef,
        id: String,
    },
}

impl Target {
    fn level(&self) -> LevelRef {
        match self {
            Self::Block { level, .. } | Self::Item { level, .. } => *level,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Block { id, .. } | Self::Item { id, .. } => id,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Self::Block { .. } => "block",
            Self::Item { .. } => "item",
        }
    }
}

/// A target held for one metal: lighting the other one (or relighting) drops it.
#[derive(Clone, Debug)]
struct Held {
    metal: Metal,
    target: Target,
}

/// A pushed coin in flight. It is followed after its pusher lets go (steel off, out of range)
/// until it hits or settles: its real move each tick, already stopped at blocks by the
/// engine, is what it can hit.
struct Flight {
    coin: EntityRef,
    shooter: EntityRef,
    last: Vec3,
    /// The speed the coin had at the end of the last tick: its next real move is at most this
    /// (plus a little gravity). A longer move was not flight but a teleport.
    expected: f64,
    /// The moves made between reported teleports in this tick (each `last` → that teleport's
    /// start), in order, still to be swept at the tick's end.
    before_jumps: Vec<(Vec3, Vec3)>,
    ticks_left: u8,
}

/// How long a coin is followed after its last push, counted from the tick after it.
const FLIGHT_TICKS: u8 = 40;
/// Stretches kept between reported teleports of one coin in one tick.
const MAX_JUMPS_PER_TICK: usize = 8;
/// Slack on a coin's expected move: item gravity (0.04) and rounding.
const COIN_STEP_SLACK: f64 = 0.1;

/// A coin's hit, dealt after the tick's locks are released.
pub(crate) struct Hit {
    victim: EntityRef,
    coin: EntityRef,
    shooter: EntityRef,
    kind: String,
    speed: f64,
    damage: f32,
}

#[derive(Default)]
pub(crate) struct Powers {
    held: BTreeMap<u128, Held>,
    /// Coins that already hit: never a target again, so one coin hits once.
    spent: HashSet<EntityRef>,
    flights: Vec<Flight>,
}

/// The metal a player acts with: iron or steel, burning or flared (the core lights one only).
fn acting(session: &Session, uuid: u128) -> Option<(Metal, bool)> {
    let state = session.saved(uuid)?.state();
    [Metal::Iron, Metal::Steel].into_iter().find_map(|metal| {
        match state.metals[metal.index()].burn {
            BurnState::Off => None,
            burn => Some((metal, burn == BurnState::Flared)),
        }
    })
}

fn v3([x, y, z]: [f64; 3]) -> Vec3 {
    Vec3 { x, y, z }
}

fn fmt3(v: Vec3) -> String {
    format!("{:.3},{:.3},{:.3}", v.x, v.y, v.z)
}

fn distance(a: [f64; 3], b: Vec3) -> f64 {
    physics::norm(Vec3 {
        x: b.x - a[0],
        y: b.y - a[1],
        z: b.z - a[2],
    })
}

impl Powers {
    /// One server tick of force. Returns the coin hits to deal once the locks are released.
    pub(crate) fn tick(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        tick: u64,
    ) -> Vec<Hit> {
        let mut online = host.players().server_players();
        online.sort_by_key(|player| player.uuid());
        self.held
            .retain(|uuid, _| online.iter().any(|p| p.uuid() == *uuid));
        let read: &dyn Host = host;
        self.spent.retain(|coin| coin.position(read).is_some());
        // Last tick's coin moves are resolved first, with the pushers they had: a push this tick
        // must not take over a segment it did not cause.
        let mut hits = Vec::new();
        self.follow_flights(host, tick, &mut hits);
        for player in online {
            let uuid = player.uuid();
            match acting(session, uuid) {
                None => {
                    if let Some(held) = self.held.get(&uuid) {
                        let empty = session.saved(uuid).is_some_and(|saved| {
                            saved.state().metals[held.metal.index()].reserve_milli == 0
                        });
                        self.clear(uuid, if empty { "exhausted" } else { "off" });
                    }
                }
                Some((metal, flared)) => {
                    if self.held.get(&uuid).is_some_and(|held| held.metal != metal) {
                        self.clear(uuid, "switched");
                    }
                    self.act(host, session, player, metal, flared, tick);
                }
            }
        }
        hits
    }

    pub(crate) fn clear(&mut self, uuid: u128, reason: &str) {
        if self.held.remove(&uuid).is_some() {
            eprintln!("MISTBRN event=target_cleared reason={reason}");
        }
    }

    pub(crate) fn died(&mut self, uuid: u128) {
        self.clear(uuid, "death");
    }

    /// A new level: the target belongs to the old one, and iron and steel go out. The shutdown
    /// reads nothing from the game, so it cannot be skipped by a missing position.
    pub(crate) fn changed_dimension(&mut self, session: &mut Session, uuid: u128) {
        self.clear(uuid, "dimension");
        let burning = acting(session, uuid).is_some();
        let (result, changed) = run_core(
            session,
            uuid,
            Vec3::default(),
            Command::Extinguish { metal: None },
        );
        if burning || changed > 0 {
            eprintln!("MISTBRN event=shutdown reason=dimension result={result} changed={changed}");
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn act(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        player: PlayerRef,
        metal: Metal,
        flared: bool,
        tick: u64,
    ) {
        let uuid = player.uuid();
        let rules = ForceRules::for_flare(flared);
        let read: &dyn Host = host;
        let body = player.entity();
        let Some(level) = player.level(read) else {
            self.clear(uuid, "invalid");
            return;
        };
        if self
            .held
            .get(&uuid)
            .is_some_and(|held| held.target.level() != level)
        {
            // The tick can come before `player_changed_dimension`: the same shutdown.
            self.changed_dimension(session, uuid);
            return;
        }
        if self.held.get(&uuid).is_some_and(|held| {
            matches!(held.target, Target::Item { entity, .. } if self.spent.contains(&entity))
        }) {
            // Another player's push already spent this coin on a body.
            self.clear(uuid, "hit");
        }
        // A player whose geometry cannot be read drops the target: when the reading comes back
        // the player aims anew instead of resuming on what they may no longer look at.
        let (Some(feet), Some(velocity), Some(eye)) = (
            body.position(read),
            body.velocity(read),
            body.eye_position(read),
        ) else {
            self.clear(uuid, "invalid");
            return;
        };
        if !self.held.contains_key(&uuid) {
            let report = crate::scenario::commands_enabled() && tick.is_multiple_of(20);
            let Some(target) = acquire(read, level, body, rules.range, &self.spent, report) else {
                return;
            };
            let at = target_point(read, level, &target).map_or_else(String::new, fmt3);
            eprintln!(
                "MISTBRN event=target_acquired kind={} id={} at={at}",
                target.kind(),
                target.id()
            );
            self.held.insert(uuid, Held { metal, target });
        }
        let Some(target) = self.held.get(&uuid).map(|held| held.target.clone()) else {
            return;
        };
        let Some(point) = target_point(read, level, &target) else {
            let reason = match target {
                Target::Block { .. } => "broken",
                Target::Item { .. } => "gone",
            };
            self.clear(uuid, reason);
            return;
        };
        if !in_sight(read, level, eye, point, &target) {
            self.clear(uuid, "blocked");
            return;
        }
        let (target_body, target_velocity) = match &target {
            Target::Block { .. } => (Body::Anchor, Vec3::default()),
            Target::Item { entity, .. } => (
                Body::Item {
                    on_ground: entity.on_ground(read).unwrap_or(false),
                },
                entity.velocity(read).map(v3).unwrap_or_default(),
            ),
        };
        let direction = if metal == Metal::Iron {
            Direction::Pull
        } else {
            Direction::Push
        };
        let input = StepInput {
            player: Vec3 {
                x: feet[0],
                y: feet[1] + BODY_MIDDLE,
                z: feet[2],
            },
            player_velocity: physics::client_velocity(v3(velocity)),
            target: point,
            target_velocity,
            body: target_body,
            direction,
        };
        let outcome = match physics::step(&rules, &input) {
            Ok(outcome) => outcome,
            Err(StepError::OutOfRange) => {
                self.clear(uuid, "range");
                return;
            }
            Err(StepError::NonFinite | StepError::Coincident | StepError::InvalidRules) => {
                self.clear(uuid, "invalid");
                return;
            }
        };
        if let Some(next) = outcome.player_velocity
            && player.set_velocity(host, [next.x, next.y, next.z])
            && outcome.player_delta.y > 0.0
        {
            let _ = body.reset_fall_distance(host);
        }
        eprintln!(
            "MISTBRN p4 tick={tick} metal={} pos={} vel={} target={} dist={:.3} dv={} anchor={}",
            metal_name(metal),
            fmt3(v3(feet)),
            fmt3(v3(velocity)),
            target.kind(),
            outcome.distance,
            fmt3(outcome.player_delta),
            outcome.anchored
        );
        let (Some(next), Target::Item { entity, id, .. }) = (outcome.target_velocity, &target)
        else {
            return;
        };
        let written = entity.set_velocity(host, [next.x, next.y, next.z]);
        let speed = physics::norm(next);
        // Any velocity written on a followed coin (a pull too) is what it moves by next.
        if written {
            for flight in self.flights.iter_mut().filter(|f| f.coin == *entity) {
                flight.expected = speed;
            }
        }
        eprintln!(
            "MISTBRN p4item tick={tick} entity={} pos={} vel={} speed={speed:.6}",
            entity.id(),
            fmt3(point),
            fmt3(next)
        );
        if direction == Direction::Push && is_coin(id) {
            self.pushed_coin(*entity, body, point, speed, tick);
        }
    }

    /// Moves every followed coin's segment forward and collects at most one hit per coin.
    fn follow_flights(&mut self, host: &mut dyn ServerHost, tick: u64, hits: &mut Vec<Hit>) {
        for mut flight in std::mem::take(&mut self.flights) {
            let read: &dyn Host = host;
            let (Some(level), Some([x, y, z])) =
                (flight.coin.level(read), flight.coin.position(read))
            else {
                continue;
            };
            if self.spent.contains(&flight.coin) {
                continue;
            }
            let now = Vec3 { x, y: y + 0.125, z };
            eprintln!(
                "MISTBRN p4coin tick={tick} entity={} pos={}",
                flight.coin.id(),
                fmt3(now)
            );
            // The moves to sweep: any stretch before a reported teleport, then the one since.
            // Each must fit the coin's own speed; a longer one was an unreported teleport and
            // hurts nothing.
            let bound = flight.expected + COIN_STEP_SLACK;
            flight.expected = flight
                .coin
                .velocity(read)
                .map_or(0.0, |v| physics::norm(v3(v)));
            let segments = std::mem::take(&mut flight.before_jumps)
                .into_iter()
                .chain(std::iter::once((flight.last, now)));
            let found = segments
                .filter(|(a, b)| distance([a.x, a.y, a.z], *b) <= bound)
                .find_map(|(a, b)| coin_victim(read, level, flight.shooter, flight.coin, a, b));
            if let Some(hit) = found {
                // The flight ends at the body: the coin stops and is never a target again.
                let _ = flight.coin.set_velocity(host, [0.0, 0.0, 0.0]);
                self.spent.insert(flight.coin);
                let coin = flight.coin;
                let holders: Vec<u128> = self
                    .held
                    .iter()
                    .filter(|(_, held)| {
                        matches!(held.target, Target::Item { entity, .. } if entity == coin)
                    })
                    .map(|(uuid, _)| *uuid)
                    .collect();
                for uuid in holders {
                    self.clear(uuid, "hit");
                }
                hits.push(hit);
                continue;
            }
            flight.last = now;
            flight.ticks_left = flight.ticks_left.saturating_sub(1);
            if flight.ticks_left > 0 {
                self.flights.push(flight);
            }
        }
    }

    /// A command teleported an entity within its level (`entity_teleport`): a followed coin's
    /// segment restarts at the destination, so the jump is never swept for bodies.
    pub(crate) fn teleported(&mut self, entity: u32, from: [f64; 3], to: [f64; 3]) {
        let point = |[x, y, z]: [f64; 3]| Vec3 { x, y: y + 0.125, z };
        for flight in self.flights.iter_mut().filter(|f| f.coin.id() == entity) {
            // Keep every stretch flown between jumps; the jumps themselves are never swept.
            // Bounded: a command loop teleporting one coin cannot grow it without end.
            if flight.before_jumps.len() < MAX_JUMPS_PER_TICK {
                flight.before_jumps.push((flight.last, point(from)));
            }
            flight.last = point(to);
        }
    }

    /// Starts or refreshes the flight of a coin pushed this tick to `speed`.
    fn pushed_coin(
        &mut self,
        coin: EntityRef,
        shooter: EntityRef,
        at: Vec3,
        speed: f64,
        tick: u64,
    ) {
        match self.flights.iter_mut().find(|f| f.coin == coin) {
            Some(flight) => {
                flight.shooter = shooter;
                flight.expected = speed;
                flight.ticks_left = FLIGHT_TICKS;
            }
            None => {
                eprintln!(
                    "MISTBRN p4coin tick={tick} entity={} pos={}",
                    coin.id(),
                    fmt3(at)
                );
                self.flights.push(Flight {
                    coin,
                    shooter,
                    last: at,
                    expected: speed,
                    before_jumps: Vec::new(),
                    ticks_left: FLIGHT_TICKS,
                });
            }
        }
    }
}

/// Deals the tick's coin hits. Call with no session or power lock held.
pub(crate) fn deal_hits(host: &mut dyn ServerHost, hits: Vec<Hit>) {
    for hit in hits {
        let read: &dyn Host = host;
        let before = read.living(hit.victim).and_then(|l| l.health(read));
        let dealt = hit.victim.hurt(
            host,
            COIN_DAMAGE,
            hit.damage,
            Some(hit.coin),
            Some(hit.shooter),
        );
        let read: &dyn Host = host;
        let after = read.living(hit.victim).and_then(|l| l.health(read));
        let health = |h: Option<f32>| h.map_or_else(|| "none".into(), |h| format!("{h:.3}"));
        eprintln!(
            "MISTBRN event=coin_hit entity={} speed={:.6} damage={:.3} dealt={} health_before={} health_after={}",
            hit.kind,
            hit.speed,
            hit.damage,
            dealt == Some(true),
            health(before),
            health(after)
        );
    }
}

/// The first living body the coin's real move (`from` → `to`, last tick to this one) crossed,
/// if it moved fast enough to hurt. The engine already stopped that move at blocks, so a body
/// behind a wall is never on it; the clip guards the start inside a block.
fn coin_victim(
    host: &dyn Host,
    level: LevelRef,
    shooter: EntityRef,
    coin: EntityRef,
    from: Vec3,
    to: Vec3,
) -> Option<Hit> {
    let start = [from.x, from.y, from.z];
    let end = [to.x, to.y, to.z];
    let speed = distance(start, to);
    let damage = physics::coin_damage(speed)?;
    // A failed block read (a dynamic shape, an unloaded column) cancels the hit.
    let clip = level.clip(host, start, end, ClipBlock::Collider, ClipFluid::None)?;
    let wall = if clip.miss {
        speed
    } else {
        distance(start, v3(clip.location))
    };
    let victim = level
        .ray_trace_entities(host, start, end, COIN_MARGIN, Some(shooter))?
        .into_iter()
        .take_while(|hit| hit.distance < wall)
        .map(|hit| hit.entity)
        .find(|entity| {
            *entity != coin
                && host
                    .living(*entity)
                    .is_some_and(|l| l.alive(host) == Some(true))
        })?;
    Some(Hit {
        victim,
        coin,
        shooter,
        kind: victim.kind(host).map(|k| k.id(host)).unwrap_or_default(),
        speed,
        damage,
    })
}

/// Whether nothing solid stands between the eyes and a held target.
fn in_sight(host: &dyn Host, level: LevelRef, eye: [f64; 3], point: Vec3, target: &Target) -> bool {
    let to = [point.x, point.y, point.z];
    let Some(hit) = level.clip(host, eye, to, ClipBlock::Outline, ClipFluid::None) else {
        return false;
    };
    if hit.miss {
        return true;
    }
    match target {
        Target::Block { pos, .. } => hit.block == *pos,
        Target::Item { .. } => {
            distance(eye, v3(hit.location)) + SIGHT_SLACK >= distance(eye, point)
        }
    }
}

/// The nearest metal along the view within `range`: an item before the first block, or that
/// block. Blocks stop the ray: metal behind a wall is not reached.
fn acquire(
    host: &dyn Host,
    level: LevelRef,
    body: EntityRef,
    range: f64,
    spent: &HashSet<EntityRef>,
    report: bool,
) -> Option<Target> {
    let eye = body.eye_position(host)?;
    let look = body.view_vector(host)?;
    if !v3(eye).is_finite() || !v3(look).is_finite() {
        return None;
    }
    let end = [
        eye[0] + look[0] * range,
        eye[1] + look[1] * range,
        eye[2] + look[2] * range,
    ];
    let clip = level.clip(host, eye, end, ClipBlock::Outline, ClipFluid::None);
    let block = clip.filter(|hit| !hit.miss && !hit.world_border);
    let block_distance = block.map_or(range, |hit| distance(eye, v3(hit.location)));
    let hits = level
        .ray_trace_entities(host, eye, end, RAY_MARGIN, Some(body))
        .unwrap_or_default();
    if report {
        let seen: Vec<String> = hits
            .iter()
            .map(|hit| {
                let kind = hit.entity.kind(host).map(|k| k.id(host));
                format!("{}@{:.2}", kind.unwrap_or_default(), hit.distance)
            })
            .collect();
        eprintln!(
            "MISTBRN event=target_search eye={} look={} block={:?} block_distance={block_distance:.3} entities={}",
            fmt3(v3(eye)),
            fmt3(v3(look)),
            block.map(|hit| hit.block),
            seen.join(";")
        );
    }
    let item = hits
        .into_iter()
        .take_while(|hit| hit.distance <= block_distance)
        .filter(|hit| !spent.contains(&hit.entity))
        .find_map(|hit| {
            let id = host.item_entity(hit.entity)?.stack(host)?.item().id(host);
            is_metal(&id).then_some(Target::Item {
                level,
                entity: hit.entity,
                id,
            })
        });
    if item.is_some() {
        return item;
    }
    let hit = block?;
    let id = level.block(host, hit.block)?.id(host);
    is_metal(&id).then_some(Target::Block {
        level,
        pos: hit.block,
        id,
    })
}

/// Where the force acts on a target, `None` once it is no longer the metal it was.
fn target_point(host: &dyn Host, level: LevelRef, target: &Target) -> Option<Vec3> {
    match target {
        Target::Block { pos, id, .. } => {
            let now = level.block(host, *pos)?.id(host);
            (now == *id).then(|| Vec3 {
                x: f64::from(pos[0]) + 0.5,
                y: f64::from(pos[1]) + 0.5,
                z: f64::from(pos[2]) + 0.5,
            })
        }
        Target::Item { entity, id, .. } => {
            let now = host.item_entity(*entity)?.stack(host)?.item().id(host);
            if now != *id {
                return None;
            }
            let [x, y, z] = entity.position(host)?;
            Some(Vec3 { x, y: y + 0.125, z })
        }
    }
}

/// Applies one command through the core, with the player's own last tick (a snapshot older
/// than it is refused as `StaleTick`). The command is tried on a copy first: a refusal or a
/// command that changes nothing leaves the save untouched (no revision, no write). Returns the
/// outcome as a log word and the number of accepted changes.
fn run_core(
    session: &mut Session,
    uuid: u128,
    position: Vec3,
    command: Command,
) -> (String, usize) {
    let Some(saved) = session.saved(uuid) else {
        return ("not_active".into(), 0);
    };
    let mut probe = saved.state().clone();
    let ctx = snapshot(uuid, position, probe.last_tick.unwrap_or(Tick(0)));
    let effects: Vec<Effect> = match apply_command(&mut probe, command.clone(), &ctx) {
        Ok(effects) => effects,
        Err(error) => {
            let word = format!("{error:?}");
            eprintln!("MISTBRN event=core_refused result={word}");
            return (word, 0);
        }
    };
    if effects.is_empty() {
        return ("ok".into(), 0);
    }
    let result = session.change(uuid, true, |saved| {
        update_player(saved, |state, _| {
            let _ = apply_command(state, command, &ctx);
            Ok(())
        })
    });
    match result {
        Ok(()) => ("ok".into(), effects.len()),
        Err(error) => {
            eprintln!("MISTBRN event=core_refused result={error}");
            (error, 0)
        }
    }
}

fn snapshot(uuid: u128, position: Vec3, tick: Tick) -> WorldSnapshot {
    WorldSnapshot {
        tick,
        world_id: world_id(),
        actor: player_id(uuid),
        position,
        targets: Vec::new(),
        authority: Authority::Server,
        // Test commands and safety shutdowns only: P7a passes the client's real input state.
        input: InputContext {
            gameplay_allowed: true,
        },
    }
}

fn parse_metal(name: &str) -> Option<Metal> {
    Metal::ALL.into_iter().find(|m| metal_name(*m) == name)
}

/// `/mistbrn-test …`, registered only when `MISTBRN_TEST_SCENARIO=p4`.
pub(crate) const TEST_COMMANDS: [&str; 4] = [
    "mistbrn-test burn <metal:word>",
    "mistbrn-test burn <metal:word> <mode:word>",
    "mistbrn-test off",
    "mistbrn-test credit <metal:word> <milli:integer>",
];

pub(crate) struct TestCommand<'a> {
    pub(crate) literals: &'a [String],
    pub(crate) metal: Option<&'a str>,
    pub(crate) mode: Option<&'a str>,
    pub(crate) milli: Option<i32>,
}

/// Runs one test command for `player`. Burning goes through the core's `apply_command`;
/// crediting unlocks and credits without a vial (test runs only).
pub(crate) fn run_test_command(
    host: &mut dyn ServerHost,
    session: &mut Session,
    powers: &mut Powers,
    player: PlayerRef,
    command: &TestCommand<'_>,
) {
    let uuid = player.uuid();
    let Some(position) = player.entity().position(host).map(v3) else {
        return;
    };
    let word = command.literals.get(1).map(String::as_str);
    let mut changed_save = false;
    match (word, command.metal.and_then(parse_metal)) {
        (Some("off"), _) => {
            let (result, changed) =
                run_core(session, uuid, position, Command::Extinguish { metal: None });
            if result == "ok" {
                powers.clear(uuid, "off");
            }
            changed_save = changed > 0;
            eprintln!("MISTBRN event=burn metal=all burn=off result={result} changed={changed}");
        }
        (Some("burn"), Some(metal)) => {
            let flared = command.mode == Some("flare");
            let core = if flared {
                Command::Flare { metal }
            } else {
                Command::Activate { metal }
            };
            let (result, changed) = run_core(session, uuid, position, core);
            // A new lighting aims anew: the old target is not kept across it.
            if changed > 0 {
                powers.clear(uuid, "relit");
            }
            changed_save = changed > 0;
            eprintln!(
                "MISTBRN event=burn metal={} burn={} result={result} changed={changed}",
                metal_name(metal),
                if flared { "flared" } else { "burning" }
            );
        }
        (Some("credit"), Some(metal)) => {
            let amount = i64::from(command.milli.unwrap_or(0));
            let result = session.change(uuid, true, |saved| {
                let rules = saved.rules().clone();
                update_player(saved, |state, accounting| {
                    state.metals[metal.index()].unlocked = true;
                    credit_reserve(state, accounting, &rules, metal, amount)
                })
            });
            changed_save = result.is_ok();
            eprintln!(
                "MISTBRN event=test_credit metal={} milli={amount} result={}",
                metal_name(metal),
                result.err().unwrap_or_else(|| "ok".into())
            );
        }
        _ => eprintln!("MISTBRN event=test_command result=unknown"),
    }
    // Only a command that changed the save writes it now; pending tick changes keep their
    // own schedule.
    if changed_save && let Some(player) = host.player(uuid) {
        session.flush(host, player);
    }
}
