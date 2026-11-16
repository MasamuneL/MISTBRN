# P2 N7 writer controller — preparation, 2026-11-16

## Before implementation

Read CONTINUE-PROMPT, latest CHECKPOINT schema16/relevo, P2-native-writer-boundaries, the existing GDB helper, cuts controller/suite/oracle, frozen cuts1 native_test, and build receipt. HEAD 8bbdf739; branch codex/alomancia-p2-gates. Root alone owns builds, game runs, manifests, CHECKPOINT and reopen orchestration. This task writes only tools/test-native-writer.ps1, tools/inspect-native-writer.cjs and this document. No game, build, Cargo, SDK change, cleanup or new subagent was performed.

## Controller contract

Requires PowerShell 7.4+ (Start-Process -Environment; host observed 7.6.9), node, existing bundled GDB and pinned p2-cuts1 engine SHA256 8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70. The controller does not build, invoke the launcher or reuse GDB offsets with a newer engine.

Every new RunId reserves runtime/native-writer-runs/RunId; reuse fails. Before launch, result.json records inputs and events atomically via temporary + File.Replace/Move. It compares each frozen staging source against the cuts1 build receipt, using the separately recorded transformed adapter Cargo manifest hash. Current core, guard, descriptor and other inputs must still match. Only the three explicitly identified adapter Rust files may have newer WIP hashes; these are recorded as NOT compiled in cuts1. The known launcher hash B48F4913... is recorded as build-tool drift; the launcher is not executed.

SourceWorld must be an existing closed provisioned owner world under RuntimeRoot; WorldRoot must be a NEW runtime/game-p2-*/saves/name. Linked paths and source containing destination are rejected. The controller preserves a complete source backup in the run directory, copies that into WorldRoot, and compares every file hash. It launches only its own hidden integrated engine, with scenario roundtrip and a process-local environment.

At Ready it issues real control save, copies primary/old/new as baseline A, and requires A's inventory to be empty. It then issues ordinary command give @s minecraft:iron_ingot 3 and requires the compiled roundtrip driver's PASS. The resulting B consumes one item and credits 1000 Iron: two iron items in slot 0; SavedPlayer revision increments once; ledger gains native-roundtrip-<A revision>. No new native driver or synthetic save callback is involved. Disk A must remain byte-for-byte unchanged before and after attaching GDB.

The controller starts bundled GDB with redirected stdin/stdout/stderr, sources the helper, issues writer-arm OWNED_PID OWNER MODE RECEIPT and verifies armed PID, owner, mode, engine path/hash. It then continues and dispatches real save through its own asynchronous node control process. The blocked control helper has its existing ten-second network timeout; its response does not establish a boundary.

A pass requires both boundary.json and exactly one N7_BOUNDARY_HELD JSON record with identical content, the expected boundary, matching PID/path/hash, a nonempty list of all stopped threads belonging to that PID, all_stop=true, and successful NtWriteFile/rename return. The controller records the receipt before killing its original owned engine Process. Every kill additionally checks PID, process start time and executable path. It never resumes or detaches a held target. After target death it closes its own control/debugger processes and preserves all three player-file presence states and hashes. Failures retain result, logs, full backup, clone and available postmortem snapshots; a failed attempt stays failed and needs a new RunId.

## Offline oracle

The oracle derives B solely from A plus the specified give-three/consume-one/credit transaction. It compares the entire decoded SavedPlayer (all reserves, revisions, ticks, rules, unlocks and ledger), complete typed inventory, the entire foreign NeoForgeData namespace, allomancy state metadata, and envelope metadata. Ordinary world/player motion fields are outside this economic oracle. Legal empty NBT list child 0 is accepted.

| Boundary | Primary | Old | New |
| --- | --- | --- | --- |
| write-phase | A | exact baseline old, if present | B |
| after-primary-to-old | ABSENT | A | B |
| after-new-to-primary | B | A | ABSENT |

Every presence/absence is required, not merely tolerated. Captured B's compressed-file hash and byte count must also match the debugger's actual write buffer. A mixed inventory/payload/foreign namespace fails.

The absent-primary case remains absent. There is no old-to-primary substitution, repair or automatic adoption. Root must reopen a different COMPLETE copy: expect guard rejection for after-primary-to-old, acceptance of A for write-phase, B for after-new-to-primary. Preserve and compare the original cut clone manifest. For accepted reopen use oracle --mode reopen --expect A/B with the original baseline and baseline-old. Any later full restoration must explicitly restore a complete world backup to another new copy, never only player files.

## Root invocation

For each mode use unique run/world names, serially, with no game/build/debugger active:

    & ./tools/test-native-writer.ps1 -RuntimeRoot $runtime -RunId writer-write-phase-1 -SourceWorld "$runtime/backups/p2-owner-after-reopen" -WorldRoot "$runtime/game-p2-writer/saves/write-phase-1" -OwnerUuid 3efd2c0e-f0de-3e85-b045-dec265b3f79c -Mode write-phase

Repeat only the invocation structure for after-primary-to-old and after-new-to-primary, each with its own new RunId/WorldRoot. A controller pass is writer-boundary-plus-offline-oracle only; reopen remains explicitly pending in its receipt.

## Preparation checks after implementation

PowerShell AST parsed with zero errors; node --check and writer oracle --self-test passed. The oracle also passed read-only against the real p2-owner-after-reopen backup, whose baseline revision is 2 and whose empty inventory is legal child 0. That offline fixture predicts transaction native-roundtrip-2; the actual trial baseline after reconnect/save will determine its own revision and transaction.

Read-only staged/current comparison found no frozen-source drift. Current differences were precisely launcher plus adapter lib.rs, save.rs and native_test.rs. The host supports Start-Process -Environment. Earlier sandbox node invocation from the worktree failed Access denied before execution; rerunning from the permitted workspace using absolute paths passed. No native gate or debugger boundary was executed by this task.

Limits remain unchanged: write-phase is after a complete successful NtWriteFile and before fs::write close/return. It is not interruption inside kernel IO, not partial bytes, not fsync/power-loss durability, and does not alone close N7 or P2.

Final static GDB command check caught that source treats quoted filenames literally in this bundled build. Controller corrected to source plus the raw absolute helper path; the identical unquoted source command registered writer-arm successfully (exit 0). The known missing Python gdb package warning remains; helper uses built-in _gdb. No inferior was started or attached.
