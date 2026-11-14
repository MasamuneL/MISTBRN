use allomancy_core::*;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn rules() -> ReserveRules {
    ReserveRules {
        consumption_milli_per_tick: [1; 8],
        base_capacity_milli: [10_000; 8],
        reduction_milli: 200,
        capacity_bonus_milli: 0,
    }
}

fn fresh() -> SavedPlayer {
    SavedPlayer::new(PlayerId::new("player-1"), WorldId::new("world-1"), rules()).unwrap()
}

fn funded() -> SavedPlayer {
    let mut saved = fresh();
    update_player(&mut saved, |state, accounting| {
        state.selected_pair = Pair::CopperBronze;
        state.last_tick = Some(Tick(19));
        for metal in Metal::ALL {
            let i = metal.index();
            state.metals[i].unlocked = true;
            state.metals[i].reserve_milli = 100 + i as u64;
            state.metals[i].burn = if i % 2 == 0 {
                BurnState::Burning
            } else {
                BurnState::Flared
            };
            accounting.remainder[i] = 123 + i as u16;
        }
        Ok(())
    })
    .unwrap();
    saved
}

fn pending(saved: &SavedPlayer, id: &str, metal: Metal, amount: i64) -> PreparedCredit {
    match prepare_credit(saved, TransactionId::new(id), metal, amount).unwrap() {
        CreditPreparation::Pending(prepared) => prepared,
        CreditPreparation::Replay(_) => panic!("a new ID must prepare a candidate"),
    }
}

fn decode(bytes: &[u8]) -> Result<SavedPlayer, SaveError> {
    decode_save(bytes, &WorldId::new("world-1"), &PlayerId::new("player-1"))
}

fn wire(saved: &SavedPlayer) -> Value {
    serde_json::from_slice(&encode_save(saved).unwrap()).unwrap()
}

fn decode_value(value: Value) -> Result<SavedPlayer, SaveError> {
    decode(&serde_json::to_vec(&value).unwrap())
}

#[test]
fn round_trip_keeps_fractional_fuel_burns_tick_rules_and_receipt() {
    let mut saved = funded();
    let prepared = pending(&saved, "drink-1", Metal::Iron, 37);
    commit_credit(&mut saved, prepared).unwrap();
    let bytes = encode_save(&saved).unwrap();
    assert_eq!(decode(&bytes).unwrap(), saved);
    assert_eq!(encode_save(&decode(&bytes).unwrap()).unwrap(), bytes);
    assert_eq!(saved.state().metals[0].reserve_milli, 137);
    assert_eq!(saved.accounting().remainder[0], 123);
    assert_eq!(saved.ledger().len(), 1);
}

#[test]
fn future_schema_is_detected_before_its_incompatible_payload() {
    assert_eq!(
        decode(br#"{"schema_version":99,"other_future_data":true}"#),
        Err(SaveError::UnsupportedVersion(99))
    );
    assert_eq!(
        decode(br#"{"schema_version":0,"state":false}"#),
        Err(SaveError::UnsupportedVersion(0))
    );
    assert!(decode(br#"{"schema_version":1,"schema_version":99}"#).is_err());
}

#[test]
fn truncated_extra_tokens_wrong_numbers_enums_and_arrays_are_rejected() {
    let bytes = encode_save(&funded()).unwrap();
    assert!(decode(&bytes[..bytes.len() - 1]).is_err());
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b" true");
    assert!(decode(&trailing).is_err());
    for (pointer, value) in [
        ("/saved_player/state/metals/0/reserve_milli", json!(-1)),
        ("/saved_player/state/metals/0/reserve_milli", json!(1.5)),
        ("/saved_player/state/metals/0/burn", json!("FutureBurn")),
        ("/saved_player/state/metals", json!([])),
        ("/saved_player/accounting/remainder", json!(vec![0; 7])),
    ] {
        let mut value_wire = wire(&fresh());
        *value_wire.pointer_mut(pointer).unwrap() = value;
        assert!(decode_value(value_wire).is_err(), "accepted {pointer}");
    }
    assert!(decode(br#"{"schema_version":18446744073709551616}"#).is_err());
    assert_eq!(
        decode(&vec![b' '; MAX_SAVE_BYTES + 1]),
        Err(SaveError::InputTooLarge)
    );
}

#[test]
fn all_nested_objects_reject_unknown_and_duplicate_fields() {
    for pointer in [
        "",
        "/saved_player",
        "/saved_player/state",
        "/saved_player/state/metals/0",
        "/saved_player/accounting",
        "/saved_player/rules",
    ] {
        let mut value = wire(&fresh());
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("future_field".into(), json!(true));
        assert!(
            decode_value(value).is_err(),
            "accepted unknown at {pointer}"
        );
    }
    let raw = String::from_utf8(encode_save(&fresh()).unwrap()).unwrap();
    let duplicate = raw.replacen("\"revision\":0", "\"revision\":0,\"revision\":0", 1);
    assert!(decode(duplicate.as_bytes()).is_err());
    let mut missing = wire(&fresh());
    missing["saved_player"]
        .as_object_mut()
        .unwrap()
        .remove("accounting");
    assert!(decode_value(missing).is_err());
}

#[test]
fn identities_are_bound_and_never_normalized_or_recreated() {
    let bytes = encode_save(&fresh()).unwrap();
    assert_eq!(
        decode_save(&bytes, &WorldId::new("other"), &PlayerId::new("player-1")),
        Err(SaveError::WorldMismatch)
    );
    assert_eq!(
        decode_save(&bytes, &WorldId::new("world-1"), &PlayerId::new("other")),
        Err(SaveError::PlayerMismatch)
    );
    for bad in [String::new(), "x".repeat(257), "bad\nID".into()] {
        assert!(
            SavedPlayer::new(PlayerId::new(bad.clone()), WorldId::new("world-1"), rules()).is_err()
        );
        let mut value = wire(&fresh());
        value["saved_player"]["state"]["world_id"] = json!(bad);
        assert!(decode_value(value).is_err());
    }
    let unusual = SavedPlayer::new(
        PlayerId::new(" Player Ω "),
        WorldId::new("World A"),
        rules(),
    )
    .unwrap();
    let restored = decode_save(
        &encode_save(&unusual).unwrap(),
        &WorldId::new("World A"),
        &PlayerId::new(" Player Ω "),
    )
    .unwrap();
    assert_eq!(restored.state().player_id.as_str(), " Player Ω ");
}

#[test]
fn codec_validates_rules_and_reserves_on_both_input_and_output() {
    for (pointer, bad) in [
        ("/rules/reduction_milli", json!(201)),
        ("/rules/capacity_bonus_milli", json!(501)),
        ("/rules/consumption_milli_per_tick/7", json!(0)),
        ("/rules/base_capacity_milli/7", json!(0)),
        ("/accounting/remainder/0", json!(1000)),
        ("/state/metals/0/reserve_milli", json!(10001)),
        ("/state/metals/0/burn", json!("Burning")),
    ] {
        let mut value = wire(&fresh());
        *value["saved_player"].pointer_mut(pointer).unwrap() = bad;
        assert!(
            decode_value(value.clone()).is_err(),
            "decode accepted {pointer}"
        );
        let typed: SavedPlayer = serde_json::from_value(value["saved_player"].clone()).unwrap();
        assert!(encode_save(&typed).is_err(), "encode accepted {pointer}");
    }
    let mut empty_with_fraction = wire(&fresh());
    empty_with_fraction["saved_player"]["accounting"]["remainder"][0] = json!(1);
    assert!(decode_value(empty_with_fraction).is_err());
}

#[test]
fn prepare_contains_complete_candidate_but_does_not_accept_it() {
    let saved = funded();
    let original = saved.clone();
    let prepared = pending(&saved, "drink", Metal::Steel, 9);
    assert_eq!(saved, original);
    assert_eq!(prepared.candidate().state().metals[1].reserve_milli, 110);
    assert_eq!(prepared.candidate().revision(), saved.revision() + 1);
    assert_eq!(prepared.candidate().ledger().len(), 1);
    assert_eq!(prepared.receipt().committed_revision, saved.revision() + 1);
    assert!(prepared.is_current(&saved));
    let stored_candidate = decode(&encode_save(prepared.candidate()).unwrap()).unwrap();
    assert_eq!(stored_candidate, *prepared.candidate());
}

#[test]
fn discarded_preparation_leaves_id_available_and_crash_snapshots_replay_once() {
    let mut saved = fresh();
    let old_snapshot = encode_save(&saved).unwrap();
    let failed_host = pending(&saved, "vial", Metal::Iron, 40);
    drop(failed_host);
    assert_eq!(saved.ledger().len(), 0);
    let accepted = pending(&saved, "vial", Metal::Iron, 40);
    let new_snapshot = encode_save(accepted.candidate()).unwrap();
    commit_credit(&mut saved, accepted).unwrap();
    let mut recovered_old = decode(&old_snapshot).unwrap();
    let before_commit = pending(&recovered_old, "vial", Metal::Iron, 40);
    commit_credit(&mut recovered_old, before_commit).unwrap();
    assert_eq!(recovered_old.state().metals[0].reserve_milli, 40);
    let recovered_new = decode(&new_snapshot).unwrap();
    assert!(matches!(
        prepare_credit(&recovered_new, TransactionId::new("vial"), Metal::Iron, 40),
        Ok(CreditPreparation::Replay(_))
    ));
    assert_eq!(recovered_new, saved);
}

#[test]
fn replay_after_reload_keeps_original_receipt_and_conflicts_do_not_mutate() {
    let mut saved = fresh();
    let prepared = pending(&saved, "same", Metal::Iron, 17);
    let receipt = commit_credit(&mut saved, prepared).unwrap();
    let mut restored = decode(&encode_save(&saved).unwrap()).unwrap();
    on_reconnect(&mut restored).unwrap();
    let before = restored.clone();
    assert_eq!(
        prepare_credit(&restored, TransactionId::new("same"), Metal::Iron, 17).unwrap(),
        CreditPreparation::Replay(receipt)
    );
    for (metal, amount) in [(Metal::Steel, 17), (Metal::Iron, 18)] {
        assert_eq!(
            prepare_credit(&restored, TransactionId::new("same"), metal, amount),
            Err(SaveError::TransactionConflict)
        );
    }
    assert_eq!(restored, before);
}

#[test]
fn negative_overflow_capacity_and_bad_transaction_reject_without_acceptance() {
    let mut saved = funded();
    let before = saved.clone();
    for amount in [-1, i64::MIN, 10_000] {
        assert!(
            prepare_credit(&saved, TransactionId::new("invalid"), Metal::Iron, amount).is_err()
        );
        assert_eq!(saved, before);
    }
    assert_eq!(
        prepare_credit(&saved, TransactionId::new(""), Metal::Iron, 1),
        Err(SaveError::InvalidIdentity)
    );
    let zero = pending(&saved, "zero", Metal::Iron, 0);
    commit_credit(&mut saved, zero).unwrap();
    assert_eq!(saved.state(), before.state());
    assert_eq!(saved.revision(), before.revision() + 1);
    let mut huge = fresh();
    let mut huge_rules = rules();
    huge_rules.base_capacity_milli = [u64::MAX; 8];
    replace_rules(&mut huge, huge_rules).unwrap();
    update_player(&mut huge, |state, _| {
        state.metals[0].reserve_milli = u64::MAX;
        Ok(())
    })
    .unwrap();
    assert!(matches!(
        prepare_credit(&huge, TransactionId::new("overflow"), Metal::Iron, 1),
        Err(SaveError::InvalidReserves(
            ReserveError::ArithmeticOverflow(Metal::Iron)
        ))
    ));
}

#[test]
fn ledger_saturation_preserves_old_replay_and_rejects_new_ids() {
    let mut value = wire(&fresh());
    value["saved_player"]["revision"] = json!(MAX_LEDGER_ENTRIES);
    value["saved_player"]["ledger"] = Value::Array((0..MAX_LEDGER_ENTRIES).map(|i| json!({"transaction":format!("tx-{i:04}"),"payload":{"metal":"Iron","amount_milli":0},"committed_revision":i+1})).collect());
    let saved = decode_value(value.clone()).unwrap();
    assert!(matches!(
        prepare_credit(&saved, TransactionId::new("tx-0000"), Metal::Iron, 0),
        Ok(CreditPreparation::Replay(_))
    ));
    assert_eq!(
        prepare_credit(&saved, TransactionId::new("new-id"), Metal::Iron, 1),
        Err(SaveError::LedgerFull)
    );
    value["saved_player"]["ledger"].as_array_mut().unwrap().push(json!({"transaction":"zz-extra","payload":{"metal":"Iron","amount_milli":0},"committed_revision":MAX_LEDGER_ENTRIES+1}));
    value["saved_player"]["revision"] = json!(MAX_LEDGER_ENTRIES + 1);
    assert!(decode_value(value).is_err());
}

#[test]
fn ledger_rejects_duplicate_ids_revisions_unknown_payload_and_bad_order() {
    let entry = json!({"transaction":"a","payload":{"metal":"Iron","amount_milli":0},"committed_revision":1});
    for entries in [
        json!([entry.clone(), entry.clone()]),
        json!([entry.clone(), {"transaction":"b","payload":{"metal":"Iron","amount_milli":0},"committed_revision":1}]),
        json!([{ "transaction":"z","payload":{"metal":"Iron","amount_milli":0},"committed_revision":2}, entry.clone()]),
        json!([{ "transaction":"a","payload":{"metal":"Iron","amount_milli":-1},"committed_revision":1}]),
        json!([{ "transaction":"a","payload":{"metal":"Iron","amount_milli":0},"committed_revision":0}]),
        json!([{ "transaction":"a","payload":{"metal":"Iron","amount_milli":0},"committed_revision":3}]),
        json!([{ "transaction":"a","payload":{"metal":"Iron","amount_milli":0,"future":true},"committed_revision":1}]),
        json!([{ "transaction":"a","payload":{"metal":"Iron","amount_milli":0},"committed_revision":1,"future":true}]),
    ] {
        let mut value = wire(&fresh());
        value["saved_player"]["revision"] = json!(2);
        value["saved_player"]["ledger"] = entries;
        assert!(decode_value(value).is_err());
    }
}

#[test]
fn revision_invalidates_preparation_after_noop_aba_consumption_and_rules_change() {
    let mut saved = funded();
    let stale = pending(&saved, "pending", Metal::Iron, 2);
    let initial = saved.revision();
    update_player(&mut saved, |_, _| Ok(())).unwrap();
    assert_eq!(saved.revision(), initial + 1);
    let after_noop = saved.clone();
    assert_eq!(
        commit_credit(&mut saved, stale),
        Err(SaveError::StalePreparation)
    );
    assert_eq!(saved, after_noop);
    let stale = pending(&saved, "pending", Metal::Iron, 2);
    let pair = saved.state().selected_pair;
    update_player(&mut saved, |state, _| {
        state.selected_pair = Pair::IronSteel;
        Ok(())
    })
    .unwrap();
    update_player(&mut saved, |state, _| {
        state.selected_pair = pair;
        Ok(())
    })
    .unwrap();
    assert!(!stale.is_current(&saved));
    assert_eq!(
        commit_credit(&mut saved, stale),
        Err(SaveError::StalePreparation)
    );
    let stale = pending(&saved, "pending", Metal::Iron, 2);
    let rules = saved.rules().clone();
    update_player(&mut saved, |state, accounting| {
        consume_reserves(state, accounting, &rules, 1)?;
        Ok(())
    })
    .unwrap();
    assert_eq!(saved.accounting().remainder[0], 923);
    assert_eq!(
        commit_credit(&mut saved, stale),
        Err(SaveError::StalePreparation)
    );
    let stale = pending(&saved, "pending", Metal::Iron, 2);
    replace_rules(&mut saved, rules).unwrap();
    assert_eq!(
        commit_credit(&mut saved, stale),
        Err(SaveError::StalePreparation)
    );
}

#[test]
fn rejected_mutations_rollback_all_fields_and_do_not_advance_revision() {
    let mut saved = funded();
    let before = saved.clone();
    assert_eq!(
        update_player(&mut saved, |state, accounting| {
            state.selected_pair = Pair::IronSteel;
            accounting.remainder[7] = 1000;
            Ok(())
        }),
        Err(SaveError::InvalidReserves(ReserveError::InvalidRemainder(
            Metal::Bronze
        )))
    );
    assert_eq!(saved, before);
    assert!(
        update_player(&mut saved, |state, _| {
            state.player_id = PlayerId::new("other");
            Ok(())
        })
        .is_err()
    );
    assert_eq!(saved, before);
    assert!(
        update_player(&mut saved, |state, _| {
            state.metals[0].reserve_milli = 1;
            Err(ReserveError::NegativeCredit)
        })
        .is_err()
    );
    assert_eq!(saved, before);
    let mut invalid_rules = rules();
    invalid_rules.base_capacity_milli = [1; 8];
    assert!(replace_rules(&mut saved, invalid_rules).is_err());
    assert_eq!(saved, before);
}

#[test]
fn death_and_reconnect_preserve_unlocks_ledger_and_revision_but_differ_on_fractions() {
    let mut saved = funded();
    let accepted = pending(&saved, "history", Metal::Iron, 3);
    commit_credit(&mut saved, accepted).unwrap();
    let before = saved.clone();
    let pending_reconnect = pending(&saved, "later", Metal::Iron, 2);
    on_reconnect(&mut saved).unwrap();
    assert_eq!(saved.accounting(), before.accounting());
    assert_eq!(saved.state().last_tick, None);
    assert_eq!(saved.revision(), before.revision() + 1);
    for metal in Metal::ALL {
        let i = metal.index();
        assert_eq!(
            saved.state().metals[i].reserve_milli,
            before.state().metals[i].reserve_milli
        );
        assert!(saved.state().metals[i].unlocked);
        assert_eq!(saved.state().metals[i].burn, BurnState::Off);
    }
    assert_eq!(
        commit_credit(&mut saved, pending_reconnect),
        Err(SaveError::StalePreparation)
    );
    let pending_death = pending(&saved, "later", Metal::Iron, 2);
    on_death(&mut saved).unwrap();
    assert_eq!(saved.accounting().remainder, [0; 8]);
    assert_eq!(saved.state().selected_pair, before.state().selected_pair);
    assert_eq!(saved.ledger(), before.ledger());
    assert_eq!(saved.revision(), before.revision() + 2);
    assert!(
        saved.state().metals.iter().all(|metal| metal.unlocked
            && metal.reserve_milli == 0
            && metal.burn == BurnState::Off)
    );
    assert_eq!(
        commit_credit(&mut saved, pending_death),
        Err(SaveError::StalePreparation)
    );
    on_death(&mut saved).unwrap();
    assert_eq!(saved.revision(), before.revision() + 3);
}

#[test]
fn revision_overflow_blocks_credit_lifecycle_and_mutators_without_changes() {
    let mut value = wire(&fresh());
    value["saved_player"]["revision"] = json!(u64::MAX);
    let mut saved = decode_value(value).unwrap();
    let before = saved.clone();
    assert_eq!(
        prepare_credit(&saved, TransactionId::new("new"), Metal::Iron, 1),
        Err(SaveError::RevisionOverflow)
    );
    assert_eq!(
        update_player(&mut saved, |_, _| Ok(())),
        Err(SaveError::RevisionOverflow)
    );
    assert_eq!(
        replace_rules(&mut saved, rules()),
        Err(SaveError::RevisionOverflow)
    );
    assert_eq!(on_death(&mut saved), Err(SaveError::RevisionOverflow));
    assert_eq!(on_reconnect(&mut saved), Err(SaveError::RevisionOverflow));
    assert_eq!(saved, before);
}

#[test]
fn fixture_v0_migration_is_explicit_offline_and_refuses_lost_economy() {
    let state = PlayerState::new(PlayerId::new("player-1"), WorldId::new("world-1"));
    let fixture = json!({"schema_version":0,"state":state,"rules":rules()});
    let bytes = serde_json::to_vec(&fixture).unwrap();
    assert_eq!(decode(&bytes), Err(SaveError::UnsupportedVersion(0)));
    let migrated =
        migrate_fixture_v0(&bytes, &WorldId::new("world-1"), &PlayerId::new("player-1")).unwrap();
    assert_eq!(migrated.revision(), 0);
    assert_eq!(migrated.accounting().remainder, [0; 8]);
    assert_eq!(decode(&encode_save(&migrated).unwrap()).unwrap(), migrated);
    for pointer in ["/state/metals/0/reserve_milli", "/state/last_tick"] {
        let mut unsafe_fixture = fixture.clone();
        *unsafe_fixture.pointer_mut(pointer).unwrap() = json!(1);
        assert!(
            migrate_fixture_v0(
                &serde_json::to_vec(&unsafe_fixture).unwrap(),
                &WorldId::new("world-1"),
                &PlayerId::new("player-1")
            )
            .is_err()
        );
    }
}

#[test]
fn v1_requires_last_tick_presence_but_accepts_explicit_null() {
    let mut value = wire(&fresh());
    assert_eq!(value["saved_player"]["state"]["last_tick"], Value::Null);
    assert_eq!(decode_value(value.clone()).unwrap().state().last_tick, None);
    value["saved_player"]["state"]
        .as_object_mut()
        .unwrap()
        .remove("last_tick");
    assert_eq!(decode_value(value), Err(SaveError::InvalidSchema));
}

#[test]
fn fixture_v0_requires_last_tick_presence_but_accepts_explicit_null() {
    let mut fixture = json!({"schema_version":0,"state":fresh().state(),"rules":rules()});
    let world = WorldId::new("world-1");
    let player = PlayerId::new("player-1");
    assert_eq!(fixture["state"]["last_tick"], Value::Null);
    assert_eq!(
        migrate_fixture_v0(&serde_json::to_vec(&fixture).unwrap(), &world, &player)
            .unwrap()
            .state()
            .last_tick,
        None
    );
    fixture["state"]
        .as_object_mut()
        .unwrap()
        .remove("last_tick");
    assert_eq!(
        migrate_fixture_v0(&serde_json::to_vec(&fixture).unwrap(), &world, &player),
        Err(SaveError::InvalidSchema)
    );
}

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

#[test]
fn json_structs_require_objects_instead_of_serde_positional_sequences() {
    for pointer in [
        "/saved_player",
        "/saved_player/state",
        "/saved_player/state/metals/0",
        "/saved_player/rules",
        "/saved_player/accounting",
    ] {
        let mut value = wire(&fresh());
        let object = value.pointer_mut(pointer).unwrap();
        let positional = match pointer {
            "/saved_player" => json!([
                object["state"],
                object["accounting"],
                object["rules"],
                object["revision"],
                object["ledger"]
            ]),
            "/saved_player/state" => json!([
                object["player_id"],
                object["world_id"],
                object["last_tick"],
                object["selected_pair"],
                object["metals"]
            ]),
            "/saved_player/state/metals/0" => {
                json!([object["unlocked"], object["reserve_milli"], object["burn"]])
            }
            "/saved_player/rules" => json!([
                object["consumption_milli_per_tick"],
                object["base_capacity_milli"],
                object["reduction_milli"],
                object["capacity_bonus_milli"]
            ]),
            _ => json!([object["remainder"]]),
        };
        *object = positional;
        assert!(
            decode_value(value).is_err(),
            "accepted positional struct at {pointer}"
        );
    }
}

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "allomancy-persistence-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self) -> PathBuf {
        self.0.join("player.json")
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn offline_export_replaces_complete_snapshot_and_preserves_valid_backup() {
    let dir = TempDir::new();
    let path = dir.file();
    let old = fresh();
    write_export_atomic(&path, &old).unwrap();
    let mut new = old.clone();
    let prepared = pending(&new, "commit", Metal::Iron, 23);
    commit_credit(&mut new, prepared).unwrap();
    write_export_atomic(&path, &new).unwrap();
    let loaded =
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1"))
            .unwrap();
    assert_eq!(loaded.source, ExportSource::Primary);
    assert_eq!(loaded.saved, new);
    assert_eq!(
        decode(&fs::read(path.with_extension("json.bak")).unwrap()).unwrap(),
        old
    );
    fs::remove_file(&path).unwrap();
    let recovered =
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1"))
            .unwrap();
    assert_eq!(recovered.source, ExportSource::Backup);
    assert_eq!(recovered.saved, old);
    assert!(!path.exists());
}

#[test]
fn offline_temp_and_backup_destination_errors_do_not_change_primary() {
    for blocked_suffix in ["json.tmp", "json.bak"] {
        let dir = TempDir::new();
        let path = dir.file();
        let old = fresh();
        write_export_atomic(&path, &old).unwrap();
        let original = fs::read(&path).unwrap();
        fs::create_dir(path.with_extension(blocked_suffix)).unwrap();
        let mut new = old.clone();
        on_reconnect(&mut new).unwrap();
        assert!(write_export_atomic(&path, &new).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }
}

#[test]
fn offline_recovery_does_not_overwrite_corrupt_or_future_files_or_downgrade() {
    let dir = TempDir::new();
    let path = dir.file();
    write_export_atomic(&path, &fresh()).unwrap();
    let mut next = fresh();
    on_reconnect(&mut next).unwrap();
    write_export_atomic(&path, &next).unwrap();
    assert!(write_export_atomic(&path, &fresh()).is_err());
    let future = br#"{"schema_version":9,"new_format":true}"#;
    fs::write(&path, future).unwrap();
    assert_eq!(
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1")),
        Err(SaveError::UnsupportedVersion(9))
    );
    assert!(write_export_atomic(&path, &next).is_err());
    assert_eq!(fs::read(&path).unwrap(), future);
    fs::write(&path, b"truncated{").unwrap();
    let recovered =
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1"))
            .unwrap();
    assert_eq!(recovered.source, ExportSource::Backup);
    assert_eq!(fs::read(&path).unwrap(), b"truncated{");
    assert!(write_export_atomic(&path, &next).is_err());
    fs::write(path.with_extension("json.bak"), b"also-corrupt").unwrap();
    assert!(
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1"))
            .is_err()
    );
}

#[test]
fn offline_export_preserves_a_future_backup_even_when_primary_is_valid() {
    let dir = TempDir::new();
    let path = dir.file();
    write_export_atomic(&path, &fresh()).unwrap();
    let old_primary = fs::read(&path).unwrap();
    let future = br#"{"schema_version":99,"future":true}"#;
    fs::write(path.with_extension("json.bak"), future).unwrap();
    let mut next = fresh();
    on_reconnect(&mut next).unwrap();
    assert_eq!(
        write_export_atomic(&path, &next),
        Err(SaveError::UnsupportedVersion(99))
    );
    assert_eq!(fs::read(&path).unwrap(), old_primary);
    assert_eq!(fs::read(path.with_extension("json.bak")).unwrap(), future);
}

#[cfg(windows)]
#[test]
fn offline_primary_rename_failure_keeps_old_snapshot_and_valid_backup() {
    use std::os::windows::fs::OpenOptionsExt;
    let dir = TempDir::new();
    let path = dir.file();
    let old = fresh();
    write_export_atomic(&path, &old).unwrap();
    // Allow readers/writers but deny FILE_SHARE_DELETE, so actual replacement fails.
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(&path)
        .unwrap();
    let mut new = old.clone();
    let prepared = pending(&new, "rename", Metal::Iron, 31);
    commit_credit(&mut new, prepared).unwrap();
    assert_eq!(write_export_atomic(&path, &new), Err(SaveError::Io));
    assert_eq!(decode(&fs::read(&path).unwrap()).unwrap(), old);
    assert_eq!(
        decode(&fs::read(path.with_extension("json.bak")).unwrap()).unwrap(),
        old
    );
    assert!(!path.with_extension("json.tmp").exists());
    drop(held);
    write_export_atomic(&path, &new).unwrap();
    assert_eq!(decode(&fs::read(&path).unwrap()).unwrap(), new);
}

#[test]
fn interrupted_partial_temporary_is_neither_promoted_nor_overwritten() {
    let dir = TempDir::new();
    let path = dir.file();
    let old = fresh();
    write_export_atomic(&path, &old).unwrap();
    let partial = b"{\"schema_version\":1,\"saved_player\":";
    fs::write(path.with_extension("json.tmp"), partial).unwrap();
    let recovered =
        read_export_with_backup(&path, &WorldId::new("world-1"), &PlayerId::new("player-1"))
            .unwrap();
    assert_eq!(recovered.saved, old);
    assert_eq!(recovered.source, ExportSource::Primary);
    let mut next = old.clone();
    on_reconnect(&mut next).unwrap();
    assert_eq!(write_export_atomic(&path, &next), Err(SaveError::Io));
    assert_eq!(fs::read(path.with_extension("json.tmp")).unwrap(), partial);
    assert_eq!(decode(&fs::read(&path).unwrap()).unwrap(), old);
}
