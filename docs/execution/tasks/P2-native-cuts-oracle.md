# Offline native cut oracle

Run only on closed, isolated copies. Capture the owner's primary DAT immediately after the pre-arm control save completes, and optionally capture its old DAT as well. Keep those immutable along with the controller receipt, exact paused trace and all attempts. This tool never opens the game or changes a world.

```powershell
node tools/inspect-native-cuts.cjs --baseline <baseline-owner.dat> --baseline-old <baseline-owner.dat_old> --world <closed-world> --owner <uuid> --world-id <32-lowercase-hex> --mode cut --expect A --transaction native-cut-<token> --slot <slot>
```

Omit `--baseline-old` if absent. Owner UUID is lowercase hyphenated. The token and slot must be the exact values used by the controller. Select the required state with --expect A or --expect B; the current no-save cut profile requires A for all four frontiers. For the reopened **second copy**, use the same expected state, baseline, transaction and slot with `--mode reopen`. Preserve JSON stdout and the exit code as evidence. Exit 0 and `oracle: PASS` mean every present owner primary, old and new DAT matched an allowed complete state; an absent primary, malformed NBT, duplicate NBT compound name, trailing bytes, incompatible identity/schema, storage fault or mismatch fails. Missing optional old/new files are explicitly recorded. SHA-256 hashes identify the baseline and every present inspected target. An optional baseline-old must equal A or its exact pre-reconnect predecessor and is accepted only as old, not as the cut primary or new file.

A is the full baseline inventory, saved player, envelope metadata and all other NeoForge namespaces. B is A with one iron ingot removed from the exact slot, 1000 milli-Iron credited, revision incremented, and the exact transaction and receipt inserted in sorted ledger order. All remaining fields must match. Each file must match the selected whole state, so a mixture of A inventory and B economic state fails. At a cut, primary and new must match the selected state; old may also match the validated baseline-old. On reopen, primary must match the selected state after the explicit reconnect transform (revision +1, last_tick null, every burn Off); old/new may match the selected state, its reconnect transform, or the validated baseline-old. Thus an A cut cannot pass as B on reopen. It does not prove which file the engine loaded, whether a cut was reached, or that a transaction was acknowledged durably. The separate controller and reopen receipts establish those facts.

```powershell
node tools/inspect-native-cuts.cjs --self-test
```

The self-check exercises complete A/B matching and rejects mixed inventory, changed foreign data, and an unexpected A-to-B transition. It is not a substitute for checking controller traces or running the oracle on actual closed copies.
