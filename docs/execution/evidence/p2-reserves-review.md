# P2A — revisión independiente de reservas

Fecha: 2026-11-13, America/Mexico_City. Revisor delegado independiente del implementador y del integrador; modelo exacto no expuesto. Worktree: `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`. Rama: `codex/alomancia-p2`.

**Base:** `24b4b2c5759d746caab21b67f68f6766d688548e`. **Corte inmutable revisado:** `5f679d06297cddef25031d239df95b4693fca5d0`.

## Dictamen

**Spec compliance P2A: approved. Code quality/correctness P2A: approved.** Sin hallazgos accionables P0–P3 en el alcance revisado. Esta aprobación corresponde a reservas puras P2A; no cierra la fase P2 ni acredita codec, SDK, inventario, autoridad de red o durabilidad.

## Alcance y método

Se leyeron el diseño aprobado, plan y sección P2, fichas P2/P2-reserves, `docs/reserves.md`, `evidence/p2-reserves.md`, y el diff completo de `lib.rs`, `src/reserves.rs` y `tests/reserves.rs` contra la base. Se consultaron también state/types/commands del corte para comprobar unidades, orden y fronteras. Todas las fuentes y documentos del dictamen proceden de `git show 5f679d0:<ruta>` o `git diff 24b4b2c 5f679d0`; los cambios concurrentes P2B/P3 del worktree quedan excluidos. La revisión aplica el criterio de simplicidad Ponytail y verifica evidencia antes del cierre.

No se ejecutaron Cargo, launcher, SDK, juego, instalaciones, commits ni subagentes. No se modificaron fuentes, manifiestos, lock, CHECKPOINT ni archivos ajenos. Se inspeccionaron directamente los logs locales y sus hashes; no se presenta esa inspección como una nueva ejecución de pruebas.

## Comprobaciones de contrato y código

Referencias de línea corresponden exclusivamente al corte `5f679d0`.

| Criterio | Evidencia y conclusión |
| --- | --- |
| Unidades y dt | `reserves.rs:133–140`: tasa entera en milésimas/tick, reducción con denominador 1000, ticks explícitos y resto sumado una sola vez. La división euclídea y la conservación de su resto hacen equivalente agrupar ticks representables sin transiciones intermedias. Cero ticks conserva el estado válido; `last_tick` queda intacto. |
| Fracciones y pausa | `reserves.rs:12–15,85–88,128–148`: accounting serializable separado, resto <1000; apagar conserva fracción y reserva. Cambiar reducción mantiene la unidad. El test de 17 ticks confirma reserva 87/resto 600; pausa, cero dt y cambio de reducción tienen aserciones conductuales. |
| Normal/avivado y ocho metales | `reserves.rs:126–131`: `Metal::ALL`, normal 1/avivado 3, independiente de pareja seleccionada. Tests contrastan ambos modos con reducción y agotan simultáneamente los ocho en orden explícito. Potencia x2 pertenece a P4 y no se implementa aquí. |
| Agotamiento | `reserves.rs:141–145`: comparación >= apaga tanto al alcanzar como al exceder la reserva, borra fracción y devuelve el metal una sola vez en orden fijo. Tests incluyen 10/11/100 ticks y crédito posterior sin reactivar. |
| Topes y capacidad | `reserves.rs:33–61`: reducción <=200 y bonus <=500, sin clamp; tasas/capacidades positivas en todos los índices, capacidad floor(base*(1000+bonus)/1000). El producto de u64 por u16 cabe en u128 y el resultado exige u64. Tests ejercitan límites, rechazo 201/501, redondeo 101→151 y overflow del último metal apagado. |
| Rechazos sin mutación | `reserves.rs:70–90,102–111,122–152`: validación global previa; consumo calcula en arrays locales y solo aplica al terminar. Numerador u128 y cociente u64 comprobados, incluso si la reserva finita se agotaría. Tests comprueban estado/accounting y bytes intactos ante regla/estado/resto inválidos y overflow tardío en Bronze. Crédito rechaza i64::MIN/negativos, overflow y exceso de capacidad antes de asignar, sin aceptar parcialmente. |
| Serde y ciclo de vida | `reserves.rs:4–15,156–162`: reglas/accounting serializan; serde no sustituye validación. El test round-trip guarda estado, accounting y reglas, apaga explícitamente para simular reconexión y conserva resto sin consumo offline. Muerte vacía reservas/restos y apaga, preservando identidad/desbloqueos/selección/tick; es idempotente y admite limpieza de estado inconsistente. Reconexión automática sigue siendo obligación del consumidor futuro. |
| Alcance y calidad | `lib.rs` añade solo módulo/reexport. State/types/commands no cambian respecto a la base; Activate/Flare/Drink continúan rechazados y advance_tick no consume. Código directo, sin dependencias nuevas ni abstracciones innecesarias. Helpers y documento exigen llamador confiable, sin atribuirse autenticación, deduplicación o confirmación durable. |

## Evidencia existente cotejada

| Registro | Resultado observado directamente |
| --- | --- |
| `p2-reserves-red.log` | Compila y ejecuta 16 tests; 16 fallos por assertions conductuales, sin fallo de importación/sintaxis. Exit 101 registrado por implementador en ficha/entrega. |
| `p2-reserves-green.log` | 16 tests aprobados, 0 fallidos; exit 0 registrado en ficha/entrega. |
| `p2-reserves-workspace.log` | 13 contratos P1 + 16 reservas + 1 doctest compile_fail aprobados, 0 fallidos; exit 0 registrado en ficha/entrega. |
| `p2-reserves-fmt.log` | Archivo vacío de 0 bytes, coherente con check silencioso; exit 0 registrado en ficha/entrega. El log vacío por sí solo no codifica exit status. |
| `git diff --check 24b4b2c 5f679d0` | Ejecutado por el revisor, exit 0. |

SHA256 recalculados por el revisor y coincidentes con la entrega:

- RED: `7f8892e1e74a6d12a90b357b33aed213827f4fde403b8935dadd58e52c88a002`.
- GREEN: `5b960f106d2ed19e2c726f78ad86f1499ac213495b2be9103061e3e13a9f5b9f`.
- Workspace: `a3b43c5e6a35e8b0164ed32d3344e8cb08816356e0e4e512b5557c75826945db`.
- Formato: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

## Límites y siguiente acción

La aceptación conservadora de overflow forma parte del contrato: un lote puede rechazarse aunque agotaría una reserva pequeña. La equivalencia de agrupación presupone aritmética representable y ausencia de transiciones intermedias. Créditos repetidos suman repetidamente; no son transacciones idempotentes. Serde no es un codec de guardado estricto.

P2B debe revisar su propio corte, incluidos atributos `deny_unknown_fields` posteriores, y persistir accounting junto al estado bajo identidad estable. Integración sigue debiendo apagar al reconectar, evitar consumo offline/replay, validar autoridad e inventario y probar rollback/cortes/guardado real. Ninguno de esos pendientes es un defecto de la entrega P2A acotada ni queda aprobado aquí.

Siguiente acción del principal: registrar este dictamen y continuar P2B/integración con sus gates; no marcar la fase P2 completa por esta aprobación. Los dos informes de esta revisión quedan listos para integrar; no se realizó commit ni merge.
