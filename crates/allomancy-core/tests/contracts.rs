use allomancy_core::*;
use std::collections::BTreeMap;

fn player() -> PlayerState {
    let mut state = PlayerState::new(PlayerId::new("player-a"), WorldId::new("stable-world-key"));
    state.metals[0] = MetalState {
        unlocked: true,
        reserve_milli: 12_345,
        burn: BurnState::Burning,
    };
    state.metals[1] = MetalState {
        unlocked: true,
        reserve_milli: 8_000,
        burn: BurnState::Flared,
    };
    state
}

fn snapshot() -> WorldSnapshot {
    WorldSnapshot {
        tick: Tick(10),
        world_id: WorldId::new("stable-world-key"),
        actor: PlayerId::new("player-a"),
        position: Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        targets: vec![TargetSnapshot {
            id: "object-a".into(),
            position: Vec3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        }],
        authority: Authority::Server,
        input: InputContext {
            gameplay_allowed: true,
        },
    }
}

fn reject_unchanged(
    mut state: PlayerState,
    command: Command,
    ctx: &WorldSnapshot,
    error: RuleError,
) {
    let before = state.clone();
    let bytes = serde_json::to_vec(&state).unwrap();
    assert_eq!(apply_command(&mut state, command, ctx), Err(error));
    assert_eq!(state, before);
    assert_eq!(serde_json::to_vec(&state).unwrap(), bytes);
}

fn burn_signal(metal: Metal) -> Effect {
    Effect {
        stage: EffectStage::AcceptedInMemory,
        kind: EffectKind::Signal(Signal::BurnChanged {
            metal,
            burn: BurnState::Off,
        }),
    }
}

// Catches missing authority/identity/geometry/tick checks before a real mutation.
#[test]
fn invalid_snapshot_rejects_without_changing_state() {
    let mut cases = Vec::new();
    let mut ctx = snapshot();
    ctx.authority = Authority::Client;
    cases.push((ctx, RuleError::ServerAuthorityRequired));
    let mut ctx = snapshot();
    ctx.world_id = WorldId::new("another-world");
    cases.push((ctx, RuleError::WorldMismatch));
    let mut ctx = snapshot();
    ctx.actor = PlayerId::new("another-player");
    cases.push((ctx, RuleError::PlayerMismatch));
    for coordinate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut ctx = snapshot();
        ctx.position.x = coordinate;
        cases.push((ctx, RuleError::NonFinitePosition));
        let mut ctx = snapshot();
        ctx.targets[0].position.z = coordinate;
        cases.push((ctx, RuleError::NonFiniteTarget));
    }
    let mut ctx = snapshot();
    ctx.tick = Tick(8);
    cases.push((ctx, RuleError::StaleTick));
    for (ctx, error) in cases {
        let mut state = player();
        state.last_tick = Some(Tick(9));
        reject_unchanged(state, Command::Extinguish { metal: None }, &ctx, error);
    }
}

// Every command must pass server authority, including future reward/economy paths.
#[test]
fn clients_cannot_apply_any_command() {
    let mut ctx = snapshot();
    ctx.authority = Authority::Client;
    let commands = [
        Command::Drink {
            transaction: TransactionId::new("tx"),
            slot: 0,
        },
        Command::Activate { metal: Metal::Iron },
        Command::Flare {
            metal: Metal::Steel,
        },
        Command::Extinguish { metal: None },
        Command::Purchase {
            node: NodeId::new("node"),
            transaction: TransactionId::new("tx"),
        },
        Command::Claim {
            event: EventId::new("event"),
            transaction: TransactionId::new("tx"),
        },
        Command::SelectPair {
            pair: Pair::TinPewter,
        },
    ];
    for command in commands {
        reject_unchanged(player(), command, &ctx, RuleError::ServerAuthorityRequired);
    }
}

// Catches activation that bypasses lock/reserve checks or silently implements P4.
#[test]
fn activating_and_flaring_are_validated_then_explicitly_unimplemented() {
    for (command, operation) in [
        (
            Command::Activate { metal: Metal::Iron },
            Operation::Activate,
        ),
        (Command::Flare { metal: Metal::Iron }, Operation::Flare),
    ] {
        let mut locked = player();
        locked.metals[0].unlocked = false;
        reject_unchanged(
            locked,
            command.clone(),
            &snapshot(),
            RuleError::MetalLocked(Metal::Iron),
        );
        let mut empty = player();
        empty.metals[0].reserve_milli = 0;
        reject_unchanged(
            empty,
            command.clone(),
            &snapshot(),
            RuleError::EmptyReserve(Metal::Iron),
        );
        reject_unchanged(
            player(),
            command,
            &snapshot(),
            RuleError::NotImplemented(operation),
        );
    }
}

// Catches premature consumption, purchases, milestone rewards or fake success.
#[test]
fn economy_and_progression_are_explicitly_unimplemented() {
    for (command, operation) in [
        (
            Command::Drink {
                transaction: TransactionId::new("tx"),
                slot: 3,
            },
            Operation::Drink,
        ),
        (
            Command::Purchase {
                node: NodeId::new("node"),
                transaction: TransactionId::new("tx"),
            },
            Operation::Purchase,
        ),
        (
            Command::Claim {
                event: EventId::new("untrusted-reference"),
                transaction: TransactionId::new("tx"),
            },
            Operation::Claim,
        ),
    ] {
        reject_unchanged(
            player(),
            command,
            &snapshot(),
            RuleError::NotImplemented(operation),
        );
    }
}

// Catches ignored menu/chat context while preserving safety shutdown.
#[test]
fn captured_input_blocks_actions_but_allows_extinguish() {
    let mut ctx = snapshot();
    ctx.input.gameplay_allowed = false;
    reject_unchanged(
        player(),
        Command::SelectPair {
            pair: Pair::TinPewter,
        },
        &ctx,
        RuleError::InputCaptured,
    );
    reject_unchanged(
        player(),
        Command::Activate { metal: Metal::Iron },
        &ctx,
        RuleError::InputCaptured,
    );
    let mut state = player();
    assert_eq!(
        apply_command(&mut state, Command::Extinguish { metal: None }, &ctx),
        Ok(vec![burn_signal(Metal::Iron), burn_signal(Metal::Steel)])
    );
    assert_eq!(state.metals[0].burn, BurnState::Off);
    assert_eq!(state.metals[1].burn, BurnState::Off);
}

// Catches missing state mutation, cross-metal mutation or consumption on shutdown.
#[test]
fn extinguish_one_is_real_and_idempotent_even_when_locked_and_empty() {
    let mut state = player();
    state.metals[0].unlocked = false;
    state.metals[0].reserve_milli = 0;
    let mut expected = state.clone();
    expected.metals[0].burn = BurnState::Off;
    let command = Command::Extinguish {
        metal: Some(Metal::Iron),
    };
    assert_eq!(
        apply_command(&mut state, command.clone(), &snapshot()),
        Ok(vec![burn_signal(Metal::Iron)])
    );
    assert_eq!(state, expected);
    assert_eq!(apply_command(&mut state, command, &snapshot()), Ok(vec![]));
    assert_eq!(state, expected);
}

// Catches nondeterministic order and omission of any of the eight metal states.
#[test]
fn extinguish_all_has_fixed_order_for_all_metals() {
    let mut state = player();
    for metal in &mut state.metals {
        metal.burn = BurnState::Flared;
    }
    assert_eq!(
        apply_command(&mut state, Command::Extinguish { metal: None }, &snapshot()),
        Ok(vec![
            burn_signal(Metal::Iron),
            burn_signal(Metal::Steel),
            burn_signal(Metal::Tin),
            burn_signal(Metal::Pewter),
            burn_signal(Metal::Zinc),
            burn_signal(Metal::Brass),
            burn_signal(Metal::Copper),
            burn_signal(Metal::Bronze),
        ])
    );
    assert!(
        state
            .metals
            .iter()
            .all(|metal| metal.burn == BurnState::Off)
    );
    assert_eq!(state.metals[0].reserve_milli, 12_345);
}

// Equal nonempty transitions catch hidden clock/RNG/dependency-driven decisions.
#[test]
fn identical_inputs_produce_identical_nonempty_effects_and_state() {
    let mut left = player();
    let mut right = left.clone();
    let left_effects =
        apply_command(&mut left, Command::Extinguish { metal: None }, &snapshot()).unwrap();
    let right_effects =
        apply_command(&mut right, Command::Extinguish { metal: None }, &snapshot()).unwrap();
    assert_eq!(
        left_effects,
        vec![burn_signal(Metal::Iron), burn_signal(Metal::Steel)]
    );
    assert_eq!(left_effects, right_effects);
    assert_eq!(left, right);
    assert_eq!(left.metals[0].burn, BurnState::Off);
}

// Catches selection toggling combustion or silently failing on locked pairs.
#[test]
fn pair_selection_only_changes_selection_and_is_idempotent() {
    let mut state = player();
    let mut expected = state.clone();
    expected.selected_pair = Pair::CopperBronze;
    let command = Command::SelectPair {
        pair: Pair::CopperBronze,
    };
    assert_eq!(
        apply_command(&mut state, command.clone(), &snapshot()),
        Ok(vec![Effect {
            stage: EffectStage::AcceptedInMemory,
            kind: EffectKind::Signal(Signal::PairSelected {
                pair: Pair::CopperBronze
            })
        }])
    );
    assert_eq!(state, expected);
    assert_eq!(apply_command(&mut state, command, &snapshot()), Ok(vec![]));
}

// Catches fuel use/powers, replayed ticks and retrograde state changes in P1.
#[test]
fn tick_is_monotonic_and_does_not_run_future_systems() {
    let mut state = player();
    let mut expected = state.clone();
    expected.last_tick = Some(Tick(10));
    assert_eq!(advance_tick(&mut state, &snapshot()), vec![]);
    assert_eq!(state, expected);
    assert_eq!(advance_tick(&mut state, &snapshot()), vec![]);
    assert_eq!(state, expected);
    let mut ctx = snapshot();
    ctx.tick = Tick(9);
    assert_eq!(advance_tick(&mut state, &ctx), vec![]);
    assert_eq!(state, expected);
    ctx.tick = Tick(u64::MAX);
    expected.last_tick = Some(Tick(u64::MAX));
    assert_eq!(advance_tick(&mut state, &ctx), vec![]);
    assert_eq!(state, expected);
    assert_eq!(advance_tick(&mut state, &ctx), vec![]);
    assert_eq!(state, expected);
}

// Catches advance_tick bypassing the same trusted-boundary checks as commands.
#[test]
fn invalid_tick_snapshots_leave_state_intact() {
    for case in 0..5 {
        let mut ctx = snapshot();
        match case {
            0 => ctx.authority = Authority::Client,
            1 => ctx.world_id = WorldId::new("wrong"),
            2 => ctx.actor = PlayerId::new("wrong"),
            3 => ctx.position.y = f64::NAN,
            _ => ctx.targets[0].position.x = f64::INFINITY,
        }
        let mut state = player();
        let before = state.clone();
        assert_eq!(advance_tick(&mut state, &ctx), vec![]);
        assert_eq!(state, before);
    }
}

// Catches event construction from client contexts or nonfinite Host observations.
#[test]
fn verified_events_require_server_observation_and_capture_identity() {
    let id = EventId::new("observed-event");
    let kind = EventKind::ResearchObserved {
        research: ResearchId::new("research-a"),
    };
    let ctx = snapshot();
    let event = VerifiedEvent::from_server_observation(id.clone(), kind.clone(), &ctx).unwrap();
    assert_eq!(event.id(), &id);
    assert_eq!(event.tick(), Tick(10));
    assert_eq!(event.world_id(), &WorldId::new("stable-world-key"));
    assert_eq!(event.actor(), &PlayerId::new("player-a"));
    assert_eq!(event.kind(), &kind);
    let mut ctx = snapshot();
    ctx.authority = Authority::Client;
    assert_eq!(
        VerifiedEvent::from_server_observation(id.clone(), kind.clone(), &ctx),
        Err(RuleError::ServerAuthorityRequired)
    );
    let mut ctx = snapshot();
    ctx.position.z = f64::NAN;
    assert_eq!(
        VerifiedEvent::from_server_observation(id.clone(), kind.clone(), &ctx),
        Err(RuleError::NonFinitePosition)
    );
    let mut ctx = snapshot();
    ctx.targets[0].position.y = f64::INFINITY;
    assert_eq!(
        VerifiedEvent::from_server_observation(id, kind, &ctx),
        Err(RuleError::NonFiniteTarget)
    );
}

// Catches lossy float-based quantities or IDs that cannot preserve stable keys.
#[test]
fn state_and_opaque_identifiers_round_trip_without_losing_integer_reserves() {
    let mut state = player();
    state.metals[0].reserve_milli = u64::MAX;
    let world = WorldState {
        world_id: state.world_id.clone(),
        players: BTreeMap::from([(state.player_id.clone(), state)]),
    };
    let bytes = serde_json::to_vec(&world).unwrap();
    assert_eq!(serde_json::from_slice::<WorldState>(&bytes).unwrap(), world);
    let command = Command::Purchase {
        node: NodeId::new("node:opaque/α"),
        transaction: TransactionId::new("transaction:opaque"),
    };
    assert_eq!(
        serde_json::from_str::<Command>(&serde_json::to_string(&command).unwrap()).unwrap(),
        command
    );
    let ids = (ResearchId::new("research"), EventId::new("event"));
    assert_eq!(
        serde_json::from_str::<(ResearchId, EventId)>(&serde_json::to_string(&ids).unwrap())
            .unwrap(),
        ids
    );
}
