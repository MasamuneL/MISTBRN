//! Tin and pewter's powers (P8), server side.
//!
//! Each server tick reconciles what a player has lit with what this module put on them: tin's
//! hidden night vision, the stun in bright light, pewter's attribute modifiers, the flare's
//! tiredness, the drag after a long flare and clarity (darkness and blindness shortened: the
//! game's effect hooks only see content mods' own effects). Hooks read the same state (by
//! entity id): the damage and fall hooks scale pewter's damage, the sound hook marks what tin
//! hears. Nothing here is saved: putting the metal out, dying,
//! changing dimension or reconnecting undoes it (`event=power_cleared`).
//!
//! Hooks may fire while this module's own tick holds its lock: they use `try_lock` and skip,
//! which is what this module's own changes want.
use crate::bridge::Session;
use allomancy_core::{
    BurnState, CATALOG, DODGE_COOLDOWN, DODGE_COST_MILLI, DODGE_IFRAMES, EVIDENCE_FALL,
    EVIDENCE_HEARD_UNSEEN, EVIDENCE_PEWTER_FALL, EVIDENCE_TIN_DARK, EventKind, FLARE_EXHAUSTION,
    HEARING_RANGE, Metal, NIGHT_VISION_RENEW, NIGHT_VISION_TICKS, NODE_REFLEXES, PRACTICE_FALL,
    PRACTICE_HEARD_HIT, PRACTICE_PEWTER_FALL, PRACTICE_PEWTER_FIGHT, PewterBody,
    RESEARCH_THRESHOLD, SENSES_MAX, STUN_COOLDOWN, STUN_LIGHT, TinPewter, dodge_velocity,
    encode_senses, is_hostile, spend_reserve, update_player,
};
use mod_api::{
    ActiveEffect, AttributeModifier, AttributeOperation, EffectKey, EntityRef, Host, HurtSource,
    Identifier, LevelRef, LightLayer, LivingRef, ModifierLifetime, PayloadBytes, PayloadId,
    PlayerRef, Registrar, ServerHost,
};
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub(crate) const SENSES_CHANNEL: &str = "mistbrn:senses";
/// The senses list goes at most this often.
const SENSES_EVERY_TICKS: u64 = 5;
/// The stun's blindness is left alone by clarity for this long after it is given.
const STUN_GRACE: u64 = 45;
/// Hearing listens this often, for creatures that moved more than `MOVED` blocks since.
const LISTEN_EVERY_TICKS: u64 = 5;
const MOVED: f64 = 0.05;
/// Tin's darkness check (research evidence) runs this often.
const DARK_EVERY_TICKS: u64 = 20;

/// Pewter's attribute modifiers: attribute, modifier path (`mistbrn:<path>`).
const BODY: [(&str, &str); 3] = [
    ("minecraft:attack_damage", "pewter_attack"),
    ("minecraft:movement_speed", "pewter_speed"),
    ("minecraft:knockback_resistance", "pewter_knockback"),
];

/// Damage types a dodge ignores: melee and projectiles.
const DODGED: [&str; 11] = [
    "minecraft:mob_attack",
    "minecraft:mob_attack_no_aggro",
    "minecraft:player_attack",
    "minecraft:arrow",
    "minecraft:trident",
    "minecraft:mob_projectile",
    "minecraft:thrown",
    "minecraft:sting",
    "minecraft:spit",
    "minecraft:fireball",
    "minecraft:wind_charge",
];

#[derive(Clone, Copy)]
struct Effects {
    night_vision: EffectKey,
    blindness: EffectKey,
    darkness: EffectKey,
    slowness: EffectKey,
    mining_fatigue: EffectKey,
}

static EFFECTS: OnceLock<Effects> = OnceLock::new();

/// Requests the vanilla effects the powers use (registration only).
pub(crate) fn register(registrar: &mut dyn Registrar) {
    let _ = EFFECTS.set(Effects {
        night_vision: registrar.effect_key("minecraft:night_vision"),
        blindness: registrar.effect_key("minecraft:blindness"),
        darkness: registrar.effect_key("minecraft:darkness"),
        slowness: registrar.effect_key("minecraft:slowness"),
        mining_fatigue: registrar.effect_key("minecraft:mining_fatigue"),
    });
}

/// What this module did to one player, and what its hooks need.
#[derive(Default)]
struct Player {
    entity: u32,
    rules: TinPewter,
    tin: Option<bool>,
    pewter: Option<bool>,
    /// Pewter's modifiers on the player, as applied.
    body: Option<PewterBody>,
    night_vision: bool,
    flared_since: Option<u64>,
    last_stun: Option<u64>,
    dodge_ready: u64,
    iframes_until: u64,
    /// Creatures heard: entity id → (marked until, out of sight when marked).
    heard: BTreeMap<u32, (u64, bool)>,
    sent: Vec<u32>,
    sent_at: Option<u64>,
    /// Evidence seen this lighting (tin's darkness, a creature heard unseen).
    dark_seen: bool,
    unseen_seen: bool,
    /// Login, respawn, dimension: clear everything this module may have left, then reconcile.
    fresh: bool,
    /// The darkness and blindness clarity last shortened: (tick, duration it was left with).
    clarified: [Option<(u64, i32)>; 2],
}

/// A player with tin lit, for the sound hook.
struct Hearer {
    uuid: u128,
    dimension: String,
    eye: [f64; 3],
    hearing: Option<(f64, u64)>,
}

#[derive(Default)]
pub(crate) struct TinPewterPowers {
    players: BTreeMap<u128, Player>,
    by_entity: BTreeMap<u32, u128>,
    hearers: Vec<Hearer>,
    /// Where each creature near a tin player was at the last listen.
    last_positions: BTreeMap<u32, [f64; 3]>,
    tick: u64,
    /// Falls of pewter players, checked the next tick: (player, distance).
    falls: Vec<(u128, f64)>,
    observations: Vec<(u128, EventKind)>,
}

fn id(path: &str) -> Option<Identifier> {
    Identifier::from_parts("mistbrn", path).ok()
}

fn lit(burn: BurnState) -> Option<bool> {
    match burn {
        BurnState::Off => None,
        BurnState::Burning => Some(false),
        BurnState::Flared => Some(true),
    }
}

/// Ours: the hidden night vision this module gives (no particles, no icon, never longer).
fn ours(effect: &ActiveEffect) -> bool {
    effect.amplifier == 0
        && effect.ambient
        && !effect.visible
        && !effect.show_icon
        && (0..=NIGHT_VISION_TICKS).contains(&effect.duration)
}

fn remove_night_vision(host: &mut dyn ServerHost, living: LivingRef, effects: Effects) {
    let read: &dyn Host = host;
    if living
        .effect(read, effects.night_vision)
        .is_some_and(|effect| ours(&effect))
    {
        let _ = living.remove_effect(host, effects.night_vision);
    }
}

fn set_body(host: &mut dyn ServerHost, living: LivingRef, body: Option<PewterBody>) {
    for (attribute, path) in BODY {
        let Some(modifier) = id(path) else { continue };
        let Some(body) = body else {
            let _ = living.remove_attribute_modifier(host, attribute, &modifier);
            continue;
        };
        let (amount, operation) = match path {
            "pewter_attack" => (body.attack, AttributeOperation::AddMultipliedBase),
            "pewter_speed" => (body.speed, AttributeOperation::AddMultipliedBase),
            _ => (body.knockback, AttributeOperation::AddValue),
        };
        let _ = living.replace_attribute_modifier(
            host,
            attribute,
            AttributeModifier::new(modifier, amount, operation),
            ModifierLifetime::Transient,
        );
    }
}

/// Light at the head as the game shows it: block light, or sky light less the night's dimming.
fn light_at(read: &dyn Host, level: LevelRef, eye: [f64; 3]) -> i32 {
    let at = eye.map(|v| v.floor() as i32);
    let block = level.brightness(read, LightLayer::Block, at).unwrap_or(0);
    let sky = level.brightness(read, LightLayer::Sky, at).unwrap_or(0)
        - level.get_sky_darken(read).unwrap_or(0);
    block.max(sky)
}

/// Whether `living` has darkness or blindness on (the client's fog check too).
pub(crate) fn dark(host: &dyn Host, living: LivingRef) -> bool {
    EFFECTS.get().is_some_and(|effects| {
        [effects.darkness, effects.blindness]
            .into_iter()
            .any(|key| living.effect(host, key).is_some())
    })
}

fn evidence(bit: u8) -> EventKind {
    EventKind::ResearchEvidence {
        research: RESEARCH_THRESHOLD,
        evidence: bit,
    }
}

fn practice(category: u8) -> EventKind {
    EventKind::PracticeAction { category }
}

fn distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl TinPewterPowers {
    /// The tick's reconciliation, after the key requests and the consumption.
    pub(crate) fn tick(&mut self, host: &mut dyn ServerHost, session: &Session, tick: u64) {
        self.tick = tick;
        let Some(&effects) = EFFECTS.get() else {
            return;
        };
        let online = host.players().server_players();
        self.players
            .retain(|uuid, _| online.iter().any(|p| p.uuid() == *uuid));
        self.by_entity.clear();
        self.hearers.clear();
        for player in online {
            let uuid = player.uuid();
            let Some(saved) = session.saved(uuid) else {
                continue;
            };
            let state = saved.state();
            let tin = lit(state.metals[Metal::Tin.index()].burn);
            let pewter = lit(state.metals[Metal::Pewter.index()].burn);
            let rules = TinPewter::new(saved.progress(), &CATALOG);
            let entry = self.players.entry(uuid).or_insert_with(|| Player {
                fresh: true,
                ..Player::default()
            });
            let body = player.entity();
            entry.entity = body.id();
            entry.rules = rules;
            self.by_entity.insert(entry.entity, uuid);
            let read: &dyn Host = host;
            let Some(living) = read.living(body) else {
                continue;
            };
            // Pewter's modifiers carry this mod's own ids: always safe to take off. Night
            // vision is only taken off once this session gave it (see `tin_tick`).
            if entry.fresh {
                set_body(host, living, None);
                entry.body = None;
                entry.heard.clear();
                entry.fresh = false;
            }
            let (was_tin, was_pewter) = (entry.tin, entry.pewter);
            entry.tin = tin;
            entry.pewter = pewter;
            tin_tick(host, player, living, entry, effects, tick);
            pewter_tick(host, player, living, entry, effects, was_pewter, tick);
            if (was_tin.is_some() && tin.is_none()) || (was_pewter.is_some() && pewter.is_none()) {
                eprintln!(
                    "MISTBRN event=power_cleared reason=off tin={} pewter={}",
                    tin.is_some(),
                    pewter.is_some()
                );
            }
            if tin.is_some() && !entry.dark_seen && tick.is_multiple_of(DARK_EVERY_TICKS) {
                let read: &dyn Host = host;
                let dark = [effects.darkness, effects.blindness]
                    .into_iter()
                    .any(|key| living.effect(read, key).is_some());
                if dark {
                    entry.dark_seen = true;
                    self.observations.push((uuid, evidence(EVIDENCE_TIN_DARK)));
                }
            }
            send_senses(host, player, entry, tick);
            let read: &dyn Host = host;
            if tin.is_some()
                && let (Some(eye), Some(dimension)) = (
                    body.eye_position(read),
                    player
                        .level(read)
                        .and_then(|level| level.dimension().id(read)),
                )
            {
                self.hearers.push(Hearer {
                    uuid,
                    dimension,
                    eye,
                    hearing: rules.hearing(),
                });
            }
        }
        if self.hearers.is_empty() {
            self.last_positions.clear();
        } else if tick.is_multiple_of(LISTEN_EVERY_TICKS) {
            self.listen(host);
        }
        // Falls survived: the player is still alive the tick after landing.
        for (uuid, distance) in std::mem::take(&mut self.falls) {
            let read: &dyn Host = host;
            let alive = host
                .player(uuid)
                .and_then(|p| read.living(p.entity()))
                .and_then(|l| l.health(read))
                .is_some_and(|health| health > 0.0);
            eprintln!("MISTBRN event=pewter_fall distance={distance:.1} survived={alive}");
            if !alive {
                continue;
            }
            if distance >= PRACTICE_FALL {
                self.observations
                    .push((uuid, practice(PRACTICE_PEWTER_FALL)));
            }
            if distance >= EVIDENCE_FALL {
                self.observations
                    .push((uuid, evidence(EVIDENCE_PEWTER_FALL)));
            }
        }
    }

    /// The server tick, before this tick's key requests (the dodge's cooldown counts from it).
    pub(crate) fn begin(&mut self, tick: u64) {
        self.tick = tick;
    }

    pub(crate) fn take_observations(&mut self) -> Vec<(u128, EventKind)> {
        std::mem::take(&mut self.observations)
    }

    /// Respawn, dimension change: what this module put on the player is undone on its next
    /// tick, then reconciled with what is lit.
    pub(crate) fn clear(&mut self, uuid: u128, reason: &str) {
        if let Some(entry) = self.players.get_mut(&uuid) {
            if entry.tin.is_some() || entry.pewter.is_some() || entry.body.is_some() {
                eprintln!("MISTBRN event=power_cleared reason={reason}");
            }
            entry.fresh = true;
            entry.flared_since = None;
            entry.iframes_until = 0;
            entry.tin = None;
            entry.pewter = None;
        }
    }

    /// A logout forgets the player. Night vision this session gave is removed if the player is
    /// still reachable; otherwise what is left of it (20 s at most) runs out after the next login.
    pub(crate) fn logged_out(&mut self, host: &mut dyn ServerHost, uuid: u128) {
        self.clear(uuid, "logout");
        let gave = self
            .players
            .remove(&uuid)
            .is_some_and(|entry| entry.night_vision);
        if let (Some(player), Some(&effects)) = (host.player(uuid), EFFECTS.get()) {
            let read: &dyn Host = host;
            if let Some(living) = read.living(player.entity()) {
                if gave {
                    remove_night_vision(host, living, effects);
                }
                set_body(host, living, None);
            }
        }
    }

    fn player_of(&self, entity: u32) -> Option<(u128, &Player)> {
        let uuid = *self.by_entity.get(&entity)?;
        Some((uuid, self.players.get(&uuid)?))
    }

    /// Hearing, every few ticks: creatures within reach of a tin player that moved since the
    /// last listen (steps, flight, a hit's knockback) and are not silent. The pinned game asks
    /// its sound hooks only on the client, so the server listens to motion instead.
    fn listen(&mut self, host: &dyn Host) {
        let mut moved: BTreeMap<u32, (LevelRef, EntityRef)> = BTreeMap::new();
        let mut positions = BTreeMap::new();
        for hearer in &self.hearers {
            let reach = hearer
                .hearing
                .map_or(HEARING_RANGE, |(range, _)| range.max(HEARING_RANGE));
            let Some(level) = host
                .player(hearer.uuid)
                .and_then(|player| player.level(host))
            else {
                continue;
            };
            let low = hearer.eye.map(|v| v - reach);
            let high = hearer.eye.map(|v| v + reach);
            for entity in level.entities_in(host, low, high).unwrap_or_default() {
                if !self.creature(host, entity) || entity.is_silent(host) == Some(true) {
                    continue;
                }
                let Some(at) = entity.position(host) else {
                    continue;
                };
                let id = entity.id();
                if self
                    .last_positions
                    .get(&id)
                    .is_some_and(|before| distance(*before, at) > MOVED)
                {
                    moved.insert(id, (level, entity));
                }
                positions.insert(id, at);
            }
        }
        self.last_positions = positions;
        for (level, entity) in moved.into_values() {
            let (Some(at), Some(dimension)) = (entity.position(host), level.dimension().id(host))
            else {
                continue;
            };
            let near: Vec<usize> = (0..self.hearers.len())
                .filter(|&i| {
                    let hearer = &self.hearers[i];
                    if hearer.dimension != dimension {
                        return false;
                    }
                    let reach = hearer
                        .hearing
                        .map_or(HEARING_RANGE, |(range, _)| range.max(HEARING_RANGE));
                    distance(hearer.eye, at) <= reach
                })
                .collect();
            self.heard_from(host, level, entity, near);
        }
    }

    /// A living creature that is not a player this module knows.
    fn creature(&self, host: &dyn Host, entity: EntityRef) -> bool {
        !self.by_entity.contains_key(&entity.id())
            && host
                .living(entity)
                .and_then(|l| l.alive(host))
                .unwrap_or(false)
    }

    fn heard_from(
        &mut self,
        host: &dyn Host,
        level: LevelRef,
        source: EntityRef,
        near: Vec<usize>,
    ) {
        let Some(position) = source.position(host) else {
            return;
        };
        let middle =
            source
                .eye_position(host)
                .unwrap_or([position[0], position[1] + 0.5, position[2]]);
        let tick = self.tick;
        for i in near {
            let hearer = &self.hearers[i];
            let dist = distance(hearer.eye, middle);
            let unseen = level.line_of_sight(host, hearer.eye, middle) == Some(false);
            let Some(entry) = self.players.get_mut(&hearer.uuid) else {
                continue;
            };
            if unseen && dist <= HEARING_RANGE && !entry.unseen_seen {
                entry.unseen_seen = true;
                self.observations
                    .push((hearer.uuid, evidence(EVIDENCE_HEARD_UNSEEN)));
            }
            if let Some((range, memory)) = hearer.hearing
                && dist <= range
            {
                let fresh = !entry.heard.contains_key(&source.id());
                entry.heard.insert(source.id(), (tick + memory, unseen));
                if fresh {
                    eprintln!(
                        "MISTBRN event=heard entity={} distance={dist:.1} unseen={unseen}",
                        source.id()
                    );
                }
            }
        }
    }

    /// A hit about to land on `entity`: a dodge cancels it (true), pewter takes less (the new
    /// damage), and a creature heard out of sight, hit by its hearer, is practice.
    pub(crate) fn incoming_damage(
        &mut self,
        entity: i32,
        source: &HurtSource<'_>,
        damage: f32,
    ) -> (bool, Option<f32>) {
        let tick = self.tick;
        let Ok(victim) = u32::try_from(entity) else {
            return (false, None);
        };
        if let Ok(attacker) = u32::try_from(source.causing)
            && let Some(&uuid) = self.by_entity.get(&attacker)
            && let Some(entry) = self.players.get_mut(&uuid)
            && let Some((until, unseen)) = entry.heard.get_mut(&victim)
            && *unseen
            && *until >= tick
        {
            *unseen = false;
            self.observations.push((uuid, practice(PRACTICE_HEARD_HIT)));
            eprintln!("MISTBRN event=heard_hit entity={victim}");
        }
        let Some((_, player)) = self.player_of(victim) else {
            return (false, None);
        };
        if player.iframes_until >= tick && DODGED.contains(&source.damage_type) {
            eprintln!("MISTBRN event=dodged damage_type={}", source.damage_type);
            return (true, None);
        }
        // Falls have their own multiplier (`fall`).
        let Some(flared) = player
            .pewter
            .filter(|_| source.damage_type != "minecraft:fall")
        else {
            return (false, None);
        };
        let taken = damage * player.rules.body(flared).damage_taken;
        eprintln!("MISTBRN event=pewter_damage from={damage:.2} to={taken:.2}");
        (false, Some(taken))
    }

    /// Pewter's jump on the server, from the burn and ranks it holds (the client predicts the
    /// same from what it was last sent).
    pub(crate) fn jump(&self, entity: u32) -> Option<f32> {
        let (_, player) = self.player_of(entity)?;
        player.pewter.map(|_| player.rules.jump())
    }

    /// A fall about to be handled (server): pewter's damage multiplier, or None.
    pub(crate) fn fall(&mut self, entity: u32, distance: f64) -> Option<f32> {
        let (uuid, player) = self.player_of(entity)?;
        let flared = player.pewter?;
        let multiplier = player.rules.body(flared).fall;
        if distance >= PRACTICE_FALL {
            self.falls.push((uuid, distance));
        }
        eprintln!(
            "MISTBRN event=pewter_fall_damage distance={distance:.1} multiplier={multiplier:.2}"
        );
        Some(multiplier)
    }

    /// A creature died: a hostile one killed by a player with pewter lit is practice.
    pub(crate) fn died(&mut self, kind: &str, source: &HurtSource<'_>) {
        let Some((uuid, player)) = u32::try_from(source.causing)
            .ok()
            .and_then(|killer| self.player_of(killer))
        else {
            return;
        };
        if player.pewter.is_some() && is_hostile(kind) {
            self.observations
                .push((uuid, practice(PRACTICE_PEWTER_FIGHT)));
            eprintln!("MISTBRN event=pewter_kill kind={kind}");
        }
    }

    /// Reflexes' dodge (op 10): the technique owned, tin and pewter lit, the cooldown over and
    /// the pewter there; the pewter is spent in the save, then the player moves.
    pub(crate) fn dodge(
        &mut self,
        host: &mut dyn ServerHost,
        session: &mut Session,
        player: PlayerRef,
        direction: u8,
    ) -> Result<(), String> {
        let tick = self.tick;
        let uuid = player.uuid();
        let saved = session.saved(uuid).ok_or("not_active")?;
        if saved.progress().rank(NODE_REFLEXES) == 0 {
            return Err("NotOwned".into());
        }
        let state = saved.state();
        let off = |metal: Metal| state.metals[metal.index()].burn == BurnState::Off;
        if off(Metal::Tin) || off(Metal::Pewter) {
            return Err("NotLit".into());
        }
        let entry = self.players.get_mut(&uuid).ok_or("not_active")?;
        if tick < entry.dodge_ready {
            return Err("Cooldown".into());
        }
        let read: &dyn Host = host;
        let [yaw, _] = player.entity().rotation(read).ok_or("no_body")?;
        let velocity = dodge_velocity(yaw, direction).ok_or("Direction")?;
        // Checked before moving, spent after: a dodge that cannot move costs nothing. Both
        // happen in this one call, under the session and powers locks.
        let left = state.metals[Metal::Pewter.index()].reserve_milli;
        if left < DODGE_COST_MILLI {
            return Err("NotEnough".into());
        }
        if !player.set_velocity(host, velocity) {
            return Err("NoMove".into());
        }
        session.change(uuid, true, |saved| {
            let rules = saved.rules().clone();
            update_player(saved, |state, accounting| {
                spend_reserve(state, accounting, &rules, Metal::Pewter, DODGE_COST_MILLI)
            })
        })?;
        entry.dodge_ready = tick + DODGE_COOLDOWN;
        entry.iframes_until = tick + DODGE_IFRAMES;
        eprintln!(
            "MISTBRN event=dodge spent={DODGE_COST_MILLI} left={} velocity={:.2},{:.2},{:.2}",
            left - DODGE_COST_MILLI,
            velocity[0],
            velocity[1],
            velocity[2]
        );
        Ok(())
    }
}

fn tin_tick(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    living: LivingRef,
    entry: &mut Player,
    effects: Effects,
    tick: u64,
) {
    let Some(flared) = entry.tin else {
        if entry.night_vision {
            remove_night_vision(host, living, effects);
            entry.night_vision = false;
            eprintln!("MISTBRN event=power power=night_vision state=off");
        }
        entry.heard.clear();
        entry.dark_seen = false;
        entry.unseen_seen = false;
        return;
    };
    let read: &dyn Host = host;
    // A potion's night vision is left alone; ours is renewed before it runs low.
    let renew = living
        .effect(read, effects.night_vision)
        .is_none_or(|effect| ours(&effect) && effect.duration <= NIGHT_VISION_RENEW);
    if renew {
        let given = living.add_effect(
            host,
            &ActiveEffect::new(
                effects.night_vision,
                NIGHT_VISION_TICKS,
                0,
                true,
                false,
                false,
            ),
        );
        if !entry.night_vision && given {
            eprintln!("MISTBRN event=power power=night_vision state=on");
        }
        entry.night_vision |= given;
    }
    entry.heard.retain(|_, (until, _)| *until >= tick);
    clarity(host, living, entry, effects, tick);
    if !flared || entry.last_stun.is_some_and(|at| tick < at + STUN_COOLDOWN) {
        return;
    }
    let read: &dyn Host = host;
    let (Some(level), Some(eye)) = (player.level(read), player.entity().eye_position(read)) else {
        return;
    };
    let light = light_at(read, level, eye);
    let ticks = entry.rules.stun();
    if light >= STUN_LIGHT && ticks > 0 {
        entry.last_stun = Some(tick);
        let _ = living.add_effect(
            host,
            &ActiveEffect::new(effects.blindness, ticks as i32, 0, false, true, true),
        );
        eprintln!("MISTBRN event=stun light={light} ticks={ticks}");
    }
}

/// Tin's clarity: a darkness or blindness not yet shortened (one this module did not leave)
/// keeps only its share. The game's effect hooks only see content mods' own effects, so this
/// runs each tick instead.
fn clarity(
    host: &mut dyn ServerHost,
    living: LivingRef,
    entry: &mut Player,
    effects: Effects,
    tick: u64,
) {
    if !entry.rules.has_clarity() {
        return;
    }
    for (slot, key) in [effects.darkness, effects.blindness]
        .into_iter()
        .enumerate()
    {
        let read: &dyn Host = host;
        let Some(active) = living.effect(read, key) else {
            entry.clarified[slot] = None;
            continue;
        };
        // Ours: what was left, counted down since (a tick or two of slack).
        let ours = entry.clarified[slot].is_some_and(|(at, left)| {
            let expected = i64::from(left) - (tick - at) as i64;
            (i64::from(active.duration) - expected).abs() <= 2
        });
        // The stun's own blindness is already its length.
        let stun = slot == 1 && entry.last_stun.is_some_and(|at| tick - at <= STUN_GRACE);
        if ours || stun || active.duration < 0 {
            continue;
        }
        let left = entry.rules.clarity(active.duration);
        let _ = living.remove_effect(host, key);
        if left > 0 {
            let _ = living.add_effect(
                host,
                &ActiveEffect::new(
                    key,
                    left,
                    active.amplifier,
                    active.ambient,
                    active.visible,
                    active.show_icon,
                ),
            );
        }
        entry.clarified[slot] = Some((tick, left));
        eprintln!("MISTBRN event=clarity from={} to={left}", active.duration);
    }
}

fn pewter_tick(
    host: &mut dyn ServerHost,
    player: PlayerRef,
    living: LivingRef,
    entry: &mut Player,
    effects: Effects,
    was: Option<bool>,
    tick: u64,
) {
    // Lit, flared or not, or a new rank bought while it burns: the modifiers follow.
    let wanted = entry.pewter.map(|flared| entry.rules.body(flared));
    if entry.body != wanted {
        set_body(host, living, wanted);
        let shown = match entry.pewter {
            None => "off",
            Some(false) => "burning",
            Some(true) => "flared",
        };
        let attack = wanted.map_or(0.0, |body| body.attack);
        eprintln!("MISTBRN event=power power=pewter_body state={shown} attack={attack:.2}");
        entry.body = wanted;
    }
    if entry.pewter == Some(true) {
        let _ = player.add_exhaustion(host, FLARE_EXHAUSTION);
        entry.flared_since.get_or_insert(tick);
        return;
    }
    let since = entry.flared_since.take();
    if was != Some(true) {
        return;
    }
    if let Some(ticks) = since.and_then(|since| entry.rules.drag(tick - since)) {
        for (key, amplifier) in [(effects.slowness, 1), (effects.mining_fatigue, 0)] {
            let _ = living.add_effect(
                host,
                &ActiveEffect::new(key, ticks as i32, amplifier, false, true, true),
            );
        }
        eprintln!("MISTBRN event=drag ticks={ticks}");
    }
}

fn send_senses(host: &mut dyn ServerHost, player: PlayerRef, entry: &mut Player, tick: u64) {
    let ids: Vec<u32> = entry.heard.keys().copied().take(SENSES_MAX).collect();
    if ids == entry.sent
        || entry
            .sent_at
            .is_some_and(|at| tick < at + SENSES_EVERY_TICKS)
    {
        return;
    }
    let channel = PayloadId(SENSES_CHANNEL.into());
    let accepted = player
        .send_payload(host, &channel, PayloadBytes(encode_senses(&ids)))
        .is_ok();
    entry.sent_at = Some(tick);
    if accepted {
        eprintln!("MISTBRN event=senses_sent count={}", ids.len());
        entry.sent = ids;
    }
}
