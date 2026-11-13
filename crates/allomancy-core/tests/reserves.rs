use allomancy_core::*;

fn rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [1; 8],
        base_capacity_milli: [1_000; 8],
        reduction_milli: 200,
        capacity_bonus_milli: 0,
    }
}

fn player() -> PlayerState {
    let mut state = PlayerState::new(PlayerId::new("player"), WorldId::new("stable-world"));
    for metal in &mut state.metals {
        metal.unlocked = true;
        metal.reserve_milli = 100;
    }
    state.metals[0].burn = BurnState::Burning;
    state.last_tick = Some(Tick(42));
    state
}

fn reject_consumption_unchanged(
    mut state: PlayerState,
    mut accounting: ReserveAccounting,
    rules: &ReserveRules,
    ticks: u64,
    error: ReserveError,
) {
    let before = (state.clone(), accounting.clone());
    let bytes = serde_json::to_vec(&before).unwrap();
    assert_eq!(
        consume_reserves(&mut state, &mut accounting, rules, ticks),
        Err(error)
    );
    assert_eq!((&state, &accounting), (&before.0, &before.1));
    assert_eq!(serde_json::to_vec(&(state, accounting)).unwrap(), bytes);
}

// Catches per-call truncation that gives free fuel when ticks are split.
#[test]
fn grouped_ticks_equal_separate_ticks_with_fractional_consumption() {
    let mut grouped = player();
    let mut separate = grouped.clone();
    let mut grouped_accounting = ReserveAccounting::default();
    let mut separate_accounting = grouped_accounting.clone();
    assert_eq!(
        consume_reserves(&mut grouped, &mut grouped_accounting, &rules(), 17),
        Ok(vec![])
    );
    for _ in 0..17 {
        consume_reserves(&mut separate, &mut separate_accounting, &rules(), 1).unwrap();
    }
    assert_eq!(grouped.metals[0].reserve_milli, 87);
    assert_eq!(grouped_accounting.remainder[0], 600);
    assert_eq!(
        (grouped, grouped_accounting),
        (separate, separate_accounting)
    );
}

// Catches lost fractions, consumption while off, and mutation on zero dt.
#[test]
fn fractions_wait_for_a_whole_milli_and_pause_without_reset() {
    let mut state = player();
    let mut accounting = ReserveAccounting::default();
    consume_reserves(&mut state, &mut accounting, &rules(), 1).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 100);
    assert_eq!(accounting.remainder[0], 800);
    state.metals[0].burn = BurnState::Off;
    let paused = (state.clone(), accounting.clone());
    consume_reserves(&mut state, &mut accounting, &rules(), 1_000).unwrap();
    assert_eq!((&state, &accounting), (&paused.0, &paused.1));
    state.metals[0].burn = BurnState::Burning;
    consume_reserves(&mut state, &mut accounting, &rules(), 0).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 100);
    assert_eq!(accounting.remainder[0], 800);
    consume_reserves(&mut state, &mut accounting, &rules(), 1).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 99);
    assert_eq!(accounting.remainder[0], 600);
    assert_eq!(state.last_tick, Some(Tick(42)));
}

// Catches a > instead of >= depletion check and stale debt on refilling.
#[test]
fn exact_depletion_and_overshoot_extinguish_and_clear_fraction() {
    for ticks in [10, 11, 100] {
        let mut state = player();
        state.metals[0].reserve_milli = 8;
        let mut accounting = ReserveAccounting::default();
        assert_eq!(
            consume_reserves(&mut state, &mut accounting, &rules(), ticks),
            Ok(vec![Metal::Iron])
        );
        assert_eq!(state.metals[0].reserve_milli, 0);
        assert_eq!(state.metals[0].burn, BurnState::Off);
        assert_eq!(accounting.remainder[0], 0);
        credit_reserve(&mut state, &accounting, &rules(), Metal::Iron, 1).unwrap();
        assert_eq!(state.metals[0].reserve_milli, 1);
        assert_eq!(state.metals[0].burn, BurnState::Off);
    }
}

// Catches applying x2 power to fuel, or applying reduction twice to flare.
#[test]
fn flare_consumes_three_times_normal_with_the_same_reduction() {
    let mut state = player();
    state.metals[1].burn = BurnState::Flared;
    let mut rules = rules();
    rules.consumption_milli_per_tick = [2; 8];
    rules.reduction_milli = 100;
    let mut accounting = ReserveAccounting::default();
    consume_reserves(&mut state, &mut accounting, &rules, 10).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 82);
    assert_eq!(state.metals[1].reserve_milli, 46);
    assert_eq!(state.metals[0].burn, BurnState::Burning);
    assert_eq!(state.metals[1].burn, BurnState::Flared);
}

// Catches omitted metals, pair-dependent consumption, or unstable result order.
#[test]
fn simultaneous_depletion_visits_all_eight_metals_in_fixed_order() {
    let mut state = player();
    state.selected_pair = Pair::CopperBronze;
    let mut rules = rules();
    rules.reduction_milli = 0;
    rules.consumption_milli_per_tick = [1, 2, 3, 4, 5, 6, 7, 8];
    for (i, metal) in state.metals.iter_mut().enumerate() {
        metal.reserve_milli = (i as u64 + 1) * 10;
        metal.burn = BurnState::Burning;
    }
    let mut accounting = ReserveAccounting::default();
    assert_eq!(
        consume_reserves(&mut state, &mut accounting, &rules, 10),
        Ok(vec![
            Metal::Iron,
            Metal::Steel,
            Metal::Tin,
            Metal::Pewter,
            Metal::Zinc,
            Metal::Brass,
            Metal::Copper,
            Metal::Bronze,
        ])
    );
    assert!(
        state
            .metals
            .iter()
            .all(|m| m.reserve_milli == 0 && m.burn == BurnState::Off)
    );
    assert_eq!(accounting.remainder, [0; 8]);
}

// Catches an off-by-one limit, wrong percentage denominator, or unbounded bonus.
#[test]
fn maximum_reduction_and_capacity_bonus_apply_to_real_consumption_and_credit() {
    let mut state = player();
    let mut rules = rules();
    rules.base_capacity_milli = [100; 8];
    rules.consumption_milli_per_tick = [10; 8];
    rules.capacity_bonus_milli = 500;
    let mut accounting = ReserveAccounting::default();
    assert_eq!(rules.capacity(Metal::Iron), Ok(150));
    credit_reserve(&mut state, &accounting, &rules, Metal::Iron, 50).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 150);
    consume_reserves(&mut state, &mut accounting, &rules, 10).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 70);
    rules.base_capacity_milli[0] = 101;
    assert_eq!(rules.capacity(Metal::Iron), Ok(151));
}

// Catches clamping invalid rules and validating only the first active metal.
#[test]
fn invalid_rules_reject_without_any_partial_consumption() {
    let mut cases = Vec::new();
    let mut r = rules();
    r.reduction_milli = 201;
    cases.push((r, ReserveError::ReductionLimitExceeded));
    let mut r = rules();
    r.capacity_bonus_milli = 501;
    cases.push((r, ReserveError::CapacityBonusLimitExceeded));
    let mut r = rules();
    r.consumption_milli_per_tick[7] = 0;
    cases.push((r, ReserveError::ZeroConsumption(Metal::Bronze)));
    let mut r = rules();
    r.base_capacity_milli[7] = 0;
    cases.push((r, ReserveError::ZeroCapacity(Metal::Bronze)));
    for (r, error) in cases {
        assert_eq!(r.validate(), Err(error));
        reject_consumption_unchanged(player(), ReserveAccounting::default(), &r, 10, error);
    }
}

// Catches late-state validation that leaves an earlier metal already consumed.
#[test]
fn invalid_state_or_deserialized_remainder_rejects_the_whole_batch() {
    let mut cases = Vec::new();
    let mut state = player();
    state.metals[7].reserve_milli = 1_001;
    cases.push((
        state,
        ReserveAccounting::default(),
        ReserveError::ReserveAboveCapacity(Metal::Bronze),
    ));
    let mut state = player();
    state.metals[7].unlocked = false;
    state.metals[7].burn = BurnState::Burning;
    cases.push((
        state,
        ReserveAccounting::default(),
        ReserveError::LockedBurn(Metal::Bronze),
    ));
    let mut state = player();
    state.metals[7].reserve_milli = 0;
    state.metals[7].burn = BurnState::Flared;
    cases.push((
        state,
        ReserveAccounting::default(),
        ReserveError::EmptyBurn(Metal::Bronze),
    ));
    let accounting =
        serde_json::from_str::<ReserveAccounting>(r#"{"remainder":[0,0,0,0,0,0,0,1000]}"#).unwrap();
    cases.push((
        player(),
        accounting,
        ReserveError::InvalidRemainder(Metal::Bronze),
    ));
    let mut state = player();
    state.metals[7].reserve_milli = 0;
    let mut accounting = ReserveAccounting::default();
    accounting.remainder[7] = 1;
    cases.push((
        state,
        accounting,
        ReserveError::InvalidRemainder(Metal::Bronze),
    ));
    for (state, accounting, error) in cases {
        assert_eq!(validate_reserves(&state, &accounting, &rules()), Err(error));
        reject_consumption_unchanged(state, accounting, &rules(), 10, error);
    }
}

// Catches unchecked numerator/quotient overflow or saturation after partial work.
#[test]
fn overflow_in_the_last_active_metal_leaves_all_reserves_and_remainders_intact() {
    for ticks in [2, u64::MAX] {
        let mut state = player();
        state.metals[7].burn = BurnState::Flared;
        let mut r = rules();
        r.consumption_milli_per_tick[7] = u64::MAX;
        let mut accounting = ReserveAccounting::default();
        accounting.remainder[0] = 123;
        reject_consumption_unchanged(
            state,
            accounting,
            &r,
            ticks,
            ReserveError::ArithmeticOverflow(Metal::Bronze),
        );
    }
}

// Catches capacity multiplication overflow even for a metal currently switched off.
#[test]
fn overflowing_capacity_rules_reject_before_mutation() {
    let mut r = rules();
    r.base_capacity_milli[7] = u64::MAX;
    r.capacity_bonus_milli = 500;
    assert_eq!(
        r.capacity(Metal::Iron),
        Err(ReserveError::ArithmeticOverflow(Metal::Bronze))
    );
    reject_consumption_unchanged(
        player(),
        ReserveAccounting::default(),
        &r,
        10,
        ReserveError::ArithmeticOverflow(Metal::Bronze),
    );
}

// Catches negative-to-unsigned casts, wrapping addition, and partial capacity fill.
#[test]
fn signed_credit_rejects_negative_overflow_and_capacity_excess_unchanged() {
    let mut state = player();
    let mut r = rules();
    let accounting = ReserveAccounting::default();
    for (amount, error) in [
        (i64::MIN, ReserveError::NegativeCredit),
        (-1, ReserveError::NegativeCredit),
        (901, ReserveError::CapacityExceeded(Metal::Iron)),
    ] {
        let before = state.clone();
        assert_eq!(
            credit_reserve(&mut state, &accounting, &r, Metal::Iron, amount),
            Err(error)
        );
        assert_eq!(state, before);
    }
    r.base_capacity_milli = [u64::MAX; 8];
    state.metals[0].reserve_milli = u64::MAX - 1;
    let before = state.clone();
    assert_eq!(
        credit_reserve(&mut state, &accounting, &r, Metal::Iron, 2),
        Err(ReserveError::ArithmeticOverflow(Metal::Iron))
    );
    assert_eq!(state, before);
}

// Catches credit bypassing validation of unrelated corrupt state/rules.
#[test]
fn credit_validates_the_entire_state_before_accepting_any_amount() {
    let mut state = player();
    state.metals[7].reserve_milli = 1_001;
    let before = state.clone();
    assert_eq!(
        credit_reserve(
            &mut state,
            &ReserveAccounting::default(),
            &rules(),
            Metal::Iron,
            1
        ),
        Err(ReserveError::ReserveAboveCapacity(Metal::Bronze))
    );
    assert_eq!(state, before);
    let mut r = rules();
    r.reduction_milli = 201;
    assert_eq!(
        credit_reserve(
            &mut state,
            &ReserveAccounting::default(),
            &r,
            Metal::Iron,
            0
        ),
        Err(ReserveError::ReductionLimitExceeded)
    );
    assert_eq!(state, before);
}

// Catches credit changing activation, unlocking a metal, or deleting fractional debt.
#[test]
fn credit_is_exact_and_zero_is_idempotent_without_touching_other_fields() {
    let mut state = player();
    state.metals[1].unlocked = false;
    let mut accounting = ReserveAccounting::default();
    accounting.remainder[1] = 999;
    let mut expected = state.clone();
    expected.metals[1].reserve_milli = 123;
    credit_reserve(&mut state, &accounting, &rules(), Metal::Steel, 23).unwrap();
    assert_eq!(state, expected);
    assert_eq!(accounting.remainder[1], 999);
    credit_reserve(&mut state, &accounting, &rules(), Metal::Steel, 0).unwrap();
    assert_eq!(state, expected);
}

// Catches not persisting remainder, and recreating it on reconnect to gift fuel.
#[test]
fn serialized_accounting_preserves_pending_consumption_across_reconnect() {
    let mut state = player();
    let mut accounting = ReserveAccounting::default();
    consume_reserves(&mut state, &mut accounting, &rules(), 1).unwrap();
    state.metals[0].burn = BurnState::Off;
    let bytes = serde_json::to_vec(&(state.clone(), accounting.clone(), rules())).unwrap();
    let (mut restored, mut restored_accounting, restored_rules): (
        PlayerState,
        ReserveAccounting,
        ReserveRules,
    ) = serde_json::from_slice(&bytes).unwrap();
    assert_eq!((&restored, &restored_accounting), (&state, &accounting));
    consume_reserves(
        &mut restored,
        &mut restored_accounting,
        &restored_rules,
        1_000,
    )
    .unwrap();
    restored.metals[0].burn = BurnState::Burning;
    consume_reserves(&mut restored, &mut restored_accounting, &restored_rules, 1).unwrap();
    assert_eq!(restored.metals[0].reserve_milli, 99);
    assert_eq!(restored_accounting.remainder[0], 600);
}

// Catches treating old remainder as a fraction of changed rules instead of milli/1000.
#[test]
fn reduction_changes_keep_the_existing_fraction_in_the_same_unit() {
    let mut state = player();
    let mut accounting = ReserveAccounting::default();
    consume_reserves(&mut state, &mut accounting, &rules(), 1).unwrap();
    let mut r = rules();
    r.reduction_milli = 0;
    consume_reserves(&mut state, &mut accounting, &r, 1).unwrap();
    assert_eq!(state.metals[0].reserve_milli, 99);
    assert_eq!(accounting.remainder[0], 800);
}

// Catches debt surviving death, and unrelated progression/identity fields being cleared.
#[test]
fn death_clears_all_fuel_burns_and_remainders_but_preserves_other_state() {
    let mut state = player();
    state.metals[7].unlocked = false;
    state.metals[7].burn = BurnState::Flared;
    state.selected_pair = Pair::TinPewter;
    let mut accounting = ReserveAccounting {
        remainder: [999; 8],
    };
    let mut expected = state.clone();
    for metal in &mut expected.metals {
        metal.reserve_milli = 0;
        metal.burn = BurnState::Off;
    }
    clear_reserves_on_death(&mut state, &mut accounting);
    assert_eq!(state, expected);
    assert_eq!(accounting, ReserveAccounting::default());
    clear_reserves_on_death(&mut state, &mut accounting);
    assert_eq!(state, expected);
    validate_reserves(&state, &accounting, &rules()).unwrap();
}
