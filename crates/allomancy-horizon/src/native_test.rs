use crate::save::{CreditFailpoint, CreditIntent, NativeTestConfig, SaveBackend};
use allomancy_core::{Metal, TransactionId, WorldId, load_world_identity};
use horizon_client::mod_api::{Hooks, Host};
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
            let driver = if scenario == "observe" {
                observe
            } else {
                roundtrip
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
    if scenario != "observe" && scenario != "roundtrip" {
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
    Ok((
        NativeTestConfig {
            world_root: root,
            world_id,
        },
        scenario,
    ))
}

fn observe(_: &SaveBackend, _: &mut dyn Host) {}

fn roundtrip(backend: &SaveBackend, host: &mut dyn Host) {
    if ATTEMPTED.load(Ordering::Acquire) {
        return;
    }
    for player in host.server_players() {
        let Some((before, generation)) = backend.native_test_state(player.uuid) else {
            continue;
        };
        let Some(stacks) = host.player_stacks(player.uuid) else {
            continue;
        };
        let Some((slot, stack)) = stacks.iter().take(36).enumerate().find_map(|(i, stack)| {
            stack
                .as_ref()
                .filter(|s| s.count >= 1)
                .map(|s| (i, s.clone()))
        }) else {
            continue;
        };
        if ATTEMPTED.swap(true, Ordering::AcqRel) {
            return;
        }
        let transaction = TransactionId::new(format!("native-roundtrip-{}", before.revision()));
        if before
            .ledger()
            .iter()
            .any(|entry| entry.transaction == transaction)
        {
            eprintln!(
                "ALLOMANCY_NATIVE_TEST scenario=roundtrip result=FAIL reason=transaction_exists"
            );
            return;
        }
        let mut reduced = stack.clone();
        reduced.count -= 1;
        let replacement = (reduced.count > 0).then_some(reduced);
        let intent = || CreditIntent {
            player: player.uuid,
            generation,
            transaction: transaction.clone(),
            metal: Metal::Iron,
            amount_milli: 1000,
            slot,
            expected: Some(stack.clone()),
            replacement: replacement.clone(),
        };
        let result = (|| -> Result<(), &'static str> {
            let receipt = backend
                .credit_for_native_test(host, intent(), CreditFailpoint::None)
                .map_err(|_| "credit")?;
            let (after, after_generation) = backend
                .native_test_state(player.uuid)
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
                || host
                    .player_stacks(player.uuid)
                    .and_then(|s| s.get(slot).cloned())
                    != Some(replacement.clone())
            {
                return Err("credit_verification");
            }
            let replay = backend
                .credit_for_native_test(host, intent(), CreditFailpoint::None)
                .map_err(|_| "replay")?;
            if replay != receipt
                || backend.native_test_state(player.uuid) != Some((after, generation))
                || host
                    .player_stacks(player.uuid)
                    .and_then(|s| s.get(slot).cloned())
                    != Some(replacement.clone())
            {
                return Err("replay_verification");
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                eprintln!("ALLOMANCY_NATIVE_TEST scenario=roundtrip result=PASS durable=false")
            }
            Err(reason) => eprintln!(
                "ALLOMANCY_NATIVE_TEST scenario=roundtrip result=FAIL reason={reason} durable=false"
            ),
        }
        return;
    }
}
