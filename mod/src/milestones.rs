//! Milestones seen in the game (P6). The hooks only queue: each claim writes the save in its own
//! host transaction, which the server tick opens (`Session::claim`), as a vial's credit does.
//!
//! - Bastion: every 20 ticks, a player of the level in place standing in a bastion piece.
//! - Elder guardian: its death, seen by `living_death` and confirmed by `living_death_post`
//!   (a later listener may cancel it).
//! - Ancient city: a player's open fills an ancient city chest (`container_loot_filled`).
//! - Dragon: no death hook fires for it, so a dragon that was hit is read each level tick
//!   until its health reaches 0 (its death animation starts there).
//!
//! A boss is claimed by every player whose hit took health in the 600 ticks before its death
//! (`BossLedger`, fed by `living_damage_post`), counted by the attacker's entity id as the
//! damage source names it. A claim whose write fails for a reason that may pass is retried
//! each tick for up to a minute; a refusal (`AlreadyClaimed`) or a blocked save ends it.
use crate::bridge::Session;
use allomancy_core::{
    BossLedger, MILESTONE_ANCIENT_CITY, MILESTONE_BASTION, MILESTONE_DRAGON,
    MILESTONE_ELDER_GUARDIAN,
};
use mod_api::{Host, HurtSource, LevelRef, ServerHost, StructureSearch};
use std::collections::BTreeMap;

const BASTION: &str = "minecraft:bastion_remnant";
const ANCIENT_CITY_TABLES: [&str; 2] = [
    "minecraft:chests/ancient_city",
    "minecraft:chests/ancient_city_ice_box",
];
const ELDER_GUARDIAN: &str = "minecraft:elder_guardian";
const DRAGON: &str = "minecraft:ender_dragon";
/// How often each player's bastion presence is read.
const STRUCTURE_EVERY_TICKS: u64 = 20;
/// How many ticks a claim whose write failed is retried.
const MAX_RETRIES: u16 = 1200;

/// Who a queued claim is for: a player, or an attacker's entity id resolved at the server tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Claimant {
    Player(u128),
    Entity(u32),
}

#[derive(Default)]
pub(crate) struct Milestones {
    /// The server tick being run (the one after the last `ServerTickEnd`).
    tick: u64,
    ledger: BossLedger,
    /// The milestone of each boss with recorded hits.
    bosses: BTreeMap<i32, u8>,
    queued: Vec<(Claimant, u8)>,
    /// Bosses whose death `living_death` saw this tick, until `living_death_post` confirms it.
    dying: BTreeMap<i32, u8>,
    /// Claims whose write failed for a reason that may pass, with the attempts made.
    retries: BTreeMap<(u128, u8), u16>,
    /// Test runs only (`/mistbrn-test bastion-find|probe`): answered in the player's level pass.
    probes: Vec<(u128, bool)>,
}

fn boss_milestone(kind: &str) -> Option<u8> {
    match kind {
        ELDER_GUARDIAN => Some(MILESTONE_ELDER_GUARDIAN),
        DRAGON => Some(MILESTONE_DRAGON),
        _ => None,
    }
}

/// An attacker's entity id as the ledger keys it (ids are non-negative; 0 is "none").
fn attacker(source: &HurtSource<'_>) -> Option<u128> {
    u32::try_from(source.causing)
        .ok()
        .filter(|&id| id != 0)
        .map(u128::from)
}

impl Milestones {
    /// A hit on a boss (`living_damage_post`) that took health: its attacker is remembered.
    pub(crate) fn damaged(&mut self, entity: i32, kind: &str, source: &HurtSource<'_>, dmg: f32) {
        let (Some(bit), Some(by)) = (boss_milestone(kind), attacker(source)) else {
            return;
        };
        if dmg > 0.0 {
            self.ledger.hit(entity, by, self.tick);
            self.bosses.insert(entity, bit);
        }
    }

    /// A mob is about to die (`living_death`): an elder guardian becomes a candidate. Its
    /// killing blow is already in the ledger (`living_damage_post` comes first).
    pub(crate) fn dying(&mut self, entity: u32, kind: &str) {
        if let (Some(bit), Ok(boss)) = (boss_milestone(kind), i32::try_from(entity)) {
            self.dying.insert(boss, bit);
        }
    }

    /// A death finished (`living_death_post`, never after a cancelled one): a candidate's
    /// claimants are queued.
    pub(crate) fn died(&mut self, entity: u32) {
        if let Some((boss, bit)) = i32::try_from(entity)
            .ok()
            .and_then(|boss| self.dying.remove(&boss).map(|bit| (boss, bit)))
        {
            self.boss_died(boss, bit);
        }
    }

    fn boss_died(&mut self, boss: i32, bit: u8) {
        self.bosses.remove(&boss);
        let claimants = self.ledger.death(boss, self.tick);
        eprintln!(
            "MISTBRN event=boss_died entity={boss} milestone={bit} claimants={}",
            claimants.len()
        );
        for by in claimants {
            if let Ok(id) = u32::try_from(by) {
                self.queued.push((Claimant::Entity(id), bit));
            }
        }
    }

    /// A container filled by a player's open (`container_loot_filled`).
    pub(crate) fn loot(&mut self, table: &str, opener: Option<u128>) {
        if let Some(uuid) = opener
            && ANCIENT_CITY_TABLES.contains(&table)
        {
            self.queued
                .push((Claimant::Player(uuid), MILESTONE_ANCIENT_CITY));
        }
    }

    /// Queues a test probe for `uuid`: the nearest bastion start (`find`), or a block in a
    /// bastion piece near the player.
    pub(crate) fn test_probe(&mut self, uuid: u128, find: bool) {
        self.probes.push((uuid, find));
    }

    /// Answers the queued probes of players in `level` (reads need the level in place).
    fn answer_probes(&mut self, host: &mut dyn ServerHost, level: LevelRef) {
        let read: &dyn Host = host;
        let probes = std::mem::take(&mut self.probes);
        for (uuid, find) in probes {
            let Some(player) = host.player(uuid) else {
                continue;
            };
            let at = player.entity().block_position(read);
            if player.level(read) != Some(level) || at.is_none() {
                self.probes.push((uuid, find));
                continue;
            }
            let [x, y, z] = at.unwrap_or_default();
            let line = if find {
                match level.find_nearest_structure(
                    read,
                    BASTION,
                    StructureSearch::new([x, y, z], 100),
                ) {
                    Some(found) => {
                        let [fx, _, fz] = found.position;
                        format!("found=true x={fx} z={fz}")
                    }
                    None => "found=false".into(),
                }
            } else {
                // Columns every 4 blocks around the player, each scanned bottom to top.
                let inside = (-6..=6)
                    .flat_map(|dx| (-6..=6).map(move |dz| (x + 4 * dx, z + 4 * dz)))
                    .find_map(|(cx, cz)| {
                        (5..=120).find_map(|cy| {
                            level
                                .structures_at(read, [cx, cy, cz])?
                                .iter()
                                .any(|s| s.structure == BASTION && s.in_piece)
                                .then_some([cx, cy, cz])
                        })
                    });
                match inside {
                    Some([px, py, pz]) => format!("found=true x={px} y={py} z={pz}"),
                    None => "found=false".into(),
                }
            };
            let name = if find {
                "bastion-find"
            } else {
                "bastion-probe"
            };
            eprintln!("MISTBRN_TEST scenario={name} {line}");
        }
    }

    /// Per level, while it is in place (`level_tick_post`): dragons hit to death and players
    /// standing in a bastion.
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
        if !self.probes.is_empty() {
            self.answer_probes(host, level);
        }
        let read: &dyn Host = host;
        let dragons: Vec<i32> = self
            .bosses
            .iter()
            .filter(|&(_, &bit)| bit == MILESTONE_DRAGON)
            .map(|(&boss, _)| boss)
            .collect();
        for boss in dragons {
            let health = u32::try_from(boss)
                .ok()
                .and_then(|id| read.entity(id))
                .filter(|entity| entity.level(read) == Some(level))
                .and_then(|entity| read.living(entity))
                .and_then(|living| living.health(read));
            if health.is_some_and(|h| h <= 0.0) {
                self.boss_died(boss, MILESTONE_DRAGON);
            }
        }
        if !self.tick.is_multiple_of(STRUCTURE_EVERY_TICKS) {
            return;
        }
        for player in host.players().server_players() {
            let read: &dyn Host = host;
            let uuid = player.uuid();
            let pending = session
                .saved(uuid)
                .is_some_and(|saved| !saved.progress().claimed(MILESTONE_BASTION));
            if !pending || player.level(read) != Some(level) {
                continue;
            }
            let inside = player
                .entity()
                .block_position(read)
                .and_then(|pos| level.structures_at(read, pos))
                .is_some_and(|found| {
                    found
                        .iter()
                        .any(|at| at.structure == BASTION && at.in_piece)
                });
            if inside {
                self.queued
                    .push((Claimant::Player(uuid), MILESTONE_BASTION));
            }
        }
    }

    /// The claims queued this tick, by player UUID, each once; then the next tick begins.
    /// An attacker that is no online player (a mob, a player gone) claims nothing.
    /// Claims still being retried come back with them.
    pub(crate) fn take(&mut self, host: &mut dyn ServerHost, tick: u64) -> Vec<(u128, u8)> {
        let queued = std::mem::take(&mut self.queued);
        let by_entity = queued.iter().any(|(c, _)| matches!(c, Claimant::Entity(_)));
        let players = if by_entity {
            host.players().server_players()
        } else {
            Vec::new()
        };
        let mut claims: Vec<(u128, u8)> = queued
            .into_iter()
            .filter_map(|(claimant, bit)| match claimant {
                Claimant::Player(uuid) => Some((uuid, bit)),
                Claimant::Entity(id) => players
                    .iter()
                    .find(|player| player.entity().id() == id)
                    .map(|player| (player.uuid(), bit)),
            })
            .collect();
        claims.extend(self.retries.keys().copied());
        claims.sort_unstable();
        claims.dedup();
        // A death `living_death` saw but no `living_death_post` confirmed was cancelled.
        self.dying.clear();
        self.tick = tick + 1;
        self.ledger.prune(self.tick);
        let ledger = &self.ledger;
        self.bosses.retain(|boss, _| ledger.tracks(*boss));
        claims
    }

    /// The outcome of a claim attempt: settled (written, refused or blocked) or retried.
    pub(crate) fn outcome(&mut self, uuid: u128, bit: u8, result: &Result<u64, String>) {
        let key = (uuid, bit);
        match result {
            Err(error) if retryable(error) => {
                let attempts = self.retries.entry(key).or_insert(0);
                *attempts += 1;
                if *attempts >= MAX_RETRIES {
                    self.retries.remove(&key);
                    eprintln!("MISTBRN event=milestone_dropped bit={bit} error={error}");
                }
            }
            _ => {
                self.retries.remove(&key);
            }
        }
    }
}

/// Whether a failed claim may succeed later: not a refusal of the claim itself, nor a save
/// the session blocked after the host committed.
fn retryable(error: &str) -> bool {
    !matches!(error, "AlreadyClaimed" | "UnknownMilestone")
        && !error.starts_with("commit_after_host")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Catches a failed write losing the claim, or a refusal being retried forever.
    #[test]
    fn failed_writes_are_retried_and_refusals_settle() {
        let mut m = Milestones::default();
        m.outcome(7, 1, &Err("write:io".into()));
        assert_eq!(m.retries.get(&(7, 1)), Some(&1));
        m.outcome(7, 1, &Ok(3));
        assert!(m.retries.is_empty());
        m.outcome(7, 2, &Err("write:io".into()));
        m.outcome(7, 2, &Err("AlreadyClaimed".into()));
        assert!(m.retries.is_empty());
        m.outcome(7, 4, &Err("commit_after_host:Duplicate".into()));
        assert!(m.retries.is_empty());
        for _ in 0..MAX_RETRIES {
            m.outcome(9, 8, &Err("nested_transaction".into()));
        }
        assert!(m.retries.is_empty(), "dropped after the last attempt");
    }

    // Catches a death another listener cancelled still being claimed.
    #[test]
    fn only_a_confirmed_death_queues_its_claimants() {
        let mut m = Milestones::default();
        m.ledger.hit(5, 42, 0);
        m.dying(5, ELDER_GUARDIAN);
        m.dying.clear(); // the tick ended with no death_post: cancelled
        m.died(5);
        assert!(m.queued.is_empty());
        m.dying(5, ELDER_GUARDIAN);
        m.died(5);
        assert_eq!(
            m.queued,
            vec![(Claimant::Entity(42), MILESTONE_ELDER_GUARDIAN)]
        );
        m.dying(6, "minecraft:zombie");
        assert!(m.dying.is_empty());
    }
}
