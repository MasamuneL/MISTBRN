# N5 FIFO controller — 2026-11-16

Agent /root/review_n5. Root authorized exclusive new tools/native-fifo-debug.gdb, tools/test-native-fifo.ps1 and this task. Rust frozen; no Cargo/build/game/SDK/Git mutation or subagents. Build is root-owned p2-gates2 (gates1 failed precompile launcher validation); no new build assumed complete.

## BEFORE implementation

Implement bounded auxiliary against root-owned live dedicated server+guest, exact PID/start/path/hash, prepared sync baseline A and live ingot. Resolve this engine write_dat_as symbol rather than old cuts1 offsets. Filter exact guest DAT entry then actual NtWriteFile import call on same thread; save all-stop receipt with thread PTID and handle/gzip. Outside debugger verify Windows TID owner and description horizon-player-writer, duplicate/query file handle path equals guest.dat_new. SuspendThread only that TID from suspendcount0; detach debugger to run main; wait exact tokenized inventory_helper_begin; ResumeThread exactly once; require real same-player callback between begin/end/count>0 and scenarioPASS. Never kill externally owned server/client. Failures release only own suspension and detach after verified debugger stop; uncertain detach leaves explicit attention required.

Trigger: real console tick rate1 shortens countdown to300 (SDKmisc84/game15363/autosave_interval=max100,rate*300); console tick rate1000 leaves short countdown unchanged and runs it quickly. game8037-8043 natural autosave calls save_game_with(false); dedicated metadata enqueues guest snapshot, finish(false) does not wait. save-all is waittrue and cannot supply N5. Natural event and pending writer evidence determine success, not elapsed sleep. Poll delays are deadline scheduling only.

WorldSaved(false) may be reached while writer is already all-stop; armfile must exist before continue/trigger. Main can have printed helperbegin before the debugger catches writer; captured NtWrite entry still proves that the queued target write has not run to completion. Require no helperend/callback before own suspension release.

This proves dedicated guest FIFO barrier, not integrated owner pending (that route is synchronous), in-kernel partial write, fsync or P2 completion. Root retains full baseline/closed-copy/offline/reopen oracle responsibility.
## AFTER implementation / source freeze

Created only the two authorized tools plus this note. No Rust/SDK changes, no game launched, no controller executed against a server, no suspend/resume exercised on any target. Runtime N5 remains unverified.

- test-native-fifo.ps1 AST parse PASS after final edit.
- Bundled GDB loaded native-fifo-debug.gdb and registered fifo-arm without inferior; command help PASS. Known upstream missing Python gdb module warning remains, helper imports builtin _gdb and works.
- Exact controller MI functions extracted and exercised against standalone GDB with no inferior: source command and response pump PASS. Initial smoke exposed GDB source treating quoted filename as literal; corrected source command to unquoted whole filename (forward slashes). fifo-arm keeps argv quoting through gdb.string_to_argv.
- PInvoke declarations compiled and queried ONLY this validation PowerShell process/thread, with no suspend: thread owner and GetThreadDescription PASS. First probe rejected a valid nonzero successful HRESULT0x10000000; corrected success condition to HRESULT>=0. No target was affected by that probe.
- ObjDump symbol selector returns exactly one write_dat_as on cuts1 and new gates2. New gates2 engine SHA2560484F38F4BCF1F24224E046D6D3690C185D148367C6DB1EF01DAC342F2B1A0E8; static GDB resolution write_dat_as0x142eb11d0 and NtWriteFile0x1452da60c PASS. These are preferred static VAs only; controller resolves relocated addresses through its attached engine and uses NO cuts1 instruction offsets.

Frozen tools SHA256:
- native-fifo-debug.gdb 0ACB9555F97E14089E59148E8829BD9E6AC47D1264091C290E552C7F3ACC8E78
- test-native-fifo.ps1 D602EEDE04164E12B21966F85A25BC9E5C8F028ED8BF31F4925972F8A0AA1E15

### Caller contract / usage

Root launches server with native-test scenario autosave-pending and same RunToken (arm absent at process config), starts real loopback guest, observes Ready, supplies guest main-slot iron and performs real synchronous baseline save. Ensure principal AND dat_old exist and copy/hash full baseline A in parent harness before invoking. No other save/controller runs concurrently. Root retains launch Process object, PID/start/executable/commandline and loopback-only listener proof; helper checks these again, including dedicated/nogui flags and no nonloopback listeners. Root owns subsequent save B, clean guest departure/server stop, closed full manifests, offline oracle and separate reopen clone.

Example caller expression (variables owned by root harness):

```powershell
& (Join-Path $PSScriptRoot 'test-native-fifo.ps1') -RuntimeRoot $runtime -BuildName $BuildName -ServerPid $server.Id -ServerStartUtc $server.StartTime.ToUniversalTime() -ServerControl $se -ServerStderr $serr -WorldRoot $world -GuestUuid $guestUuid32 -RunToken $RunToken -RunDirectory (Join-Path $run 'fifo')
```

RunDirectory must be a NEW child directory (parent run may already exist); all target paths must be inside runtime. GuestUuid is32hex without hyphens. No EULA decisions are made by this auxiliary. User acceptance/loopback authorization is already handled by root harness.

result.json records initial exact engine/helper/controller hashes, principal/old baseline hashes, real command replies, GDB-held receipt, verified native writer TID/name/IO target, SuspendThread previous0, ResumeThread previous1, token/player/transaction-specific begin/callback/end log offsets and callback count. Runtime pass requires NtWriteFile call-entry all-stop on exact guest.dat_new while real writer not completed, verified writer suspended while main runs, no callback/end prior to own resume, and callback strictly inside begin/end followed by autosave-pending PASS. Poll intervals never supply proof. Root must review final stderr and closed files; pass=true is explicitly runtime scope with offline_oracle=pending.

Cleanup never kills root's server/guest. It balances only its own thread suspension and detaches only after stopped inferior is observed. A debugger that cannot be stopped/detached is retained and result marks attention_required_debugger_attached with debugger PID; root must resolve before another engine run. Do not reuse a failed RunDirectory or pretend a timeout is success. Arm remains in the disposable prepared clone for audit.

### Bounded remaining uncertainty

Live GDB attach, Windows PTID shape/description, DuplicateHandle path query, debugger SuspendThread interaction and real autosave timing have not been exercised. Each has an explicit rejecting check; any failure is a preserved failed attempt requiring diagnosis. This helper stops BEFORE NtWriteFile executes, proving an outstanding immutable snapshot on the real player writer, rather than a partly completed kernel write. The distinct N7 script has different boundaries. N5 integrated-owner async-pending is unavailable by SDK design; owner synchronous case and dedicated-guest FIFO evidence must be described separately.

### N9 source ruling (no Rust additions)

For real guest death-before-logout, tick freeze is a plausible existing control route: native_lifecycle emits frozen ticks with world_running=false and backend tick drain guard skips them; tick_lan continues processing connections/guests. kill sets live health0, guest tick can emit native_player_died/death_queued, then genuine guest quit invokes logging_out→drain. Runtime must demonstrate exact death_queued before logging_out before death_applied, no intervening running backend tick, and a real departure; source analysis does not guarantee the ordering. No new synthetic hook or SDK modification introduced.