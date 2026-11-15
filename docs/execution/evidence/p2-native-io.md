# N8 parcial — error real de escritura del propietario

Build p2-owner1 sin rebuild ni cambios Rust/SDK, engine AB56DEDB6756DF75E1BE706A7D0E717B18E8A6FD2C2263F72D939A22475D6DF3. Rama IO desde owner0264460. Seis ensayos nuevos sobre copias cerradas; las33 verificaciones anteriores no se repitieron.

| Run | Resultado |
|---|---|
| owner-io-create-new1 | Handle FileShare.None sobre .dat_new tras Ready+savebaseline. Save real devuelve JSON errorOS32/helperexit1. DAT y old conservan hashesA. El temporal vacío lo creó el harness; no es snapshot escrito por el juego. |
| owner-io-deny-rename1 | Handle Read, FileShare.ReadWrite sin Delete sobre ownerDAT. Juego escribe temporalB completo, rename primary→old fallaOS32, controlerror/exit1. DAT y old conservan hashesA. |
| owner-io-create-new-reopen1 | Copia del fallo con temporal vacío; apertura/cierre0. Oráculo confirma A. |
| owner-io-deny-rename-reopen1 | Copia del fallo con temporalB; apertura/cierre0. Oráculo confirma A/inventario2, nunca B/inventario3. |
| owner-io-missing-owner1 | DAT del owner real ausente, old completo válido. Guard rechaza antes login; hashes preservados. Requiere restauración completa offline. |
| owner-io-truncated-owner1 | DAT del owner real truncado, old completo válido. Guard rechaza antes login; hashes preservados. Sin fallback parcial. |

Los errores IO se observan en control directo save integrado. Harness mantiene handle hasta matar SU proceso y esperar terminación; no quit ni save de recuperación. killed_after_expected_io_error no es cierre limpio ni corte entre renombres. Timeout, transporte, error sin OS5/32/33 o cambio de hashes fallan. Sol revisó harness antes de ejecutar (DF5C6653).

Oráculo previo: dos paresA (reserva1000/revisión5/ledger Tx native-roundtrip-3 receipt4/inventario2), temporal renameB completo/inventario3 y mismo namespace, temporal create vacío. Después reopen: cuatro snapshots reserva1000/revisión6/ledger exacto/inventario2. Revisión aumenta por reconnect. Fuentes reproducibles y JSON adjuntos con hashes. Originales fallidos y fixtures no se reabrieron ni repararon.

Política N0 bloquea primary ausente/corrupto aunque old válido; acredita rechazo conservador, no recuperación automática. Se requiere restaurar copia COMPLETA de mundo cerrado, nunca namespace/ledger aislado. .fixture-original y source completo conservados.

Límites: no fsync/energía, cortesN6/N7 ni erroresFIFO. El pin descarta algunos Result de finish_player_autosaves; owner síncrono sí propaga vía Level.write_player→write_level_for_role→control_save. Treinta y nueve ensayos no completan P2/N8. Receipts en native-lifecycle-runs y native-open-runs; p2-io-checks.json consolida6 nuevos además de33 previos.