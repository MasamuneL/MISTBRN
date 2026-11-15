# P2 N8 — IO Windows real en owner integrado

Estado: harness implementado, ejecución pendiente; no Rust/SDK/build nuevo. Rama codex/alomancia-p2-io desde owner0264460. Root único escritor de lifecycle y evidence; Sol revisión sólo lectura.

Antes: ampliar harness lifecycle con io-create-new/io-deny-rename. Mismo engine p2-owner1, modo observe. Copias cerradas independientes desde game-p2-owner-econ/saves/credit (B1000/ledger1/item2). Ready, save real baseline, hashes ownerDAT/old, lock Windows después de apertura, give1 para distinguir intentoB. Control save exige exit1 y errorOS5/32/33, nunca errorgenérico ni ok sin log. Matar sólo PIDpropio antes de liberar lock, sin quit ni reintento. Principal/old deben conservar hashesA; _new puede quedar vacío(create-denied) o contenerB(rename-denied). Oráculo offline debe verificarlo; reapertura sobre otra copia, no tocar originales del fallo.

Fuente SDK: save.rs3175–3206 write_dat_as usa fs::write(.dat_new), rename(primary,old), rename(new,primary); normal write_dat no usa sync_all. Level.write_internal llama write_player? antes level.dat. Control directo save integrado propaga ese Err; no implica propagación de errores asyncFIFO (Result descartado en game.rs15384). Error esperado es sharing/access real, no failpoint del adaptador. Cortar después del error no es N7 entre renombres ni fsync.

Recuperación: guard nativo actual rechaza principal ausente o corrupto aunque old válido, exige restauración completa offline. Preparar sólo copias cerradas del ownerreal para probar rechazo sin sobrescribir, conservar originales. No activar fallback parcial ni copiar sólo JSON/ledger sobre inventario actual. Gates N6/N7 y variantesN8 siguen independientes.

## Resultado root y revisión
Seisensayos nuevos PASS y oráculosoffline PASS; ver evidence/p2-native-io.md/json y p2-io-checks.json. ReviewSol sólolecturaaprobó anteslaunch y despuésreceipts/hash reales; root fortaleciópayloadledgerIron1000 y transport/schema1/reopen segúnfeedback, sólooffline reejecutadoPASS. Harness DF5C6653 sinRustnewbuild. NoN8global/N7/fsync/FIFO. CommitlocalIO autorizado, no merge main. Todosmundos fallidos y .fixture-original intactos; sinengineactivo.
