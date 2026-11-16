# P2 native gate fixture helper

Offline preparation only. Close Horizon first. Clone the **whole world** into a fresh `runtime/game-p2-gates/saves/NAME`; never use a live world or restore only the player/ledger. The helper does not detect a running process or prove that its input is a full clone. It checks the exact path shape, rejects redirected path components and player files, and requires level owner `3efd2c0e-f0de-3e85-b045-dec265b3f79c`, marker world ID `026e465a87434711a6951c07588dc0ee`, and v1 owner payload in both primary and old. Use one new clone per invocation.

```
node tools/make-native-gate-fixture.cjs --world <runtime/game-p2-gates/saves/NAME> --kind <kind>
```

- `both-owner-invalid`: truncate both gzip snapshots to ten bytes.
- `future-owner-primary-valid-old`: set primary JSON schema to 999; retain the complete old snapshot byte for byte.
- `player-seed`: both snapshots have CopperBronze selected; all eight metals unlocked with reserves 100..107, remainders 123..130 and alternating Burning/Flared; last_tick 19; rates 1, base capacity 10000, reduction 200, bonus 0; revision 1 and `fixture-drink-1` Iron 37 ledger entry.
- `dead-player`: same seed plus NBT Health float zero, for testing dead-login processing before ready.
- `capacity-player`: same seed with Iron 10000, its configured capacity.
- `ledger-full-player`: same seed with 4096 ordered `fixture-0000` through `fixture-4095` zero-Iron entries and matching unique revisions 1..4096; saved revision 4096.

The helper first creates `.fixture-original` exclusively and copies both original snapshots there before any modification. It refuses reuse, reports SHA-256 of each original and result as JSON, and verifies bytes written. NBT editing preserves untouched uncompressed tag bytes, including unrelated namespace and inventory; gzip bytes change. These fixtures intentionally change player state offline, not as recovery. World restoration must use a separately retained full-world copy.

Local self-check: all six kinds accepted copies of the real owner files and refused a second invocation because the backup existed. The source primary and old both had SHA-256 `9905021b7846588b19641b6d486971e30613efe40427f25362a858b8fe3beac8`. This was a temporary parser/write check, not a full world, native opening, Rust decode, recovery, game, or durability test. Run native checks on full clones and preserve their emitted receipts separately.

## State oracle

```
node tools/inspect-native-gate-state.cjs --baseline <owner.dat> --world <world> --mode <mode> --owner 3efd2c0e-f0de-3e85-b045-dec265b3f79c --world-id 026e465a87434711a6951c07588dc0ee --old-expect <baseline|reconnect|same>
```

The read-only oracle compares the full typed inventory, complete SavedPlayer, foreign namespace, state transport metadata and envelope fields. `unchanged` and `identity` require exact equality to baseline. `reconnect` requires exactly one revision advance, all burns Off and null last_tick; balances and fractions stay intact. `death-login` requires exactly two revisions (death, then reconnect), zero reserves and remainders, burns Off, null last_tick, with inventory preserved. `death-respawn` expects one revision for death relative to the supplied immediate pre-death baseline and empty inventory. All other compared fields remain exact. A present `.dat_old` requires an explicit expectation, and `.dat_new` fails the oracle. Pick the baseline and expectation according to observed hook ordering; the tool does not prove a hook occurred or a test reached its intended boundary.

`node tools/make-native-gate-fixture.cjs --self-test` and `node tools/inspect-native-gate-state.cjs --self-test` pass. The latter also compared the real credit-world primary and old against the same primary baseline in unchanged mode, read-only. Sol independently reviewed fixture values against core constraints and found no schema issue. No native lifecycle result is claimed by these checks.
