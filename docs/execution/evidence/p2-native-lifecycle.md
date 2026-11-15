# Ciclo de vida nativo: resultados parciales

Build p2-owner1, SHA256 AB56DEDB6756DF75E1BE706A7D0E717B18E8A6FD2C2263F72D939A22475D6DF3. Se clonó el mundo cerrado `game-p2-owner-econ/saves/credit`: propietario AllomancyQA, reserva de hierro1000, ledger de una transacción e inventario de dos lingotes. Sin reconstruir ni modificar el SDK.

| Ensayo | Resultado real |
|---|---|
| owner-death-respawn1 | kill real, death_queued/death_applied, pantalla de muerte, respawn y vuelta a playing; quit/WorldCloseFinal/exit0. Reserva0; ledger exacto retenido; inventario vacío por muerte vanilla. |
| owner-save-pause2 | Control directo save-all, delta save_observed2, paused=true y resume/playing; quit/WorldCloseFinal/exit0. Reserva1000, ledger e inventario intactos. |
| owner-dimension1 | execute en Nether y teleport real; dimension_changed, estado y NBT Nether; quit/WorldCloseFinal/exit0. Mismo WorldId, reserva1000, ledger e inventario intactos. |
| owner-death-replay1 | Proceso nuevo desde copia de muerte/respawn; replay sin give y sin ítem. PASS/quit/exit0. Reserva permanece0 y ledger exacto; no se vuelve a acreditar ni consumir. |

El inspector independiente `p2-native-lifecycle-inspect.cjs` verifica ocho snapshots DAT/old: identidad, formato, reservas, reglas, selección, desbloqueos, accounting, ledger, inventario, salud y dimensión. Resultado JSON adjunto. Desbloqueos estaban false y fracciones0: no acredita conservación de valores distintos ni XP/conocimiento no implementados.

`owner-save-pause1` es un fallo conservado del harness: `command save-all flush` devolvió chat unknown aunque transporte ok. El SDK restringe ese comando vanilla a Dedicated (`commands/profiles.rs:22–39`); control directo `save-all` sí llama al servidor integrado (`control.rs:1213–1232`). Corrección revisada por Sol: directo save-all y fallo inmediato ante chat unknown. No se cambió Rust ni se repitió ninguna prueba previa válida.

El escenario death-logout no se ejecutó: quit detiene el servidor y el contrato SDK omite logging_out al detener (`native_hooks.rs:6980–6983`), emitiendo logged_out. No puede acreditar muerte→logging_out antes del siguiente tick. El gate temporal queda pendiente de ruta real de salida y barrera opt-in; acelerar comandos no basta. Tampoco se acreditan fsync, cortes, writer FIFO, conservación de fracciones no nulas ni reinicio estando muerto. P2/N9 siguen parciales.

Receipts y logs originales: runtime/native-lifecycle-runs/{owner-death-respawn1,owner-save-pause1,owner-save-pause2,owner-dimension1}; replay en runtime/native-credit-runs/owner-death-replay1. Los receipts guardan hashes de build/harness y comandos/respuestas redactados. El fallo usa killed_after_failure; los cuatro éxitos exigen cierre normal, jamás PASS por matar el proceso.