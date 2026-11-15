# P2 crédito nativo: roundtrip2 y lectura offline

2026-11-15. Ensayo integrado con engine p2-native2, perfil de jugador owner llamado Player y mundo dedicado CreditP2. El receipt native-credit-runs/roundtrip2/result.json registra PASS, give y quit con exit 0, proceso 11684 con exit 0 y terminación espontánea después de quit. stderr registra login/ready, credit_accepted revision=4 durable=false, credit_replay, driver PASS durable=false y WorldCloseFinal. El comando de fixture devolvió “Gave 3 [Iron Ingot] to Player”.

Lectura offline posterior con Node del baseline runtime/backups/p2-aligned-after-reopen y del mundo runtime/game-p2-credit2/saves/CreditP2. Sin abrir juego, modificar DAT ni ejecutar Cargo. El lector temporal target/p2-native-credit-inspect.cjs copia el parser NBT de target/p2-native-identity-inspect.cjs y cambia únicamente el recorrido/extracción; consume gzip, exige consumo completo y rechaza claves NBT duplicadas. No sustituye decode_save de Rust ni verifica independientemente todas las reglas del codec.

Comando desde worktree:
```powershell
node target/p2-native-credit-inspect.cjs
```
Exit 0; produjo cuatro objetos JSON, uno por principal y backup en cada mundo. Un segundo comando PowerShell canalizó ese lector a ConvertFrom-Json y contrastó identidad, schema, reserva, revisión, ledger, inventario y hashes de ambas copias; exit 0, salida: ORACLE PASS: 4 snapshots, identity, schema, reserves, ledger and inventory.

| Snapshot | SHA-256 cada .dat y .dat_old | Iron reserve_milli | revision | ledger | inventario |
|---|---|---:|---:|---|---|
| Baseline | 6928235DD3376B2ADA73567DCB3C3D4EE0992414D6261B5393C9EDFC7C071FA2 | 0 | 2 | vacío | vacío |
| Después de quit | 620F84A195F6ABB1FB09F419BAC83CA1183FE84052DDD144E8268E83E9069430 | 1000 | 4 | una entrada | slot 0, minecraft:iron_ingot count 2 |

En los cuatro archivos UUID NBT a01e3843e5213998958af459800e4d11 coincide con payload.saved_player.state.player_id y el WorldId coincide entre baseline y resultado. El transporte NeoForgeData["allomancy:state"].format_version es 1 y payload ByteArray contiene JSON schema_version 1; Iron es saved_player.state.metals[0].reserve_milli. La entrada observada es transaction native-roundtrip-3, payload metal Iron/amount_milli 1000, committed_revision 4. El ID generado por el driver codifica revisión 3 inmediatamente antes del crédito; por tanto crédito 3→4, aunque el baseline offline anterior tenía revisión 2. No se atribuye sin traza adicional el incremento 2→3. Las dos copias de cada pareja tienen el mismo hash en esta lectura.

SHA-256 de fuentes de evidencia:
- receipt: EDE56F30C420AB0AD424B30E850207FC7B2C1CCBFE27411C1AE689A6C14A8632
- stderr: 1E89EB356FDFCCF107BC934B733DCB3BB8B22CA5EDB950658CCBE961F3AFFCBD
- stdout: 68AE468E84E579F1D61C7E334E6BF7528130D072DEF436D85BC8BF5DAD9FAE56
- engine según receipt: 0A51CCBF7D1D695B0467E41842214B9F4E3ABCFF7CD418CE7AE97F391DB9E0BB
- parser original: 563885696733D974A4FEED6F535D39B1B894C2547AB050A36E523230D671A7B3
- lector temporal: DB843C937B4BC1774064E61C7762162A148658AAB873F8CE340E69856E8AF749

Esto acredita el snapshot y replay en memoria del perfil owner Player en esta corrida, con cierre observado; no acredita replay tras reinicio, fsync ni durabilidad ante pérdida de energía, ni otros jugadores o rutas concurrentes. N1 sigue parcial.
