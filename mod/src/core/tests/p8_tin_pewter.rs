//! P8, tin and pewter: lighting limits, R on a pair, the dodge's spend and the catalog.
use allomancy_core::*;

fn state() -> PlayerState {
    let mut state = PlayerState::new(PlayerId::new("p"), WorldId::new("w"));
    for metal in Metal::ALL {
        state.metals[metal.index()] = MetalState {
            unlocked: true,
            reserve_milli: 10_000,
            burn: BurnState::Off,
        };
    }
    state.selected_pair = Pair::TinPewter;
    state
}

fn ctx() -> WorldSnapshot {
    WorldSnapshot {
        tick: Tick(1),
        world_id: WorldId::new("w"),
        actor: PlayerId::new("p"),
        position: Vec3::default(),
        targets: Vec::new(),
        authority: Authority::Server,
        input: InputContext {
            gameplay_allowed: true,
        },
    }
}

fn lit(state: &PlayerState) -> Vec<Metal> {
    Metal::ALL
        .into_iter()
        .filter(|m| state.metals[m.index()].burn != BurnState::Off)
        .collect()
}

// Catches a limit that counts wrongly, ignores the nodes, or lets a refused light change state.
#[test]
fn metals_lit_at_once_keep_the_limit() {
    let mut s = state();
    for metal in [Metal::Tin, Metal::Pewter] {
        apply_command(&mut s, Command::Activate { metal }, &ctx()).unwrap();
    }
    assert_eq!(lit(&s), [Metal::Tin, Metal::Pewter]);
    let before = s.clone();
    assert_eq!(
        apply_command(&mut s, Command::Activate { metal: Metal::Iron }, &ctx()),
        Err(RuleError::TooManyBurning)
    );
    assert_eq!(s, before);
    // Re-lighting or flaring a lit metal is never past the limit.
    apply_command(&mut s, Command::Flare { metal: Metal::Tin }, &ctx()).unwrap();
    apply_command_with(&mut s, Command::Activate { metal: Metal::Iron }, &ctx(), 3).unwrap();
    assert_eq!(lit(&s), [Metal::Iron, Metal::Tin, Metal::Pewter]);
    // Steel puts iron out: still three lit, within 3.
    apply_command_with(
        &mut s,
        Command::Activate {
            metal: Metal::Steel,
        },
        &ctx(),
        3,
    )
    .unwrap();
    assert_eq!(lit(&s), [Metal::Steel, Metal::Tin, Metal::Pewter]);
    // Copper has no power yet.
    assert_eq!(
        apply_command_with(
            &mut s,
            Command::Activate {
                metal: Metal::Copper
            },
            &ctx(),
            4
        ),
        Err(RuleError::NotImplemented(Operation::Activate))
    );
}

// Catches R that flares one of two lit metals, or acts with none lit.
#[test]
fn r_flares_both_lit_metals_of_the_pair() {
    let mut s = state();
    assert_eq!(resolve_control(&s, ControlRequest::ToggleFlare), None);
    apply_command(
        &mut s,
        Command::Activate {
            metal: Metal::Pewter,
        },
        &ctx(),
    )
    .unwrap();
    assert_eq!(
        resolve_control(&s, ControlRequest::ToggleFlare),
        Some(Command::Flare {
            metal: Metal::Pewter
        })
    );
    apply_command(&mut s, Command::Activate { metal: Metal::Tin }, &ctx()).unwrap();
    let flare = resolve_control(&s, ControlRequest::ToggleFlare).unwrap();
    assert_eq!(
        flare,
        Command::FlarePair {
            pair: Pair::TinPewter,
            flared: true
        }
    );
    assert_eq!(apply_command(&mut s, flare, &ctx()).unwrap().len(), 2);
    assert!(
        [Metal::Tin, Metal::Pewter]
            .iter()
            .all(|m| s.metals[m.index()].burn == BurnState::Flared)
    );
    let back = resolve_control(&s, ControlRequest::ToggleFlare).unwrap();
    apply_command(&mut s, back, &ctx()).unwrap();
    assert!(
        [Metal::Tin, Metal::Pewter]
            .iter()
            .all(|m| s.metals[m.index()].burn == BurnState::Burning)
    );
    // One flared, one burning: R flares both.
    apply_command(&mut s, Command::Flare { metal: Metal::Tin }, &ctx()).unwrap();
    assert_eq!(
        resolve_control(&s, ControlRequest::ToggleFlare),
        Some(Command::FlarePair {
            pair: Pair::TinPewter,
            flared: true
        })
    );
    let mut none = state();
    assert_eq!(
        apply_command(
            &mut none,
            Command::FlarePair {
                pair: Pair::TinPewter,
                flared: true
            },
            &ctx()
        ),
        Err(RuleError::NothingBurning)
    );
}

// Catches a partial spend, a negative reserve, or a metal left lit with nothing in it.
#[test]
fn a_spend_is_all_or_nothing() {
    let rules = ReserveRules {
        consumption_milli_per_tick: [10; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    };
    let mut s = state();
    let mut accounting = ReserveAccounting::default();
    s.metals[Metal::Pewter.index()].burn = BurnState::Burning;
    spend_reserve(&mut s, &mut accounting, &rules, Metal::Pewter, 500).unwrap();
    assert_eq!(s.metals[Metal::Pewter.index()].reserve_milli, 9_500);
    let before = s.clone();
    assert_eq!(
        spend_reserve(&mut s, &mut accounting, &rules, Metal::Pewter, 9_501),
        Err(ReserveError::NotEnough(Metal::Pewter))
    );
    assert_eq!(s, before);
    spend_reserve(&mut s, &mut accounting, &rules, Metal::Pewter, 9_500).unwrap();
    assert_eq!(s.metals[Metal::Pewter.index()].burn, BurnState::Off);
}

// Catches nodes the map or the server would not find, and a limit the nodes cannot raise.
#[test]
fn the_tin_pewter_catalog_holds() {
    assert_eq!(validate_catalog(&CATALOG), Ok(()));
    for code in [
        NODE_SIMULTANEOUS,
        NODE_TRANSITION,
        NODE_STEEL_BODY,
        NODE_TIN_RANGE,
        NODE_TIN_CLARITY,
        NODE_HEARING,
        NODE_HEARING_RANGE,
        NODE_HEARING_MEMORY,
        NODE_PEWTER_STRENGTH,
        NODE_PEWTER_ENDURANCE,
        NODE_PEWTER_JUMP,
        NODE_JUMP_HEIGHT,
        NODE_JUMP_CUSHION,
        NODE_REFLEXES,
    ] {
        assert!(node_def(code, &CATALOG).is_some(), "{code}");
    }
    assert_eq!(
        node_def(NODE_STEEL_BODY, &CATALOG).unwrap().pool,
        Pool::Bridge(Pair::IronSteel, Pair::TinPewter)
    );
    assert!(practice_def(PRACTICE_PEWTER_FIGHT, &CATALOG).is_some());
    assert_eq!(
        research_def(RESEARCH_THRESHOLD, &CATALOG).unwrap().pair,
        Pair::TinPewter
    );
    assert_eq!(Progress::default().burn_limit(&CATALOG), BASE_BURN_LIMIT);
}
