# P2 native economy/lifecycle completion

Date: 2026-11-15. Agent /root/p2_economy_lifecycle. Base 8bbdf739.

## Before implementation

Exclusive source writes save.rs/native_test.rs; root approved only the tiny conditional logged_out callback plus Host parameter rename in lib.rs. No Cargo/build/game/SDK edits/commits. Reuse existing opt-in runtime/world marker restrictions, core preparation and inline Host transaction path. Root owns native execution, manifests and integration.

Accepted scenarios: conflict, replay-new, capacity, full-ledger, stale-slot, stale-generation, reentrant, disconnected-intent. Offline capacity/full-ledger/rich-N9 fixture production is assigned to Terra. The reentrant scenario induces an actual recursive call to credit_for_native_test while the outer entry is Busy and after releasing session mutex. This is adapter-induced recursion, not spontaneous SDK reentry. Snapshot checks compare full SavedPlayer+generation, whole inventory and NeoForgeData.

N9 route investigation: original SDK native_hooks.rs6982 documents logging_out absent on server stop; native_players_leaving11492 emits logged_out while hub guests remain listed. Thus quit cannot prove disconnected intent nor death before next tick. Real HostEvent::Left in game/lan.rs2180 temporarily restores guest for logging_out then removes before logged_out2252. Genuine departure route still under investigation.

## Verification contract

Focused static RED before source edits: assert scenario wiring, recursive shared entry and conditional actual logged_out callback exist. GREEN repeats these structural checks and rustfmt parser/format check only; no claim native pass or behavioral coverage until root SDK build and real game evidence. Native scenario assertions remain the meaningful runtime checks.

## Source freeze / handoff

Added eight opt-in scenario names to both config allowlist and driver dispatch. Existing runtime subtree/marker/world ID checks remain intact. New mutations use the same prepare/apply/inline/commit path; only test-only recursion injection added in save.rs. No gameplay changes, grants, direct DAT writes, SDK edits or economic sidecar. The disconnected test keeps only a captured server-resolved intent in process-local test context; that context is not persistent economy.

| Scenario | Prerequisite | Required native result |
| --- | --- | --- |
| conflict | Existing ledger entry; inventory can be empty | Same T, amount+1 gives Codec(TransactionConflict); SavedPlayer/generation, whole inventory and NeoForgeData exactly unchanged. |
| replay-new | Existing ledger plus one live item | Existing T receipt unchanged and all snapshots unchanged; a distinct native-roundtrip-REV then consumes exactly one, credits Iron1000, advances one revision and appends one ledger entry; replay of new T unchanged. |
| capacity | Iron exactly at rules capacity plus live item | Codec(InvalidReserves(CapacityExceeded(Iron))), all snapshots unchanged. |
| full-ledger | 4096 valid sorted ledger entries plus live item | Codec(LedgerFull), all snapshots unchanged. |
| stale-slot | Occupied live slot | Expected None against occupied slot gives Slot, all snapshots unchanged. This is a stale expected-slot input, not an induced SDK race. |
| stale-generation | Live item | Captured generation+1 gives StaleConnection, all snapshots unchanged. |
| reentrant | Live item and available capacity/ledger | Induced recursive actual credit_for_native_test returns Busy while outer Busy; Host snapshots unchanged by inner; outer credits once and replay unchanged. |
| disconnected-intent | Live item to capture intent, then actual departure | Armed trace only at capture, PASS only from actual LoggedOut callback with UUID absent from server_players, backend entry retired, shared credit returns Unavailable and Host snapshots unchanged. Quit cannot pass. |

All completed driver assertions emit `ALLOMANCY_NATIVE_TEST scenario=NAME result=PASS durable=false`; failures emit result=FAIL reason=CODE. Reject scenarios additionally emit `event=rejected scenario=NAME reason=DEBUG_ERROR unchanged=true`. replay-new emits `event=replay_unchanged scenario=replay-new unchanged=true` before credit. Reentry emits `event=reentrant_rejected inner=Busy outer=Busy induced_adapter_recursion=true unchanged=true`. Disconnected capture emits only `event=disconnect_armed player=HEX durable=false`, never premature PASS; callback PASS additionally has `departed=true reason=Unavailable unchanged=true`.

Existing inventory permits scenarios to act immediately once ready. Root harness must classify initial live-item fixtures accordingly rather than demand give precedes every PASS. conflict needs no give. replay-new/capacity/full-ledger can use complete cloned owner inventory if a live item exists; fresh empty fixtures need controller give.

### Verification observed

- Focused static RED before edits exit1: scenarios/callback absent, expected failure.
- Same focused wiring check after edits exit0/GREEN, extended to recursive shared entry and real lib callback. Static wiring only; not runtime behavior.
- Pinned rustfmt.exe --edition 2024 on three source files exit0; subsequent --check exit0. This parses/formats source; does not type-check SDK.
- Scoped git diff --check exit0. No Cargo, native build or engine launched. Root must compile once and run real native assertions plus offline oracles and independent review before crediting gates.
- First attempted edit used unavailable python and exited1 before any source writes; retried with existing Node. No dependency installed.

Frozen SHA256 (three source files):

- save.rs: 4FF83C2F41EE34E930688E68BE766035085D20F20B5CE33FC80EA2E8747F532E
- native_test.rs: A8CBADCEA060AFA0E394BB55AE5C7CCF576A849A19019586727BECBD244AA20F
- lib.rs: BEBB0CC93BC0719273A0C151E1D3C79E6D0B938E2C395BE8D0942BCB78BF06BA

### Lifecycle ruling and fixture review

Original SDK pin fdc87ae game/commands/extra.rs729 /kick explicitly excludes singleplayer_owner, rejects an unpublished integrated server, and reports owner.failed when count0. native_hooks.rs6982-3 explicitly says logging_out is not emitted on server stop, while native_players_leaving11492 emits logged_out with placed guests still listed. Real HostEvent::Left game/lan.rs2180 restores the guest temporarily, invokes logging_out2191 while Host helpers can access it, removes it, saves guest, and then invokes logged_out2252. Thus real guest connection departure is a feasible SDK route; integrated owner quit and /kick cannot supply it. No real logout-before-next-tick evidence produced, and disconnected-intent remains pending until an actual departure can be driven. Dedicated EULA remains root/user-controlled; not accepted by this agent.

N9 rich/dead fixtures are offline-only Terra helper, reviewed read-only against core: eight positive reserves100..107, remainders123..130 below1000, all unlocked, CopperBronze selected, valid nondefault reduction200/capacity10000, unique valid ledger; capacity10000 and 4096 sorted ledger fixture valid. Login on_reconnect turns burns Off and preserves fraction/reserves/rules/ledger; dead login on_death clears all reserves/remainders and preserves unlocks/rules/ledger. Root owns native tests; these source/schema observations are not N9 native evidence.

## Before dedicated authority extension

Root reports explicit user acceptance of Minecraft EULA and authorization for a temporary loopback test server. This agent still does not build/start a server. Original SDK mod_api.rs697-703 exposes NativeAuthority::{LocalAuthoritative,IntegratedServer,DedicatedServer,RemoteClient}; exact DedicatedServer confirmed before edit. Existing tick guard admits only IntegratedServer, so dedicated native driver and queued-death drain never run. Scoped prechange check exits1/RED with expected message authoritative dedicated ticks excluded. Extend only the existing ServerTickEnd world_running=true guard to IntegratedServer|DedicatedServer; no client/local authority admitted. N5 test-only arming/callback proof remains in coordination before final source freeze.
