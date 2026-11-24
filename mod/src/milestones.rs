//! Milestones seen in the game (P6). The hooks only queue: each claim writes the save in its own
//! host transaction, which the server tick opens (`Session::claim`), as a vial's credit does.
//!
//! - Bastion: every 20 ticks, a player of the level in place standing in a bastion piece.
//! - Elder guardian: its death (`living_death`).
//! - Ancient city: a player's open fills an ancient city chest (`container_loot_filled`).
//! - Dragon: no death hook fires for it, so a dragon that was hit is read each level tick
//!   until its health reaches 0 (its death animation starts there).
//!
//! A boss is claimed by every player whose hit landed in the 600 ticks before its death
//! (`BossLedger`), counted by the attacker's entity id as the damage source names it.
use crate::bridge::Session;
use allomancy_core::{
    BossLedger, MILESTONE_ANCIENT_CITY, MILESTONE_BASTION, MILESTONE_DRAGON,
    MILESTONE_ELDER_GUARDIAN,
};
use mod_api::{Host, HurtSource, LevelRef, ServerHost};
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
    /// A hit on a boss (`living_damage_pre`) that takes health: its attacker is remembered.
    pub(crate) fn damaged(&mut self, entity: i32, kind: &str, source: &HurtSource<'_>, dmg: f32) {
        let (Some(bit), Some(by)) = (boss_milestone(kind), attacker(source)) else {
            return;
        };
        if dmg > 0.0 {
            self.ledger.hit(entity, by, self.tick);
            self.bosses.insert(entity, bit);
        }
    }

    /// A mob's death (`living_death`): an elder guardian's claimants are queued.
    pub(crate) fn died(&mut self, entity: u32, kind: &str, source: &HurtSource<'_>) {
        let (Some(bit), Ok(boss)) = (boss_milestone(kind), i32::try_from(entity)) else {
            return;
        };
        if let Some(by) = attacker(source) {
            self.ledger.hit(boss, by, self.tick);
        }
        self.boss_died(boss, bit);
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
    pub(crate) fn take(&mut self, host: &mut dyn ServerHost, tick: u64) -> Vec<(u128, u8)> {
        let players = host.players().server_players();
        let mut claims: Vec<(u128, u8)> = std::mem::take(&mut self.queued)
            .into_iter()
            .filter_map(|(claimant, bit)| match claimant {
                Claimant::Player(uuid) => Some((uuid, bit)),
                Claimant::Entity(id) => players
                    .iter()
                    .find(|player| player.entity().id() == id)
                    .map(|player| (player.uuid(), bit)),
            })
            .collect();
        claims.sort_unstable();
        claims.dedup();
        self.tick = tick + 1;
        self.ledger.prune(self.tick);
        let ledger = &self.ledger;
        self.bosses.retain(|boss, _| ledger.tracks(*boss));
        claims
    }
}
