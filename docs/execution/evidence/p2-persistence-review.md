# P2B — revisión independiente de persistencia

Fecha: 2026-11-13 America/Mexico_City. Agente `/root/p2_persistence_review`, modelo heredado. Revisión del diff `5f679d06297cddef25031d239df95b4693fca5d0..67f4d76ecd5c02f90cbaca2beaad29200cb8be0c`, rama `codex/alomancia-p2`, worktree `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`.

**Dictamen: changes_requested para P2B parcial por un hallazgo P2 de cumplimiento.** No se aprueba P2 nativa, backend Horizon, N0 ni gameplay. No se modificó código ni se ejecutó Cargo. El hallazgo se comunicó al principal antes de cerrar esta revisión.

## Hallazgo requerido

### [P2] Exigir presencia de last_tick en el JSON estricto

Ubicación: `crates/allomancy-core/src/persistence.rs:210`, en `validate_object_shapes`; relacionado con `crates/allomancy-core/src/state.rs:25` (`last_tick: Option<Tick>`). El contrato `docs/persistence.md:59` exige rechazar campos ausentes también en PlayerState. La guardia solo comprueba que `state` sea objeto y después delega su contenido al derive Deserialize. `deny_unknown_fields` no exige un campo Option: serde convierte su ausencia en None.

Caso concreto a incorporar como regresión: producir un snapshot válido con `encode_save`, eliminar únicamente `/saved_player/state/last_tick` y llamar a `decode_save` con identidades correctas. El recorrido actual acepta ese documento y construye `last_tick = None`; no distingue ausencia de `"last_tick":null`. La fixture offline v0 tiene el mismo hueco al omitir `/state/last_tick`. Esto instala un documento incompleto y contradice el esquema congelado, aunque no cambie por sí solo saldos/ledger.

La conclusión es estática, sustentada en las fuentes exactas, **no un test ejecutado en esta revisión**. En el cache local de la versión fijada serde 1.0.229, `src/private/de.rs:22-60` documenta y aplica missing Option -> None mediante `deserialize_option`/`visit_none`; serde_derive 1.0.229 `src/de.rs:791-798` usa ese missing_field para campos sin deserialize_with. PlayerState carece de override y el codec no contiene una guardia contains_key para last_tick. Los tests entregados solo eliminan `accounting` como caso de campo ausente y por ello no detectan esta excepción.

Corrección mínima recomendada: exigir presencia de last_tick en la guardia wire existente, tanto para v1 como para fixture v0, conservando null explícito como válido y sin ampliar DTOs ni cambiar la forma P1 en memoria. Añadir regresiones de ausencia y conservar round-trip con null y con Tick. Revisor/implementador deben coordinar el nuevo corte y sus gates; este agente no implementa.

## Cumplimiento revisado

| Área | Resultado del examen del corte |
| --- | --- |
| JSON, versiones y límites | Entrada y salida 4 MiB, probe tipado de versión antes del DTO, objetos antes del decode estricto, rechazo de desconocidos/duplicados por derive. Brecha única identificada: ausencia de last_tick. IDs opacos de 1..256 bytes UTF-8 y sin controles; mundo/jugador esperado por igualdad exacta. |
| Reglas y reservas | validate_save llama validate_reserves completo; el diff P2A solo añade atributos strict, sin cambiar fórmula/reglas. Encoder también valida. |
| Ledger | Vec estrictamente ordenado, IDs únicos y payload exacto metal/i64 no negativo; recibos entre 1 y revisión actual con revisiones únicas. Máximo4096 sin expulsión, replay previo al control de saturación; conflicto no muta. |
| Revisiones y rollback puro | Mutadores clonan, checked_add y validan antes de sustituir. No-op y ABA incrementan revisión; error conserva estado. Preparación conserva preimagen completa y commit exige igualdad completa, incluyendo revisión. |
| Prepare/candidate/commit | candidate contiene saldo, contabilidad, reglas, nueva revisión y recibo antes de encode/aplicación Host. Descartar no consume ID. La API no prueba aceptación del Host; orden síncrono y rollback siguen siendo contrato del adaptador. |
| Ciclo de vida | Muerte vacía reserva/restos y apaga; conserva identidad, pareja, desbloqueos, reglas, ledger. Reconexión apaga/resetea tick y conserva saldo/restos; no consume offline. |
| Fixture v0 | Separada del decode producción, artificial/offline, exige cero combustible/Off/None y reglas/IDs válidos. No inventa ledger/restos históricos. Misma excepción de campo ausente señalada. |
| Export offline | Encode y validación de principal/backup previos preceden a writes; rechaza futuro/corrupto/downgrade. create_new para temporales, rename mismo directorio, lector sin writes y origen explícito. Fallo final puede actualizar backup al principal anterior válido, conservando snapshot completo. No se identifica defecto adicional dentro de acceso exclusivo y fallos ensayados. |
| Alcance y calidad | Sin DTO espejo ni nuevas dependencias: serde_json ya fijado pasa a producción. Cambios compartidos acotados a módulo/reexport y cuatro atributos. No fsync, power-loss, economía sidecar ni crash Horizon acreditados. |

## Evidencia cotejada en lectura

Se leyeron diseño, plan P2, contratos P1, contrato reservas P2A, docs/persistence.md, ficha y entrega P2B; se inspeccionaron implementación/test mediante `git show` del commit fijo y el diff de los archivos compartidos autorizados. HEAD observado es el corte asignado. Trabajo concurrente ajeno permaneció intacto.

Get-FileHash SHA256 de los seis archivos fuente/documento compartido, Cargo.lock, docs/persistence.md y los cinco logs coincide con los valores entregados. Hashes centrales:

| Archivo | SHA-256 |
| --- | --- |
| src/persistence.rs | 356f47c4e007975190ce72b16438f218bd567f9f62aede15d3ac99bdc19873fb |
| tests/persistence.rs | 0bbc1ea7b8664c7b4c15b2af74a6c7d2240f61d5f67e91e3605a9319751322df |
| docs/persistence.md | c4ede1c8286e641fd5e3995925da495499b28a97222ef16a41faabf5351e2e8d |
| Cargo.lock | 4515b7aa2d1a0c667a9d98b78c6e0b524a5025a5df09b0aaab90fc138cefa36b |
| p2-persistence-green.log | f60c870e4f11178f6d28a45a871088f16157bb52cc8c8fc11f6cc06f18cf05f7 |
| p2-persistence-workspace.log | ac69e0c5a2a150d2f2c1122c70fa35c34c4b8896f88971218148d71632723926 |

Los logs existentes contienen RED inicial0/20, RED guardas20/22, GREEN final24/24 y workspace13contratos+24persistencia+16reservas+1doctest sin fallos. La prueba Windows usa un handle real sin FILE_SHARE_DELETE y verifica error de rename, snapshot previo y retry; otra conserva temporal parcial. El log fmt es vacío y coincide con su hash; el exit0 de fmt procede de la ejecución acreditada por el principal, no se deduce del archivo vacío. No son nuevas ejecuciones de este revisor y no se repiten gates inmutados.

La cobertura respalda las ramas revisadas, pero no prueba la presencia obligatoria del Option ni sustituye ensayos nativos. Las simulaciones old/new snapshots no prueban inventario, autosave, caída del proceso Horizon ni durabilidad.

## Relevo y límites

Resolver el hallazgo y someter el nuevo diff/test de ausencia a revisión antes de aprobar el núcleo P2B. La decisión arquitectónica de Astra que exige N0 mixin antes de load permanece cerrada y fuera de esta aprobación: no se implementó ni verificó aquí. Identidad estable real, NeoForgeData/merge ajeno, raíz corrupta, inventario exacto/rollback, callback síncrono, autosave y cortes nativos siguen siendo gates del principal. Ninguna afirmación de esta revisión habilita beber o gameplay.
## Rereview fix1 — dictamen vigente

Fecha: 2026-11-13 America/Mexico_City. Alcance exclusivamente el diff fix1 contra `67f4d76ecd5c02f90cbaca2beaad29200cb8be0c`; el informe anterior se conserva como historia del corte inicial.

**Hallazgo [P2] missing last_tick: ADDRESSED. Dictamen vigente del núcleo P2B: approved, parcial al codec/ledger/export offline revisados. No se identifica rotura introducida por fix1.** No se aprueba N0, P2 nativa, integración Horizon ni gameplay.

La guarda nueva en `crates/allomancy-core/src/persistence.rs:212` consulta `saved["state"].get("last_tick").is_none()` después de exigir que state sea objeto y devuelve InvalidSchema si falta la clave. La misma función recibe tanto el saved_player de v1 como la raíz fixture v0, por lo que cierra ambos recorridos. get distingue una clave ausente de Some(Value::Null): null explícito sigue llegando al Option tipado como None. El Tick numérico válido sigue llegando al decode tipado; no se modificaron sus tipos ni reglas. El sondeo de versión conserva su precedencia sobre esta guardia.

Las dos regresiones conductuales nuevas comprueban primero null válido y después eliminan únicamente last_tick para exigir InvalidSchema, una por formato. El round-trip no trivial existente conserva Tick(19). No se cambia API, PlayerState, módulos, manifiestos o lock frente al baseline; se inspeccionó el diff de esos compartidos, que es vacío. La aclaración contractual expresa presencia obligatoria sin cambiar formato/esquema. La modificación de producción es una guarda local compartida; no introduce rutas nuevas de mutación ni escrituras.

Evidencia revisada: logs fix1 RED y GREEN y workspace, informe del implementador y diff de fuente/tests/contrato. El RED contiene 24 pruebas aprobadas y exactamente las dos regresiones fallidas porque el valor devuelto era Ok con last_tick None; el GREEN contiene 26/26 y workspace 13contratos+26persistencia+16reservas+1doctest sin fallos. Exit101/0/0 y fmt0 son ejecuciones acreditadas por implementador/principal; este revisor leyó los logs y no ejecutó Cargo. fmt.log vacío no se usa aisladamente para inferir exit0.

Los siete SHA-256 siguientes se cotejaron contra entrega y coinciden:

| Archivo | SHA-256 fix1 |
| --- | --- |
| src/persistence.rs | 200a8b6eb1a8ade154f47e8179ded4a2a395546270b0151d68c4ca406ffa79a7 |
| tests/persistence.rs | ecb0c410ceb49b517ca15be45bea26f495fca370ff5d1427156bbf3ce81f677a |
| docs/persistence.md | 761b3a386c165849756cd57cf09066cbff69166080ca7accca9539fe80213808 |
| p2-persistence-fix1-red.log | efd6d0ed53f90ec60efe141afb148f45c84d5dc564218cb622b35dd2b7dd6d42 |
| p2-persistence-fix1-green.log | 1d0fb36c3b59e701e36f4d9d93f27710c8bcb82de596a5684eb97ab552f39677 |
| p2-persistence-fix1-workspace.log | b75497d81e03bb2c7d7179a1a043e83fa44491516f1ad64465065a1d2428eef5 |
| p2-persistence-fix1-fmt.log | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |

`git diff --check 67f4d76 --` para fuente/test/contrato exit0; solo aviso de normalización EOL de docs. La revisión acotada no reabre la auditoría general ni repite gates inmutados. Persisten íntegros los límites previos: export offline sin fsync/power-loss, simulaciones de snapshots sin crash Horizon, ausencia de economía sidecar, y gates nativos N0 mixin antes de load/identidad/NeoForgeData/inventario/rollback/autosave pendientes del principal. Siguiente acción: principal integra el corte fix1 aprobado del núcleo y continúa el gate nativo separado.