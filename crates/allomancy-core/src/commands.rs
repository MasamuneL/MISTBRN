use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Authority {
    Client,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputContext {
    pub gameplay_allowed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TargetSnapshot {
    pub id: String,
    pub position: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldSnapshot {
    pub tick: Tick,
    pub world_id: WorldId,
    pub actor: PlayerId,
    pub position: Vec3,
    pub targets: Vec<TargetSnapshot>,
    pub authority: Authority,
    pub input: InputContext,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Drink {
        transaction: TransactionId,
        slot: u32,
    },
    Activate {
        metal: Metal,
    },
    Flare {
        metal: Metal,
    },
    Extinguish {
        metal: Option<Metal>,
    },
    Purchase {
        node: NodeId,
        transaction: TransactionId,
    },
    Claim {
        event: EventId,
        transaction: TransactionId,
    },
    SelectPair {
        pair: Pair,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Drink,
    Activate,
    Flare,
    Purchase,
    Claim,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleError {
    ServerAuthorityRequired,
    WorldMismatch,
    PlayerMismatch,
    NonFinitePosition,
    NonFiniteTarget,
    StaleTick,
    InputCaptured,
    MetalLocked(Metal),
    EmptyReserve(Metal),
    NotImplemented(Operation),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectStage {
    Proposed,
    AcceptedInMemory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signal {
    BurnChanged { metal: Metal, burn: BurnState },
    PairSelected { pair: Pair },
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectKind {
    Physics {
        target: PlayerId,
        impulse: Vec3,
    },
    Attributes {
        player: PlayerId,
        attribute: String,
        value_milli: i64,
    },
    Inventory {
        transaction: TransactionId,
        slot: u32,
    },
    Signal(Signal),
    Persist {
        transaction: TransactionId,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Effect {
    pub stage: EffectStage,
    pub kind: EffectKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    ResearchObserved { research: ResearchId },
    MilestoneReached { key: String },
}

/// Trusted-adapter data, deliberately not deserializable from client messages.
///
/// ```compile_fail
/// use allomancy_core::VerifiedEvent;
/// let _: VerifiedEvent = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedEvent {
    id: EventId,
    tick: Tick,
    world_id: WorldId,
    actor: PlayerId,
    kind: EventKind,
}

impl VerifiedEvent {
    pub fn from_server_observation(
        id: EventId,
        kind: EventKind,
        ctx: &WorldSnapshot,
    ) -> Result<Self, RuleError> {
        if ctx.authority != Authority::Server {
            return Err(RuleError::ServerAuthorityRequired);
        }
        validate_geometry(ctx)?;
        Ok(Self {
            id,
            tick: ctx.tick,
            world_id: ctx.world_id.clone(),
            actor: ctx.actor.clone(),
            kind,
        })
    }
    pub fn id(&self) -> &EventId {
        &self.id
    }
    pub fn tick(&self) -> Tick {
        self.tick
    }
    pub fn world_id(&self) -> &WorldId {
        &self.world_id
    }
    pub fn actor(&self) -> &PlayerId {
        &self.actor
    }
    pub fn kind(&self) -> &EventKind {
        &self.kind
    }
}

fn validate_geometry(ctx: &WorldSnapshot) -> Result<(), RuleError> {
    if !ctx.position.is_finite() {
        return Err(RuleError::NonFinitePosition);
    }
    if ctx
        .targets
        .iter()
        .any(|target| !target.position.is_finite())
    {
        return Err(RuleError::NonFiniteTarget);
    }
    Ok(())
}

fn validate_context(state: &PlayerState, ctx: &WorldSnapshot) -> Result<(), RuleError> {
    if ctx.authority != Authority::Server {
        return Err(RuleError::ServerAuthorityRequired);
    }
    if state.world_id != ctx.world_id {
        return Err(RuleError::WorldMismatch);
    }
    if state.player_id != ctx.actor {
        return Err(RuleError::PlayerMismatch);
    }
    validate_geometry(ctx)?;
    if state.last_tick.is_some_and(|tick| ctx.tick < tick) {
        return Err(RuleError::StaleTick);
    }
    Ok(())
}

fn signal(signal: Signal) -> Effect {
    Effect {
        stage: EffectStage::AcceptedInMemory,
        kind: EffectKind::Signal(signal),
    }
}

pub fn apply_command(
    state: &mut PlayerState,
    command: Command,
    ctx: &WorldSnapshot,
) -> Result<Vec<Effect>, RuleError> {
    validate_context(state, ctx)?;
    if !ctx.input.gameplay_allowed && !matches!(&command, Command::Extinguish { .. }) {
        return Err(RuleError::InputCaptured);
    }
    match command {
        Command::Activate { metal } | Command::Flare { metal } => {
            let entry = &state.metals[metal.index()];
            if !entry.unlocked {
                return Err(RuleError::MetalLocked(metal));
            }
            if entry.reserve_milli == 0 {
                return Err(RuleError::EmptyReserve(metal));
            }
            Err(RuleError::NotImplemented(
                if matches!(command, Command::Activate { .. }) {
                    Operation::Activate
                } else {
                    Operation::Flare
                },
            ))
        }
        Command::Drink { .. } => Err(RuleError::NotImplemented(Operation::Drink)),
        Command::Purchase { .. } => Err(RuleError::NotImplemented(Operation::Purchase)),
        Command::Claim { .. } => Err(RuleError::NotImplemented(Operation::Claim)),
        Command::Extinguish { metal } => {
            let mut effects = Vec::new();
            for current in Metal::ALL {
                if metal.is_some_and(|selected| selected != current) {
                    continue;
                }
                let entry = &mut state.metals[current.index()];
                if entry.burn != BurnState::Off {
                    entry.burn = BurnState::Off;
                    effects.push(signal(Signal::BurnChanged {
                        metal: current,
                        burn: BurnState::Off,
                    }));
                }
            }
            Ok(effects)
        }
        Command::SelectPair { pair } => {
            if state.selected_pair == pair {
                return Ok(Vec::new());
            }
            state.selected_pair = pair;
            Ok(vec![signal(Signal::PairSelected { pair })])
        }
    }
}

pub fn advance_tick(state: &mut PlayerState, ctx: &WorldSnapshot) -> Vec<Effect> {
    if validate_context(state, ctx).is_err() || state.last_tick == Some(ctx.tick) {
        return Vec::new();
    }
    state.last_tick = Some(ctx.tick);
    Vec::new()
}
