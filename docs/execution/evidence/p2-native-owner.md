# P2 owner: apertura y reapertura, oráculo offline

2026-11-15. Oráculo PASS del mundo cerrado runtime/game-p2-owner/saves/OwnerP2 tras owner-new1 y owner-reopen1 con perfil AllomancyQA. Build p2-owner1 receipt succeeded/exit 0/native_test; ambos receipts de apertura passed, salida espontánea 0, engine SHA256 AB56DEDB6756DF75E1BE706A7D0E717B18E8A6FD2C2263F72D939A22475D6DF3.

El oráculo independiente deriva B mediante MD5 de UTF-8 OfflinePlayer:AllomancyQA y fija bits UUID versión 3 y variante RFC: B = 3efd2c0e-f0de-3e85-b045-dec265b3f79c, distinto de a01e3843-e521-3998-958a-f459800e4d11 (Player). La regla coincide con SDK horizon-protocol/src/crypto.rs:161-163, sin usar el cálculo del adaptador. Data.singleplayer_uuid de level.dat y level.dat_old coincide con B; solo existen B.dat y B.dat_old en players/data, y ambos tienen UUID NBT y payload.saved_player.state.player_id iguales a B. En ambos, world_id = 026e465a87434711a6951c07588dc0ee, ocho reservas en cero, ledger vacío y revision 2; transporte y schema JSON son versión 1.

| Archivo | SHA-256 |
|---|---|
| level.dat | 6F5C6B508741178D156C01CA03FE78DED51E9E79DA1A5F569F7EF5A6A60FADE4 |
| level.dat_old | 0CAA4C2695BF628F7B2F460B4060DE5C139B56B78176338C52C774B9EA15FDDA |
| B.dat y B.dat_old (cada uno) | 99060DD10B4E8B21975D5DF57236D6197F4314C4B24A78EC6F39E9DE8BCF8044 |

Desde worktree se ejecutó node target/p2-native-owner-inspect.cjs; exit 0 y salida oracle PASS con cuatro objetos y los hashes anteriores. El lector temporal SHA256 DD38B09216ACD0B5EFB8DC4C7FCDCD510E23A42328AAEA61F69D97D0BFA7AD53 reutiliza parser NBT de target/p2-native-identity-inspect.cjs SHA256 563885696733D974A4FEED6F535D39B1B894C2547AB050A36E523230D671A7B3. Lee gzip, verifica bounds, duplicados y consumo completo; JSON.parse y las comprobaciones indicadas no sustituyen validación Rust integral del codec.

owner-new1/stderr registra bootstrap accepted in memory; durable=false, WorldOpenReady, backend ready y WorldCloseFinal. owner-reopen1 registra Primary, WorldOpenReady, backend ready y WorldCloseFinal sin otra traza bootstrap; junto con el snapshot muestra reapertura de esa identidad, sin inferir normalización o migración de mundos existentes. Hashes:
- owner-new1/result.json A20BDA99FDA549AF0559CFB38BE4C4FE5B5CE67AE193D6F9EA93E25018DA48E8
- owner-new1/stderr.log 30B4FAE434CA06757EF238D746B2AC2E78032E71CF4C70225D13E5BA3F0AF4A6
- owner-reopen1/result.json 8B7BFDD5A31CB9EAB19F6D80C4CE376C7F3EF0C3C9309D8D44D34B05B2C69E39
- owner-reopen1/stderr.log BC13AC4699080D8CBA7A8BEF860B19B26294CAED8309E63DFB3549D8ECE82AE2
- p2-owner1/receipt.json 5DCD54F08068EFF53AD51DE12937713CEF1806BA63BEC2190582E0820FC188E0

La evidencia se limita a este perfil y mundo dedicado; no demuestra fsync, durabilidad ante pérdida de energía ni migración de otros mundos. No se ejecutaron juego o Cargo durante el oráculo ni se modificaron datos de mundo.
