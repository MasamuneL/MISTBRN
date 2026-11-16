# Dedicated resume review — 2026-11-16

Scope: only tools/test-native-dedicated.ps1 and this report. No engine, build, debugger, world edit, or receipt modification executed.

Recovered dedicated-observe-2 result.json is failed / pass=false / error="Guest quit timeout." Both owned PIDs were killed by its finally block. Server stderr records dedicated admitted, WorldOpenReady, login, ready, logging_out, save_observed(durable=false), logged_out. Server stdout confirms loopback join and departure. The sole guest quit reply is {"ok":true}. These establish observations, not successful shutdown, N5, P9, or a full offline oracle. Preserve the entire run.

SDK pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca inspected read-only:
- game/control.rs:1253 calls begin_closing_save(true), so quit requests application shutdown, not merely a return to menu.
- game/saving.rs:62–81 disconnects a remote session and sets Stage::Quit; poll_closing_save:132 returns true for that stage.
- game.rs:18725 requests event-loop exit when closing completes.
- net.rs:459–469 explicitly detaches the reader on disconnect/drop; a blocking reader join is not supported as the cause.
- game.rs:598–603 documents --headless as the automated client route with no renderer/window. game.rs:20443 selects run_headless; game/headless.rs:51–74 pumps idle until its exit flag and calls shut_down.

The exact runtime location of the offscreen shutdown stall was not captured. Do not claim a GPU defect or a proven root cause. The bounded mitigation replaces only the guest --offscreen flag with --headless, bypassing window/renderer teardown while retaining the real network client, control loop, server login and logout callbacks. This is suitable for the P2 infrastructure scenario; it supplies no visual/HUD evidence. The next new run must validate actual guest and server exit 0. The helper now explicitly rejects either nonzero exit (previously it recorded exit codes without asserting them), and saves each exit observation before continuing. It still leaves pass=false and requires review/offline oracle.

Validation: PowerShell AST parse returned 0 errors, exit 0. No native run performed. Helper SHA256 FA8C9FEB03FB0C82DC4B386E912E2806AE862046173083AAA54C8A3736DCA442.

Next root action: review this two-part diff and run a new unique observe attempt serially with the existing p2-gates2 build after current inputs/hash/process checks; never reuse dedicated-observe-2 or replace its failure. Inspect full closed files/authority/identity and actual exits before using the dedicated route for FIFO or lifecycle gates.