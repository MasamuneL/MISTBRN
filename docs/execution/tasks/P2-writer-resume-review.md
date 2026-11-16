# Writer resume review — 2026-11-16

Exclusive scope: tools/test-native-writer.ps1, tools/native-writer-debug.gdb, this report. No game, build or debugger was started. No checkpoint, receipts, snapshots, offsets, engine pins or oracles modified.

Recovered writer-write-phase-1 result: failed / Timeout waiting debugger arm / owned_process_exited_after_failure, with cleanup error Owned process did not exit after kill. Preserve the complete attempt. gdb.stderr.log identifies native-writer-debug.gdb:6 Error in sourced command file: Cannot change this setting while the inferior is running; subsequent writer-arm was undefined. gdb.stdout shows an attached stopped thread, but the session already had a live inferior when source attempted set non-stop off. Source aborted before Python command registration. No writer frontier was captured; no N7 claim follows.

Correction:
- Controller passes -iex "set non-stop off" before executable/-p attach. GDB early initialization selects all-stop before a live inferior exists.
- Sourced helper no longer changes the target mode. It explicitly rejects non-stop=true and prints N7_HELPER_READY all_stop=true only after successful writer-arm registration.
- Controller waits for that marker before dispatching writer-arm. Both stderr and stdout are checked for sourced-file errors, undefined commands, mode errors and N7_DEBUG_ERROR; these fail immediately rather than waiting 60 seconds for an impossible receipt. Arm wait also checks traceback errors.
- Existing all-stop receipt and per-thread stopped checks, target ownership, offsets, successful IO/rename checks, exact engine hash, kill boundaries and offline oracles remain intact.

The missing Python gdb package / _execute_unwinders warnings predate source and are preserved. This patch continues using built-in _gdb as previously designed; it does not claim those installation warnings harmless for every operation. A new unique runtime attempt must validate actual helper initialization, arming and all-stop capture. Do not reuse writer-write-phase-1. Root must ensure all previous PIDs are absent before any retry; this static review did not independently perform a process census.

Validation: PowerShell AST parse 0 errors / command exit 0. Static review verified non-stop mutation exists only in pre-attach -iex and helper retains a rejecting assertion. No GDB execution during concurrent dedicated trial.

SHA256:
- test-native-writer.ps1: 97232CB98B342A897CAC93E4E5B53B4BF710958D7C449ACEBE2187356CC492A3
- native-writer-debug.gdb: 2D3B23813C64B2EB0697B6CA0CF80AB0048948B4875BDF1EF3421844EE52FD9C
## Ownership launch recovery — subsequent failed attempts

writer-write-phase-2 armed successfully but failed constructing the async save process because Get-Command node returned multiple Application entries and .Source became an array. Root corrected selection to the first Application; that exact scalar selection remains in the helper.

writer-write-phase-3 failed GetFullPath(empty). Its last recorded phase is before_debugger_attach_B_in_memory, processes contains only engine PID17396, and gdb.stdout.log does not exist. Controller creates that file only after Process.Start(gdb) followed by Owned(gdb). This localizes the empty MainModule.FileName to initial GDB ownership capture, before async node launch. It does not prove why Windows had no module path at that instant. Cleanup correctly refused to kill a process without verified ownership; preserve the error and full failed run. A read-only elevated census during this review found no horizon-engine/gdb process.

Owned now waits at most five seconds (50ms polls) for an actual nonempty module path. It pins the Process object's PID and first readable start time, opens a fresh Process for that exact PID, verifies matching start ticks, and requires its actual full image path equal the expected executable. Exit, identity change or nonmatching image fails immediately. Missing/inaccessible initial image retries only within the bound, then fails with PID, expected executable and last inspection issue. No ownership is inferred from the launch path. Stop-Owned remains unchanged: it still requires a verified receipt and revalidates live PID/start/path immediately before kill. If ownership cannot be established, cleanup continues refusing an unverified kill.

Validation: PowerShell AST 0 errors. No games, builds, debugger or test process executed. Helper frozen SHA256 D9503BF602FC8025C9B03A42B94171EE241995A99A736D9DA52A8F09EC67EA66. GDB helper unchanged from previous correction. Root must review before a new unique attempt; this static correction does not certify capture success.
