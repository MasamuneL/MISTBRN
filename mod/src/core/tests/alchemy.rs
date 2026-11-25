//! P3c: the basic-metals vial credits every unlocked metal up to its capacity, once.
use allomancy_core::*;

fn world() -> WorldId {
    WorldId::new("local")
}

fn player() -> PlayerId {
    PlayerId::new("0123456789abcdef0123456789abcdef")
}

fn fresh() -> SavedPlayer {
    let rules = ReserveRules {
        consumption_milli_per_tick: [10; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    };
    SavedPlayer::new(player(), world(), rules).unwrap()
}

/// Iron, steel and tin unlocked; iron nearly full, steel full, tin empty.
fn mixed() -> SavedPlayer {
    let mut saved = fresh();
    update_player(&mut saved, |state, _| {
        for metal in [Metal::Iron, Metal::Steel, Metal::Tin] {
            state.metals[metal.index()].unlocked = true;
        }
        state.metals[Metal::Iron.index()].reserve_milli = 90_000;
        state.metals[Metal::Steel.index()].reserve_milli = 100_000;
        Ok(())
    })
    .unwrap();
    saved
}

fn reserves(saved: &SavedPlayer) -> [u64; 8] {
    saved.state().metals.map(|m| m.reserve_milli)
}

const AFTER: [u64; 8] = [100_000, 100_000, 25_000, 0, 0, 0, 0, 0];

#[test]
fn credits_only_unlocked_metals_up_to_capacity() {
    let saved = mixed();
    let (prepared, credited) = prepare_basic_vial(&saved, 25_000).unwrap();
    assert_eq!(credited, [10_000, 0, 25_000, 0, 0, 0, 0, 0]);
    assert_eq!(reserves(prepared.candidate()), AFTER);
    // Nothing unlocks or changes selection; the save itself is untouched until commit.
    assert_eq!(
        prepared.candidate().state().metals.map(|m| m.unlocked),
        saved.state().metals.map(|m| m.unlocked)
    );
    assert_eq!(
        prepared.candidate().state().selected_pair,
        saved.state().selected_pair
    );
    assert_eq!(reserves(&saved), [90_000, 100_000, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn nothing_to_credit_is_refused_and_prepares_nothing() {
    assert_eq!(
        prepare_basic_vial(&fresh(), 25_000),
        Err(SaveError::NothingToCredit)
    );
    let mut full = mixed();
    update_player(&mut full, |state, _| {
        state.metals[Metal::Iron.index()].reserve_milli = 100_000;
        state.metals[Metal::Tin.index()].reserve_milli = 100_000;
        Ok(())
    })
    .unwrap();
    let before = full.clone();
    assert_eq!(
        prepare_basic_vial(&full, 25_000),
        Err(SaveError::NothingToCredit)
    );
    assert_eq!(full, before);
    assert_eq!(
        prepare_basic_vial(&mixed(), -1),
        Err(SaveError::InvalidReserves(ReserveError::NegativeCredit))
    );
}

#[test]
fn commits_exactly_once_and_stale_preparations_are_refused() {
    let mut saved = mixed();
    let (first, _) = prepare_basic_vial(&saved, 25_000).unwrap();
    let (replay, _) = prepare_basic_vial(&saved, 25_000).unwrap();
    assert_eq!(commit_credit(&mut saved, first), Ok(TransactionSeq(1)));
    let after = saved.clone();
    assert_eq!(commit_credit(&mut saved, replay), Err(SaveError::Duplicate));
    assert_eq!(saved, after);
    assert_eq!(reserves(&saved), AFTER);

    let (stale, _) = prepare_basic_vial(&saved, 25_000).unwrap();
    on_death(&mut saved).unwrap();
    let after = saved.clone();
    assert_eq!(
        commit_credit(&mut saved, stale),
        Err(SaveError::StalePreparation)
    );
    assert_eq!(saved, after);
}

// Cut before the host commit keeps the old file; cut after has the credit exactly once.
#[test]
fn cut_before_or_after_commit_yields_old_or_new_never_both() {
    let saved = mixed();
    let old_bytes = encode_save(&saved).unwrap();
    let (prepared, _) = prepare_basic_vial(&saved, 25_000).unwrap();
    let new_bytes = encode_save(prepared.candidate()).unwrap();

    let old = decode_save(&old_bytes, &world(), &player()).unwrap();
    assert_eq!(reserves(&old), [90_000, 100_000, 0, 0, 0, 0, 0, 0]);
    assert_eq!(old.last_transaction(), 0);

    let mut new = decode_save(&new_bytes, &world(), &player()).unwrap();
    assert_eq!(reserves(&new), AFTER);
    assert_eq!(new.last_transaction(), 1);
    assert_eq!(commit_credit(&mut new, prepared), Err(SaveError::Duplicate));
    assert_eq!(reserves(&new), AFTER);
}

#[test]
fn alembic_brews_in_two_stages_and_nothing_else() {
    assert_eq!(brew_result(SALT, WATER_VIAL), Some(BRINE_VIAL));
    assert_eq!(
        brew_result("mistbrn:iron_flakes", BRINE_VIAL),
        Some("mistbrn:vial_iron")
    );
    assert_eq!(
        brew_result("mistbrn:bronze_flakes", BRINE_VIAL),
        Some("mistbrn:vial_bronze")
    );
    // Wrong stage, finished vials and unknown ingredients brew nothing.
    assert_eq!(brew_result(SALT, BRINE_VIAL), None);
    assert_eq!(brew_result("mistbrn:iron_flakes", WATER_VIAL), None);
    assert_eq!(
        brew_result("mistbrn:iron_flakes", "mistbrn:vial_iron"),
        None
    );
    assert_eq!(brew_result("minecraft:sugar", WATER_VIAL), None);
    assert_eq!(brew_result(SALT, EMPTY_VIAL), None);
    // Every filled vial is brewable from its flakes.
    for (vial, _) in FILLED_VIALS {
        let flakes = vial.replace("vial_", "") + "_flakes";
        assert_eq!(brew_result(&flakes, BRINE_VIAL), Some(vial), "{flakes}");
        assert!(is_alembic_ingredient(&flakes));
        assert!(is_alembic_vial(vial));
    }
    assert!(is_alembic_ingredient(SALT));
    assert!(!is_alembic_ingredient(WATER_VIAL));
    assert!(is_alembic_vial(WATER_VIAL) && is_alembic_vial(BRINE_VIAL));
    assert!(!is_alembic_vial(EMPTY_VIAL) && !is_alembic_vial(BASIC_VIAL));
}

#[test]
fn beach_salt_is_about_one_in_twenty_and_varies_with_time() {
    let hits = (0..20_000)
        .filter(|&i| beach_salt("minecraft:overworld", [i % 100, 63, i / 100], 0))
        .count();
    assert!((800..1_200).contains(&hits), "{hits}");
    // The same position does not always give salt.
    let same = (0..2_000u64)
        .filter(|&t| beach_salt("minecraft:overworld", [5, 63, 5], t))
        .count();
    assert!((50..150).contains(&same), "{same}");
}
