//! Iron pulls and steel pushes (P4), server side.
//!
//! Each tick, for a player burning iron or steel: keep or find a metal target along the view,
//! ask `allomancy_core::physics::step` for the new velocities and hand them to the engine,
//! which moves both bodies with its own collision. The core decides; this file observes and
//! applies. Log lines follow the contract in `docs/plan.md` (P4).
use crate::bridge::{Session, metal_name, player_id, world_id};
use allomancy_core::physics::{self, Body, Direction, ForceRules, StepError, StepInput};
use allomancy_core::{
    Authority, BurnState, Command, InputContext, Metal, Tick, Vec3, WorldSnapshot, apply_command,
    credit_reserve, is_coin, is_metal, update_player,
};
use mod_api::{ClipBlock, ClipFluid, EntityRef, Host, LevelRef, PlayerRef, ServerHost};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

/// The middle of a standing player's body, above the feet.
const BODY_MIDDLE: f64 = 0.9;
/// Entities are hit by a ray grown by this margin, as vanilla picks projectiles' targets.
const RAY_MARGIN: f64 = 0.3;
/// Damage type of a coin's hit (`minecraft:thrown`, as a snowball's).
const COIN_DAMAGE: &str = "minecraft:thrown";

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

    fn kind(&self) -> &'static str {
        match self {
            Self::Block { .. } => "block",
            Self::Item { .. } => "item",
        }
    }
}

#[derive(Default)]
pub(crate) struct Powers {
    targets: BTreeMap<u128, Target>,
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

impl Powers {
    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, session: &mut Session, tick: u64) {
        let mut online = host.players().server_players();
        online.sort_by_key(|player| player.uuid());
        self.targets
            .retain(|uuid, _| online.iter().any(|p| p.uuid() == *uuid));
        for player in online {
            let uuid = player.uuid();
            let Some((metal, flared)) = acting(session, uuid) else {
                if self.targets.contains_key(&uuid) {
                    let empty = session.saved(uuid).is_some_and(|saved| {
                        [Metal::Iron, Metal::Steel]
                            .iter()
                            .any(|m| saved.state().metals[m.index()].reserve_milli == 0)
                    });
                    self.clear(uuid, if empty { "exhausted" } else { "off" });
                }
                continue;
            };
            self.act(host, session, player, metal, flared, tick);
        }
    }

    fn clear(&mut self, uuid: u128, reason: &str) {
        if self.targets.remove(&uuid).is_some() {
            eprintln!("MISTBRN event=target_cleared reason={reason}");
        }
    }

    pub(crate) fn died(&mut self, uuid: u128) {
        self.clear(uuid, "death");
    }

    /// A new level: the target belongs to the old one. Iron and steel go out.
    pub(crate) fn changed_dimension(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        uuid: u128,
    ) {
        self.leave_level(host, session, uuid);
    }

    fn leave_level(&mut self, host: &mut dyn ServerHost, session: &mut Session, uuid: u128) {
        self.clear(uuid, "dimension");
        let Some(position) = host
            .player(uuid)
            .and_then(|player| player.entity().position(host))
        else {
            return;
        };
        for metal in [Metal::Iron, Metal::Steel] {
            run_core(
                session,
                uuid,
                v3(position),
                Command::Extinguish { metal: Some(metal) },
            );
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
        let (Some(level), Some(feet), Some(velocity)) =
            (player.level(read), body.position(read), body.velocity(read))
        else {
            return;
        };
        if let Entry::Vacant(slot) = self.targets.entry(uuid)
            && let Some(target) = acquire(
                read,
                level,
                body,
                rules.range,
                crate::scenario::enabled() && tick.is_multiple_of(20),
            )
        {
            let at = target_point(read, level, &target).map_or_else(String::new, fmt3);
            let id = match &target {
                Target::Block { id, .. } | Target::Item { id, .. } => id.as_str(),
            };
            eprintln!(
                "MISTBRN event=target_acquired kind={} id={id} at={at}",
                target.kind()
            );
            slot.insert(target);
        }
        let player_point = Vec3 {
            x: feet[0],
            y: feet[1] + BODY_MIDDLE,
            z: feet[2],
        };
        let Some(target) = self.targets.get(&uuid).cloned() else {
            return;
        };
        if target.level() != level {
            // The tick can come before `player_changed_dimension`: the same clear.
            return self.leave_level(host, session, uuid);
        }
        let Some(point) = target_point(read, level, &target) else {
            let reason = match target {
                Target::Block { .. } => "broken",
                Target::Item { .. } => "gone",
            };
            self.clear(uuid, reason);
            return;
        };
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
            player: player_point,
            player_velocity: physics::client_velocity(v3(velocity)),
            target: point,
            target_velocity,
            body: target_body,
            direction,
        };
        let outcome = match physics::step(&rules, &input) {
            Ok(outcome) => outcome,
            Err(StepError::OutOfRange) => return self.clear(uuid, "range"),
            Err(StepError::NonFinite | StepError::Coincident) => return,
        };
        if let Some(next) = outcome.player_velocity {
            let _ = player.set_velocity(host, [next.x, next.y, next.z]);
            if outcome.player_delta.y > 0.0 {
                let _ = body.reset_fall_distance(host);
            }
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
        if let (Some(next), Target::Item { entity, id, .. }) = (outcome.target_velocity, &target) {
            let _ = entity.set_velocity(host, [next.x, next.y, next.z]);
            eprintln!(
                "MISTBRN p4item tick={tick} entity={} pos={} vel={}",
                entity.id(),
                fmt3(point),
                fmt3(next)
            );
            if direction == Direction::Push && is_coin(id) {
                self.coin_hit(host, level, body, *entity, point, next, uuid);
            }
        }
    }

    /// A pushed coin hurts the first living body its next move crosses, then stops.
    #[allow(clippy::too_many_arguments)]
    fn coin_hit(
        &mut self,
        host: &mut dyn ServerHost,
        level: LevelRef,
        shooter: EntityRef,
        coin: EntityRef,
        from: Vec3,
        velocity: Vec3,
        uuid: u128,
    ) {
        let speed = physics::norm(velocity);
        let Some(damage) = physics::coin_damage(speed) else {
            return;
        };
        let to = [
            from.x + velocity.x,
            from.y + velocity.y,
            from.z + velocity.z,
        ];
        let read: &dyn Host = host;
        let Some(hits) =
            level.ray_trace_entities(read, [from.x, from.y, from.z], to, 0.1, Some(shooter))
        else {
            return;
        };
        let Some(victim) = hits.into_iter().map(|hit| hit.entity).find(|entity| {
            *entity != coin
                && read
                    .living(*entity)
                    .is_some_and(|l| l.alive(read) == Some(true))
        }) else {
            return;
        };
        let kind = victim.kind(read).map(|k| k.id(read)).unwrap_or_default();
        let _ = victim.hurt(host, COIN_DAMAGE, damage, Some(coin), Some(shooter));
        let _ = coin.set_velocity(host, [0.0, 0.0, 0.0]);
        eprintln!("MISTBRN event=coin_hit entity={kind} speed={speed:.3} damage={damage:.3}");
        self.clear(uuid, "gone");
    }
}

/// The nearest metal along the view within `range`: an item before the first block, or that
/// block. Blocks stop the ray: metal behind a wall is not reached.
fn acquire(
    host: &dyn Host,
    level: LevelRef,
    body: EntityRef,
    range: f64,
    report: bool,
) -> Option<Target> {
    let eye = body.eye_position(host)?;
    let look = body.view_vector(host)?;
    let end = [
        eye[0] + look[0] * range,
        eye[1] + look[1] * range,
        eye[2] + look[2] * range,
    ];
    let clip = level.clip(host, eye, end, ClipBlock::Outline, ClipFluid::None);
    if report {
        let at = clip.map(|hit| {
            let id = level.block(host, hit.block).map(|k| k.id(host));
            format!("{:?}:{:?}:{id:?}", hit.miss, hit.block)
        });
        eprintln!(
            "MISTBRN event=target_search eye={} look={} clip={at:?}",
            fmt3(v3(eye)),
            fmt3(v3(look))
        );
    }
    let block = clip.filter(|hit| !hit.miss && !hit.world_border);
    let block_distance = block.map_or(range, |hit| {
        physics::norm(Vec3 {
            x: hit.location[0] - eye[0],
            y: hit.location[1] - eye[1],
            z: hit.location[2] - eye[2],
        })
    });
    let hits = level
        .ray_trace_entities(host, eye, end, RAY_MARGIN, Some(body))
        .unwrap_or_default();
    if report {
        let kinds: Vec<String> = hits
            .iter()
            .map(|hit| {
                let kind = hit
                    .entity
                    .kind(host)
                    .map(|k| k.id(host))
                    .unwrap_or_default();
                format!("{kind}@{:.2}", hit.distance)
            })
            .collect();
        eprintln!(
            "MISTBRN event=target_search_entities block_distance={block_distance:.3} hits={}",
            kinds.join(";")
        );
    }
    let item = hits
        .into_iter()
        .take_while(|hit| hit.distance <= block_distance)
        .find_map(|hit| {
            let stack = host.item_entity(hit.entity)?.stack(host)?;
            let id = stack.item().id(host);
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

/// Where the force acts on a target, `None` once it is no longer a metal target.
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
        Target::Item { entity, .. } => {
            host.item_entity(*entity)?.stack(host)?;
            let [x, y, z] = entity.position(host)?;
            Some(Vec3 { x, y: y + 0.125, z })
        }
    }
}

/// Applies one command through the core, with the player's own last tick (a snapshot older
/// than it is refused as `StaleTick`). Returns the outcome as a log word.
fn run_core(session: &mut Session, uuid: u128, position: Vec3, command: Command) -> String {
    let mut outcome = Ok(());
    let result = session.change(uuid, true, |saved| {
        update_player(saved, |state, _| {
            let tick = state.last_tick.unwrap_or(Tick(0));
            let ctx = snapshot(uuid, position, tick);
            outcome = apply_command(state, command, &ctx).map(|_| ());
            Ok(())
        })
    });
    let word = match (result, outcome) {
        (Err(error), _) => error,
        (Ok(()), Err(error)) => format!("{error:?}"),
        (Ok(()), Ok(())) => "ok".into(),
    };
    if word != "ok" {
        eprintln!("MISTBRN event=core_refused result={word}");
    }
    word
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

/// `/mistbrn-test …` (registered only while `MISTBRN_TEST_SCENARIO` is set).
pub(crate) const TEST_COMMANDS: [&str; 4] = [
    "mistbrn-test burn <metal:word>",
    "mistbrn-test burn <metal:word> <mode:word>",
    "mistbrn-test off",
    "mistbrn-test credit <metal:word> <milli:integer>",
];

/// Runs one test command for `player`. Burning goes through the core's `apply_command`
/// with the tick's last known state; crediting unlocks and credits without a vial.
pub(crate) fn run_test_command(
    host: &mut dyn ServerHost,
    session: &mut Session,
    player: PlayerRef,
    literals: &[String],
    metal: Option<&str>,
    mode: Option<&str>,
    milli: Option<i32>,
) {
    let uuid = player.uuid();
    let Some(position) = player.entity().position(host).map(v3) else {
        return;
    };
    let words: Vec<&str> = literals.iter().map(String::as_str).collect();
    match (words.get(1).copied(), metal.and_then(parse_metal)) {
        (Some("off"), _) => {
            let result = run_core(session, uuid, position, Command::Extinguish { metal: None });
            eprintln!("MISTBRN event=burn metal=all burn=off result={result}");
        }
        (Some("burn"), Some(metal)) => {
            let flared = mode == Some("flare");
            let command = if flared {
                Command::Flare { metal }
            } else {
                Command::Activate { metal }
            };
            let result = run_core(session, uuid, position, command);
            eprintln!(
                "MISTBRN event=burn metal={} burn={} result={result}",
                metal_name(metal),
                if flared { "flared" } else { "burning" }
            );
        }
        (Some("credit"), Some(metal)) => {
            let amount = i64::from(milli.unwrap_or(0));
            let result = session.change(uuid, true, |saved| {
                let rules = saved.rules().clone();
                update_player(saved, |state, accounting| {
                    state.metals[metal.index()].unlocked = true;
                    credit_reserve(state, accounting, &rules, metal, amount)
                })
            });
            eprintln!(
                "MISTBRN event=test_credit metal={} milli={amount} result={}",
                metal_name(metal),
                result.err().unwrap_or_else(|| "ok".into())
            );
        }
        _ => eprintln!("MISTBRN event=test_command result=unknown"),
    }
    if let Some(player) = host.player(uuid) {
        session.flush(host, player);
    }
}
