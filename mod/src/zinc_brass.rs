//! Zinc and brass's powers (P8), server side.
//!
//! Each level's pass (`level_tick_post`, while the level is in place: rays and entity reads of
//! a parked level answer nothing) looks along each lit player's view for one compatible
//! creature per metal. Zinc provokes it (its target becomes the player), brass soothes it (it
//! lets the player go and cannot pick them again). Owned techniques add courage for an ally in
//! the view, a retreat for the soothed and, with both metals, selective influence.
//!
//! Influence is kept per creature (`Hold`), not per player: a creature's own level pass ends,
//! keeps and lets go of it, with the player entity id it was taken for, wherever the player is
//! by then. Immunity is per creature and metal, shared by every player. `living_change_target`
//! enforces the same resolution as the pass: the oldest provocation wins; a soothed creature
//! never takes a player who soothes it.
//!
//! An effect lasts while the metal holds it and `after` ticks once let go (bosses: 40 ticks at
//! most); then that creature ignores the same metal for a while. Nothing here is saved:
//! putting the metals out lets go; dying, changing dimension or leaving ends every hold and
//! courage the player gave at once (`event=power_cleared`).
//!
//! Hooks may fire while this module's own pass holds its lock: they use `try_lock` and skip,
//! which is what this module's own target changes want.
use crate::bridge::Session;
use allomancy_core::{
    BurnState, CATALOG, EMOTION_PAIR_TICKS, EVIDENCE_BOTH, EVIDENCE_PROVOKED_NEUTRAL,
    EVIDENCE_SOOTHED_CHASER, EventKind, Influence, Metal, PRACTICE_ALLY, PRACTICE_PROVOKE,
    PRACTICE_SOOTHE, PROVOKE_NEAR, RESEARCH_EMOTION, SELECTIVE_RADIUS, SOOTHE_PRACTICE, ZB_AFTER,
    ZincBrass, is_boss, is_compatible, is_hostile, is_neutral,
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
/// An ally looks for the creature its player hit this far from itself.
const ALLY_SEARCH: f64 = 24.0;
/// A refused creature in the view (incompatible, immune) is logged at most this often.
const QUIET_TICKS: u64 = 20;

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

/// A creature under one player's metal.
struct Hold {
    entity: EntityRef,
    level: LevelRef,
    zinc: bool,
    player: u128,
    /// The player's body when the hold began (the target zinc holds, the one brass refuses).
    player_entity: u32,
    player_living: Option<LivingRef>,
    /// Taken by the view (one per metal and player), not by selective influence's area.
    gazed: bool,
    neutral: bool,
    hostile: bool,
    influence: Influence,
    /// Brass: it was chasing the player on its own when soothed (not by this player's zinc).
    chaser: bool,
    /// Zinc: it started farther than `PROVOKE_NEAR`.
    far: bool,
    /// The effect was seen working (zinc: it targets the player; brass: it does not).
    verified: bool,
    /// Practice already counted for this hold.
    counted: bool,
    next_path: u64,
    retreating: bool,
    reached: bool,
    /// Death, dimension, login, logout: ended on its level's next pass, with no immunity.
    ending: Option<&'static str>,
}

impl Hold {
    fn live(&self, after: u64, tick: u64) -> bool {
        self.ending.is_none() && self.influence.active(after, tick)
    }
}

/// Courage given to an ally: what to take off if the player leaves before it runs out.
struct Ally {
    entity: EntityRef,
    level: LevelRef,
    player: u128,
    given: u64,
    ticks: i32,
    amplifier: i32,
    /// The creature it was last sent at.
    led: Option<u32>,
    ending: bool,
}

impl Ally {
    fn until(&self) -> u64 {
        self.given + u64::try_from(self.ticks).unwrap_or(0)
    }
}

#[derive(Default)]
struct Player {
    entity: u32,
    living: Option<LivingRef>,
    rules: ZincBrass,
    zinc: Option<bool>,
    brass: Option<bool>,
    /// The last creature (never a player) the player hurt, and when.
    struck: Option<(u32, u64)>,
    /// When each creature was first seen provoked and soothed (the third evidence).
    provoked_at: BTreeMap<u32, u64>,
    soothed_at: BTreeMap<u32, u64>,
}

#[derive(Default)]
pub(crate) struct ZincBrassPowers {
    players: BTreeMap<u128, Player>,
    by_entity: BTreeMap<u32, u128>,
    holds: Vec<Hold>,
    allies: Vec<Ally>,
    /// (creature id, zinc) → the tick it stops ignoring that metal, for every player.
    immune: BTreeMap<(u32, bool), u64>,
    /// (creature id, zinc) → until when a refusal is not logged again.
    quiet: BTreeMap<(u32, bool), u64>,
    /// (boss id, zinc) → the first hold's start: every player's hold of that metal on the boss
    /// shares it, so all are cut at the same 40th tick.
    boss_cut: BTreeMap<(u32, bool), u64>,
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

fn target_id(host: &dyn Host, mob: MobRef) -> Option<u32> {
    mob.target(host).map(|t| t.entity().id())
}

fn player_position(host: &dyn Host, uuid: u128) -> Option<[f64; 3]> {
    host.player(uuid).and_then(|p| p.entity().position(host))
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

/// Lets a creature go: a provoked one drops the player it was provoked against (and a neutral
/// one its anger), a retreating one stops.
/// True when, afterwards, the creature no longer targets the player because of zinc.
fn release(host: &mut dyn ServerHost, hold: &Hold) -> bool {
    let read: &dyn Host = host;
    let Some(mob) = read.mob(hold.entity) else {
        return false;
    };
    if hold.zinc {
        if target_id(read, mob) == Some(hold.player_entity) {
            let _ = mob.set_target(host, None);
        }
        let _ = mob.set_aggressive(host, false);
        if hold.neutral {
            let _ = calm(host, hold.entity);
        }
        let read: &dyn Host = host;
        target_id(read, mob) != Some(hold.player_entity)
    } else {
        if hold.retreating {
            let _ = mob.stop(host);
        }
        true
    }
}

/// What the live holds on one creature want, as the pass and the hook both resolve it: the
/// oldest provocation's player keeps it; every player soothing it is refused.
struct Wanted {
    keep: Option<(u32, Option<LivingRef>)>,
    refuse: Vec<u32>,
}

fn after_of(players: &BTreeMap<u128, Player>, hold: &Hold) -> u64 {
    players
        .get(&hold.player)
        .map_or(ZB_AFTER, |p| p.rules.after(hold.zinc))
}

fn wanted(holds: &[Hold], players: &BTreeMap<u128, Player>, id: u32, tick: u64) -> Wanted {
    let live: Vec<&Hold> = holds
        .iter()
        .filter(|h| h.entity.id() == id && h.live(after_of(players, h), tick))
        .collect();
    let keep = live
        .iter()
        .filter(|h| h.zinc)
        .min_by_key(|h| (h.influence.started, h.player))
        .map(|h| (h.player_entity, h.player_living));
    let refuse = live
        .iter()
        .filter(|h| !h.zinc)
        .map(|h| h.player_entity)
        .collect();
    Wanted { keep, refuse }
}

impl ZincBrassPowers {
    /// The server tick, before the levels' passes: players who left end their influence.
    pub(crate) fn begin(&mut self, host: &dyn ServerHost, tick: u64) {
        self.tick = tick;
        let online = host.players().server_players();
        let gone: Vec<u128> = self
            .players
            .keys()
            .copied()
            .filter(|uuid| !online.iter().any(|p| p.uuid() == *uuid))
            .collect();
        for uuid in gone {
            self.logged_out(uuid);
        }
        self.immune.retain(|_, until| *until > tick);
        self.quiet.retain(|_, until| *until > tick);
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
        let mut online = host.players().server_players();
        let read: &dyn Host = host;
        online.retain(|player| player.level(read) == Some(level));
        for &player in &online {
            self.update_player(host, session, player);
        }
        self.end_holds(host, level);
        for &player in &online {
            self.acquire(host, level, player);
        }
        self.keep_holds(host, level);
        self.keep_allies(host, level);
    }

    /// What the player has lit and owns; a metal put out (or selective influence's area left)
    /// lets its creatures go.
    fn update_player(&mut self, host: &mut dyn ServerHost, session: &Session, player: PlayerRef) {
        let tick = self.tick;
        let uuid = player.uuid();
        let Some(saved) = session.saved(uuid) else {
            return;
        };
        let state = saved.state();
        let zinc = lit(state.metals[Metal::Zinc.index()].burn);
        let brass = lit(state.metals[Metal::Brass.index()].burn);
        let body = player.entity();
        let read: &dyn Host = host;
        let entry = self.players.entry(uuid).or_default();
        entry.entity = body.id();
        entry.living = read.living(body);
        entry.rules = ZincBrass::new(saved.progress(), &CATALOG);
        self.by_entity.insert(body.id(), uuid);
        if (entry.zinc.is_some() && zinc.is_none()) || (entry.brass.is_some() && brass.is_none()) {
            eprintln!(
                "MISTBRN event=power_cleared reason=off zinc={} brass={}",
                zinc.is_some(),
                brass.is_some()
            );
        }
        entry.zinc = zinc;
        entry.brass = brass;
        let selective = entry.rules.selective() && zinc.is_some() && brass.is_some();
        let at = body.position(read);
        let here = player.level(read);
        for hold in self.holds.iter_mut().filter(|h| h.player == uuid) {
            let on = if hold.zinc { zinc } else { brass };
            // Selective influence's area (and, while it is on, every soothed creature): only
            // within reach.
            let away = || {
                here != Some(hold.level)
                    || hold
                        .entity
                        .position(read)
                        .zip(at)
                        .is_none_or(|(p, a)| distance(p, a) > SELECTIVE_RADIUS)
            };
            let outside = if selective && !hold.zinc {
                away()
            } else {
                !hold.gazed
            };
            if on.is_none() || outside {
                hold.influence.released.get_or_insert(tick);
            }
        }
    }

    /// Ends this level's holds whose time is over (or that a clear ended), before the views.
    fn end_holds(&mut self, host: &mut dyn ServerHost, level: LevelRef) {
        let tick = self.tick;
        let mut kept = Vec::new();
        for hold in std::mem::take(&mut self.holds) {
            if hold.level != level {
                kept.push(hold);
                continue;
            }
            let read: &dyn Host = host;
            let after = after_of(&self.players, &hold);
            let gone = !alive(read, hold.entity);
            let reason = if gone {
                "gone"
            } else if let Some(reason) = hold.ending {
                reason
            } else if !hold.influence.active(after, tick) {
                "time"
            } else {
                kept.push(hold);
                continue;
            };
            let cleared = !gone && release(host, &hold);
            // An effect that ran its course, or gave way to the other metal, leaves the
            // creature immune to that metal (a boss longer) for every player.
            if matches!(reason, "time" | "switched") {
                let key = (hold.entity.id(), hold.zinc);
                let immunity = if hold.influence.boss {
                    allomancy_core::BOSS_IMMUNITY
                } else {
                    allomancy_core::ZB_IMMUNITY
                };
                let longest = self
                    .immune
                    .get(&key)
                    .copied()
                    .unwrap_or(0)
                    .max(tick + immunity);
                self.immune.insert(key, longest);
            }
            eprintln!(
                "MISTBRN event=zb_end metal={} entity={} reason={reason} cleared={cleared} tick={tick}",
                metal_name(hold.zinc),
                hold.entity.id()
            );
        }
        self.holds = kept;
        // A boss's shared start lasts while some hold of that metal is on it.
        let holds = &self.holds;
        self.boss_cut
            .retain(|&(id, zinc), _| holds.iter().any(|h| h.entity.id() == id && h.zinc == zinc));
    }

    /// The creature in the view, per lit metal; with selective influence, the ones around.
    fn acquire(&mut self, host: &mut dyn ServerHost, level: LevelRef, player: PlayerRef) {
        let uuid = player.uuid();
        let Some(entry) = self.players.get(&uuid) else {
            return;
        };
        let (rules, zinc_on, brass_on) = (entry.rules, entry.zinc, entry.brass);
        let body = player.entity();
        let selective = rules.selective() && zinc_on.is_some() && brass_on.is_some();
        for (zinc, on) in [(true, zinc_on), (false, brass_on)] {
            let Some(flared) = on else {
                continue;
            };
            let read: &dyn Host = host;
            let Some(target) = gaze(read, level, body, rules.range(zinc, flared)) else {
                continue;
            };
            if ally_of(host, target, uuid) {
                if zinc {
                    self.courage(host, level, uuid, target);
                }
                continue;
            }
            // With both metals lit, the view belongs to zinc; brass soothes around it.
            let provoked = self.holds.iter().any(|h| {
                h.player == uuid && h.zinc && h.entity == target && h.influence.released.is_none()
            });
            if !zinc && (selective || provoked) {
                continue;
            }
            self.start(host, level, uuid, target, zinc, true);
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
            let taken = self
                .holds
                .iter()
                .any(|h| h.player == uuid && h.entity == other && h.influence.released.is_none());
            if taken
                || self.by_entity.contains_key(&other.id())
                || other
                    .position(read)
                    .is_none_or(|p| distance(p, at) > SELECTIVE_RADIUS)
                || !alive(read, other)
                || ally_of(host, other, uuid)
            {
                continue;
            }
            if self.start(host, level, uuid, other, false, false) {
                let read: &dyn Host = host;
                eprintln!(
                    "MISTBRN event=influence entity={} kind={}",
                    other.id(),
                    kind(read, other)
                );
            }
        }
    }

    /// Puts a creature under a player's metal. False when it is not a new hold (incompatible,
    /// immune, or this player's own hold taken back).
    fn start(
        &mut self,
        host: &mut dyn ServerHost,
        level: LevelRef,
        uuid: u128,
        target: EntityRef,
        zinc: bool,
        gazed: bool,
    ) -> bool {
        let tick = self.tick;
        let id = target.id();
        // The view moved on: this player's previous creature of that metal keeps its `after`.
        if gazed {
            for hold in self
                .holds
                .iter_mut()
                .filter(|h| h.player == uuid && h.zinc == zinc && h.gazed && h.entity != target)
            {
                hold.influence.released.get_or_insert(tick);
            }
        }
        // This player's own hold, still running: taken back as it is (its start, its boss cut
        // and its practice stay).
        if let Some(hold) = self.holds.iter_mut().find(|h| {
            h.player == uuid && h.zinc == zinc && h.entity == target && h.ending.is_none()
        }) {
            hold.influence.released = None;
            hold.gazed |= gazed;
            return false;
        }
        let read: &dyn Host = host;
        let target_kind = kind(read, target);
        // Another player's wolf is nobody's to rouse or calm (its owner gives it courage).
        let owned = target_kind == "minecraft:wolf"
            && read.living(target).and_then(|l| l.owner(read)).is_some();
        let refused = if !is_compatible(&target_kind) || owned {
            Some("zb_incompatible")
        } else if self
            .immune
            .get(&(id, zinc))
            .is_some_and(|&until| tick < until)
        {
            Some("zb_immune")
        } else {
            None
        };
        if let Some(event) = refused {
            if self
                .quiet
                .get(&(id, zinc))
                .is_none_or(|&until| tick >= until)
            {
                self.quiet.insert((id, zinc), tick + QUIET_TICKS);
                eprintln!(
                    "MISTBRN event={event} metal={} entity={id} kind={target_kind}",
                    metal_name(zinc)
                );
            }
            return false;
        }
        let Some(entry) = self.players.get(&uuid) else {
            return false;
        };
        let player_entity = entry.entity;
        let dist = target
            .position(read)
            .zip(player_position(read, uuid))
            .map_or(f64::MAX, |(p, a)| distance(p, a));
        // A chase this player's own zinc imposed is not one brass can take credit for.
        let imposed = self
            .holds
            .iter()
            .any(|h| h.player == uuid && h.zinc && h.entity == target && h.ending.is_none());
        let chaser = !zinc
            && !imposed
            && read
                .mob(target)
                .is_some_and(|mob| target_id(read, mob) == Some(player_entity));
        // The player's own hold of the other metal on it gives way to this one.
        for hold in self
            .holds
            .iter_mut()
            .filter(|h| h.player == uuid && h.zinc != zinc && h.entity == target)
        {
            hold.ending.get_or_insert("switched");
        }
        let boss = is_boss(&target_kind);
        let started = if boss {
            *self.boss_cut.entry((id, zinc)).or_insert(tick)
        } else {
            tick
        };
        self.holds.push(Hold {
            entity: target,
            level,
            zinc,
            player: uuid,
            player_entity,
            player_living: entry.living,
            gazed,
            neutral: is_neutral(&target_kind),
            hostile: is_hostile(&target_kind),
            influence: Influence {
                started,
                released: None,
                boss,
            },
            chaser,
            far: dist > PROVOKE_NEAR,
            verified: false,
            counted: false,
            next_path: tick,
            retreating: false,
            reached: false,
            ending: None,
        });
        if zinc {
            eprintln!(
                "MISTBRN event=zinc_provoke entity={id} kind={target_kind} boss={boss} tick={tick}"
            );
        } else {
            eprintln!(
                "MISTBRN event=brass_soothe entity={id} kind={target_kind} boss={boss} chaser={chaser}"
            );
            if is_neutral(&target_kind) {
                let changed = calm(host, target);
                eprintln!("MISTBRN event=brass_calm entity={id} changed={changed}");
            }
        }
        true
    }

    /// Keeps this level's live holds as their metal wants, and counts what they achieved.
    fn keep_holds(&mut self, host: &mut dyn ServerHost, level: LevelRef) {
        let tick = self.tick;
        for index in 0..self.holds.len() {
            let hold = &self.holds[index];
            let Some(entry) = self.players.get(&hold.player) else {
                continue;
            };
            if hold.level != level || !hold.live(entry.rules.after(hold.zinc), tick) {
                continue;
            }
            let rules = entry.rules;
            let id = hold.entity.id();
            let read: &dyn Host = host;
            let Some(mob) = read.mob(hold.entity) else {
                continue;
            };
            let want = wanted(&self.holds, &self.players, id, tick);
            match (want.keep, target_id(read, mob)) {
                (Some((keeper, Some(living))), current) if current != Some(keeper) => {
                    let _ = mob.set_target(host, Some(living));
                }
                (None, Some(current)) if want.refuse.contains(&current) => {
                    let _ = mob.set_target(host, None);
                }
                _ => {}
            }
            let hold = &self.holds[index];
            if hold.zinc
                && want
                    .keep
                    .is_some_and(|(keeper, _)| keeper == hold.player_entity)
            {
                let _ = mob.set_aggressive(host, true);
            }
            let read: &dyn Host = host;
            let current = target_id(read, mob);
            let at = player_position(read, hold.player);
            let dist = hold
                .entity
                .position(read)
                .zip(at)
                .map_or(f64::MAX, |(p, a)| distance(p, a));
            let hold = &mut self.holds[index];
            let mine = current == Some(hold.player_entity);
            let mut seen = Vec::new();
            let mut first_seen = false;
            if hold.zinc {
                if mine && !hold.verified {
                    hold.verified = true;
                    first_seen = true;
                    eprintln!("MISTBRN event=zinc_verified entity={id} tick={tick}");
                    if hold.neutral {
                        seen.push(evidence(EVIDENCE_PROVOKED_NEUTRAL));
                    }
                }
                if mine && hold.far && !hold.counted && dist <= PROVOKE_NEAR {
                    hold.counted = true;
                    seen.push(practice(PRACTICE_PROVOKE));
                    eprintln!("MISTBRN event=provoke_near entity={id} distance={dist:.1}");
                }
            } else {
                if !mine && !hold.verified {
                    hold.verified = true;
                    first_seen = true;
                    eprintln!("MISTBRN event=brass_verified entity={id} tick={tick}");
                    if hold.chaser && hold.hostile {
                        seen.push(evidence(EVIDENCE_SOOTHED_CHASER));
                    }
                }
                if let Some((blocks, speed)) = rules.retreat() {
                    if dist < blocks && tick >= hold.next_path {
                        hold.next_path = tick + RETREAT_EVERY_TICKS;
                        hold.retreating = true;
                        retreat(host, mob, at, blocks, speed);
                    } else if hold.retreating && !hold.reached && dist >= blocks {
                        hold.reached = true;
                        eprintln!("MISTBRN event=retreat_reached entity={id} distance={dist:.1}");
                    }
                }
                if hold.verified
                    && hold.chaser
                    && hold.hostile
                    && !hold.counted
                    && tick.saturating_sub(hold.influence.started) >= SOOTHE_PRACTICE
                {
                    hold.counted = true;
                    seen.push(practice(PRACTICE_SOOTHE));
                    eprintln!("MISTBRN event=soothe_held entity={id}");
                }
            }
            let (zinc, uuid) = (hold.zinc, hold.player);
            if let Some(entry) = self.players.get_mut(&uuid) {
                entry
                    .provoked_at
                    .retain(|_, at| tick.saturating_sub(*at) <= EMOTION_PAIR_TICKS);
                entry
                    .soothed_at
                    .retain(|_, at| tick.saturating_sub(*at) <= EMOTION_PAIR_TICKS);
                // Provoked and soothed, both seen working, close together: the third evidence.
                if first_seen {
                    let (mine_at, other_at) = if zinc {
                        (&mut entry.provoked_at, &entry.soothed_at)
                    } else {
                        (&mut entry.soothed_at, &entry.provoked_at)
                    };
                    if other_at.contains_key(&id) {
                        seen.push(evidence(EVIDENCE_BOTH));
                    }
                    mine_at.insert(id, tick);
                }
            }
            self.observations
                .extend(seen.into_iter().map(|kind| (uuid, kind)));
        }
    }

    /// Courage on an ally in zinc's view: strength and speed, once until they run out.
    fn courage(&mut self, host: &mut dyn ServerHost, level: LevelRef, uuid: u128, ally: EntityRef) {
        let tick = self.tick;
        let Some(entry) = self.players.get(&uuid) else {
            return;
        };
        let (Some((amplifier, ticks)), Some(&effects)) = (entry.rules.courage(), EFFECTS.get())
        else {
            return;
        };
        // One leader per ally: whoever gave the courage that is running.
        if self
            .allies
            .iter()
            .any(|a| a.entity == ally && !a.ending && tick < a.until())
        {
            return;
        }
        let read: &dyn Host = host;
        let Some(living) = read.living(ally) else {
            return;
        };
        let duration = i32::try_from(ticks).unwrap_or(i32::MAX);
        let amplifier = i32::from(amplifier);
        let strength = living.add_effect(
            host,
            &ActiveEffect::new(effects.strength, duration, amplifier, true, false, false),
        );
        let speed = living.add_effect(
            host,
            &ActiveEffect::new(effects.speed, duration, amplifier, true, false, false),
        );
        if !strength && !speed {
            return;
        }
        self.allies.retain(|a| a.entity != ally);
        self.allies.push(Ally {
            entity: ally,
            level,
            player: uuid,
            given: tick,
            ticks: duration,
            amplifier,
            led: None,
            ending: false,
        });
        eprintln!(
            "MISTBRN event=courage entity={} level={} ticks={ticks} strength={strength} speed={speed}",
            ally.id(),
            amplifier + 1
        );
    }

    /// This level's allies: an ended one loses the courage this module gave it (and the
    /// creature it was sent at); the others attack what their player last hurt.
    fn keep_allies(&mut self, host: &mut dyn ServerHost, level: LevelRef) {
        let tick = self.tick;
        let Some(&effects) = EFFECTS.get() else {
            return;
        };
        let mut kept = Vec::new();
        for mut ally in std::mem::take(&mut self.allies) {
            if ally.level != level {
                kept.push(ally);
                continue;
            }
            let read: &dyn Host = host;
            if !alive(read, ally.entity) || tick >= ally.until() {
                continue;
            }
            let Some(mob) = read.mob(ally.entity) else {
                continue;
            };
            if ally.ending {
                take_courage(host, &ally, effects, tick);
                let read: &dyn Host = host;
                if ally.led.is_some() && target_id(read, mob) == ally.led {
                    let _ = mob.set_target(host, None);
                }
                eprintln!("MISTBRN event=courage_cleared entity={}", ally.entity.id());
                continue;
            }
            let struck = self
                .players
                .get(&ally.player)
                .and_then(|p| p.struck)
                .filter(|&(victim, at)| {
                    tick.saturating_sub(at) <= STRUCK_TICKS && victim != ally.entity.id()
                })
                .map(|(victim, _)| victim);
            // One try per victim: a refused target is not asked for again every tick.
            let victim = struck
                .filter(|&v| target_id(read, mob) != Some(v) && ally.led != Some(v))
                .and_then(|v| {
                    let center = ally.entity.position(read)?;
                    near(read, level, center, ALLY_SEARCH)
                        .into_iter()
                        .find(|e| e.id() == v && alive(read, *e))
                        .filter(|e| kind(read, *e) != "minecraft:player")
                        .and_then(|e| read.living(e))
                });
            if let Some(living) = victim {
                let led = mob.set_target(host, Some(living));
                ally.led = Some(living.entity().id());
                eprintln!(
                    "MISTBRN event=ally_target entity={} victim={} ok={led}",
                    ally.entity.id(),
                    living.entity().id()
                );
            }
            kept.push(ally);
        }
        self.allies.extend(kept);
    }

    /// `living_change_target`: the same resolution as the pass. The oldest provocation's player
    /// is kept; a soothing player is never taken.
    pub(crate) fn change_target(&self, mob: MobRef, new_target: &mut Option<LivingRef>) {
        let id = mob.entity().id();
        if !self.holds.iter().any(|h| h.entity.id() == id) {
            return;
        }
        let want = wanted(&self.holds, &self.players, id, self.tick);
        let new = new_target.map(|t| t.entity().id());
        match want.keep {
            Some((keeper, Some(living))) if new != Some(keeper) => {
                *new_target = Some(living);
                eprintln!("MISTBRN event=zinc_hold entity={id}");
            }
            None if new.is_some_and(|n| want.refuse.contains(&n)) => {
                *new_target = None;
                eprintln!("MISTBRN event=brass_block entity={id}");
            }
            _ => {}
        }
    }

    /// A hit that took health: the player's last hit (never on a player, for allies) and an
    /// ally with courage hitting a creature (practice).
    pub(crate) fn damaged(
        &mut self,
        entity: i32,
        kind: &str,
        source: &HurtSource<'_>,
        health: f32,
    ) {
        let (Ok(victim), Ok(attacker)) = (u32::try_from(entity), u32::try_from(source.causing))
        else {
            return;
        };
        if health <= 0.0 || kind == "minecraft:player" {
            return;
        }
        let tick = self.tick;
        if let Some(entry) = self
            .by_entity
            .get(&attacker)
            .and_then(|uuid| self.players.get_mut(uuid))
            .filter(|entry| entry.entity == attacker)
        {
            entry.struck = Some((victim, tick));
        }
        let courage = self
            .allies
            .iter()
            .find(|a| a.entity.id() == attacker && !a.ending && tick < a.until());
        if let Some(ally) = courage {
            self.observations
                .push((ally.player, practice(PRACTICE_ALLY)));
            eprintln!("MISTBRN event=ally_hit entity={attacker} victim={victim}");
        }
    }

    pub(crate) fn take_observations(&mut self) -> Vec<(u128, EventKind)> {
        std::mem::take(&mut self.observations)
    }

    /// Death, dimension change, login, logout: every hold and courage the player gave ends now
    /// for the hook and on each creature's next level pass, with the body id it was taken for.
    pub(crate) fn clear(&mut self, uuid: u128, reason: &'static str) {
        let mut any = false;
        for hold in self.holds.iter_mut().filter(|h| h.player == uuid) {
            hold.ending.get_or_insert(reason);
            any = true;
        }
        for ally in self.allies.iter_mut().filter(|a| a.player == uuid) {
            ally.ending = true;
            any = true;
        }
        if let Some(entry) = self.players.get_mut(&uuid) {
            any |= entry.zinc.is_some() || entry.brass.is_some();
            entry.zinc = None;
            entry.brass = None;
            entry.struck = None;
            entry.provoked_at.clear();
            entry.soothed_at.clear();
        }
        self.by_entity.retain(|_, owner| *owner != uuid);
        if any {
            eprintln!("MISTBRN event=power_cleared reason={reason} power=zinc_brass");
        }
    }

    /// A logout: the player is forgotten once their influence is ended.
    pub(crate) fn logged_out(&mut self, uuid: u128) {
        self.clear(uuid, "logout");
        self.players.remove(&uuid);
    }
}

/// Takes off the strength and speed this module gave an ally, and only those: the same
/// level, hidden, counting down from when they were given (another source's stay).
fn take_courage(host: &mut dyn ServerHost, ally: &Ally, effects: Effects, tick: u64) {
    let read: &dyn Host = host;
    let Some(living) = read.living(ally.entity) else {
        return;
    };
    let expected = i64::from(ally.ticks) - tick.saturating_sub(ally.given) as i64;
    for key in [effects.strength, effects.speed] {
        let read: &dyn Host = host;
        let ours = living.effect(read, key).is_some_and(|e| {
            e.amplifier == ally.amplifier
                && e.ambient
                && !e.visible
                && e.duration > 0
                && (expected - 2..=expected).contains(&i64::from(e.duration))
        });
        if ours {
            let _ = living.remove_effect(host, key);
        }
    }
}

/// Sends a soothed creature `blocks` away from the player, straight out from them.
fn retreat(
    host: &mut dyn ServerHost,
    mob: MobRef,
    from: Option<[f64; 3]>,
    blocks: f64,
    speed: f64,
) {
    let read: &dyn Host = host;
    let (Some(at), Some(from)) = (mob.entity().position(read), from) else {
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
        "MISTBRN event=retreat entity={} distance={:.1} to={:.1},{:.1},{:.1} speed={speed:.2} ok={started}",
        mob.entity().id(),
        distance(at, from),
        to[0],
        to[1],
        to[2]
    );
}
