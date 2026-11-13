# P1 — revisión independiente

Fecha: 2026-11-13, America/Mexico_City. Revisor: agente delegado independiente del implementador e integrador; modelo exacto no expuesto. Worktree único: `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`. Rama: `codex/alomancia-p1-resume`. Base: `54469d64852499e671faff2da710b8fea5de442b`. HEAD revisado: `76af126`; commit de código: `ec27064a8acc00b0e22a2ff03ffba59d089ee8c0`. PR: https://github.com/MasamuneL/MISTBRN/pull/1 (borrador según checkpoint; no se modificó su estado).

## Dictamen

**Spec compliance: approved. Code quality/correctness: approved.** No se identificaron hallazgos accionables ni bloqueantes de prioridad P0–P3 en el alcance P1 revisado. **C1 es apto para integrar P1**: harness del núcleo y contratos de datos fijados, con adaptador mínimo real comprobado. Este dictamen no ejecuta ni declara el merge y no acredita poderes, consumo, inventario, progresión, guardado durable o cooperativo.

## Alcance e inspección

Se leyeron CHECKPOINT vigente, diseño completo, sección P1 y contratos internos del plan, `docs/contracts.md`, diff completo contra la base (fuentes, tests, manifiestos/lock, helpers, README y documentos/evidencia de recuperación). Se excluyeron expresamente las dos diferencias P0 locales de terminaciones CRLF: `p0-initial-probe.rs` y `bundle/api/src/lib.rs`; `git diff --ignore-space-at-eol` confirma que no añaden comportamiento P1. El append de reanudación en CHECKPOINT es propiedad del principal.

- **Núcleo y contrato:** IDs opacos, ocho metales/cuatro parejas y milésimas enteras coinciden con los contratos. `commands.rs:192–206` valida autoridad, mundo, actor, geometría y tick antes de mutar. `apply_command` rechaza input capturado salvo apagado de seguridad; solo Extinguish y SelectPair pueden completar una transición. Activate/Flare validan bloqueo/reserva antes de NotImplemented; Drink/Purchase/Claim también rechazan sin gastar ni conceder. Apagado e idempotencia siguen Metal::ALL; selección no cambia combustión. `advance_tick` solo registra ticks estrictamente nuevos, sin consumo ni sistemas futuros.
- **Frontera confiable:** snapshot/contexto y VerifiedEvent no implementan Deserialize; campos del evento son privados. El constructor público exige contexto servidor y geometría válida, y el contrato atribuye al adaptador la observación real del Host, sin presentar ese chequeo estructural como autenticación de texto cliente. No hay ruta de red cliente ni reclamación activa en P1. Las firmas futuras están documentadas como reservadas, sin stubs que aparenten soporte.
- **Pruebas:** los trece contratos ejercitan transiciones no vacías, rechazo sin mutación (incluidos bytes serde), autoridad de todas las variantes, geometría no finita, identidad, tick, input capturado, operaciones futuras, orden de ocho metales e enteros u64::MAX. El doctest compile_fail protege la ausencia de Deserialize del evento. La inspección de fuente y dependencias concuerda con su propósito.
- **Adaptador real:** `allomancy-horizon/src/lib.rs:30–39` exige IntegratedServer y ServerTickEnd con mundo en marcha; usa UUID/posición/tick/epoch obtenidos del Host, ordena jugadores y elimina sesiones desconectadas. Input se fija conservadoramente a false y solo se solicita Extinguish. Mantiene copia previa y restaura ante rechazo o efectos inesperados. Los únicos efectos admitidos son señales AcceptedInMemory; no se confirma aplicación física o durable. Epoch está delimitado y documentado como identidad transitoria P1. Se cotejaron los tipos en la fuente API411 y las firmas generadas reales `Hooks::server_tick`, `Host::server_players` y `ServerTickEvent` del runtime fijado.
- **Manifiestos y herramientas:** el workspace puro excluye deliberadamente el adaptador, cuyo gate es el build nativo. Versiones de Rust y revisión completa Horizon están fijadas. El build rechaza destinos existentes y verifica inputs, copia solo fuentes/descriptor, anida el core dentro del API staged y transforma exactamente una coincidencia del path en la copia del manifiesto; registra transformación/hash, estados y exit code mediante receipt atómico. Smoke usa directorio de mundo nuevo y proceso propio oculto, guarda resultado/timeout y logs; el verificador busca el marcador emitido exclusivamente desde el callback servidor. No se introduce un SDK falso ni se distribuyen recursos, binarios o runtime.

## Evidencia existente cotejada en esta revisión

No se ejecutó Cargo, launcher, juego ni instalación, ni se repitieron los gates válidos. Se inspeccionaron directamente los registros en disco y se calcularon hashes de solo lectura.

| Evidencia inspeccionada | Resultado observado |
| --- | --- |
| `p1-resume-tests.log` y `p1-resume-checks.json` | 13 contratos aprobados, 0 fallidos; 1 doctest compile_fail aprobado, 0 fallidos; test_exit=0, fmt_exit=0, adapter_fmt_exit=0. El doctest sí se ejecutó; no se descartó por la incidencia histórica de rustdoc. |
| `runtime/native-builds/p1-bridge2/receipt.json` y `p1-bridge2-build.log` | status=succeeded, exit_code=0, error=null; compilación real de core/adaptador y engine contra el pin. |
| `runtime/p1-bridge2-engine/native-mods.json` | Solo allomancy 0.1.0; Horizon fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca, Rust 1.99.1 y target server. El texto incremental del log menciona caché previa, pero el registro del export final no incluye el probe. |
| `runtime/game-p1-bridge2/smoke-result.json`, `smoke.json`, `smoke-stderr.log` | timed_out=false, exit_code=0, error=null; 9 chunks; tres marcadores reales accepted=true tick_recorded=true y carga de allomancy 0.1.0. |
| SHA256 de ocho inputs registrados y tres archivos de normalización EOF de `p1-integration.json` | Cero discrepancias con los hashes finales declarados. La única diferencia de descriptor frente al receipt está documentada como eliminación de blanklines EOF sin cambio semántico. |
| SHA256 engine, stderr smoke, manifiesto staged y rustdoc oficial extraído | Los cuatro coinciden con `p1-integration.json`, incluido rustdoc 69CF788C36D4646A531BAB89024F11E8FC0130C7243F903672CC0D1BAC0E9163. |
| Hash de tests, contratos, Cargo.lock y verificador frente a `p1-recovery.json` | Los cuatro coinciden; no cambiaron desde el corte comprobado. |
| `git diff --check 54469d6 HEAD` | Exit 0. |

El runtime original se leyó únicamente desde `C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime`; no se modificó. No hubo consultas ni mensajes a terceros.

## Límites y siguiente acción

La aprobación corresponde a P1 y C1, no al ciclo jugable completo. Seleccionar pareja está habilitado en el núcleo pero no se ofrece todavía como entrada del juego; el adaptador es un harness de ticks y apagado de seguridad, conforme a P1. Serde no es codec validado de guardado y AcceptedInMemory no es recibo durable. No se exige identidad persistente de mundo en este corte; P2 deberá definirla y preservar los gates ya aceptados de corrupción NeoForgeData, rollback, replay, cortes y errores antes de habilitar consumo o recompensas.

Siguiente acción del principal: registrar este dictamen y el cierre de revisión en CHECKPOINT/documentación vigente, comprobar el corte a integrar y llevar P1 a main conforme al flujo autorizado. C1 puede cerrarse al registrar su integración; P2 conserva todos sus requisitos y pruebas propios. Esta revisión no modifica código, manifiestos, CHECKPOINT, commits ni el PR.
