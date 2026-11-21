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

/// A save with non-trivial values in every field the codec writes.
fn rich() -> SavedPlayer {
    let mut saved = fresh();
    update_player(&mut saved, |state, accounting| {
        state.selected_pair = Pair::ZincBrass;
        for (i, metal) in state.metals.iter_mut().enumerate() {
            metal.unlocked = i % 2 == 0;
            metal.reserve_milli = 1_000 * (i as u64 + 1);
        }
        state.metals[0].burn = BurnState::Burning;
        state.metals[2].burn = BurnState::Flared;
        accounting.remainder = [0, 1, 2, 3, 4, 5, 6, 999];
        Ok(())
    })
    .unwrap();
    let prepared = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    saved
}

fn decode(bytes: &[u8]) -> Result<SavedPlayer, SaveError> {
    decode_save(bytes, &world(), &player())
}

/// Re-seals a hand-edited save with a correct checksum, so only the edit is under test.
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

// Catches fields dropped or reordered by the codec; last_tick is deliberately not saved.
#[test]
fn round_trip_keeps_every_saved_field_and_drops_session_tick() {
    let mut saved = rich();
    update_player(&mut saved, |state, _| {
        state.last_tick = Some(Tick(77));
        Ok(())
    })
    .unwrap();
    let bytes = encode_save(&saved).unwrap();
    let decoded = decode(&bytes).unwrap();
    assert_eq!(decoded.state().last_tick, None);
    assert_eq!(decoded.state().metals, saved.state().metals);
    assert_eq!(decoded.state().selected_pair, Pair::ZincBrass);
    assert_eq!(decoded.accounting(), saved.accounting());
    assert_eq!(decoded.rules(), saved.rules());
    assert_eq!(decoded.revision(), saved.revision());
    assert_eq!(decoded.last_transaction(), 1);
    assert_eq!(
        encode_save(&decoded).unwrap(),
        bytes,
        "encoding is deterministic"
    );
}

// Catches accepting a save cut short at any byte (a partial write).
#[test]
fn every_truncation_is_rejected() {
    let bytes = encode_save(&rich()).unwrap();
    for len in 0..bytes.len() {
        let result = decode(&bytes[..len]);
        assert!(
            matches!(result, Err(SaveError::Truncated | SaveError::BadMagic)),
            "len {len}: {result:?}"
        );
    }
}

// Catches silently accepting flipped bits anywhere in the payload.
#[test]
fn every_single_bit_flip_is_rejected() {
    let bytes = encode_save(&rich()).unwrap();
    for i in 0..bytes.len() {
        for bit in 0..8 {
            let mut corrupt = bytes.clone();
            corrupt[i] ^= 1 << bit;
            assert!(decode(&corrupt).is_err(), "byte {i} bit {bit} accepted");
        }
    }
}

#[test]
fn trailing_bytes_magic_and_checksum_are_distinct_errors() {
    let bytes = encode_save(&rich()).unwrap();
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(decode(&longer), Err(SaveError::TrailingBytes));
    let mut magic = bytes.clone();
    magic[0] = b'X';
    assert_eq!(decode(&magic), Err(SaveError::BadMagic));
    let mut crc = bytes.clone();
    *crc.last_mut().unwrap() ^= 0xFF;
    assert_eq!(decode(&crc), Err(SaveError::ChecksumMismatch));
    assert_eq!(
        decode(&vec![0; MAX_SAVE_BYTES + 1]),
        Err(SaveError::TooLarge)
    );
}

// Catches reading a future (or pre-release) schema with today's layout.
#[test]
fn other_schemas_are_rejected_before_reading_the_payload() {
    let bytes = encode_save(&rich()).unwrap();
    for schema in [0u16, 2, u16::MAX] {
        let mut other = bytes.clone();
        other[4..6].copy_from_slice(&schema.to_le_bytes());
        let other = reseal(other);
        assert_eq!(save_schema(&other), Some(schema));
        assert_eq!(decode(&other), Err(SaveError::UnsupportedVersion(schema)));
    }
}

#[test]
fn unknown_enum_values_are_rejected() {
    let bytes = encode_save(&fresh()).unwrap();
    // Header 10 bytes, then player id (2 + 32), world id (2 + 5), then the pair byte.
    let pair_at = 10 + 2 + 32 + 2 + 5;
    let mut bad_pair = bytes.clone();
    bad_pair[pair_at] = 4;
    assert_eq!(decode(&reseal(bad_pair)), Err(SaveError::InvalidEnum));
    let mut bad_unlock = bytes.clone();
    bad_unlock[pair_at + 1] = 2;
    assert_eq!(decode(&reseal(bad_unlock)), Err(SaveError::InvalidEnum));
    let mut bad_burn = bytes.clone();
    bad_burn[pair_at + 1 + 1 + 8] = 3;
    assert_eq!(decode(&reseal(bad_burn)), Err(SaveError::InvalidEnum));
}

// Catches loading another player's (or world's) data into this player.
#[test]
fn identity_must_match_exactly() {
    let bytes = encode_save(&rich()).unwrap();
    assert_eq!(
        decode_save(&bytes, &WorldId::new("other"), &player()),
        Err(SaveError::WorldMismatch)
    );
    assert_eq!(
        decode_save(
            &bytes,
            &world(),
            &PlayerId::new("0123456789ABCDEF0123456789ABCDEF")
        ),
        Err(SaveError::PlayerMismatch)
    );
    assert_eq!(
        SavedPlayer::new(PlayerId::new(""), world(), rules()),
        Err(SaveError::InvalidIdentity)
    );
    assert_eq!(
        SavedPlayer::new(PlayerId::new("a\nb"), world(), rules()),
        Err(SaveError::InvalidIdentity)
    );
    assert_eq!(
        SavedPlayer::new(
            PlayerId::new("x".repeat(MAX_ID_BYTES + 1)),
            world(),
            rules()
        ),
        Err(SaveError::InvalidIdentity)
    );
}

// Catches decoding a structurally valid but economically impossible save.
#[test]
fn decoded_reserves_are_fully_validated() {
    let bytes = encode_save(&fresh()).unwrap();
    // First metal's reserve starts after the pair byte and the unlocked byte.
    let reserve_at = 10 + 2 + 32 + 2 + 5 + 1 + 1;
    let mut above = bytes.clone();
    above[reserve_at..reserve_at + 8].copy_from_slice(&200_000u64.to_le_bytes());
    assert_eq!(
        decode(&reseal(above)),
        Err(SaveError::InvalidReserves(
            ReserveError::ReserveAboveCapacity(Metal::Iron)
        ))
    );
}

#[test]
fn transaction_counter_cannot_exceed_revision() {
    let bytes = encode_save(&fresh()).unwrap();
    let mut bad = bytes.clone();
    let last = bad.len() - 4 - 8;
    bad[last..last + 8].copy_from_slice(&1u64.to_le_bytes());
    assert_eq!(decode(&reseal(bad)), Err(SaveError::InvalidCounters));
}

// Catches a credit applied twice: commit once, then replay the same preparation.
#[test]
fn a_prepared_credit_commits_exactly_once() {
    let mut saved = fresh();
    let first = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    let replay = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    assert_eq!(first.seq(), TransactionSeq(1));
    assert_eq!(commit_credit(&mut saved, first), Ok(TransactionSeq(1)));
    let after = saved.clone();
    assert_eq!(commit_credit(&mut saved, replay), Err(SaveError::Duplicate));
    assert_eq!(saved, after);
    assert_eq!(saved.state().metals[0].reserve_milli, 25_000);
    assert_eq!(saved.last_transaction(), 1);
}

// Catches committing a credit computed before an unrelated change (lost update).
#[test]
fn a_preparation_from_an_older_revision_is_stale() {
    let mut saved = fresh();
    let prepared = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    on_death(&mut saved).unwrap();
    let after = saved.clone();
    assert_eq!(
        commit_credit(&mut saved, prepared),
        Err(SaveError::StalePreparation)
    );
    assert_eq!(saved, after);
}

#[test]
fn preparing_never_changes_the_save_and_rejects_invalid_credits() {
    let saved = fresh();
    let before = saved.clone();
    let prepared = prepare_credit(&saved, Metal::Steel, 1).unwrap();
    assert_eq!(saved, before);
    assert_eq!(prepared.candidate().state().metals[1].reserve_milli, 1);
    assert_eq!(prepared.candidate().last_transaction(), 1);
    assert_eq!(
        prepare_credit(&saved, Metal::Iron, -1),
        Err(SaveError::InvalidReserves(ReserveError::NegativeCredit))
    );
    assert_eq!(
        prepare_credit(&saved, Metal::Iron, 100_001),
        Err(SaveError::InvalidReserves(ReserveError::CapacityExceeded(
            Metal::Iron
        )))
    );
}

// Simulated cut before the host commit: only the old bytes exist, nothing was applied.
// Simulated cut after: the new bytes hold exactly one application, and a retry is refused.
#[test]
fn cut_before_or_after_commit_yields_old_or_new_never_both() {
    let mut saved = fresh();
    let old_bytes = encode_save(&saved).unwrap();
    let prepared = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    let new_bytes = encode_save(prepared.candidate()).unwrap();

    // Cut before: the game kept the old file; loading it shows no credit and the same
    // transaction can be prepared again as seq 1.
    let reloaded_old = decode(&old_bytes).unwrap();
    assert_eq!(reloaded_old.state().metals[0].reserve_milli, 0);
    assert_eq!(
        prepare_credit(&reloaded_old, Metal::Iron, 25_000)
            .unwrap()
            .seq(),
        TransactionSeq(1)
    );

    // Cut after: the new file has the credit once; a replay of the in-flight preparation
    // against the reloaded save is a duplicate.
    let mut reloaded_new = decode(&new_bytes).unwrap();
    assert_eq!(reloaded_new.state().metals[0].reserve_milli, 25_000);
    assert_eq!(reloaded_new.last_transaction(), 1);
    assert_eq!(
        commit_credit(&mut reloaded_new, prepared),
        Err(SaveError::Duplicate)
    );
    assert_eq!(reloaded_new.state().metals[0].reserve_milli, 25_000);

    // Committing in memory yields exactly the bytes that were written.
    let prepared = prepare_credit(&saved, Metal::Iron, 25_000).unwrap();
    commit_credit(&mut saved, prepared).unwrap();
    assert_eq!(encode_save(&saved).unwrap(), new_bytes);
}

#[test]
fn death_clears_fuel_but_keeps_unlocks_and_counters() {
    let mut saved = rich();
    let unlocked: Vec<bool> = saved.state().metals.iter().map(|m| m.unlocked).collect();
    let revision = saved.revision();
    on_death(&mut saved).unwrap();
    assert!(saved.state().metals.iter().all(|m| m.reserve_milli == 0));
    assert!(
        saved
            .state()
            .metals
            .iter()
            .all(|m| m.burn == BurnState::Off)
    );
    assert_eq!(saved.accounting(), &ReserveAccounting::default());
    let after: Vec<bool> = saved.state().metals.iter().map(|m| m.unlocked).collect();
    assert_eq!(after, unlocked);
    assert_eq!(saved.last_transaction(), 1);
    assert_eq!(saved.revision(), revision + 1);
    let cleared = saved.state().clone();
    on_death(&mut saved).unwrap();
    assert_eq!(saved.state(), &cleared, "death is idempotent in state");
}

#[test]
fn reconnect_stops_burns_and_keeps_reserves_and_remainders() {
    let mut saved = rich();
    let reserves: Vec<u64> = saved
        .state()
        .metals
        .iter()
        .map(|m| m.reserve_milli)
        .collect();
    let remainders = saved.accounting().clone();
    on_reconnect(&mut saved).unwrap();
    assert!(
        saved
            .state()
            .metals
            .iter()
            .all(|m| m.burn == BurnState::Off)
    );
    let after: Vec<u64> = saved
        .state()
        .metals
        .iter()
        .map(|m| m.reserve_milli)
        .collect();
    assert_eq!(after, reserves);
    assert_eq!(saved.accounting(), &remainders);
}

// Catches a failing update leaving half-applied state.
#[test]
fn rejected_updates_and_rule_changes_leave_the_save_untouched() {
    let mut saved = rich();
    let before = saved.clone();
    assert!(
        update_player(&mut saved, |state, _| {
            state.metals[1].burn = BurnState::Burning; // locked metal
            Ok(())
        })
        .is_err()
    );
    assert_eq!(saved, before);
    let mut smaller = rules();
    smaller.base_capacity_milli = [1; 8];
    assert!(replace_rules(&mut saved, smaller).is_err());
    assert_eq!(saved, before);
}

/// Rewrites the payload (bytes after the 10-byte header, before the CRC) and re-seals it with a
/// consistent declared length, so only the field-level decoding is under test.
fn with_payload(bytes: &[u8], edit: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let mut payload = bytes[10..bytes.len() - 4].to_vec();
    edit(&mut payload);
    let mut out = bytes[..6].to_vec();
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&[0; 4]);
    reseal(out)
}

// Catches field decoding that ignores leftover payload or reads past it.
#[test]
fn payload_longer_or_shorter_than_its_fields_is_rejected() {
    let bytes = encode_save(&rich()).unwrap();
    assert_eq!(
        decode(&with_payload(&bytes, |p| p.push(0))),
        Err(SaveError::LengthMismatch)
    );
    assert_eq!(
        decode(&with_payload(&bytes, |p| {
            p.pop();
        })),
        Err(SaveError::Truncated)
    );
}

// Catches lossy identity decoding (invalid UTF-8 or control characters accepted).
#[test]
fn malformed_identity_bytes_are_rejected_on_decode() {
    let bytes = encode_save(&fresh()).unwrap();
    // Player id text starts after its 2-byte length.
    let bad_utf8 = with_payload(&bytes, |p| p[2] = 0xFF);
    assert_eq!(decode(&bad_utf8), Err(SaveError::InvalidIdentity));
    let control = with_payload(&bytes, |p| p[2] = b'\n');
    assert_eq!(decode(&control), Err(SaveError::InvalidIdentity));
}

// Catches committing one player's preparation into another player's save.
#[test]
fn a_preparation_for_another_identity_is_stale() {
    let mine = fresh();
    let mut other = SavedPlayer::new(
        PlayerId::new("ffffffffffffffffffffffffffffffff"),
        world(),
        rules(),
    )
    .unwrap();
    let prepared = prepare_credit(&mine, Metal::Iron, 1).unwrap();
    let before = other.clone();
    assert_eq!(
        commit_credit(&mut other, prepared),
        Err(SaveError::StalePreparation)
    );
    assert_eq!(other, before);
}

// Fixture for future migrations (P5 adds schema 2 and must load this file).
#[test]
fn schema_1_fixture_still_decodes() {
    let bytes = include_bytes!("fixtures/saved-player-v1.bin");
    assert_eq!(decode(bytes).unwrap(), rich());
}

#[test]
#[ignore = "regenerates tests/fixtures/saved-player-v1.bin; run only when schema 1 is defined anew"]
fn write_schema_1_fixture() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/saved-player-v1.bin"
    );
    std::fs::write(path, encode_save(&rich()).unwrap()).unwrap();
}
