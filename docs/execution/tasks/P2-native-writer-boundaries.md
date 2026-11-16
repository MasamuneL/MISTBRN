# N5/N7 writer boundaries — bounded investigation, 2026-11-15

Base 8bbdf739, branch codex/alomancia-p2-gates. Read-only SDK inspection; no game/build/install/EULA change executed by this agent. Existing cuts1 engine only.

## Verified local findings

Runtime R = C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime.
SDK S = R/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.
C = S/crates/horizon-client/src.

- No cdb/windbg/gdb/lldb found on PATH. Bundled R/launcher/rust/1.99.1-x86_64-pc-windows-gnu/mingw64/bin/gdb.exe exists and reports GDB 18.1. objdump.exe is adjacent. No install required.
- GDB Python package is absent (startup warning references build-machine directory); built-in _gdb supplies Breakpoint, Command, parse_and_eval, execute, selected_inferior, selected_thread, parameter, string_to_argv. The helper imports _gdb directly. No SDK/toolchain files changed.
- cuts1 SHA256 8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70 has COFF non-debugging function symbols, no DWARF sections. Symbol write_dat_as maps preferred VA 0x142eada20; actual runtime address must be resolved after attach (ASLR).
- C/save.rs:3175–3209 serializes/compresses first, writes temporary, renames primary to old, then new to primary. Normal path uses fs::write, no sync_all.
- Actual cuts1 disassembly: normal fs::write call offset +1072; first std rename call +1444, return +1449; second call +1720, return +1725. std rename returns 0 for success (verified test/jump branches). Its real primary syscall wrapper is MoveFileExW at preferred 0x1452d6ba4, with SetFileInformationByHandle fallback on access-denied, not an assumed syscall.
- fs::write::inner opens/truncates the temporary, calls synchronous_write, loops on short writes, closes handle. synchronous_write calls actual NtWriteFile (preferred import thunk 0x1452d6e4c), and waits if pending. First cut accepts only immediate successful NtWriteFile with successful IO_STATUS_BLOCK and full reported byte count. Any pending/error/short result stops as DEBUG_ERROR; it is not a gate.
- Entry Path is rcx UTF8 pointer / rdx length, durable r9 low byte; inferred from disassembly and still requires runtime receipt validation. At normal fs::write call rcx/rdx identifies temporary path, r8/r9 buffer/length. NtWriteFile x64 stack arguments are IOSB at rsp+40, buffer +48, length +56, return PC at rsp. Return breakpoint is thread-qualified.
- All-stop remains on; final receipt requires every thread reports stopped. Root must observe debugger stop, kill only its owned Process, and preserve artifacts before offline oracle. Helper never starts, detaches, resumes automatically after a capture/error, or kills.

## Executable root orchestration

1. Existing reviewed root harness launches cuts1 hidden and owns Process PID/starttime/executable. One game only, new closed clone and immutable full backup. Reach Ready; save baseline A and preserve/hash both player files. Mutate live inventory to distinct B through ordinary control, but do not save B yet.
2. Start bundled gdb hidden with redirected stdin/stdout/stderr, args -nx -q, exact cuts1 engine filename, and -p OWNED_PID. Attach stops target. Send `source ABS_WORKTREE/tools/native-writer-debug.gdb`, then `writer-arm OWNED_PID "ABS_OWNER.dat" MODE "ABS_RUN/boundary.json"`. Modes: write-phase, after-primary-to-old, after-new-to-primary.
3. Wait for boundary.json.armed.json and verify PID/hash/path/mode. Send `continue`; dispatch existing control-save asynchronously in a separate helper process (save reply is expected to hang at a captured boundary).
4. Poll final receipt plus debugger output. Require N7_BOUNDARY_HELD, matching PID, all_stop=true/all threads stopped, correct exact owner path/mode and successful syscall/rename result. Root kills original Process object without resuming/detaching; wait exit, stop own blocked control helper, then quit debugger after target death. On timeout/error stop only own processes and classify failed attempt, never retry same run ID.
5. Copy/hash/parse primary/new/old offline; inspect coherent A/B and source selection. Reopen another full copy through existing guard. Missing primary deliberately rejects under strict policy even when old is valid; do not weaken guard or claim automatic backup adoption. Full backup restoration is a separate explicit operation.

No launch was executed during helper preparation. GDB source parsing and binary mapping are preliminary validation, not proof that runtime offsets/argument interpretation were hit.

## Boundary semantics and limitations

write-phase captures **after actual NtWriteFile success, before fs::write closes/returns**, while the new snapshot remains unpublished. It usually contains all compressed bytes (~1 KB), so it does not establish partial-byte writes or interruption inside kernel IO. Those variants remain unverified. Root explicitly approved this precise limited scope; independent reviewer must decide remaining N7 coverage.

The rename modes capture after successful filesystem rename, before write_dat_as returns and therefore before owner SaveToFile. First expects absent primary, full old A and new B; second expects primary B and old A. Offline oracle determines actual result; receipt alone does not certify contents. No fsync/power-loss claims.

## N5 factual limitation / dedicated preparation

C/game.rs:15428–15453 contains the sole player_autosaves.enqueue call, gated by options.dedicated.is_some() AND !execution_role.is_client(). Integrated owner metadata saves synchronously after finish_player_autosaves(true), then fires owner callback inline (15415–15425). There is no real pending owner FIFO snapshot in the current integrated singleplayer route.

C/lan/playerdata.rs:246–386 creates immutable snapshots, lazy named horizon-player-writer thread, FIFO channel, pending count, completed callbacks. C/game/native_inventories.rs:393 drains completions before inventory mutation; C/game.rs:15391–15400 dispatches SaveToFile and then consumes writer error. Thus N5 FIFO requires a real dedicated guest; inserting a synthetic SaveToFile or calling the trait manually would not prove it.

No eula.txt or server.properties found by runtime file inventory. Existing binaries/resources suffice in principle; minimal server route is same engine --resources R/p2-cuts1-engine/.local/horizon-client/26.3 --dedicated --nogui --port isolated_port --universe isolated_dir --world clone. Dedicated C/game/dedicated.rs:434–457 reads eula.txt and refuses without true; S/docs/server.md:17–22 confirms. Need explicit user EULA acceptance before starting dedicated service, then loopback-only server.properties and a real guest client/protocol connection, plus new N0 dedicated validation. Existing upstream test_dedicated_server_parity.py:117 writes eula=true automatically: do not run that script as-is. No EULA file was created or changed.

N5 pending FIFO callback and logout-before-nexttick remain open; Sol owns actual logging_out path investigation. This report does not close P2.

## Preparation checks actually run

Bundled GDB loaded the new script and registered writer-arm: N7_HELPER_LOAD_PASS. It resolved the pinned mangled symbol and created/deleted an internal breakpoint against the executable without starting a process: N7_SYMBOL_BREAKPOINT_PASS. selected_inferior().progspace.filename and non-stop=false were verified; _gdb.current_progspace does not exist, and the helper does not use it. Full engine SHA256 recomputed and matches PIN. The sole FIFO enqueue and dedicated guard also match current stage plus all three available patched game.rs copies. These are preparation checks only; no runtime boundary captured yet.
