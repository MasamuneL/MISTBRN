use allomancy_core::*;

fn state() -> PlayerState {
    let mut state = PlayerState::new(PlayerId::new("p"), WorldId::new("local"));
    for metal in Metal::ALL {
        state.metals[metal.index()].unlocked = true;
        state.metals[metal.index()].reserve_milli = 1_000;
    }
    state
}

fn rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [10; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    }
}

const REQUESTS: [ControlRequest; 9] = [
    ControlRequest::CyclePair,
    ControlRequest::Toggle(Side::First),
    ControlRequest::Toggle(Side::Second),
    ControlRequest::ToggleFlare,
    ControlRequest::Off,
    ControlRequest::SelectPair(Pair::IronSteel),
    ControlRequest::SelectPair(Pair::TinPewter),
    ControlRequest::SelectPair(Pair::ZincBrass),
    ControlRequest::SelectPair(Pair::CopperBronze),
];

#[test]
fn control_round_trip() {
    for request in REQUESTS {
        assert_eq!(decode_control(&encode_control(request)), Ok(request));
    }
}

#[test]
fn control_rejects_every_other_length() {
    let good = encode_control(ControlRequest::Off);
    for len in 0..=8 {
        if len == CONTROL_LEN {
            continue;
        }
        let bytes: Vec<u8> = good.iter().copied().cycle().take(len).collect();
        assert_eq!(decode_control(&bytes), Err(ControlError::Length(len)));
    }
}

// Every byte string of the right length decodes only when it is one of the nine requests.
#[test]
fn control_rejects_every_unknown_byte() {
    let valid: Vec<[u8; 3]> = REQUESTS.iter().map(|r| encode_control(*r)).collect();
    for version in [0u8, 1, 2, 255] {
        for op in 0..=255u8 {
            for arg in [0u8, 1, 2, 3, 4, 255] {
                let bytes = [version, op, arg];
                let decoded = decode_control(&bytes);
                // The notebook's ops (P5) take any non-zero code and index.
                let notebook = ((6..=7).contains(&op) && arg >> 2 != 0 && arg & 3 != 0)
                    // The book's Conclude (P7b) takes any non-zero research code.
                    || (op == 8 && arg != 0);
                if valid.contains(&bytes) || (version == CONTROL_VERSION && notebook) {
                    assert!(decoded.is_ok(), "{bytes:?}");
                    continue;
                }
                let expected = if version != CONTROL_VERSION {
                    ControlError::Version(version)
                } else if (1..=8).contains(&op) {
                    ControlError::Arg { op, arg }
                } else {
                    ControlError::Op(op)
                };
                assert_eq!(decoded, Err(expected), "{bytes:?}");
            }
        }
    }
}

#[test]
fn cycle_goes_through_all_four_pairs() {
    let mut state = state();
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(state.selected_pair);
        let Some(Command::SelectPair { pair }) = resolve_control(&state, ControlRequest::CyclePair)
        else {
            panic!("cycle must select a pair");
        };
        state.selected_pair = pair;
    }
    assert_eq!(
        seen,
        [
            Pair::IronSteel,
            Pair::TinPewter,
            Pair::ZincBrass,
            Pair::CopperBronze
        ]
    );
    assert_eq!(state.selected_pair, Pair::IronSteel);
}

#[test]
fn toggle_lights_then_puts_out_the_side_of_the_selected_pair() {
    let mut state = state();
    assert_eq!(
        resolve_control(&state, ControlRequest::Toggle(Side::First)),
        Some(Command::Activate { metal: Metal::Iron })
    );
    assert_eq!(
        resolve_control(&state, ControlRequest::Toggle(Side::Second)),
        Some(Command::Activate {
            metal: Metal::Steel
        })
    );
    state.metals[Metal::Iron.index()].burn = BurnState::Flared;
    assert_eq!(
        resolve_control(&state, ControlRequest::Toggle(Side::First)),
        Some(Command::Extinguish {
            metal: Some(Metal::Iron)
        })
    );
    // Steel off while iron burns: X lights steel (the core puts iron out).
    assert_eq!(
        resolve_control(&state, ControlRequest::Toggle(Side::Second)),
        Some(Command::Activate {
            metal: Metal::Steel
        })
    );
    state.selected_pair = Pair::CopperBronze;
    assert_eq!(
        resolve_control(&state, ControlRequest::Toggle(Side::Second)),
        Some(Command::Activate {
            metal: Metal::Bronze
        })
    );
}

#[test]
fn flare_toggles_the_burning_metal_of_the_selected_pair() {
    let mut state = state();
    assert_eq!(resolve_control(&state, ControlRequest::ToggleFlare), None);
    state.metals[Metal::Steel.index()].burn = BurnState::Burning;
    assert_eq!(
        resolve_control(&state, ControlRequest::ToggleFlare),
        Some(Command::Flare {
            metal: Metal::Steel
        })
    );
    state.metals[Metal::Steel.index()].burn = BurnState::Flared;
    assert_eq!(
        resolve_control(&state, ControlRequest::ToggleFlare),
        Some(Command::Activate {
            metal: Metal::Steel
        })
    );
    // A metal of another pair is not the selected pair's.
    state.selected_pair = Pair::TinPewter;
    assert_eq!(resolve_control(&state, ControlRequest::ToggleFlare), None);
    assert_eq!(
        resolve_control(&state, ControlRequest::Off),
        Some(Command::Extinguish { metal: None })
    );
}

// The resolved commands still pass the core's rules: input captured refuses all but off.
#[test]
fn resolved_commands_go_through_apply_command() {
    let mut state = state();
    let mut ctx = WorldSnapshot {
        tick: Tick(1),
        world_id: WorldId::new("local"),
        actor: PlayerId::new("p"),
        position: Vec3::default(),
        targets: Vec::new(),
        authority: Authority::Server,
        input: InputContext {
            gameplay_allowed: false,
        },
    };
    let light = resolve_control(&state, ControlRequest::Toggle(Side::First)).unwrap();
    assert_eq!(
        apply_command(&mut state, light.clone(), &ctx),
        Err(RuleError::InputCaptured)
    );
    ctx.input.gameplay_allowed = true;
    assert!(apply_command(&mut state, light, &ctx).is_ok());
    assert_eq!(state.metals[Metal::Iron.index()].burn, BurnState::Burning);
    ctx.input.gameplay_allowed = false;
    let off = resolve_control(&state, ControlRequest::Off).unwrap();
    assert!(apply_command(&mut state, off, &ctx).is_ok());
    assert_eq!(state.metals[Metal::Iron.index()].burn, BurnState::Off);
}

#[test]
fn budget_allows_a_burst_then_one_every_two_ticks() {
    let mut budget = RequestBudget::new(100);
    for _ in 0..RequestBudget::CAPACITY {
        assert!(budget.allow(100));
    }
    assert!(!budget.allow(100));
    assert!(!budget.allow(101));
    assert!(budget.allow(102));
    assert!(!budget.allow(102));
    // A long pause refills to the cap, not past it.
    for _ in 0..RequestBudget::CAPACITY {
        assert!(budget.allow(10_000));
    }
    assert!(!budget.allow(10_000));
    // A tick from before (a new world) neither panics nor refills.
    assert!(!budget.allow(5));
    assert!(budget.allow(7));
}

#[test]
fn hud_round_trip_and_state() {
    let mut state = state();
    state.selected_pair = Pair::ZincBrass;
    state.metals[Metal::Iron.index()].burn = BurnState::Flared;
    state.metals[Metal::Steel.index()].unlocked = false;
    let hud = HudSnapshot::from_state(&state, &rules());
    assert_eq!(hud.pair, Pair::ZincBrass);
    assert_eq!(hud.metals[0].burn, BurnState::Flared);
    assert_eq!(hud.metals[0].reserve_milli, 1_000);
    assert_eq!(hud.metals[0].capacity_milli, 100_000);
    assert!(!hud.metals[1].unlocked);
    let bytes = encode_hud(&hud);
    assert_eq!(bytes.len(), HUD_LEN);
    assert_eq!(decode_hud(&bytes), Ok(hud));
}

#[test]
fn hud_rejects_malformed_bytes() {
    let good = encode_hud(&HudSnapshot::from_state(&state(), &rules()));
    for len in 0..HUD_LEN {
        assert_eq!(decode_hud(&good[..len]), Err(HudError::Length(len)));
    }
    let mut long = good.to_vec();
    long.push(0);
    assert_eq!(decode_hud(&long), Err(HudError::Length(HUD_LEN + 1)));
    let poke = |at: usize, value: u8| {
        let mut bytes = good;
        bytes[at] = value;
        decode_hud(&bytes)
    };
    assert_eq!(poke(0, 2), Err(HudError::Version(2)));
    assert_eq!(poke(1, 4), Err(HudError::Pair(4)));
    assert_eq!(poke(2, 2), Err(HudError::Flag(2)));
    assert_eq!(poke(3, 3), Err(HudError::Burn(3)));
    // Reserve above capacity.
    assert_eq!(poke(2 + 5, 0xFF), Err(HudError::Reserve));
}

#[test]
fn select_pair_names_that_pair_on_the_wire_and_in_the_command() {
    let state = state();
    for (code, pair) in PAIRS.into_iter().enumerate() {
        let bytes = encode_control(ControlRequest::SelectPair(pair));
        assert_eq!(bytes, [CONTROL_VERSION, 5, code as u8]);
        assert_eq!(
            resolve_control(&state, ControlRequest::SelectPair(pair)),
            Some(Command::SelectPair { pair })
        );
    }
    assert_eq!(
        decode_control(&[CONTROL_VERSION, 5, 4]),
        Err(ControlError::Arg { op: 5, arg: 4 })
    );
}

#[test]
fn selecting_a_locked_pair_is_allowed_and_lights_nothing() {
    let mut state = PlayerState::new(PlayerId::new("p"), WorldId::new("local"));
    let command = resolve_control(&state, ControlRequest::SelectPair(Pair::ZincBrass)).unwrap();
    let snapshot = WorldSnapshot {
        tick: Tick(1),
        world_id: WorldId::new("local"),
        actor: PlayerId::new("p"),
        position: Vec3::default(),
        targets: vec![],
        authority: Authority::Server,
        input: InputContext {
            gameplay_allowed: true,
        },
    };
    let effects = apply_command(&mut state, command, &snapshot).unwrap();
    assert_eq!(state.selected_pair, Pair::ZincBrass);
    assert!(state.metals.iter().all(|m| m.burn == BurnState::Off));
    assert_eq!(effects.len(), 1);
}

// ---- P5: notebook requests and the progress snapshot ----

#[test]
fn notebook_requests_round_trip_and_reject_code_or_index_zero() {
    for request in [
        ControlRequest::Purchase { code: 1, rank: 1 },
        ControlRequest::Purchase { code: 17, rank: 2 },
        ControlRequest::Purchase { code: 63, rank: 3 },
        ControlRequest::RevealHint {
            research: 1,
            index: 3,
        },
    ] {
        assert_eq!(decode_control(&encode_control(request)), Ok(request));
    }
    for (op, arg) in [(6, 0), (6, 1 << 2), (6, 0b11), (7, 0), (7, 1 << 2)] {
        assert_eq!(
            decode_control(&[1, op, arg]),
            Err(ControlError::Arg { op, arg }),
            "op {op} arg {arg}"
        );
    }
    assert_eq!(decode_control(&[1, 9, 0]), Err(ControlError::Op(9)));
}

#[test]
fn the_conclude_request_round_trips_and_rejects_research_zero() {
    for research in [1, 2, 255] {
        let request = ControlRequest::Conclude { research };
        assert_eq!(decode_control(&encode_control(request)), Ok(request));
    }
    assert_eq!(
        decode_control(&[1, 8, 0]),
        Err(ControlError::Arg { op: 8, arg: 0 })
    );
    assert_eq!(
        resolve_control(
            &PlayerState::new(PlayerId::new("p"), WorldId::new("local")),
            ControlRequest::Conclude { research: 1 }
        ),
        None
    );
}

#[test]
fn notebook_requests_are_not_power_commands() {
    let state = PlayerState::new(PlayerId::new("p"), WorldId::new("local"));
    assert_eq!(
        resolve_control(&state, ControlRequest::Purchase { code: 1, rank: 1 }),
        None
    );
}

#[test]
fn the_progress_snapshot_round_trips_and_refuses_out_of_range_values() {
    let mut saved = SavedPlayer::new(
        PlayerId::new("0123456789abcdef0123456789abcdef"),
        WorldId::new("local"),
        ReserveRules {
            consumption_milli_per_tick: [10; 8],
            base_capacity_milli: [100_000; 8],
            reduction_milli: 0,
            capacity_bonus_milli: 0,
        },
    )
    .unwrap();
    let (vial, _) = prepare_vial(&saved, Metal::Iron, 1_000).unwrap();
    commit_credit(&mut saved, vial).unwrap();
    let hint = prepare_hint(&saved, RESEARCH_ANCHOR, 1).unwrap();
    commit_credit(&mut saved, hint).unwrap();
    let bytes = encode_progress_snapshot(saved.progress());
    assert_eq!(bytes.len(), PROGRESS_LEN);
    let shown = decode_progress_snapshot(&bytes).unwrap().progress;
    assert_eq!(shown.research(RESEARCH_ANCHOR).hints, 1);
    assert_eq!(shown.pair_xp(Pair::IronSteel), 0);

    let mut rank = bytes;
    rank[17] = 4; // reach has 3 ranks
    assert_eq!(
        decode_progress_snapshot(&rank),
        Err(ProgressSnapshotError::Rank)
    );
    let mut evidence = bytes;
    evidence[17 + CATALOG.nodes.len()] = 8;
    assert_eq!(
        decode_progress_snapshot(&evidence),
        Err(ProgressSnapshotError::Research)
    );
    assert_eq!(
        decode_progress_snapshot(&bytes[1..]),
        Err(ProgressSnapshotError::Length(PROGRESS_LEN - 1))
    );
}

#[test]
fn node_status_follows_pair_requirements_and_points() {
    let progress = decode_progress_snapshot(&encode_progress_snapshot(&Progress::default()))
        .unwrap()
        .progress;
    let reach = node_def(NODE_REACH, &CATALOG).unwrap();
    let accel = node_def(NODE_ACCEL, &CATALOG).unwrap();
    assert_eq!(
        node_status(&progress, reach, |_| false, &CATALOG),
        NodeStatus::Locked
    );
    assert_eq!(
        node_status(&progress, reach, |_| true, &CATALOG),
        NodeStatus::TooExpensive
    );
    assert_eq!(
        node_status(&progress, accel, |_| true, &CATALOG),
        NodeStatus::Locked
    );
}
