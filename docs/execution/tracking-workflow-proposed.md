# Propuesta de seguimiento de ejecución

Propuesta documental; no modifica el checkpoint activo ni los criterios técnicos existentes.

## Fuentes y snapshot

Lectura: 2026-11-16 10:22 America/Mexico_City. Fuentes: `CHECKPOINT.md` (schema 16, cabecera y anexos finales) y `HANDOFF.md` del worktree activo `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN/docs/execution`.
Este es un snapshot de archivos que siguen cambiando por ejecución concurrente. No acredita estado actual de procesos, HEAD, artefactos ni resultados posteriores a la lectura. Los anexos recientes del checkpoint superan datos anteriores del handoff; no consolidar resultados por conteo.

| Bloque | Estado leído | Evidencia referida por la fuente | Bloqueo o límite leído | Próxima acción indicada |
| --- | --- | --- | --- | --- |
| P2 global | Parcial | CHECKPOINT schema 16 y anexos | No equivale a P2 completa ni autoriza merge | Continuar pendientes existentes |
| Build gates | p2-gates2 succeeded, exit 0 | `evidence/p2-gates2-build-check.json`, receipt y log propios | gates1 fallido conservado; compilación no cierra gates | Usar artefacto e inputs verificados para los ensayos previstos |
| N3/N4 | conflict2, capacity, full-ledger, stale-slot y stale-generation registrados passed | Receipts y baselines citados en anexos | conflict1 fallido conservado | Consolidar evidencia sin repetir casos válidos sin causa |
| Replay/reentrada | `native_pass_oracle_pending` en ambos intentos | gates-replay-new-1 y gates-reentrant-1 | `native_pass=True`, `pass=False`; falta oráculo | Completar inspección ya prevista antes de acreditar resultado |
| N9 rich | death-respawn, dimension y save-pause registrados PASS; dead-login-4 fallido | Anexos rich4 y sus oráculos | Inventario inesperado en dead-login-4; no aprobado | Conservar fallo; siguiente ensayo parte de snapshot de muerte nativa previsto |
| Dedicated | Última entrada: ANTES dedicated-observe1 | Entrada de operación en CHECKPOINT | Sin resultado observado en esta lectura; helper deja revisión pendiente | Terminar operación actual y registrar resultado real |
| N7 | Pendiente según fuentes leídas | Contratos/fronteras writer referidos | No sustituir frontera real por fixture o ACK | Seguir trabajo pendiente existente, sin anticipar cumplimiento |

## Formato propuesto para el checkpoint vigente

Mantener `CHECKPOINT.md` en un máximo de 80 líneas físicas, reservadas al estado que permite reanudar. El historial cronológico completo permanece separado y enlazado; no se elimina evidencia para lograr el límite.

Contenido propuesto, en este orden:

1. Fecha con zona horaria, versión del formato y ruta del worktree activo.
2. Rama y HEAD verificados, artefacto vigente y referencia de inputs/receipt.
3. Operación actual: identificador, responsable, recursos propios y último evento confirmado.
4. Tabla breve: bloque, estado, evidencia, bloqueo/límite y próxima acción.
5. Restricciones y autorizaciones vigentes relevantes para reanudar.
6. Enlaces al historial íntegro, evidencias y relevo.

Estados distinguibles: pendiente, en curso, fallido, ejecución nativa aprobada con oráculo pendiente, aprobado según criterios existentes. Un recibo ausente o una operación iniciada no se transforma en aprobación. Cada resultado conserva enlace a evidencia y limitación de alcance.

## Transición segura al terminar la operación actual

1. El único escritor del checkpoint termina la operación en curso y registra su resultado o interrupción con evidencia. Este documento no interviene en esa ejecución.
2. Vuelve a leer CHECKPOINT y HANDOFF completos; reconcilia cambios posteriores al snapshot y conserva fallos, pendientes y autorizaciones vigentes.
3. Antes de abreviar, guarda una copia íntegra, fechada y con nombre único en `docs/execution/history/`; registra su ruta y huella. No sobrescribe un archivo histórico existente.
4. Prepara el checkpoint breve aparte, enlaza el histórico y comprueba que cada estado activo, bloqueo y siguiente acción tiene respaldo. Mantiene referencias a los intentos fallidos.
5. Comprueba que el checkpoint de origen no cambió durante la preparación. Si cambió, incorpora esos cambios antes de sustituirlo; no pisa trabajo concurrente.
6. Sustituye el checkpoint sólo desde el escritor exclusivo y actualiza HANDOFF con el enlace al vigente. El relevo describe el estado confirmado al cierre, con fecha, y evita duplicar otra cronología.

## Actualizaciones posteriores

El escritor exclusivo añade eventos con identificador y evidencia al historial separado y mantiene el checkpoint breve coherente con el último evento confirmado. Los colaboradores entregan archivos propios o notas al escritor, sin editar simultáneamente CHECKPOINT/HANDOFF.
Antes de una operación registra intención y recursos; después registra resultado real, evidencia, límite y siguiente acción. Un fallo se conserva y cualquier repetición remite a su motivo y a un intento nuevo.
Este protocolo organiza información existente: no añade gates, builds, ensayos, permisos ni afirmaciones de cumplimiento.
