use crate::{Metal, Pair, PlayerId, Tick, WorldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BurnState {
    Off,
    Burning,
    Flared,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetalState {
    pub unlocked: bool,
    pub reserve_milli: u64,
    pub burn: BurnState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: PlayerId,
    pub world_id: WorldId,
    pub last_tick: Option<Tick>,
    pub selected_pair: Pair,
    pub metals: [MetalState; 8],
}

impl PlayerState {
    pub fn new(player_id: PlayerId, world_id: WorldId) -> Self {
        Self {
            player_id,
            world_id,
            last_tick: None,
            selected_pair: Pair::IronSteel,
            metals: [MetalState {
                unlocked: false,
                reserve_milli: 0,
                burn: BurnState::Off,
            }; Metal::ALL.len()],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldState {
    pub world_id: WorldId,
    pub players: BTreeMap<PlayerId, PlayerState>,
}
