# N5 source review — 2026-11-16

Agent /root/review_n5. Exclusive scope: crates/allomancy-horizon/src/{lib.rs,save.rs,native_test.rs} and this note. Root owns Cargo/build/runtime/SDK/Git/manifests/checkpoints. No subagents.

## BEFORE independent review

Read CONTINUE-PROMPT, last CHECKPOINT schema16 RELEVO REAL, current HANDOFF header, economy-completion and writer-boundaries tasks. Current source SHA256 matches the real handoff exactly:
- lib.rs C423BCA95377D94DA39E8D2262125E18CDCD204945036282B7C79DA80BB9E434
- save.rs B4F5973C1081E7496E18DFBD670B8A67EA1CE203D1C820151B19BC5C3F70A95F
- native_test.rs D1690C04B8CBF39609C722EEE9E6AE1889EC1FD7BBBC1A800A0EC875D65B8595

Intention: independently inspect N5 autosave-pending arming/token, real WorldSaved(wait=false), callback observation strictly around inventory helper, dedicated tick authority, and eight N3/N4 scenarios against pinned SDK/core. Correct only concrete minimal blockers if found; otherwise freeze unchanged. This note is published by temp file plus atomic same-directory rename. No Cargo/build/game/tests core or SDK edits authorized for this agent.

Read-only preflight: no Horizon/Cargo/rustc/gdb processes observed. Unprivileged git -C . rev-parse/status both failed Permission denied; root already investigating Git state, no Git mutation attempted. No local AGENTS.md found by root-file inventory or accessible parent candidates. Preserve all WIP and the three named unrelated changes.
## AFTER independent review / unchanged source freeze

No concrete source blocker found; no Rust source was edited. Read-only review covers the exact hashes above, which were recomputed unchanged after rustfmt --check. Pinned rustfmt 1.99.1 --edition 2024 --check on all three files exit0. This is parser/format validation plus manual SDK/type inspection, not Rust typecheck, build or native execution. No core tests repeated.

### N5 evidence contract reviewed

- lib.rs authorizes ServerTickEnd(world_running=true) only for IntegratedServer or DedicatedServer. No remote/local client tick added. Actual SDK NativeAuthority contains DedicatedServer.
- Real SDK-generated Hooks::world_saved takes &mut WorldSavedEvent (no lifetime), and the generated event has wait:bool. Current override matches this signature.
- autosave-pending selects observe as its ordinary tick driver: a server tick cannot prematurely run its transaction. Only actual WorldSaved(wait=false) plus valid armed token calls autosave_pending. Runtime/world-root marker checks still apply through run_native_test_driver/test_world.
- Token is restricted to ASCII alphanumeric, hyphen or underscore, length1..64. Initial arm path must be absent; active arm must be a regular file and not a symlink. Distinct native-autosave-arm-TOKEN path prevents using a cuts arm accidentally. Begin/end traces correlate PID, player, transaction and run_token.
- Backend prepares the credit, reserves Busy, and releases the Session mutex before Host calls. Observation begins immediately before the real Host::set_inventory_slot and ends immediately after it returns. Thread-local counter only counts real PlayerSaveToFile callbacks for the exact target UUID on the same thread. Observer does not call Host, acquire Session or commit economics.
- SDK native_set_inventory_slot calls native_edit_slots; its first operation is finish_player_autosaves(true), which drains completed callbacks before slot mutation. native_player_stacks and native_player_compound are read-only and do not drain this queue before the observation. Inline persistent-data setter does not drain either. SDK entered hook iterator permits nested dispatch; its guard is crash attribution, not a nested-hook rejection.
- Zero observed callbacks is explicitly rejected and the successful inventory mutation is rolled back before reporting FAIL. No synthetic SaveToFile, sleep or manual callback supplies the count.
- The source cannot itself prove that the writer was still pending at helper entry. Runtime controller must freeze/verify its own writer TID at the actual syscall, release it after inventory_helper_begin and observe callback_count>0 plus callback trace between begin/end. WorldSaved(false) alone and completed-but-undispatched callbacks do not prove pending IO. Immutable autosave A, live credit B, later real save and offline/reopen oracle remain required. No fsync/power-loss claim.

### Eight N3/N4 paths reviewed

| Scenario | Manual finding |
| --- | --- |
| conflict | Reuses existing transaction/payload, changes amount by checked+1; core rejects TransactionConflict before Busy/mutation; compares saved state/generation, returned inventory and NeoForgeData. |
| replay-new | Verifies existing receipt/no-change first, then distinct new transaction, one slot consumption, Iron1000, revision+1 and ledger append, then unchanged replay. Existing transaction collision fails explicitly. |
| capacity | Valid full Iron fixture causes CapacityExceeded(Iron) from prepare_credit before Busy or Host mutation; same unchanged checks. |
| full-ledger | New transaction against valid4096-entry fixture causes LedgerFull before reserve/Host mutation; no eviction. |
| stale-slot | Expected None against chosen occupied slot produces Slot; PreparedCredit remains uncommitted and outer status restored Ready. This checks stale expected input rather than an SDK race. |
| stale-generation | checked generation+1 rejected as StaleConnection before core preparation or Host mutation. |
| reentrant | Actual recursive shared credit entry outside Session mutex sees outer Busy and returns Busy; status/raw/inventory checks precede one outer commit plus unchanged replay. Induced adapter recursion remains explicitly identified. |
| disconnected-intent | Captures resolved intent only; PASS emitted solely from real LoggedOut callback after backend retirement, requiring UUID absent from server_players and Unavailable with no changes. Integrated owner quit cannot pass. |

Inventory assertion scope: Host::player_stacks returns all36 main inventory slots, not armour/offhand; compare this vector completely. The transaction itself edits one main slot. Dedicated runtime/offline oracle must inspect full persisted inventory when claiming full inventory preservation. Post-credit driver checks reserve/ledger/generation/revision/slot and replay; full rich-state/foreign-data preservation needs its offline oracle, not a wider claim from the PASS line alone.

Prerequisites remain real fixtures and actual callback order. Manual source review does not mark N3/N4/N5/N9 or P2 complete. Root may proceed to its one justified gates1 build once the other independent controller reviews are ready. Preserve cuts1 for its pinned debugger offsets; gates1 offsets require their own resolution.

### Final hashes

lib.rs C423BCA95377D94DA39E8D2262125E18CDCD204945036282B7C79DA80BB9E434
save.rs B4F5973C1081E7496E18DFBD670B8A67EA1CE203D1C820151B19BC5C3F70A95F
native_test.rs D1690C04B8CBF39609C722EEE9E6AE1889EC1FD7BBBC1A800A0EC875D65B8595

Only this review note was created/updated by this agent, through atomic same-directory replacement. All existing Rust WIP, SDK and unrelated diffs preserved. Git state remains for root to resolve; no git mutation, Cargo/build/game or dependency action executed.
## Additional read-only dead-login judgement requested by root

Root reports rich-dead-login-4 native exit0/dead-screen/quit passed, exact full SavedPlayer matched death+reconnect, but offline inventory assertion failed (baseline iron2 -> persisted empty). Preserve native receipt and oracle failure as separate facts; do not relabel this attempt fully PASS.

Pinned SDK game.rs7845 says saved death already dropped inventory/XP, then only selects Screen::Dead; that block does not clear inventory. finish_open_world12506 loads every level.inventory item regardless Health0. playerdata::apply197 similarly loads inventory without a health guard. Actual guest_die (lan_actions4292ff) requires health<=0 and !dead, emits native_player_died4355, then later drop_inventory4433; absent that event, this source alone does not prove this path caused the observed loss. Health0+inventory2 with normal keep_inventory=false is not a naturally completed death snapshot.

Independent recommendation: retain4 as nativePASS/oracleFAIL and use a new closed complete world snapshot from an actual completed death (normally empty inventory), then apply only rich namespace fixture offline to test dead-login recovery. Set expectations before launch and preserve source/fixture manifests. Full saved namespace equality is meaningful partial evidence from4, but should not suppress inventory failure or claim general loaded-dead inventory preservation. Do not alter SDK/game or mod solely to satisfy this incoherent artificial fixture. Actual rich death-respawn continues to need its own native run and oracle.

Atomic publication correction: first File.Replace(temp,dest,$null) failed with empty path conversion before replacing the destination; the BEFORE note remained intact. Retry uses .NET File.Move(temp,dest,true) for same-directory replacement, preserving the prepared complete text and removing its temp name. No source mutation occurred.