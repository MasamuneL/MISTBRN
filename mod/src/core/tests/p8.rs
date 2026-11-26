//! P8 (map branch): schema 4's newer practice windows, the flare cost and dosing nodes.
use allomancy_core::*;

fn rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [10; 8],
        base_capacity_milli: [100_000; 8],
        reduction_milli: 0,
        capacity_bonus_milli: 0,
    }
}

fn world() -> WorldId {
    WorldId::new("local")
}

fn player() -> PlayerId {
    PlayerId::new("0123456789abcdef0123456789abcdef")
}

fn decode(bytes: &[u8]) -> Result<SavedPlayer, SaveError> {
    decode_save(bytes, &world(), &player())
}

fn commit(
    saved: &mut SavedPlayer,
    prepare: impl FnOnce(&SavedPlayer) -> Result<PreparedCredit, SaveError>,
) {
    let prepared = prepare(saved).unwrap();
    commit_credit(saved, prepared).unwrap();
}

/// Iron and steel unlocked through the tutorial vial, and the dragon's 3 knowledge points.
fn knowing() -> SavedPlayer {
    let mut saved = SavedPlayer::new(player(), world(), rules()).unwrap();
    commit(&mut saved, |s| {
        prepare_vial(s, Metal::Iron, 25_000).map(|(p, _)| p)
    });
    commit(&mut saved, |s| {
        prepare_claim(s, MILESTONE_DRAGON).map(|(p, _)| p)
    });
    saved
}

fn reseal(mut bytes: Vec<u8>) -> Vec<u8> {
    let body = bytes.len() - 4;
    bytes.truncate(body);
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in &bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xEDB8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    bytes.extend_from_slice(&(!crc).to_le_bytes());
    bytes
}

const EXTRA: usize = PRACTICE_CATEGORIES - LEGACY_WINDOWS;

/// Where schema 4's `n_extra` byte sits: before its windows and the milestones byte.
fn extra_at(bytes: &[u8]) -> usize {
    bytes.len() - 4 - 1 - EXTRA * 9 - 1
}

// ---- schema 4 ----

#[test]
fn schema_4_stores_every_practice_window() {
    let saved = knowing();
    let bytes = encode_save(&saved).unwrap();
    assert_eq!(save_schema(&bytes), Some(SAVE_SCHEMA));
    assert_eq!(SAVE_SCHEMA, 4);
    assert_eq!(usize::from(bytes[extra_at(&bytes)]), EXTRA);
    assert_eq!(decode(&bytes).unwrap(), saved);
}

// Catches a reader that accepts more windows than there are categories.
#[test]
fn schema_4_refuses_too_many_newer_windows() {
    let mut bytes = encode_save(&knowing()).unwrap();
    let at = extra_at(&bytes);
    bytes[at] = EXTRA as u8 + 1;
    assert_eq!(decode(&reseal(bytes)), Err(SaveError::InvalidEnum));
}

// Fewer stored windows are unopened ones: a later schema 4 writer may store more categories.
#[test]
fn schema_4_reads_fewer_newer_windows_as_unopened() {
    let saved = knowing();
    let mut bytes = encode_save(&saved).unwrap();
    let at = extra_at(&bytes);
    bytes[at] = 0;
    bytes.drain(at + 1..at + 1 + EXTRA * 9);
    let len = (bytes.len() - 10 - 4) as u32;
    bytes[6..10].copy_from_slice(&len.to_le_bytes());
    assert_eq!(decode(&reseal(bytes)).unwrap(), saved);
}

// Catches a window opened for a category no catalog entry defines.
#[test]
fn a_window_of_an_undefined_category_is_refused() {
    let mut bytes = encode_save(&knowing()).unwrap();
    let first = extra_at(&bytes) + 1;
    bytes[first..first + 8].copy_from_slice(&5u64.to_le_bytes());
    bytes[first + 8] = 1;
    assert_eq!(
        decode(&reseal(bytes)),
        Err(SaveError::Progress(ProgressError::WindowOutOfRange))
    );
}

// Every truncation of a schema 4 save is an error, never a shorter save.
#[test]
fn every_truncation_of_schema_4_is_rejected() {
    let bytes = encode_save(&knowing()).unwrap();
    for len in 0..bytes.len() {
        assert!(decode(&bytes[..len]).is_err(), "length {len}");
    }
}

#[test]
fn schema_4_fixture_decodes() {
    let bytes = include_bytes!("fixtures/saved-player-v4.bin");
    assert_eq!(save_schema(bytes), Some(4));
    assert_eq!(decode(bytes).unwrap(), flaring());
}

#[test]
#[ignore = "regenerates tests/fixtures/saved-player-v4.bin; run only while schema 4 is current"]
fn write_schema_4_fixture() {
    assert_eq!(SAVE_SCHEMA, 4);
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/saved-player-v4.bin"
    );
    std::fs::write(path, encode_save(&flaring()).unwrap()).unwrap();
}

// ---- general nodes ----

/// Economy 1 and flare cost 1 bought with the dragon's knowledge.
fn flaring() -> SavedPlayer {
    let mut saved = knowing();
    commit(&mut saved, |s| prepare_purchase(s, NODE_ECONOMY, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_FLARE_COST, 1));
    saved
}

#[test]
fn flare_cost_needs_economy_and_lowers_the_multiplier() {
    let saved = knowing();
    assert_eq!(
        prepare_purchase(&saved, NODE_FLARE_COST, 1),
        Err(SaveError::Progress(ProgressError::MissingRequirement))
    );
    assert_eq!(saved.progress().flare_milli(&CATALOG), 3_000);
    let saved = flaring();
    assert_eq!(saved.progress().flare_milli(&CATALOG), 2_700);
}

// Catches the flare node changing plain burning, or a lost thousandth across ticks.
#[test]
fn a_cheaper_flare_consumes_exactly_its_share() {
    let saved = flaring();
    let flare = saved.progress().flare_milli(&CATALOG);
    let rules = ReserveRules {
        reduction_milli: 50,
        ..rules()
    };
    let mut state = saved.state().clone();
    state.metals[Metal::Iron.index()].burn = BurnState::Flared;
    state.metals[Metal::Steel.index()].unlocked = true;
    state.metals[Metal::Steel.index()].reserve_milli = 10_000;
    state.metals[Metal::Steel.index()].burn = BurnState::Burning;
    let mut one = state.clone();
    let mut accounting = ReserveAccounting::default();
    consume_reserves_with(&mut state, &mut accounting, &rules, 10, flare).unwrap();
    // 10 × 0.95 × 2.7 × 10 = 256.5 and 10 × 0.95 × 10 = 95.
    assert_eq!(
        state.metals[Metal::Iron.index()].reserve_milli,
        25_000 - 256
    );
    assert_eq!(accounting.remainder[Metal::Iron.index()], 500);
    assert_eq!(
        state.metals[Metal::Steel.index()].reserve_milli,
        10_000 - 95
    );
    let mut single = ReserveAccounting::default();
    for _ in 0..10 {
        consume_reserves_with(&mut one, &mut single, &rules, 1, flare).unwrap();
    }
    assert_eq!(one, state);
    assert_eq!(single, accounting);
    // The base multiplier is what consume_reserves always used.
    let mut base = saved.state().clone();
    base.metals[Metal::Iron.index()].burn = BurnState::Flared;
    let mut with = base.clone();
    let (mut a, mut b) = (ReserveAccounting::default(), ReserveAccounting::default());
    consume_reserves(&mut base, &mut a, &rules, 7).unwrap();
    consume_reserves_with(&mut with, &mut b, &rules, 7, BASE_FLARE_MILLI).unwrap();
    assert_eq!((base, a), (with, b));
}

#[test]
fn dosing_adds_its_share_up_to_the_capacity() {
    let mut saved = knowing();
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_DOSING, 1));
    let before = saved.state().metals[Metal::Steel.index()].reserve_milli;
    commit(&mut saved, |s| {
        prepare_vial(s, Metal::Steel, 25_000).map(|(p, _)| p)
    });
    assert_eq!(
        saved.state().metals[Metal::Steel.index()].reserve_milli,
        before + 27_500
    );
    // Capacity 110 000 with iron at 25 000: 3 vials fill to 107 500, the 4th fits only 2 500.
    for _ in 0..3 {
        commit(&mut saved, |s| {
            prepare_vial(s, Metal::Iron, 25_000).map(|(p, _)| p)
        });
    }
    assert_eq!(
        saved.state().metals[Metal::Iron.index()].reserve_milli,
        107_500
    );
    assert!(prepare_vial(&saved, Metal::Iron, 25_000).is_err());
}

// Catches a bonus that overfills the capacity, or one dropped when only part of it fits.
#[test]
fn dosing_fills_what_room_is_left_after_the_vial() {
    let mut saved = knowing();
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_DOSING, 1));
    // Iron at 25 000 of 110 000: two vials (27 500 each) leave 30 000; the third gets 25 000
    // and 2 500 of its bonus; then 2 500 room: refused, the vial alone does not fit.
    for _ in 0..2 {
        commit(&mut saved, |s| {
            prepare_vial(s, Metal::Iron, 25_000).map(|(p, _)| p)
        });
    }
    let iron = |s: &SavedPlayer| s.state().metals[Metal::Iron.index()].reserve_milli;
    assert_eq!(iron(&saved), 80_000);
    commit(&mut saved, |s| {
        prepare_vial(s, Metal::Iron, 26_000).map(|(p, _)| p)
    });
    assert_eq!(iron(&saved), 108_600);
    commit(&mut saved, |s| {
        prepare_vial(s, Metal::Iron, 1_000).map(|(p, _)| p)
    });
    // 1 400 of room: the 1 000 and its whole 100 bonus fit.
    assert_eq!(iron(&saved), 109_700);
}

// Catches dosing skipped by the basic-metals vial (Codex review, round 1).
#[test]
fn dosing_also_adds_to_each_metal_of_a_basic_vial() {
    let mut saved = knowing();
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_DOSING, 1));
    // Iron 25 000, steel 0 (unlocked by the first vial): both get 27 500, iron up to 110 000.
    let (prepared, credited) = prepare_basic_vial(&saved, 25_000).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    assert_eq!(credited[Metal::Iron.index()], 27_500);
    assert_eq!(credited[Metal::Steel.index()], 27_500);
    for _ in 0..2 {
        let (prepared, _) = prepare_basic_vial(&saved, 25_000).unwrap();
        commit_credit(&mut saved, prepared).unwrap();
    }
    // Iron: 25 000 + 3 × 27 500 = 107 500; one more fills the 2 500 left.
    let (_, credited) = prepare_basic_vial(&saved, 25_000).unwrap();
    assert_eq!(credited[Metal::Iron.index()], 2_500);
}
