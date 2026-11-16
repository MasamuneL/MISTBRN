use crate::save::{
    CreditFailpoint, CreditIntent, NativeTestConfig, PlayerStatus, SaveBackend, StorageError,
};
use allomancy_core::{Metal, TransactionId, WorldId, load_world_identity};
use horizon_client::mod_api::{Hooks, Host, Tag};
use std::{
    env, fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

static ATTEMPTED: AtomicBool = AtomicBool::new(false);

pub fn make_hooks() -> Box<dyn Hooks> {
    match config() {
        Ok((config, scenario)) => {
            eprintln!("ALLOMANCY_NATIVE_TEST event=enabled scenario={scenario}");
            let driver: fn(&SaveBackend, &mut dyn Host) = match scenario.as_str() {
                "observe" => observe,
                "roundtrip" => roundtrip,
                "replay" => replay,
                "rollback-inventory" => rollback_inventory,
                "rollback-inline" => rollback_inline,
                "quarantine" => quarantine,
                "cut-before-inventory" => cut_before_inventory,
                "cut-after-inventory" => cut_after_inventory,
                "cut-after-inline" => cut_after_inline,
                "cut-after-core-commit" => cut_after_core_commit,
                _ => unreachable!(),
            };
            crate::hooks_with_native_test(config, driver).0
        }
        Err(reason) => {
            eprintln!("ALLOMANCY_NATIVE_TEST event=disabled reason={reason}");
            crate::hooks_without_native_test()
        }
    }
}

fn config() -> Result<(NativeTestConfig, String), &'static str> {
    let runtime = env::var_os("ALLOMANCY_NATIVE_TEST_RUNTIME").ok_or("runtime_missing")?;
    let root = env::var_os("ALLOMANCY_NATIVE_TEST_ROOT").ok_or("root_missing")?;
    let id = env::var("ALLOMANCY_NATIVE_TEST_WORLD_ID").map_err(|_| "world_id_missing")?;
    let scenario = env::var("ALLOMANCY_NATIVE_TEST_SCENARIO").map_err(|_| "scenario_missing")?;
    if ![
        "observe",
        "roundtrip",
        "replay",
        "rollback-inventory",
        "rollback-inline",
        "quarantine",
        "cut-before-inventory",
        "cut-after-inventory",
        "cut-after-inline",
        "cut-after-core-commit",
    ]
    .contains(&scenario.as_str())
    {
        return Err("scenario_invalid");
    }
    let runtime = fs::canonicalize(runtime).map_err(|_| "runtime_invalid")?;
    let root: PathBuf = fs::canonicalize(root).map_err(|_| "root_invalid")?;
    if !runtime.is_dir() || !root.is_dir() {
        return Err("directory_invalid");
    }
    let relative = root
        .strip_prefix(&runtime)
        .map_err(|_| "root_outside_runtime")?;
    let components: Vec<_> = relative
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    let game = components
        .iter()
        .position(|c| c.starts_with("game-p2-") && c.len() > "game-p2-".len())
        .ok_or("game_dir_invalid")?;
    if !components.get(game + 1).is_some_and(|c| *c == "saves") || components.len() != game + 3 {
        return Err("saves_dir_invalid");
    }
    let world_id = WorldId::new(id);
    let marker = load_world_identity(&root)
        .map_err(|_| "marker_invalid")?
        .ok_or("marker_missing")?;
    if marker.world_id != world_id {
        return Err("world_id_mismatch");
    }
    let cut_run_token = if scenario.starts_with("cut-") {
        let token = env::var("ALLOMANCY_NATIVE_TEST_RUN_TOKEN").map_err(|_| "run_token_missing")?;
        if !valid_run_token(&token) {
            return Err("run_token_invalid");
        }
        let arm = root
            .join("allomancy")
            .join(format!("native-cut-arm-{token}"));
        match fs::symlink_metadata(arm) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => return Err("cut_arm_stale"),
            Err(_) => return Err("cut_arm_unreadable"),
        }
        Some(token)
    } else {
        None
    };
    Ok((
        NativeTestConfig {
            world_root: root,
            world_id,
            cut_run_token,
        },
        scenario,
    ))
}

fn valid_run_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 64
        && token
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

fn observe(_: &SaveBackend, _: &mut dyn Host) {}
fn roundtrip(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "roundtrip");
}
fn replay(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "replay");
}
fn rollback_inventory(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "rollback-inventory");
}
fn rollback_inline(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "rollback-inline");
}
fn quarantine(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "quarantine");
}

fn cut_before_inventory(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "cut-before-inventory");
}
fn cut_after_inventory(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "cut-after-inventory");
}
fn cut_after_inline(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "cut-after-inline");
}
fn cut_after_core_commit(b: &SaveBackend, h: &mut dyn Host) {
    run(b, h, "cut-after-core-commit");
}

fn run(backend: &SaveBackend, host: &mut dyn Host, scenario: &str) {
    if ATTEMPTED.load(Ordering::Acquire) {
        return;
    }
    let cut_token = scenario
        .starts_with("cut-")
        .then(|| backend.native_cut_run_token())
        .flatten();
    for player in host.server_players() {
        let uuid = player.uuid;
        let Some((before, generation)) = backend.native_test_state(uuid) else {
            continue;
        };
        let Some(stacks) = host.player_stacks(uuid) else {
            continue;
        };
        let entry = before.ledger().last().cloned();
        if scenario == "replay" && entry.is_none() {
            continue;
        }
        let slot_stack = stacks.iter().take(36).enumerate().find_map(|(i, s)| {
            s.as_ref()
                .filter(|s| {
                    s.count >= 1
                        && (cut_token.is_none() || s.item.as_ref() == "minecraft:iron_ingot")
                })
                .map(|s| (i, s.clone()))
        });
        if scenario != "replay" && slot_stack.is_none() {
            continue;
        }
        if ATTEMPTED.swap(true, Ordering::AcqRel) {
            return;
        }
        let result = (|| -> Result<(), &'static str> {
            let raw = host.player_compound(uuid).ok_or("raw_before")?;
            let mods = raw.get("NeoForgeData").cloned();
            let (slot, expected, replacement, transaction, metal, amount) = if scenario == "replay"
            {
                let e = entry.ok_or("ledger_missing")?;
                let stack = stacks.first().cloned().ok_or("inventory_empty")?;
                (
                    0,
                    stack.clone(),
                    stack,
                    e.transaction,
                    e.payload.metal,
                    e.payload.amount_milli,
                )
            } else {
                let (slot, stack) = slot_stack.ok_or("slot_missing")?;
                let mut reduced = stack.clone();
                reduced.count -= 1;
                let replacement = (reduced.count > 0).then_some(reduced);
                (
                    slot,
                    Some(stack),
                    replacement,
                    TransactionId::new(match cut_token {
                        Some(token) => format!("native-cut-{token}"),
                        None => format!("native-roundtrip-{}", before.revision()),
                    }),
                    Metal::Iron,
                    1000,
                )
            };
            let intent = || CreditIntent {
                player: uuid,
                generation,
                transaction: transaction.clone(),
                metal,
                amount_milli: amount,
                slot,
                expected: expected.clone(),
                replacement: replacement.clone(),
            };
            if scenario == "replay" {
                let e = before.ledger().last().ok_or("ledger_missing")?;
                let receipt = backend
                    .credit_for_native_test(host, intent(), CreditFailpoint::None)
                    .map_err(|_| "replay")?;
                if receipt.payload != e.payload
                    || receipt.committed_revision != e.committed_revision
                    || backend.native_test_state(uuid) != Some((before.clone(), generation))
                    || host.player_stacks(uuid) != Some(stacks.clone())
                    || host
                        .player_compound(uuid)
                        .and_then(|r| r.get("NeoForgeData").cloned())
                        != mods
                {
                    return Err("replay_verification");
                }
                return Ok(());
            }
            if before.ledger().iter().any(|e| e.transaction == transaction) {
                return Err("transaction_exists");
            }
            let cut_failpoint = match scenario {
                "cut-before-inventory" => Some(CreditFailpoint::PauseBeforeInventory),
                "cut-after-inventory" => Some(CreditFailpoint::PauseAfterInventory),
                "cut-after-inline" => Some(CreditFailpoint::PauseAfterInline),
                "cut-after-core-commit" => Some(CreditFailpoint::PauseAfterCoreCommit),
                _ => None,
            };
            if let Some(failpoint) = cut_failpoint {
                backend
                    .credit_for_native_test(host, intent(), failpoint)
                    .map_err(|_| "cut_credit")?;
                return Err("cut_not_reached");
            }
            let mut expected_stacks = stacks.clone();
            expected_stacks[slot] = replacement.clone();
            if scenario == "quarantine" {
                let failure = backend.credit_for_native_test(
                    host,
                    intent(),
                    CreditFailpoint::RollbackInventory,
                );
                let raw_after = host.player_compound(uuid).ok_or("raw_after")?;
                let Some(Tag::Compound(mut expected_mods)) = mods.clone() else {
                    return Err("mods_before");
                };
                expected_mods.insert(
                    "allomancy:storage_fault".into(),
                    Tag::compound([
                        ("format_version", Tag::Int(1)),
                        ("reason", Tag::String("rollback_unverified".into())),
                    ]),
                );
                if failure != Err(StorageError::RollbackFailed)
                    || backend.status(uuid) != Ok(PlayerStatus::Fault)
                    || backend.native_test_state(uuid).is_some()
                    || raw_after.get("NeoForgeData") != Some(&Tag::Compound(expected_mods))
                    || host.player_stacks(uuid) != Some(expected_stacks)
                {
                    return Err("quarantine_verification");
                }
                return Ok(());
            }
            let failpoint = match scenario {
                "rollback-inventory" => Some(CreditFailpoint::AfterInventory),
                "rollback-inline" => Some(CreditFailpoint::AfterInline),
                _ => None,
            };
            if let Some(failpoint) = failpoint {
                if backend.credit_for_native_test(host, intent(), failpoint)
                    != Err(StorageError::HostRejected)
                    || backend.status(uuid) != Ok(PlayerStatus::Ready)
                    || backend.native_test_state(uuid) != Some((before.clone(), generation))
                    || host.player_stacks(uuid) != Some(stacks.clone())
                    || host
                        .player_compound(uuid)
                        .and_then(|r| r.get("NeoForgeData").cloned())
                        != mods
                {
                    return Err("rollback_verification");
                }
            }
            let receipt = backend
                .credit_for_native_test(host, intent(), CreditFailpoint::None)
                .map_err(|_| "credit")?;
            let (after, after_generation) = backend
                .native_test_state(uuid)
                .ok_or("state_after_credit")?;
            let revision = before
                .revision()
                .checked_add(1)
                .ok_or("revision_overflow")?;
            let reserve = before.state().metals[Metal::Iron.index()]
                .reserve_milli
                .checked_add(1000)
                .ok_or("reserve_overflow")?;
            if after_generation != generation
                || after.revision() != revision
                || receipt.committed_revision != revision
                || receipt.payload.metal != Metal::Iron
                || receipt.payload.amount_milli != 1000
                || after.state().metals[Metal::Iron.index()].reserve_milli != reserve
                || after.ledger().len() != before.ledger().len() + 1
                || !before.ledger().iter().all(|e| after.ledger().contains(e))
                || !after.ledger().iter().any(|e| {
                    e.transaction == transaction
                        && e.payload == receipt.payload
                        && e.committed_revision == revision
                })
                || host.player_stacks(uuid) != Some(expected_stacks.clone())
            {
                return Err("credit_verification");
            }
            let replayed = backend
                .credit_for_native_test(host, intent(), CreditFailpoint::None)
                .map_err(|_| "replay")?;
            if replayed != receipt
                || backend.native_test_state(uuid) != Some((after, generation))
                || host.player_stacks(uuid) != Some(expected_stacks)
            {
                return Err("replay_verification");
            }
            Ok(())
        })();
        let induced = if scenario == "quarantine" || scenario.starts_with("rollback-") {
            " induced_adapter_failure=true"
        } else {
            ""
        };
        match result {
            Ok(()) => eprintln!(
                "ALLOMANCY_NATIVE_TEST scenario={scenario} result=PASS{induced} durable=false"
            ),
            Err(reason) => eprintln!(
                "ALLOMANCY_NATIVE_TEST scenario={scenario} result=FAIL reason={reason}{induced} durable=false"
            ),
        }
        return;
    }
}
