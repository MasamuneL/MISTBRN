//! P6: milestones claimed once per player, schema 3 and boss kills shared by a group.
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

fn fresh() -> SavedPlayer {
    SavedPlayer::new(player(), world(), rules()).unwrap()
}

fn decode(bytes: &[u8]) -> Result<SavedPlayer, SaveError> {
    decode_save(bytes, &world(), &player())
}

/// Iron and steel unlocked and selected through the tutorial vial, with iron fuel.
fn unlocked() -> SavedPlayer {
    let mut saved = fresh();
    let (prepared, unlocked) = prepare_vial(&saved, Metal::Iron, 25_000).unwrap();
    assert!(unlocked);
    commit_credit(&mut saved, prepared).unwrap();
    saved
}

fn claim(saved: &mut SavedPlayer, bit: u8) -> Option<Pair> {
    let (prepared, pair) = prepare_claim(saved, bit).unwrap();
    commit_credit(saved, prepared).unwrap();
    pair
}

fn pair_unlocked(saved: &SavedPlayer, pair: Pair) -> bool {
    pair.metals()
        .iter()
        .all(|metal| saved.state().metals[metal.index()].unlocked)
}

/// Every milestone claimed.
fn claimed_all() -> SavedPlayer {
    let mut saved = unlocked();
    for def in CATALOG.milestones {
        claim(&mut saved, def.bit);
    }
    saved
}

/// Re-seals a hand-edited save with a correct checksum.
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

/// `claimed_all()` encoded, edited and re-sealed.
fn edited(edit: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = encode_save(&claimed_all()).unwrap();
    edit(&mut bytes);
    reseal(bytes)
}

/// The milestones byte: the last of the payload, before the checksum.
fn milestones_at(bytes: &[u8]) -> usize {
    bytes.len() - 4 - 1
}

/// Where a metal's `unlocked` byte sits: header, both ids, the selected pair, 10 bytes a metal.
fn unlocked_at(metal: Metal) -> usize {
    10 + 2 + 32 + 2 + 5 + 1 + 10 * metal.index()
}

#[test]
fn the_catalog_is_valid_and_names_four_single_bit_milestones() {
    assert_eq!(validate_catalog(&CATALOG), Ok(()));
    assert_eq!(milestone_mask(&CATALOG), 0b1111);
    let twice = Catalog {
        milestones: &[
            MilestoneDef {
                bit: 1,
                key: "a",
                unlocks: None,
                knowledge: 0,
            },
            MilestoneDef {
                bit: 1,
                key: "b",
                unlocks: None,
                knowledge: 0,
            },
        ],
        ..CATALOG
    };
    assert_eq!(validate_catalog(&twice), Err(CatalogError::BadMilestone(1)));
    let wide = Catalog {
        milestones: &[MilestoneDef {
            bit: 3,
            key: "a",
            unlocks: None,
            knowledge: 0,
        }],
        ..CATALOG
    };
    assert_eq!(validate_catalog(&wide), Err(CatalogError::BadMilestone(3)));
}

// Catches a milestone that unlocks the wrong pair, selects it, or touches the fuel.
#[test]
fn each_milestone_unlocks_only_its_pair_without_selecting_it() {
    for (bit, pair) in [
        (MILESTONE_BASTION, Pair::TinPewter),
        (MILESTONE_ELDER_GUARDIAN, Pair::ZincBrass),
        (MILESTONE_ANCIENT_CITY, Pair::CopperBronze),
    ] {
        let mut saved = unlocked();
        let before = saved.clone();
        assert_eq!(claim(&mut saved, bit), Some(pair));
        assert!(saved.progress().claimed(bit));
        assert_eq!(saved.progress().milestones(), bit);
        assert_eq!(saved.state().selected_pair, Pair::IronSteel);
        for other in PAIRS {
            let expected = other == pair || other == Pair::IronSteel;
            assert_eq!(pair_unlocked(&saved, other), expected, "{bit} {other:?}");
        }
        for metal in Metal::ALL {
            let i = metal.index();
            assert_eq!(
                saved.state().metals[i].reserve_milli,
                before.state().metals[i].reserve_milli
            );
            assert_eq!(saved.state().metals[i].burn, before.state().metals[i].burn);
        }
        assert_eq!(saved.last_transaction(), before.last_transaction() + 1);
        assert_eq!(saved.revision(), before.revision() + 1);
    }
}

// A milestone counts even before the tutorial: its pair unlocks with iron and steel still locked.
#[test]
fn a_milestone_unlocks_its_pair_on_a_fresh_save() {
    let mut saved = fresh();
    assert_eq!(claim(&mut saved, MILESTONE_BASTION), Some(Pair::TinPewter));
    assert!(pair_unlocked(&saved, Pair::TinPewter));
    assert!(!pair_unlocked(&saved, Pair::IronSteel));
    assert_eq!(decode(&encode_save(&saved).unwrap()).unwrap(), saved);
}

// The close criterion: a duplicate unlocks nothing more and leaves the save as it was.
#[test]
fn a_second_claim_is_refused_and_changes_nothing() {
    let saved = claimed_all();
    let before = saved.clone();
    for def in CATALOG.milestones {
        assert_eq!(
            prepare_claim(&saved, def.bit).map(|_| ()),
            Err(SaveError::Progress(ProgressError::AlreadyClaimed))
        );
    }
    assert_eq!(saved, before);
    // The dragon's knowledge is granted once, however often it is asked for.
    let mut other = unlocked();
    claim(&mut other, MILESTONE_DRAGON);
    assert_eq!(
        prepare_claim(&other, MILESTONE_DRAGON).map(|_| ()),
        Err(SaveError::Progress(ProgressError::AlreadyClaimed))
    );
    assert_eq!(other.progress().earned(Pool::Knowledge, &CATALOG), 3);
}

#[test]
fn unknown_or_combined_bits_are_refused() {
    let saved = unlocked();
    for bit in [0u8, 3, 5, 16, 128, 255] {
        assert_eq!(
            prepare_claim(&saved, bit).map(|_| ()),
            Err(SaveError::Progress(ProgressError::UnknownMilestone)),
            "bit {bit}"
        );
    }
}

#[test]
fn the_dragon_grants_knowledge_that_buys_general_nodes() {
    let mut saved = unlocked();
    assert_eq!(saved.progress().earned(Pool::Knowledge, &CATALOG), 0);
    assert_eq!(claim(&mut saved, MILESTONE_DRAGON), None);
    assert_eq!(saved.progress().earned(Pool::Knowledge, &CATALOG), 3);
    for rank in [1, 2] {
        let prepared = prepare_purchase(&saved, NODE_ECONOMY, rank).unwrap();
        commit_credit(&mut saved, prepared).unwrap();
    }
    assert_eq!(saved.progress().available(Pool::Knowledge, &CATALOG), 0);
    assert_eq!(saved.rules().reduction_milli, 100);
    assert_eq!(
        prepare_purchase(&saved, NODE_CAPACITY, 1).map(|_| ()),
        Err(SaveError::Progress(ProgressError::NotEnoughPoints))
    );
}

#[test]
fn a_claim_commits_exactly_once() {
    let saved = unlocked();
    let (first, _) = prepare_claim(&saved, MILESTONE_BASTION).unwrap();
    let (again, _) = prepare_claim(&saved, MILESTONE_BASTION).unwrap();
    let mut live = saved.clone();
    commit_credit(&mut live, first).unwrap();
    assert_eq!(commit_credit(&mut live, again), Err(SaveError::Duplicate));
    assert_eq!(live.progress().milestones(), MILESTONE_BASTION);
    // Prepared against an older revision: stale, nothing applied.
    let mut moved = saved.clone();
    let (stale, _) = prepare_claim(&moved, MILESTONE_ANCIENT_CITY).unwrap();
    on_reconnect(&mut moved).unwrap();
    let before = moved.clone();
    assert_eq!(
        commit_credit(&mut moved, stale),
        Err(SaveError::StalePreparation)
    );
    assert_eq!(moved, before);
}

// A cut before the game's save keeps the old file, after it the new one: never half a claim.
#[test]
fn a_cut_yields_the_old_or_the_new_save_never_both() {
    let saved = unlocked();
    let old = encode_save(&saved).unwrap();
    let (prepared, _) = prepare_claim(&saved, MILESTONE_ELDER_GUARDIAN).unwrap();
    let new = encode_save(prepared.candidate()).unwrap();
    let old = decode(&old).unwrap();
    assert!(!old.progress().claimed(MILESTONE_ELDER_GUARDIAN));
    assert!(!pair_unlocked(&old, Pair::ZincBrass));
    let new = decode(&new).unwrap();
    assert!(new.progress().claimed(MILESTONE_ELDER_GUARDIAN));
    assert!(pair_unlocked(&new, Pair::ZincBrass));
    assert_eq!(new.last_transaction(), old.last_transaction() + 1);
}

#[test]
fn death_and_reconnect_keep_milestones_and_their_unlocks() {
    let mut saved = claimed_all();
    on_death(&mut saved).unwrap();
    on_reconnect(&mut saved).unwrap();
    assert_eq!(saved.progress().milestones(), 0b1111);
    for pair in PAIRS {
        assert!(pair_unlocked(&saved, pair));
    }
    assert_eq!(saved.progress().earned(Pool::Knowledge, &CATALOG), 3);
}

// ---- schema 3 ----

#[test]
fn schema_3_round_trips_the_milestones() {
    let saved = claimed_all();
    let bytes = encode_save(&saved).unwrap();
    assert_eq!(save_schema(&bytes), Some(3));
    assert_eq!(bytes[milestones_at(&bytes)], 0b1111);
    assert_eq!(decode(&bytes).unwrap(), saved);
}

#[test]
fn every_truncation_and_bit_flip_of_schema_3_is_rejected() {
    let bytes = encode_save(&claimed_all()).unwrap();
    for len in 0..bytes.len() {
        assert!(decode(&bytes[..len]).is_err(), "length {len}");
    }
    for at in 0..bytes.len() {
        let mut flipped = bytes.clone();
        flipped[at] ^= 0x10;
        assert!(decode(&flipped).is_err(), "byte {at}");
    }
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(decode(&longer), Err(SaveError::TrailingBytes));
}

// Catches a decoder that accepts milestone bits no catalog entry defines.
#[test]
fn unknown_milestone_bits_are_rejected_on_decode() {
    for bit in [16u8, 32, 64, 128] {
        let bytes = edited(|b| {
            let at = milestones_at(b);
            b[at] |= bit;
        });
        assert_eq!(decode(&bytes), Err(SaveError::InvalidEnum), "bit {bit}");
    }
}

// Catches a claimed milestone whose pair was locked again by hand.
#[test]
fn a_claimed_milestone_with_its_pair_locked_is_rejected() {
    for metal in [Metal::Tin, Metal::Brass, Metal::Bronze] {
        let bytes = edited(|b| b[unlocked_at(metal)] = 0);
        assert_eq!(
            decode(&bytes),
            Err(SaveError::Progress(ProgressError::PairLocked)),
            "{metal:?}"
        );
    }
    // Without the claim, the same locked pair is a valid save.
    let bytes = edited(|b| {
        b[unlocked_at(Metal::Tin)] = 0;
        let at = milestones_at(b);
        b[at] &= !MILESTONE_BASTION;
    });
    assert!(decode(&bytes).is_ok());
}

// Catches spending knowledge the dragon no longer explains.
#[test]
fn knowledge_spent_from_the_dragon_needs_its_claim() {
    let mut saved = claimed_all();
    let prepared = prepare_purchase(&saved, NODE_CAPACITY, 1).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    let mut bytes = encode_save(&saved).unwrap();
    let at = milestones_at(&bytes);
    bytes[at] &= !MILESTONE_DRAGON;
    assert_eq!(
        decode(&reseal(bytes)),
        Err(SaveError::Progress(ProgressError::Overspent))
    );
}

// Fixture for the schema after this one (it must load it).
#[test]
fn schema_3_fixture_decodes() {
    let bytes = include_bytes!("fixtures/saved-player-v3.bin");
    assert_eq!(save_schema(bytes), Some(3));
    assert_eq!(decode(bytes).unwrap(), claimed_all());
}

#[test]
#[ignore = "regenerates tests/fixtures/saved-player-v3.bin; run only while schema 3 is current"]
fn write_schema_3_fixture() {
    assert_eq!(SAVE_SCHEMA, 3);
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/saved-player-v3.bin"
    );
    std::fs::write(path, encode_save(&claimed_all()).unwrap()).unwrap();
}

// ---- notebook snapshot ----

#[test]
fn the_progress_snapshot_carries_the_milestones() {
    let saved = claimed_all();
    let bytes = encode_progress_snapshot(saved.progress());
    assert_eq!(bytes[0], PROGRESS_VERSION);
    let shown = decode_progress_snapshot(&bytes).unwrap().progress;
    assert_eq!(shown.milestones(), 0b1111);
    let mut bad = bytes;
    bad[PROGRESS_LEN - 1] |= 16;
    assert_eq!(
        decode_progress_snapshot(&bad),
        Err(ProgressSnapshotError::Milestones)
    );
}

// ---- group kills ----

const A: u128 = 1;
const B: u128 = 2;
const C: u128 = 3;

// Catches a watcher claiming, or a hit just outside the window counting.
#[test]
fn players_who_hit_within_the_window_claim_and_others_do_not() {
    let mut ledger = BossLedger::default();
    ledger.hit(7, A, 1000);
    ledger.hit(7, B, 1000 + 600);
    ledger.hit(7, C, 999);
    ledger.hit(8, C, 1600); // another boss
    assert_eq!(ledger.death(7, 1600), vec![A, B]);
    assert!(!ledger.tracks(7));
    assert!(ledger.tracks(8));
    assert_eq!(ledger.death(7, 1600), Vec::<u128>::new());
}

#[test]
fn the_latest_hit_counts_and_a_dead_boss_is_forgotten() {
    let mut ledger = BossLedger::default();
    ledger.hit(5, A, 100);
    ledger.hit(5, A, 900);
    ledger.hit(5, A, 50); // late, older report: the latest stays
    assert_eq!(ledger.death(5, 1400), vec![A]);
    ledger.hit(6, B, 10);
    ledger.forget(6);
    assert_eq!(ledger.death(6, 20), Vec::<u128>::new());
}

#[test]
fn pruning_drops_only_hits_too_old_to_count() {
    let mut ledger = BossLedger::default();
    ledger.hit(1, A, 0);
    ledger.hit(2, B, 500);
    ledger.prune(600);
    assert!(ledger.tracks(1) && ledger.tracks(2));
    ledger.prune(601);
    assert!(!ledger.tracks(1));
    assert_eq!(ledger.bosses().collect::<Vec<_>>(), vec![2]);
    // A hit reported after the death tick never counts.
    ledger.hit(3, C, 50);
    assert_eq!(ledger.death(3, 40), Vec::<u128>::new());
}
