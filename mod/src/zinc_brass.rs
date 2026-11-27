//! Zinc and brass's powers (P8), server side.
//!
//! Each level's pass (`level_tick_post`, while the level is in place: rays of a parked level
//! answer nothing) looks along each lit player's view for one compatible creature per metal.
//! Zinc provokes it (its target becomes the player), brass soothes it (it lets the player go
//! and cannot pick them again). Owned techniques add courage for an ally in the view, a retreat
//! for the soothed and, with both metals, selective influence. `living_change_target` holds the
//! choice: a provoked creature keeps the player, a soothed one never takes them.
//!
//! An effect lasts while the metal holds it and `after` ticks once let go (bosses: 40 ticks at
//! most); then that creature ignores the same metal for a while. Nothing here is saved:
//! putting the metals out, dying, changing dimension or reconnecting lets every creature go
//! (`event=power_cleared`). Courage's effects on an ally run out by themselves.
//!
//! Hooks may fire while this module's own pass holds its lock: they use `try_lock` and skip,
//! which is what this module's own target changes want.
use crate::bridge::Session;
use allomancy_core::{
    BurnState, CATALOG, EMOTION_PAIR_TICKS, EVIDENCE_BOTH, EVIDENCE_PROVOKED_NEUTRAL,
    EVIDENCE_SOOTHED_CHASER, EventKind, Influence, Metal, PRACTICE_ALLY, PRACTICE_PROVOKE,
    PRACTICE_SOOTHE, PROVOKE_NEAR, RESEARCH_EMOTION, SELECTIVE_RADIUS, SOOTHE_PRACTICE, ZincBrass,
    is_boss, is_compatible, is_hostile, is_neutral,
};
use mod_api::{
    ActiveEffect, ClipBlock, ClipFluid, EffectKey, EntityRef, Host, HurtSource, LevelRef,
    LivingRef, MobRef, PlayerRef, Registrar, ServerHost, Tag,
};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const RAY_MARGIN: f64 = 0.3;
/// The retreat's path is sent again this often (the creature's own goals may replace it).
const RETREAT_EVERY_TICKS: u64 = 10;
/// The player's last hit counts for an ally this long.
const STRUCK_TICKS: u64 = 100;
/// Allies and struck creatures are looked for this far from the player.
const ALLY_SEARCH: f64 = 24.0;
/// An incompatible creature in the view is logged at most this often.
const INCOMPATIBLE_QUIET: u64 = 20;

#[derive(Clone, Copy)]
struct Effects {
    strength: EffectKey,
    speed: EffectKey,
}

static EFFECTS: OnceLock<Effects> = OnceLock::new();

/// Requests the vanilla effects courage gives (registration only).
pub(crate) fn register(registrar: &mut dyn Registrar) {
    let _ = EFFECTS.set(Effects {
        strength: registrar.effect_key("minecraft:strength"),
        speed: registrar.effect_key("minecraft:speed"),
    });
}

/// A creature under one of this player's metals.
struct Held {
    entity: EntityRef,
    zinc: bool,
    /// Taken by the view (one per metal); false for selective influence's soothed ones.
    gazed: bool,
    /// A neutral creature: calmed when soothed, and when zinc lets it go.
    neutral: bool,
    influence: Influence,
    /// Brass: it had the player as target when soothed (a chaser).
    chaser: bool,
    /// Practice already counted for this hold.
    counted: bool,
    next_path: u64,
    /// Retreat: sent away at least once, and reached its distance.
    retreating: bool,
    reached: bool,
}

#[derive(Default)]
struct Player {
    entity: u32,
    /// The player's body as a living handle (the target zinc holds).
    living: Option<LivingRef>,
    rules: ZincBrass,
    zinc: Option<bool>,
    brass: Option<bool>,
    held: Vec<Held>,
    /// (creature id, zinc) → the tick it stops ignoring that metal.
    immune: BTreeMap<(u32, bool), u64>,
    /// The last creature the player hit, and when.
    struck: Option<(u32, u64)>,
    /// Allies with courage: id → until.
    allies: BTreeMap<u32, u64>,
    /// When each creature was last provoked and soothed (the emotion research's third evidence).
    provoked_at: BTreeMap<u32, u64>,
    soothed_at: BTreeMap<u32, u64>,
    /// Login, respawn, dimension: every creature goes, then the pass starts again.
    fresh: bool,
}

/// What `living_change_target` enforces on a creature: the player entity it must keep (zinc)
/// or never take (brass).
#[derive(Clone, Copy)]
enum Guard {
    Keep(LivingRef),
    Refuse(u32),
}

#[derive(Default)]
pub(crate) struct ZincBrassPowers {
    players: BTreeMap<u128, Player>,
    by_entity: BTreeMap<u32, u128>,
    /// Ally id → the player whose courage it carries.
    allies: BTreeMap<u32, u128>,
    guards: BTreeMap<u32, Guard>,
    tick: u64,
    observations: Vec<(u128, EventKind)>,
}

fn lit(burn: BurnState) -> Option<bool> {
    match burn {
        BurnState::Off => None,
        BurnState::Burning => Some(false),
        BurnState::Flared => Some(true),
    }
}

fn evidence(bit: u8) -> EventKind {
    EventKind::ResearchEvidence {
        research: RESEARCH_EMOTION,
        evidence: bit,
    }
}

fn practice(category: u8) -> EventKind {
    EventKind::PracticeAction { category }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn kind(host: &dyn Host, entity: EntityRef) -> String {
    entity.kind(host).map(|k| k.id(host)).unwrap_or_default()
}

fn alive(host: &dyn Host, entity: EntityRef) -> bool {
    host.living(entity)
        .and_then(|l| l.alive(host))
        .unwrap_or(false)
}

fn metal_name(zinc: bool) -> &'static str {
    if zinc { "zinc" } else { "brass" }
}

/// The player's own ally: a wolf they tamed, or an iron golem a player built (the game keeps
/// only that a player built it, not who).
fn ally_of(host: &mut dyn ServerHost, entity: EntityRef, owner: u128) -> bool {
    let read: &dyn Host = host;
    match kind(read, entity).as_str() {
        "minecraft:wolf" => read.living(entity).and_then(|l| l.owner(read)) == Some(owner),
        "minecraft:iron_golem" => {
            entity
                .nbt(host)
                .and_then(|data| Tag::Compound(data).get("PlayerCreated").cloned())
                == Some(Tag::Byte(1))
        }
        _ => false,
    }
}

/// The first living creature along the view within `range`, before the first block.
fn gaze(host: &dyn Host, level: LevelRef, body: EntityRef, range: f64) -> Option<EntityRef> {
    let eye = body.eye_position(host)?;
    let look = body.view_vector(host)?;
    let end = [
        eye[0] + look[0] * range,
        eye[1] + look[1] * range,
        eye[2] + look[2] * range,
    ];
    let block = level
        .clip(host, eye, end, ClipBlock::Outline, ClipFluid::None)
        .filter(|hit| !hit.miss && !hit.world_border)
        .map_or(range, |hit| distance(eye, hit.location));
    level
        .ray_trace_entities(host, eye, end, RAY_MARGIN, Some(body))?
        .into_iter()
        .take_while(|hit| hit.distance <= block)
        .map(|hit| hit.entity)
        .find(|&entity| alive(host, entity))
}

fn near(host: &dyn Host, level: LevelRef, at: [f64; 3], radius: f64) -> Vec<EntityRef> {
    let low = at.map(|v| v - radius);
    let high = at.map(|v| v + radius);
    level.entities_in(host, low, high).unwrap_or_default()
}

/// `NeutralMob` (26.3) keeps `anger_end_time` in game time: one in the past calms it.
fn calm(host: &mut dyn ServerHost, entity: EntityRef) -> bool {
    let data = [("anger_end_time".into(), Tag::Long(0))]
        .into_iter()
        .collect();
    entity.merge_nbt(host, data)
}

/// Lets a creature go: a provoked one drops the player as target (and a neutral one its
/// anger), a retreating one stops.
fn release(host: &mut dyn ServerHost, held: &Held, player: u32) {
    let read: &dyn Host = host;
    let Some(mob) = read.mob(held.entity) else {
        return;
    };
    if held.zinc {
        if mob.target(read).is_some_and(|t| t.entity().id() == player) {
            let _ = mob.set_target(host, None);
        }
        let _ = mob.set_aggressive(host, false);
        if held.neutral {
            let _ = calm(host, held.entity);
        }
    } else {
        let _ = mob.stop(host);
    }
}

impl ZincBrassPowers {
    /// The server tick, before the levels' passes: players who left are forgotten.
    pub(crate) fn begin(&mut self, host: &dyn ServerHost, tick: u64) {
        self.tick = tick;
        let online = host.players().server_players();
        self.players
            .retain(|uuid, _| online.iter().any(|p| p.uuid() == *uuid));
        let players = &self.players;
        self.by_entity.retain(|_, uuid| players.contains_key(uuid));
        self.allies.retain(|_, uuid| players.contains_key(uuid));
    }

    /// One level's pass, right after that level's own tick.
    pub(crate) fn tick_level(
        &mut self,
        host: &mut dyn ServerHost,
        session: &Session,
        level: LevelRef,
    ) {
        let read: &dyn Host = host;
        if !level.in_place(read) {
            return;
        }
        let tick = self.tick;
        let mut online = host.players().server_players();
        let read: &dyn Host = host;
        online.retain(|player| player.level(read) == Some(level));
        for player in online {
            let uuid = player.uuid();
            let Some(saved) = session.saved(uuid) else {
                continue;
            };
            let state = saved.state();
            let zinc = lit(state.metals[Metal::Zinc.index()].burn);
            let brass = lit(state.metals[Metal::Brass.index()].burn);
            let rules = ZincBrass::new(saved.progress(), &CATALOG);
            let body = player.entity();
            let mut entry = self.players.remove(&uuid).unwrap_or_else(|| Player {
                fresh: true,
                ..Player::default()
            });
            entry.entity = body.id();
            let read: &dyn Host = host;
            entry.living = read.living(body);
            entry.rules = rules;
            self.by_entity.insert(entry.entity, uuid);
            if entry.fresh {
                for held in std::mem::take(&mut entry.held) {
                    release(host, &held, entry.entity);
                }
                entry.fresh = false;
            }
            let (was_zinc, was_brass) = (entry.zinc, entry.brass);
            entry.zinc = zinc;
            entry.brass = brass;
            if (was_zinc.is_some() && zinc.is_none()) || (was_brass.is_some() && brass.is_none()) {
                eprintln!(
                    "MISTBRN event=power_cleared reason=off zinc={} brass={}",
                    zinc.is_some(),
                    brass.is_some()
                );
            }
            // A metal put out lets its creatures go: the effect lasts `after` more ticks.
            for held in &mut entry.held {
                let on = if held.zinc { zinc } else { brass };
                if on.is_none() && held.influence.released.is_none() {
                    held.influence.released = Some(tick);
                }
            }
            self.acquire(host, level, player, &mut entry, uuid);
            self.keep(host, level, player, &mut entry, uuid);
            self.players.insert(uuid, entry);
        }
        self.rebuild_guards();
    }

    /// The creature in the view, per lit metal; with selective influence, the ones around.
    fn acquire(
        &mut self,
        host: &mut dyn ServerHost,
        level: LevelRef,
        player: PlayerRef,
        entry: &mut Player,
        uuid: u128,
    ) {
        let body = player.entity();
        let selective = entry.rules.selective() && entry.zinc.is_some() && entry.brass.is_some();
        for (zinc, on) in [(true, entry.zinc), (false, entry.brass)] {
            let Some(flared) = on else {
                continue;
            };
            let read: &dyn Host = host;
            let Some(target) = gaze(read, level, body, entry.rules.range(zinc, flared)) else {
                continue;
            };
            let target_kind = kind(read, target);
            if ally_of(host, target, uuid) {
                if zinc {
                    self.courage(host, entry, uuid, target);
                }
                continue;
            }
            // With both metals lit, the view belongs to zinc; brass soothes around it.
            let provoked = entry
                .held
                .iter()
                .any(|h| h.zinc && h.entity == target && h.influence.released.is_none());
            if !zinc && (selective || provoked) {
                continue;
            }
            self.start(host, entry, uuid, target, &target_kind, zinc, true);
        }
        if !selective {
            return;
        }
        let read: &dyn Host = host;
        let Some(at) = body.position(read) else {
            return;
        };
        for other in near(read, level, at, SELECTIVE_RADIUS) {
            let read: &dyn Host = host;
            if other.id() == entry.entity
                || self.by_entity.contains_key(&other.id())
                || other
                    .position(read)
                    .is_none_or(|p| distance(p, at) > SELECTIVE_RADIUS)
                || entry
                    .held
                    .iter()
                    .any(|h| h.entity == other && h.influence.released.is_none())
                || !alive(read, other)
            {
                continue;
            }
            let other_kind = kind(read, other);
            if ally_of(host, other, uuid) {
                continue;
            }
            if self.start(host, entry, uuid, other, &other_kind, false, false) {
                eprintln!(
                    "MISTBRN event=influence entity={} kind={other_kind}",
                    other.id()
                );
            }
        }
    }

    /// Puts a creature under a metal. False when it cannot be (incompatible, immune, held).
    #[allow(clippy::too_many_arguments)]
    fn start(
        &mut self,
        host: &mut dyn ServerHost,
        entry: &mut Player,
        uuid: u128,
        target: EntityRef,
        target_kind: &str,
        zinc: bool,
        gazed: bool,
    ) -> bool {
        let tick = self.tick;
        let id = target.id();
        if entry
            .held
            .iter()
            .any(|h| h.zinc == zinc && h.entity == target && h.influence.released.is_none())
        {
            return false;
        }
        if entry
            .immune
            .get(&(id, zinc))
            .is_some_and(|&until| tick < until)
        {
            return false;
        }
        if !is_compatible(target_kind) {
            eprintln!(
                "MISTBRN event=zb_incompatible metal={} kind={target_kind}",
                metal_name(zinc)
            );
            entry.immune.insert((id, zinc), tick + INCOMPATIBLE_QUIET);
            return false;
        }
        // A creature still fading under this metal is taken again from the start.
        entry
            .held
            .retain(|h| !(h.zinc == zinc && h.entity == target));
        // One creature in the view per metal: the previous one keeps its `after`.
        if gazed {
            for held in &mut entry.held {
                if held.zinc == zinc && held.gazed && held.influence.released.is_none() {
                    held.influence.released = Some(tick);
                }
            }
        }
        let read: &dyn Host = host;
        let player_id = entry.entity;
        let chaser = read
            .mob(target)
            .and_then(|mob| mob.target(read))
            .is_some_and(|t| t.entity().id() == player_id);
        let boss = is_boss(target_kind);
        entry.held.push(Held {
            entity: target,
            zinc,
            gazed,
            neutral: is_neutral(target_kind),
            influence: Influence {
                started: tick,
                released: None,
                boss,
            },
            chaser,
            counted: false,
            next_path: tick,
            retreating: false,
            reached: false,
        });
        if zinc {
            eprintln!("MISTBRN event=zinc_provoke entity={id} kind={target_kind} boss={boss}");
            entry.provoked_at.insert(id, tick);
            if is_neutral(target_kind) {
                self.observations
                    .push((uuid, evidence(EVIDENCE_PROVOKED_NEUTRAL)));
            }
        } else {
            eprintln!(
                "MISTBRN event=brass_soothe entity={id} kind={target_kind} boss={boss} chaser={chaser}"
            );
            entry.soothed_at.insert(id, tick);
            if chaser && is_hostile(target_kind) {
                self.observations
                    .push((uuid, evidence(EVIDENCE_SOOTHED_CHASER)));
            }
            if is_neutral(target_kind) {
                let changed = calm(host, target);
                eprintln!("MISTBRN event=brass_calm entity={id} changed={changed}");
            }
        }
        let other = if zinc {
            &entry.soothed_at
        } else {
            &entry.provoked_at
        };
        if other
            .get(&id)
            .is_some_and(|&at| tick.saturating_sub(at) <= EMOTION_PAIR_TICKS)
        {
            self.observations.push((uuid, evidence(EVIDENCE_BOTH)));
        }
        true
    }

    /// Keeps every held creature as its metal wants, and lets go of the ones whose time ended.
    fn keep(
        &mut self,
        host: &mut dyn ServerHost,
        level: LevelRef,
        player: PlayerRef,
        entry: &mut Player,
        uuid: u128,
    ) {
        let tick = self.tick;
        let read: &dyn Host = host;
        let body = player.entity();
        let (Some(at), Some(me)) = (body.position(read), read.living(body)) else {
            return;
        };
        let mut kept = Vec::new();
        for mut held in std::mem::take(&mut entry.held) {
            let read: &dyn Host = host;
            let after = entry.rules.after(held.zinc);
            let gone = !alive(read, held.entity);
            if gone || !held.influence.active(after, tick) {
                if !gone {
                    release(host, &held, entry.entity);
                    if let Some(until) = held.influence.immune_until(after) {
                        entry.immune.insert((held.entity.id(), held.zinc), until);
                    }
                }
                eprintln!(
                    "MISTBRN event=zb_end metal={} entity={} gone={gone}",
                    metal_name(held.zinc),
                    held.entity.id()
                );
                continue;
            }
            let Some(mob) = read.mob(held.entity) else {
                kept.push(held);
                continue;
            };
            let targets_me = mob
                .target(read)
                .is_some_and(|t| t.entity().id() == entry.entity);
            let dist = held
                .entity
                .position(read)
                .map_or(f64::MAX, |p| distance(p, at));
            if held.zinc {
                if !targets_me {
                    let _ = mob.set_target(host, Some(me));
                }
                let _ = mob.set_aggressive(host, true);
                if !held.counted && dist <= PROVOKE_NEAR {
                    held.counted = true;
                    self.observations.push((uuid, practice(PRACTICE_PROVOKE)));
                    eprintln!(
                        "MISTBRN event=provoke_near entity={} distance={dist:.1}",
                        held.entity.id()
                    );
                }
            } else {
                if targets_me {
                    let _ = mob.set_target(host, None);
                }
                if let Some((blocks, speed)) = entry.rules.retreat() {
                    if dist < blocks && tick >= held.next_path {
                        held.next_path = tick + RETREAT_EVERY_TICKS;
                        held.retreating = true;
                        retreat(host, mob, at, blocks, speed);
                    } else if held.retreating && !held.reached && dist >= blocks {
                        held.reached = true;
                        eprintln!(
                            "MISTBRN event=retreat_reached entity={} distance={dist:.1}",
                            held.entity.id()
                        );
                    }
                }
                if !held.counted
                    && held.chaser
                    && tick.saturating_sub(held.influence.started) >= SOOTHE_PRACTICE
                {
                    held.counted = true;
                    self.observations.push((uuid, practice(PRACTICE_SOOTHE)));
                    eprintln!("MISTBRN event=soothe_held entity={}", held.entity.id());
                }
            }
            kept.push(held);
        }
        entry.held = kept;
        entry.immune.retain(|_, until| *until > tick);
        entry
            .provoked_at
            .retain(|_, at| tick.saturating_sub(*at) <= EMOTION_PAIR_TICKS);
        entry
            .soothed_at
            .retain(|_, at| tick.saturating_sub(*at) <= EMOTION_PAIR_TICKS);
        self.lead_allies(host, level, body, entry, uuid);
    }

    /// Courage on an ally in zinc's view: strength and speed, once until they run out.
    fn courage(
        &mut self,
        host: &mut dyn ServerHost,
        entry: &mut Player,
        uuid: u128,
        ally: EntityRef,
    ) {
        let tick = self.tick;
        let (Some((level, ticks)), Some(&effects)) = (entry.rules.courage(), EFFECTS.get()) else {
            return;
        };
        if entry
            .allies
            .get(&ally.id())
            .is_some_and(|&until| tick < until)
        {
            return;
        }
        let read: &dyn Host = host;
        let Some(living) = read.living(ally) else {
            return;
        };
        let duration = i32::try_from(ticks).unwrap_or(i32::MAX);
        let amplifier = i32::from(level);
        let strength = living.add_effect(
            host,
            &ActiveEffect::new(effects.strength, duration, amplifier, true, false, false),
        );
        let speed = living.add_effect(
            host,
            &ActiveEffect::new(effects.speed, duration, amplifier, true, false, false),
        );
        entry.allies.insert(ally.id(), tick + ticks);
        self.allies.insert(ally.id(), uuid);
        eprintln!(
            "MISTBRN event=courage entity={} level={} ticks={ticks} strength={strength} speed={speed}",
            ally.id(),
            level + 1
        );
    }

    /// Allies with courage attack the creature the player last hit.
    fn lead_allies(
        &mut self,
        host: &mut dyn ServerHost,
        level: LevelRef,
        body: EntityRef,
        entry: &mut Player,
        uuid: u128,
    ) {
        let tick = self.tick;
        entry.allies.retain(|_, until| *until > tick);
        let mine = &entry.allies;
        self.allies
            .retain(|id, owner| *owner != uuid || mine.contains_key(id));
        let Some((struck, at)) = entry.struck else {
            return;
        };
        if tick.saturating_sub(at) > STRUCK_TICKS || entry.allies.is_empty() {
            return;
        }
        let read: &dyn Host = host;
        let Some(center) = body.position(read) else {
            return;
        };
        let around = near(read, level, center, ALLY_SEARCH);
        let Some(victim) = around
            .iter()
            .copied()
            .find(|e| e.id() == struck)
            .and_then(|e| read.living(e))
        else {
            return;
        };
        for ally in around
            .into_iter()
            .filter(|e| entry.allies.contains_key(&e.id()) && e.id() != struck)
        {
            let read: &dyn Host = host;
            let Some(mob) = read.mob(ally) else {
                continue;
            };
            if mob.target(read).is_none_or(|t| t.entity().id() != struck) {
                let led = mob.set_target(host, Some(victim));
                eprintln!(
                    "MISTBRN event=ally_target entity={} victim={struck} ok={led}",
                    ally.id()
                );
            }
        }
    }

    fn rebuild_guards(&mut self) {
        self.guards.clear();
        let tick = self.tick;
        for entry in self.players.values() {
            for held in &entry.held {
                if !held.influence.active(entry.rules.after(held.zinc), tick) {
                    continue;
                }
                let id = held.entity.id();
                // Provoked by one player and soothed by another: the provocation holds.
                match (held.zinc, entry.living) {
                    (true, Some(me)) => {
                        self.guards.insert(id, Guard::Keep(me));
                    }
                    (true, None) => {}
                    (false, _) => {
                        self.guards.entry(id).or_insert(Guard::Refuse(entry.entity));
                    }
                }
            }
        }
    }

    /// `living_change_target`: a provoked creature keeps its player, a soothed one never takes
    /// its player. Changes that agree pass.
    pub(crate) fn change_target(&self, mob: MobRef, new_target: &mut Option<LivingRef>) {
        let Some(&guard) = self.guards.get(&mob.entity().id()) else {
            return;
        };
        let new = new_target.map(|t| t.entity().id());
        match guard {
            Guard::Keep(player) if new != Some(player.entity().id()) => {
                *new_target = Some(player);
                eprintln!("MISTBRN event=zinc_hold entity={}", mob.entity().id());
            }
            Guard::Refuse(player) if new == Some(player) => {
                *new_target = None;
                eprintln!("MISTBRN event=brass_block entity={}", mob.entity().id());
            }
            _ => {}
        }
    }

    /// A hit about to land: the player's last hit (for allies) and an ally's hit (practice).
    pub(crate) fn incoming_damage(&mut self, entity: i32, source: &HurtSource<'_>) {
        let (Ok(victim), Ok(attacker)) = (u32::try_from(entity), u32::try_from(source.causing))
        else {
            return;
        };
        let tick = self.tick;
        if let Some(entry) = self
            .by_entity
            .get(&attacker)
            .and_then(|uuid| self.players.get_mut(uuid))
        {
            entry.struck = Some((victim, tick));
        }
        if let Some(&uuid) = self.allies.get(&attacker)
            && self.players.get(&uuid).is_some_and(|entry| {
                entry
                    .allies
                    .get(&attacker)
                    .is_some_and(|&until| tick < until)
                    && entry.entity != victim
            })
        {
            self.observations.push((uuid, practice(PRACTICE_ALLY)));
            eprintln!("MISTBRN event=ally_hit entity={attacker} victim={victim}");
        }
    }

    pub(crate) fn take_observations(&mut self) -> Vec<(u128, EventKind)> {
        std::mem::take(&mut self.observations)
    }

    /// Respawn, dimension change, login: every creature goes on the player's next pass, and
    /// no guard holds it meanwhile.
    pub(crate) fn clear(&mut self, uuid: u128, reason: &str) {
        let Some(entry) = self.players.get_mut(&uuid) else {
            return;
        };
        if entry.zinc.is_some() || entry.brass.is_some() || !entry.held.is_empty() {
            eprintln!("MISTBRN event=power_cleared reason={reason} power=zinc_brass");
        }
        entry.fresh = true;
        entry.zinc = None;
        entry.brass = None;
        let player = entry.entity;
        self.guards.retain(|_, guard| match *guard {
            Guard::Keep(me) => me.entity().id() != player,
            Guard::Refuse(p) => p != player,
        });
    }

    /// A logout lets every creature go now, while they are still reachable.
    pub(crate) fn logged_out(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        self.clear(uuid, "logout");
        if let Some(entry) = self.players.remove(&uuid) {
            for held in &entry.held {
                release(host, held, entry.entity);
            }
            self.by_entity.remove(&entry.entity);
        }
        self.allies.retain(|_, owner| *owner != uuid);
    }
}

/// Sends a soothed creature `blocks` away from the player, straight out from them.
fn retreat(host: &mut dyn ServerHost, mob: MobRef, from: [f64; 3], blocks: f64, speed: f64) {
    let read: &dyn Host = host;
    let Some(at) = mob.entity().position(read) else {
        return;
    };
    let (dx, dz) = (at[0] - from[0], at[2] - from[2]);
    let length = (dx * dx + dz * dz).sqrt().max(0.01);
    let to = [
        from[0] + dx / length * (blocks + 1.0),
        at[1],
        from[2] + dz / length * (blocks + 1.0),
    ];
    let started = mob.move_to(host, to, speed);
    eprintln!(
        "MISTBRN event=retreat entity={} to={:.1},{:.1},{:.1} speed={speed:.2} ok={started}",
        mob.entity().id(),
        to[0],
        to[1],
        to[2]
    );
}
