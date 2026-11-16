# Oráculos offline N3/N4 — 2026-11-16

Estado: verified, revisión del principal pendiente. Archivos exclusivos: tools/inspect-native-gate-credit.cjs y evidence/p2-gate-credit-offline-review.json. Ningún receipt, mundo, fuente Rust ni checkpoint modificado; ningún juego/build repetido.

Intención: resolver los dos receipts native_pass_oracle_pending mediante archivos cerrados, sin alterar el resultado histórico del harness. Ambos siguen conservando pass=false/oracle=pending en su receipt original; el resultado suplementario enlaza esos receipts por SHA256.

Comandos (cwd worktree activo):

- node --check tools/inspect-native-gate-credit.cjs — exit 0.
- node tools/inspect-native-gate-credit.cjs --self-test — exit 0, negativos de inventario, ledger, reserva y namespace ajeno rechazados.
- node tools/inspect-native-gate-credit.cjs C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime — exit 0; JSON en evidence/p2-gate-credit-offline-review.json.

Resultados: gates-replay-new-1 y gates-reentrant-1 PASS offline. Se cotejan manifests completos de baseline, backup y mundo final con receipt; native_pass/exit0/quit/cierre normal; trazas precisas de replay o Busy inducido; ausencia de fallos/quarantine. Principal y old deben ser exactamente el mismo resultado esperado (inventario, SavedPlayer completo, reglas, restos, selección, ledger, namespace ajeno, metadata y esquema). No se permite .dat_new. World identity conserva bytes de baseline.

Expectativa derivada del driver native_test.rs: baseline rev4/Iron1000/iron2/ledger native-roundtrip-3 receipt4; login incrementa a rev5 y apaga; replay o recursión inducida no modifica estado; crédito posterior consume exactamente un iron y acredita1000, rev6 y ledger native-roundtrip-5 receipt6; replay posterior no modifica. Resultado esperado Iron2000/iron1 y sólo dos entries. No inferencia desde el resultado para aceptar revisiones variables.

Parser NBT estricto reutilizado por copia del inspector gate-state existente (UTF8 fatal, límites, duplicate keys, tipos, trailing bytes, identidad, versiones); no se editó ese inspector compartido. El self-test verifica que comparar estados completos detecta cuatro clases de corrupción.

Limitaciones: la reentrada es recursión inducida del adaptador, declarada en el driver. El oráculo no demuestra fsync/cortes writer, FIFO dedicado, logout ni P2 global. Los unlocks y restos de estos casos son triviales; usar evidencia rich independiente para N9.

Incidencia de herramienta: primer comando exploratorio node -e tuvo SyntaxError por paréntesis extra, sin escritura ni ejecución nativa. La lectura exploratoria posterior y el inspector final concluyeron exit0. No se relajó la expectativa ni se repitieron juegos.

Siguiente: revisión independiente del principal del inspector/evidencia antes de acreditar ambos casos y consolidar el bloque Transacción nativa con los restantes gates.