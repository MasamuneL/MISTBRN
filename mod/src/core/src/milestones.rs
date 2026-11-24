//! Boss kills shared by a group (P6): who may claim when a boss dies.
//!
//! The adapter records every hit a player lands on a tracked boss (an elder guardian or the
//! ender dragon) with the server tick; when the boss dies, every player whose last hit is at
//! most [`BOSS_WINDOW_TICKS`] old claims. A player who only watched never claims.
use std::collections::BTreeMap;

/// How long a hit keeps its player among a boss's claimants: 30 seconds.
pub const BOSS_WINDOW_TICKS: u64 = 600;

/// The last hit of each player on each tracked boss, by entity id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BossLedger {
    hits: BTreeMap<i32, BTreeMap<u128, u64>>,
}

impl BossLedger {
    /// `player` hit `boss` at `tick`.
    pub fn hit(&mut self, boss: i32, player: u128, tick: u64) {
        let last = self
            .hits
            .entry(boss)
            .or_default()
            .entry(player)
            .or_insert(tick);
        *last = (*last).max(tick);
    }

    /// Whether any hit on `boss` is recorded.
    pub fn tracks(&self, boss: i32) -> bool {
        self.hits.contains_key(&boss)
    }

    /// The bosses with recorded hits.
    pub fn bosses(&self) -> impl Iterator<Item = i32> + '_ {
        self.hits.keys().copied()
    }

    /// `boss` died at `tick`: its claimants, in UUID order, and the boss is forgotten.
    pub fn death(&mut self, boss: i32, tick: u64) -> Vec<u128> {
        self.hits
            .remove(&boss)
            .unwrap_or_default()
            .into_iter()
            .filter(|&(_, last)| last <= tick && tick - last <= BOSS_WINDOW_TICKS)
            .map(|(player, _)| player)
            .collect()
    }

    /// Forgets `boss` without a death (it unloaded or was removed).
    pub fn forget(&mut self, boss: i32) {
        self.hits.remove(&boss);
    }

    /// Drops hits too old to count at `tick`, and bosses left without any.
    pub fn prune(&mut self, tick: u64) {
        self.hits.retain(|_, players| {
            players.retain(|_, last| tick.saturating_sub(*last) <= BOSS_WINDOW_TICKS);
            !players.is_empty()
        });
    }
}
