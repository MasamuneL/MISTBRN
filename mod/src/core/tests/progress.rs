use allomancy_core::physics::{self, Body, Direction, ForceRules, StepInput};
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

fn commit(
    saved: &mut SavedPlayer,
    prepare: impl FnOnce(&SavedPlayer) -> Result<PreparedCredit, SaveError>,
) {
    let prepared = prepare(saved).unwrap();
    commit_credit(saved, prepared).unwrap();
}

/// A save with iron and steel unlocked through the tutorial vial.
fn unlocked() -> SavedPlayer {
    let mut saved = fresh();
    let (prepared, unlocked) = prepare_vial(&saved, Metal::Iron, 25_000).unwrap();
    assert!(unlocked);
    commit_credit(&mut saved, prepared).unwrap();
    saved
}

fn snapshot(actor: PlayerId) -> WorldSnapshot {
    WorldSnapshot {
        tick: Tick(1),
        world_id: world(),
        actor,
        position: Vec3::default(),
        targets: Vec::new(),
        authority: Authority::Server,
        input: InputContext {
            gameplay_allowed: true,
        },
    }
}

fn event(kind: EventKind) -> VerifiedEvent {
    VerifiedEvent::from_server_observation(EventId::new("e"), kind, &snapshot(player())).unwrap()
}

fn practice(category: u8) -> VerifiedEvent {
    event(EventKind::PracticeAction { category })
}

fn evidence(bit: u8) -> VerifiedEvent {
    event(EventKind::ResearchEvidence {
        research: RESEARCH_ANCHOR,
        evidence: bit,
    })
}

/// `windows` fresh windows of anchor practice: 10 XP each.
fn practise(saved: &mut SavedPlayer, windows: u64) {
    for i in 0..windows {
        let at = 10_000 + i * 1_200;
        let observed = observe(saved, &practice(PRACTICE_ANCHOR_MOVE), at).unwrap();
        assert!(matches!(observed, Observed::Practice { xp: 10, .. }));
    }
}

fn concluded() -> SavedPlayer {
    let mut saved = unlocked();
    observe(&mut saved, &evidence(EVIDENCE_SELF_PULL), 0).unwrap();
    observe(&mut saved, &evidence(EVIDENCE_ITEM_PULLED), 0).unwrap();
    commit(&mut saved, |s| prepare_conclude(s, RESEARCH_ANCHOR));
    saved
}

fn progress_error(error: ProgressError) -> Result<PreparedCredit, SaveError> {
    Err(SaveError::Progress(error))
}

// ---- catalog ----

#[test]
fn the_catalog_is_valid() {
    assert_eq!(validate_catalog(&CATALOG), Ok(()));
}

static NODE_A: NodeDef = NodeDef {
    code: 1,
    key: "a",
    pool: Pool::Knowledge,
    effect: NodeEffect::ReachBlocks,
    ranks: &[Rank { cost: 1, value: 1 }],
    requires: &[],
};

// Catches a catalog edit that would make a node unbuyable or a code ambiguous.
#[test]
fn broken_catalogs_are_rejected() {
    let check = |nodes: &'static [NodeDef]| validate_catalog(&Catalog { nodes, ..CATALOG });
    static CYCLE: [NodeDef; 2] = [
        NodeDef {
            requires: &[Requirement::Node { code: 2, rank: 1 }],
            ..NODE_A
        },
        NodeDef {
            code: 2,
            requires: &[Requirement::Node { code: 1, rank: 1 }],
            ..NODE_A
        },
    ];
    assert!(matches!(check(&CYCLE), Err(CatalogError::Cycle(_))));
    static DUP: [NodeDef; 2] = [NODE_A, NODE_A];
    assert_eq!(check(&DUP), Err(CatalogError::DuplicateCode(1)));
    static MISSING: [NodeDef; 1] = [NodeDef {
        requires: &[Requirement::Discovery { code: 9 }],
        ..NODE_A
    }];
    assert_eq!(check(&MISSING), Err(CatalogError::MissingRequirement(1)));
    static RANK: [NodeDef; 1] = [NodeDef {
        requires: &[Requirement::Node { code: 1, rank: 2 }],
        ..NODE_A
    }];
    assert_eq!(
        check(&RANK),
        Err(CatalogError::RequirementRankOutOfRange(1))
    );
    static WIDE: [NodeDef; 1] = [NodeDef { code: 64, ..NODE_A }];
    assert_eq!(check(&WIDE), Err(CatalogError::CodeOutOfRange(64)));
    static OVER: [NodeDef; 1] = [NodeDef {
        effect: NodeEffect::ReductionMilli,
        ranks: &[Rank {
            cost: 1,
            value: 201,
        }],
        ..NODE_A
    }];
    assert_eq!(check(&OVER), Err(CatalogError::EffectOutOfLimits(1)));
    static EMPTY: [NodeDef; 1] = [NodeDef {
        ranks: &[],
        ..NODE_A
    }];
    assert_eq!(check(&EMPTY), Err(CatalogError::NoRanks(1)));
}

#[test]
fn points_follow_the_thresholds_then_the_step() {
    let points = |xp| points_for_xp(xp, &CATALOG);
    assert_eq!(points(0), 0);
    assert_eq!(points(99), 0);
    assert_eq!(points(100), 1);
    assert_eq!(points(1_000), 5);
    assert_eq!(points(1_399), 5);
    assert_eq!(points(1_400), 6);
    assert_eq!(points(u32::MAX), 5 + (u32::MAX - 1_000) / 400);
    assert_eq!(xp_to_next_point(0, &CATALOG), 100);
    assert_eq!(xp_to_next_point(1_000, &CATALOG), 400);
    assert_eq!(xp_to_next_point(1_100, &CATALOG), 300);
}

// ---- tutorial vial ----

#[test]
fn the_first_iron_or_steel_vial_unlocks_and_selects_the_pair_once() {
    let mut saved = fresh();
    update_player(&mut saved, |state, _| {
        state.selected_pair = Pair::TinPewter;
        Ok(())
    })
    .unwrap();
    let (prepared, unlocked) = prepare_vial(&saved, Metal::Steel, 25_000).unwrap();
    assert!(unlocked);
    assert_eq!(prepared.seq(), TransactionSeq(1));
    commit_credit(&mut saved, prepared).unwrap();
    let state = saved.state();
    assert!(state.metals[Metal::Iron.index()].unlocked);
    assert!(state.metals[Metal::Steel.index()].unlocked);
    assert_eq!(state.selected_pair, Pair::IronSteel);
    assert_eq!(state.metals[Metal::Steel.index()].reserve_milli, 25_000);

    // The second vial only credits.
    let (prepared, unlocked) = prepare_vial(&saved, Metal::Iron, 25_000).unwrap();
    assert!(!unlocked);
    commit_credit(&mut saved, prepared).unwrap();
    assert_eq!(saved.last_transaction(), 2);
}

// Catches the tutorial unlocking pairs P6 reserves for milestones.
#[test]
fn other_vials_credit_but_never_unlock() {
    for metal in [
        Metal::Tin,
        Metal::Pewter,
        Metal::Zinc,
        Metal::Brass,
        Metal::Copper,
        Metal::Bronze,
    ] {
        let mut saved = fresh();
        let (prepared, unlocked) = prepare_vial(&saved, metal, 25_000).unwrap();
        assert!(!unlocked, "{metal:?}");
        commit_credit(&mut saved, prepared).unwrap();
        assert!(
            saved.state().metals.iter().all(|m| !m.unlocked),
            "{metal:?}"
        );
        assert_eq!(saved.state().metals[metal.index()].reserve_milli, 25_000);
        assert_eq!(saved.state().selected_pair, Pair::IronSteel);
    }
}

#[test]
fn a_refused_vial_unlocks_nothing() {
    let saved = fresh();
    assert!(prepare_vial(&saved, Metal::Iron, 100_001).is_err());
    assert!(prepare_vial(&saved, Metal::Iron, -1).is_err());
}

// ---- practice ----

#[test]
fn practice_falls_off_within_a_window_and_recovers_after_it() {
    let mut saved = unlocked();
    let gains: Vec<Observed> = (0..5)
        .map(|i| observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 100 + i).unwrap())
        .collect();
    let got = |xp, count| Observed::Practice {
        category: PRACTICE_ANCHOR_MOVE,
        xp,
        count,
    };
    assert_eq!(
        gains,
        [
            got(10, 1),
            got(5, 2),
            got(2, 3),
            Observed::Nothing,
            Observed::Nothing
        ]
    );
    assert_eq!(saved.progress().pair_xp(Pair::IronSteel), 17);
    // Other categories have their own windows.
    assert!(matches!(
        observe(&mut saved, &practice(PRACTICE_COIN_JUMP), 105).unwrap(),
        Observed::Practice { xp: 15, .. }
    ));
    // 1199 ticks after the start: still the same window.
    assert_eq!(
        observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 1_299).unwrap(),
        Observed::Nothing
    );
    // 1200: a new one.
    assert_eq!(
        observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 1_300).unwrap(),
        got(10, 1)
    );
    // A clock before the window's start (another save's game time) opens a new window too.
    assert_eq!(
        observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 5).unwrap(),
        got(10, 1)
    );
}

// Catches a counter that wraps and pays full experience again (Gemini review, P5 design).
#[test]
fn a_burst_of_actions_never_pays_again_inside_its_window() {
    let mut saved = unlocked();
    let mut total = 0;
    for _ in 0..300 {
        if let Observed::Practice { xp, .. } =
            observe(&mut saved, &practice(PRACTICE_COIN_HIT), 50).unwrap()
        {
            total += xp;
        }
    }
    assert_eq!(total, 10 + 5 + 2);
    assert_eq!(saved.progress().windows()[2].count, 3);
}

#[test]
fn a_spent_window_leaves_the_save_untouched() {
    let mut saved = unlocked();
    for _ in 0..3 {
        observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 7).unwrap();
    }
    let before = saved.clone();
    observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 8).unwrap();
    assert_eq!(saved, before, "no revision for nothing");
}

#[test]
fn practice_needs_the_pair_and_this_player() {
    let mut locked = fresh();
    assert_eq!(
        observe(&mut locked, &practice(PRACTICE_ANCHOR_MOVE), 0),
        Err(SaveError::Progress(ProgressError::PairLocked))
    );
    let mut saved = unlocked();
    let other = VerifiedEvent::from_server_observation(
        EventId::new("e"),
        EventKind::PracticeAction { category: 0 },
        &snapshot(PlayerId::new("ffffffffffffffffffffffffffffffff")),
    )
    .unwrap();
    let before = saved.clone();
    assert_eq!(
        observe(&mut saved, &other, 0),
        Err(SaveError::Progress(ProgressError::WrongActor))
    );
    assert_eq!(
        observe(&mut saved, &practice(9), 0),
        Err(SaveError::Progress(ProgressError::UnknownCategory))
    );
    assert_eq!(saved, before);
}

// Catches practice experience credited to whatever pair is selected.
#[test]
fn practice_goes_to_its_own_pair_whatever_is_selected() {
    let mut saved = unlocked();
    update_player(&mut saved, |state, _| {
        state.selected_pair = Pair::ZincBrass;
        Ok(())
    })
    .unwrap();
    observe(&mut saved, &practice(PRACTICE_ANCHOR_MOVE), 0).unwrap();
    assert_eq!(saved.progress().pair_xp(Pair::IronSteel), 10);
    assert_eq!(saved.progress().pair_xp(Pair::ZincBrass), 0);
}

// ---- research ----

#[test]
fn evidence_counts_before_any_hint_and_once_per_kind() {
    let mut saved = unlocked();
    let seen = |bit, ready| Observed::Evidence {
        research: RESEARCH_ANCHOR,
        bit,
        ready,
    };
    assert_eq!(
        observe(&mut saved, &evidence(EVIDENCE_SELF_PULL), 0).unwrap(),
        seen(EVIDENCE_SELF_PULL, false)
    );
    assert_eq!(
        observe(&mut saved, &evidence(EVIDENCE_SELF_PULL), 1).unwrap(),
        Observed::Nothing
    );
    assert_eq!(
        observe(&mut saved, &evidence(EVIDENCE_PUSH_LIFT), 2).unwrap(),
        seen(EVIDENCE_PUSH_LIFT, true)
    );
    assert_eq!(saved.progress().research(RESEARCH_ANCHOR).hints, 0);
    // Not a single known bit: refused.
    for bad in [3, 8] {
        assert_eq!(
            observe(&mut saved, &evidence(bad), 3),
            Err(SaveError::Progress(ProgressError::EvidenceOutOfRange))
        );
    }
}

#[test]
fn concluding_needs_enough_evidence_and_happens_once() {
    let mut saved = unlocked();
    assert_eq!(
        prepare_conclude(&saved, RESEARCH_ANCHOR),
        progress_error(ProgressError::NotReady)
    );
    observe(&mut saved, &evidence(EVIDENCE_ITEM_PULLED), 0).unwrap();
    assert_eq!(
        prepare_conclude(&saved, RESEARCH_ANCHOR),
        progress_error(ProgressError::NotReady)
    );
    observe(&mut saved, &evidence(EVIDENCE_PUSH_LIFT), 0).unwrap();
    let prepared = prepare_conclude(&saved, RESEARCH_ANCHOR).unwrap();
    let replay = prepare_conclude(&saved, RESEARCH_ANCHOR).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    assert_eq!(commit_credit(&mut saved, replay), Err(SaveError::Duplicate));
    assert!(saved.progress().discovered(RESEARCH_ANCHOR));
    assert_eq!(saved.progress().pair_xp(Pair::IronSteel), 200);
    assert_eq!(saved.progress().available(Pool::Knowledge, &CATALOG), 3);
    assert_eq!(
        prepare_conclude(&saved, RESEARCH_ANCHOR),
        progress_error(ProgressError::AlreadyConcluded)
    );
    // Evidence after the discovery adds nothing.
    let before = saved.clone();
    assert_eq!(
        observe(&mut saved, &evidence(EVIDENCE_SELF_PULL), 0).unwrap(),
        Observed::Nothing
    );
    assert_eq!(saved, before);
}

#[test]
fn hints_are_revealed_in_order_and_cost_nothing() {
    let mut saved = unlocked();
    assert_eq!(
        prepare_hint(&saved, RESEARCH_ANCHOR, 2),
        progress_error(ProgressError::HintSkipped)
    );
    commit(&mut saved, |s| prepare_hint(s, RESEARCH_ANCHOR, 1));
    assert_eq!(
        prepare_hint(&saved, RESEARCH_ANCHOR, 1),
        progress_error(ProgressError::HintKnown)
    );
    commit(&mut saved, |s| prepare_hint(s, RESEARCH_ANCHOR, 2));
    commit(&mut saved, |s| prepare_hint(s, RESEARCH_ANCHOR, 3));
    assert_eq!(
        prepare_hint(&saved, RESEARCH_ANCHOR, 4),
        progress_error(ProgressError::HintsOutOfRange)
    );
    assert_eq!(
        prepare_hint(&saved, 9, 1),
        progress_error(ProgressError::UnknownResearch)
    );
    assert_eq!(saved.progress().research(RESEARCH_ANCHOR).hints, 3);
    assert_eq!(saved.progress().pair_xp(Pair::IronSteel), 0);
}

// ---- purchases ----

#[test]
fn a_purchase_applies_once_and_names_its_rank() {
    let mut saved = unlocked();
    practise(&mut saved, 10); // 100 XP: one point
    let prepared = prepare_purchase(&saved, NODE_REACH, 1).unwrap();
    let replay = prepare_purchase(&saved, NODE_REACH, 1).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    assert_eq!(commit_credit(&mut saved, replay), Err(SaveError::Duplicate));
    assert_eq!(saved.progress().rank(NODE_REACH), 1);
    // The same request again is refused, never a second rank.
    assert_eq!(
        prepare_purchase(&saved, NODE_REACH, 1),
        progress_error(ProgressError::AlreadyOwned)
    );
    assert_eq!(
        prepare_purchase(&saved, NODE_REACH, 3),
        progress_error(ProgressError::RankSkipped)
    );
    assert_eq!(
        prepare_purchase(&saved, NODE_REACH, 2),
        progress_error(ProgressError::NotEnoughPoints)
    );
    assert_eq!(
        prepare_purchase(&saved, NODE_REACH, 4),
        progress_error(ProgressError::RankOutOfRange)
    );
    assert_eq!(
        prepare_purchase(&saved, 9, 1),
        progress_error(ProgressError::UnknownNode)
    );
    assert_eq!(
        saved
            .progress()
            .available(Pool::Pair(Pair::IronSteel), &CATALOG),
        0
    );
}

#[test]
fn requirements_and_locked_pairs_block_purchases() {
    let locked = fresh();
    assert_eq!(
        prepare_purchase(&locked, NODE_REACH, 1),
        progress_error(ProgressError::PairLocked)
    );
    let mut saved = unlocked();
    practise(&mut saved, 25); // 250 XP: two points
    assert_eq!(
        prepare_purchase(&saved, NODE_ACCEL, 1),
        progress_error(ProgressError::MissingRequirement)
    );
    assert_eq!(
        prepare_purchase(&saved, NODE_COIN_JUMP, 1),
        progress_error(ProgressError::MissingRequirement)
    );
    commit(&mut saved, |s| prepare_purchase(s, NODE_REACH, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_ACCEL, 1));
    assert_eq!(saved.progress().rank(NODE_ACCEL), 1);
}

#[test]
fn the_discovery_opens_coin_jump_and_knowledge_buys_general_nodes() {
    let mut saved = concluded(); // 200 XP: one point; 3 knowledge
    assert_eq!(
        prepare_purchase(&saved, NODE_COIN_JUMP, 1),
        progress_error(ProgressError::NotEnoughPoints)
    );
    practise(&mut saved, 5); // 250 XP: two points
    commit(&mut saved, |s| prepare_purchase(s, NODE_COIN_JUMP, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_ECONOMY, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_ECONOMY, 2));
    assert_eq!(saved.rules().reduction_milli, 100);
    assert_eq!(saved.progress().available(Pool::Knowledge, &CATALOG), 0);
    assert_eq!(
        prepare_purchase(&saved, NODE_CAPACITY, 1),
        progress_error(ProgressError::NotEnoughPoints)
    );
    // Pair points are never spent on general nodes, nor the other way round.
    assert_eq!(
        saved
            .progress()
            .available(Pool::Pair(Pair::IronSteel), &CATALOG),
        0
    );
}

#[test]
fn capacity_nodes_raise_the_capacity_through_the_rules() {
    let mut saved = concluded();
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    assert_eq!(saved.rules().capacity_bonus_milli, 100);
    assert_eq!(saved.rules().capacity(Metal::Iron), Ok(110_000));
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 2));
    assert_eq!(saved.rules().capacity(Metal::Iron), Ok(125_000));
    // Rules that contradict the nodes are refused.
    let mut wrong = saved.rules().clone();
    wrong.capacity_bonus_milli = 0;
    let before = saved.clone();
    assert_eq!(
        replace_rules(&mut saved, wrong),
        Err(SaveError::Progress(ProgressError::RulesMismatch))
    );
    assert_eq!(saved, before);
}

#[test]
fn progress_survives_death_and_reconnect() {
    let mut saved = concluded();
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    observe(&mut saved, &practice(PRACTICE_COIN_HIT), 40).unwrap();
    let progress = saved.progress().clone();
    let rules = saved.rules().clone();
    on_death(&mut saved).unwrap();
    assert_eq!(saved.progress(), &progress);
    assert_eq!(saved.rules(), &rules);
    assert!(saved.state().metals.iter().all(|m| m.reserve_milli == 0));
    on_reconnect(&mut saved).unwrap();
    assert_eq!(saved.progress(), &progress);
    let reloaded = decode(&encode_save(&saved).unwrap()).unwrap();
    assert_eq!(reloaded.progress(), &progress);
}

// ---- codec ----

/// A save with something in every progress field: nodes reach 1 and capacity 1, the anchor
/// research concluded with one hint, xp, and a used practice window.
fn learned() -> SavedPlayer {
    let mut saved = concluded();
    practise(&mut saved, 5);
    commit(&mut saved, |s| prepare_purchase(s, NODE_REACH, 1));
    commit(&mut saved, |s| prepare_purchase(s, NODE_CAPACITY, 1));
    commit(&mut saved, |s| prepare_hint(s, RESEARCH_ANCHOR, 1));
    observe(&mut saved, &practice(PRACTICE_COIN_JUMP), 123_456).unwrap();
    saved
}

#[test]
fn the_current_schema_round_trips_every_progress_field() {
    let saved = learned();
    let bytes = encode_save(&saved).unwrap();
    assert_eq!(save_schema(&bytes), Some(3));
    assert_eq!(decode(&bytes).unwrap(), saved);
}

#[test]
fn every_truncation_and_bit_flip_of_schema_2_is_rejected() {
    let bytes = encode_save(&learned()).unwrap();
    for len in 0..bytes.len() {
        assert!(decode(&bytes[..len]).is_err(), "length {len}");
    }
    for at in 0..bytes.len() {
        let mut flipped = bytes.clone();
        flipped[at] ^= 0x10;
        assert!(decode(&flipped).is_err(), "byte {at}");
    }
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

/// Where the progress block starts, for this test's 32-byte player id and world `local`:
/// header, ids, pair, metals, remainders, rules, counters.
const PROGRESS_AT: usize = 10 + 2 + 32 + 2 + 5 + 1 + 8 * 10 + 8 * 2 + 8 * 8 * 2 + 2 + 2 + 8 + 8;
/// `learned()`'s layout after PROGRESS_AT: 4 × xp, n_nodes, nodes (reach, capacity), then
/// n_research and the anchor research entry.
const NODES_AT: usize = PROGRESS_AT + 16 + 1;
const RESEARCH_AT: usize = NODES_AT + 2 * 3 + 1;

fn edited(edit: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut bytes = encode_save(&learned()).unwrap();
    edit(&mut bytes);
    reseal(bytes)
}

// Catches a decoder that trusts saved progress it would never have produced.
#[test]
fn impossible_progress_is_rejected_on_decode() {
    let invalid = |error| Err(SaveError::Progress(error));
    assert_eq!(
        decode(&edited(|_| ())).map(|s| s.progress().clone()),
        Ok(learned().progress().clone())
    );
    // Experience below what the owned nodes cost.
    let poor = edited(|b| b[PROGRESS_AT..PROGRESS_AT + 4].copy_from_slice(&0u32.to_le_bytes()));
    // (the conclusion's own 200 is checked first)
    assert_eq!(
        decode(&poor),
        invalid(ProgressError::ConcludedWithoutExperience)
    );
    // An unknown node code (reach 1 → 9: still in order).
    assert_eq!(
        decode(&edited(|b| b[NODES_AT] = 9)),
        invalid(ProgressError::UnknownNode)
    );
    // Rank 0 and a rank beyond the node's.
    assert_eq!(
        decode(&edited(|b| b[NODES_AT + 2] = 0)),
        invalid(ProgressError::RankOutOfRange)
    );
    assert_eq!(
        decode(&edited(|b| b[NODES_AT + 2] = 4)),
        invalid(ProgressError::RankOutOfRange)
    );
    // Codes out of order (capacity 17 before reach 1).
    let unsorted = edited(|b| {
        let first = b[NODES_AT..NODES_AT + 3].to_vec();
        let second = b[NODES_AT + 3..NODES_AT + 6].to_vec();
        b[NODES_AT..NODES_AT + 3].copy_from_slice(&second);
        b[NODES_AT + 3..NODES_AT + 6].copy_from_slice(&first);
    });
    assert_eq!(decode(&unsorted), Err(SaveError::InvalidEnum));
    // Research entry: code:u16 | evidence | hints | concluded.
    let evidence_at = RESEARCH_AT + 2;
    assert_eq!(
        decode(&edited(|b| b[evidence_at] = EVIDENCE_SELF_PULL)),
        invalid(ProgressError::ConcludedWithoutEvidence)
    );
    assert_eq!(
        decode(&edited(|b| b[evidence_at] |= 8)),
        invalid(ProgressError::EvidenceOutOfRange)
    );
    assert_eq!(
        decode(&edited(|b| b[evidence_at + 1] = 4)),
        invalid(ProgressError::HintsOutOfRange)
    );
    assert_eq!(
        decode(&edited(|b| b[evidence_at + 2] = 2)),
        Err(SaveError::InvalidEnum)
    );
    assert_eq!(
        decode(&edited(|b| b[RESEARCH_AT] = 9)),
        invalid(ProgressError::UnknownResearch)
    );
}

#[test]
fn rules_that_do_not_match_the_nodes_are_rejected_on_decode() {
    // capacity_bonus_milli is the u16 just before revision and last_transaction.
    let at = PROGRESS_AT - 16 - 2;
    let bytes = edited(|b| b[at..at + 2].copy_from_slice(&0u16.to_le_bytes()));
    assert_eq!(
        decode(&bytes),
        Err(SaveError::Progress(ProgressError::RulesMismatch))
    );
}

// Catches a schema 3 reader that loses or invents anything of a P5 save (P6 migration).
#[test]
fn schema_2_fixture_migrates_to_schema_3_keeping_every_field() {
    let bytes = include_bytes!("fixtures/saved-player-v2.bin");
    assert_eq!(save_schema(bytes), Some(2));
    let migrated = decode(bytes).unwrap();
    let expected = learned();
    assert_eq!(migrated.state(), expected.state());
    assert_eq!(migrated.accounting(), expected.accounting());
    assert_eq!(migrated.rules(), expected.rules());
    assert_eq!(migrated.progress(), expected.progress());
    assert_eq!(migrated.progress().milestones(), 0);
    assert_eq!(migrated.revision(), expected.revision());
    assert_eq!(migrated.last_transaction(), expected.last_transaction());
    assert_eq!(migrated, expected);
    let rewritten = encode_save(&migrated).unwrap();
    assert_eq!(save_schema(&rewritten), Some(3));
    assert_eq!(rewritten.len(), bytes.len() + 1);
    assert_eq!(decode(&rewritten).unwrap(), migrated);
}

// Catches a schema 2 reader that accepts a cut or padded P5 save.
#[test]
fn every_truncation_of_the_schema_2_fixture_is_rejected() {
    let bytes = include_bytes!("fixtures/saved-player-v2.bin");
    for len in 0..bytes.len() {
        assert!(decode(&bytes[..len]).is_err(), "length {len}");
    }
    let mut longer = bytes.to_vec();
    longer.push(0);
    assert_eq!(decode(&longer), Err(SaveError::TrailingBytes));
}

// ---- force ----

#[test]
fn nodes_tune_iron_and_steel() {
    let saved = learned(); // reach 1
    let tuned = tuned_force(&ForceRules::BURNING, saved.progress(), &CATALOG);
    assert_eq!(tuned.range, ForceRules::BURNING.range + 2.0);
    assert_eq!(tuned.accel, ForceRules::BURNING.accel);
    assert_eq!(tuned.push_below_gain, 1.0);
    let base = tuned_force(&ForceRules::FLARED, &Progress::default(), &CATALOG);
    assert_eq!(base, ForceRules::FLARED);
}

// Catches the coin jump bonus leaking into pulls or sideways pushes.
#[test]
fn the_push_below_gain_applies_only_to_a_push_off_an_anchor_underfoot() {
    let gained = ForceRules {
        push_below_gain: 1.25,
        ..ForceRules::BURNING
    };
    let input = |target: Vec3, direction| StepInput {
        player: Vec3 {
            x: 0.0,
            y: 0.9,
            z: 0.0,
        },
        player_velocity: Vec3::default(),
        target,
        target_velocity: Vec3::default(),
        body: Body::Anchor,
        direction,
    };
    let below = Vec3 {
        x: 0.0,
        y: -1.5,
        z: 0.0,
    };
    let side = Vec3 {
        x: 3.0,
        y: 0.9,
        z: 0.0,
    };
    let lift = |rules: &ForceRules, target, direction| {
        physics::step(rules, &input(target, direction))
            .unwrap()
            .player_delta
    };
    let plain = lift(&ForceRules::BURNING, below, Direction::Push);
    let boosted = lift(&gained, below, Direction::Push);
    assert!(boosted.y > plain.y);
    assert_eq!(
        lift(&gained, side, Direction::Push),
        lift(&ForceRules::BURNING, side, Direction::Push)
    );
    assert_eq!(
        lift(&gained, below, Direction::Pull),
        lift(&ForceRules::BURNING, below, Direction::Pull)
    );
    let weaker = ForceRules {
        push_below_gain: 0.5,
        ..ForceRules::BURNING
    };
    assert!(!weaker.is_valid());
}

// Catches schema 2 states the operations can never produce (Codex review, P5 final).
#[test]
fn impossible_windows_and_conclusions_are_rejected_on_decode() {
    // Window counts: anchor_move's window is the first of the three, after the research.
    let window_count_at = RESEARCH_AT + 5 + 8;
    let saved = learned();
    assert_eq!(saved.progress().windows()[0].count, 1);
    assert_eq!(
        decode(&edited(|b| b[window_count_at] = 4)),
        Err(SaveError::Progress(ProgressError::WindowOutOfRange))
    );
    assert!(decode(&edited(|b| b[window_count_at] = 3)).is_ok());
    // A conclusion worth 200 experience with less experience saved.
    let xp = edited(|b| b[PROGRESS_AT..PROGRESS_AT + 4].copy_from_slice(&199u32.to_le_bytes()));
    assert!(matches!(decode(&xp), Err(SaveError::Progress(_))));
}

#[test]
fn evidence_needs_the_pair_but_hints_alone_do_not() {
    let mut saved = fresh();
    commit(&mut saved, |s| prepare_hint(s, RESEARCH_ANCHOR, 1));
    assert_eq!(saved.progress().research(RESEARCH_ANCHOR).hints, 1);
    assert_eq!(decode(&encode_save(&saved).unwrap()).unwrap(), saved);
}

#[test]
fn an_unused_window_with_a_start_is_rejected() {
    // The third window (coin_hit) is unused in `learned()`: give it a start and no count.
    let start_at = RESEARCH_AT + 5 + 2 * 9;
    assert_eq!(learned().progress().windows()[2].count, 0);
    assert_eq!(
        decode(&edited(|b| b[start_at] = 7)),
        Err(SaveError::Progress(ProgressError::WindowOutOfRange))
    );
}
