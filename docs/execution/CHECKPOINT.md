# Checkpoint vigente — MISTBRN
schema_version: 17
fecha: 2026-11-16 America/Mexico_City
worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN
rama: codex/alomancia-p2-gates
HEAD: 8bbdf7397fecf9ddc8f92d63d66c396bf198fa21
estado: P0/P1 integrated; P2 parcial; P3/V1 pendientes de cierre P2

## Operación y artefacto
- Reanudación: sin horizon-engine/cargo/rustc/gdb activos; launcher de usuario vivo, no tocar.
- dedicated-observe-2 terminó FAILED: Guest quit timeout; harness mató PID propios 34188/18800. Logs prueban login/logging_out/logged_out, no salida limpia ni PASS.
- Receipt: runtime/native-dedicated-runs/dedicated-observe-2/result.json (runtime original compartido).
- Build vigente p2-gates2 succeeded/exit0/native-test; engine SHA256 0484F38F4BCF1F24224E046D6D3690C185D148367C6DB1EF01DAC342F2B1A0E8.
- Inputs/core único: evidence/p2-gates2-build-check.json. Revalidar antes de nuevo ensayo.
- cuts1 retenido para N7, owner1/Native2/P1/P0/SDK/mundos/backups intactos.

## Bloques del plan actualizado
| Bloque | Estado y evidencia | Próxima acción / límite |
|---|---|---|
| Núcleo/formato | Tests y revisión previos conservados en historia; core sin cambios | Auditar cierre de bloque; no repetir por rutina |
| Transacción nativa N0–N4 | N0 owner revisado; N1 variantes nuevas; conflict2/capacity/full-ledger/stale-slot/stale-generation PASS | Replay-new/reentrant oracle PASS revisado; consolidar bloque y revisar cobertura |
| Guardado/recuperación N5–N8 | N6 reviewed en HEAD; IO OS32/fullrestore evidencias conservadas | N5 FIFO y N7 fronteras writer reales pendientes; dedicated fallo cierre |
| Ciclo de vida N9 | rich reconnect3/death-respawn4/dimension4/save-pause4 native+oracle PASS | dead-login4 fallo inventario; partir de muerte nativa; logout/generaciones pendientes |

## Trabajo actual exclusivo
- Principal: integración, manifiestos, Cargo.lock, builds/ensayos seriales, checkpoint/HANDOFF y commits.
- Colaboradores finalizados; sin escritores ni ensayos activos al guardar snapshot.
- N7 diagnóstico: unwinder Python ausente y buffering de mensajes son hipótesis, no causa demostrada.
- Replay/reentrant: oráculos offline PASS revisados por principal; evidence/p2-gate-credit-offline-review.json. Sin repetir juego.
- Diario vigente: tasks/P2-resume-2026-11-16.md; writer-write-phase-1 FAILED arm GDB, sin frontera; cleanup completo. Dedicated3 FAILED WaitFor tras guest0. Dedicated4 guest0/server0+oracle+revisión PASS infraestructura P2. Writer2 FAILED node array; Writer3 FAILED ownership path vacío; fix revisado. Writer4 FAILED timeout frontera tras arm/breakpoint real, sin receipt; cleanup ejecutado. Sin builds nuevos.

## Restricciones vigentes
- PLAN-ADJUSTMENTS.md prevalece; cuatro bloques reviewed antes de cerrar P2; después P3 mínimo/V1, C2 conserva alcance.
- Dedicated es infraestructura P2, no P9. EULA y servidor 127.0.0.1 ya autorizados por usuario.
- No reset/clean; preservar fallos, backups y tres diffs ajenos: p0-initial-probe.rs, p2-horizon-preflight.md y bundle/api/src/lib.rs.
- Snapshot WIP de traslado en rama gates: no implica P2 reviewed/integrated. README/plan conservan estado parcial. Consultar HEAD real tras commit.
- No repetir evidencia válida; cargo test --workspace excluye adaptador Horizon.
- Sin merge/main hasta pruebas y revisión. No limpieza adicional justificada.

## Historia y fuentes
- Historia íntegra: docs/execution/history/CHECKPOINT-full-20261116-155900.md
- SHA256 historia: 28FAFF757E381398C70F40A04EBE87AEC43AFA7C6D6EF5ED7B58098EECC2ECEB
- HANDOFF.md aporta contexto; este checkpoint es la entrada vigente.
- PLAN-ADJUSTMENTS.md; tracking-workflow-proposed.md; plan 2026-11-12-alomancia.md.
- Fallos preservados: gates1 packaging, conflict1 export, rich-dead-login3 timeout/4 oracle, dedicated-observe1 whitelist/2 quit timeout.
- Siguiente: diagnosticar stop interno GDB writer4 y validar solución antes de nuevo N7; N5 FIFO/N9 pendientes. No bloque P2 cerrado ni merge.

- Traslado: evidence/p2-portable-run-evidence.zip + manifiesto SHA256 conservan receipts/logs; no incluyen SDK, engines ni mundos. Ver DEVICE-HANDOFF.md antes de ensayos en otro equipo.
