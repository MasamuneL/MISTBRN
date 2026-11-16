# Gate credit helper — 2026-11-16

Before: helper and ficha absent. HEAD supplied by root 8bbdf739 on codex/alomancia-p2-gates. No Horizon/cargo/rustc/gdb process observed; git status from sandbox could not read worktree metadata. Root owns checkpoints/builds/serial execution. No game or build run by this agent.

## After preparation

`tools/test-native-gate-credit.ps1` SHA256 880D3EC0B6892F2C5326882C266AE66A6760B2226282DEBCD4F7A43B4F67B537; PowerShell AST parse: zero errors. No game, fixture, build or oracle has been executed by this agent. The helper covers seven integrated-owner scenarios: conflict, replay-new, capacity, full-ledger, stale-slot, stale-generation and reentrant. Disconnected intent requires the separate real dedicated guest path. It never issues give: live owner inventory and existing ledger must already satisfy the driver; an unsuitable fixture times out and does not pass. Conflict needs the ledger but can use an empty inventory. The source is the closed complete `runtime/game-p2-owner-econ/saves/credit`; each fresh RunId creates `runtime/game-p2-gates/saves/RunId`, a full retained backup and a full fixture baseline. Source, backup, initial clone and post-fixture baseline are manifested with path/size/SHA256 and compared. Only capacity and full-ledger invoke `capacity-player` and `ledger-full-player`, respectively. It records the post-close world and rechecks the untouched baseline.

The build receipt must be succeeded, exit 0 and native-test, with all recorded inputs present and byte-identical. `-AllowLauncherDrift` explicitly permits drift only for the exact launcher path in the receipt and records the exception; inspect whether that exception is justified for the new build. Engine, receipt, input, harness, control, fixture and oracle hashes are recorded. Only own process object is killed on timeout/failure. The existing `horizon-control.cjs` supplies a ten-second socket timeout and validates endpoint/auth; this harness additionally requires `quit` reply `ok=true`, matching SDK `game/control.rs:376,1253`.

Driver PASS, ready, clean exit and WorldCloseFinal are mandatory; any native FAIL, fault, quarantine or rollback-unverified fails. The relevant rejection, replay or reentry trace is required. For rejections an explicitly supplied `-OldExpect baseline|reconnect|same` invokes `inspect-native-gate-state.cjs` in reconnect mode against the complete post-fixture baseline. It checks the full typed inventory, SavedPlayer, foreign namespace, state metadata, envelope and specified old-file expectation; a passing offline oracle is required before the receipt `pass=true`. Root must choose the old-file expectation using observed save ordering, not guess until one passes. Without `-OldExpect`, rejection remains `native_pass_oracle_pending`. For replay-new and reentrant, the existing oracle does not describe the credit: they always remain `native_pass_oracle_pending`, `pass=false`; root must independently check reconnect plus exact revision, ledger entry/payload, inventory decrement, foreign namespace, and replay. The native reentry is intentionally induced, not spontaneous SDK reentrancy.

Suggested commands after root completes the build and freezes/reviews inputs, each run serialized with a fresh RunId:

```powershell
$h = 'C:\Users\alanj.VIKINGO\.codex\worktrees\mistbrn-p1\MISTBRN\tools\test-native-gate-credit.ps1'
$r = 'C:\Users\alanj.VIKINGO\OneDrive\Documents\ChatGPT\MISTBRN\experiments\horizon-probe\runtime'
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-conflict-1 -Scenario conflict
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-replay-new-1 -Scenario replay-new
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-capacity-1 -Scenario capacity
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-full-ledger-1 -Scenario full-ledger
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-stale-slot-1 -Scenario stale-slot
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-stale-generation-1 -Scenario stale-generation
& $h -RuntimeRoot $r -BuildName p2-gates1 -RunId gate-reentrant-1 -Scenario reentrant
```

These commands deliberately omit `-OldExpect`; the offline rejection check can instead be run afterward against the retained baseline and closed world using the documented oracle and an evidence-supported old expectation. The fixture itself requires the known owner and world identity, valid primary and old files and real paths. Clone completion and a process-name preflight cannot alone prove no external writer exists: root must ensure the source is closed and all builds/games are serial. Full-world byte manifests establish copy integrity but are not a semantic player oracle. AST validation does not prove runtime behavior, argument quoting, world initialization, control response or that a new build includes the intended sources; inspect the new receipt and logs. No evidence is claimed from preparation.