# P1-core — informe de implementación

Fecha: 2026-11-13, America/Mexico_City. Base: `54469d64852499e671faff2da710b8fea5de442b`, rama `codex/alomancia-p1`. Workspace utilizado explícitamente: `C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN`.

Estado: implementación lista para revisión; **gate de doctest pendiente por componente rustdoc ausente**. No declara P1 verificado ni build/ejecución del adaptador. Sin commits; sin modificación de manifiestos, lock, bridge o código del SDK por este implementador.

## Archivos y comportamiento

- `crates/allomancy-core/src/{lib,types,state,commands}.rs`: núcleo independiente del juego, ocho metales y cuatro parejas, IDs opacos serde, cantidades enteras de milésimas, estado/snapshot/comandos/efectos y eventos confiables.
- `crates/allomancy-core/tests/contracts.rs`: trece pruebas reales contra el núcleo, sin mocks del SDK ni lecturas del código fuente.
- `docs/contracts.md`: contrato congelado y anunciado al principal antes de implementar comportamiento y antes del consumidor.
- `docs/execution/tasks/P1-core.md`: registro antes/después actualizado con reemplazo atómico y backup `.bak`.

Solo están habilitados apagar uno/todos y seleccionar pareja. Apagar produce transición real y señales aceptadas en memoria, en orden fijo de metales; repetición idempotente. Snapshot inválido o comando rechazado conserva estado exactamente igual, también sus bytes serde. Activate/Flare verifican desbloqueo/reserva y rechazan NotImplemented; Drink/Purchase/Claim rechazan NotImplemented sin consumo ni premios. Tick válido estrictamente nuevo actualiza únicamente last_tick; duplicado/anterior o inválido no altera nada. No consumo, economía, poderes, catálogos ni codec de guardado anticipados.

VerifiedEvent tiene campos privados, constructor desde observación del adaptador servidor, validación de geometría y carece de Deserialize. La autenticidad del Host es responsabilidad de la frontera confiable; no deserializar authority ni construir evidencia a partir de mensajes cliente. Las firmas futuras se documentan reservadas y no se exportan como stubs.

## RED y verificaciones observadas

| Comando | Exit | Resultado observado |
| --- | --- | --- |
| `cargo test --locked --offline --workspace` (RED) | 101 | Compila 13 casos; 11 fallan por comportamiento ausente, 2 pasan. Fallos pertinentes: autoridad, validación, apagado, selección, tick y constructor de evento. |
| `cargo test --locked --offline --workspace` (primer GREEN) | 0 | 13/13 pruebas de integración ejecutadas y aprobadas, 0 unitarias. Salida llegó a Doc-tests sin resultado del ejemplo, por lo que no acredita ese gate. |
| `cargo fmt --all -- --check` | 0 | Formato verificado. Formateo mutante previo limitado a los cinco archivos Rust propios. |
| `cargo test --locked --offline --doc --verbose` (diagnóstico) | 101 | rustdoc.exe ausente, os error 2, proceso nunca ejecutado. **No es un fallo de reglas ni un doctest probado.** Principal avisado; restaurará componente oficial. |

Logs locales ignorados: `docs/execution/evidence/p1-core-red.log` y `p1-core-green.log`; este último incluye fmt y diagnóstico fallido sin ocultarlo. Avisos del primer GREEN: el filesystem OneDrive no permitió hardlinks del cache incremental y rustc copió archivos. No se observaron otros avisos de fuente.

Entorno de ejecución: binarios portables del runtime original `experiments/horizon-probe/runtime/launcher/rust/1.99.1-x86_64-pc-windows-gnu/bin`; PATH incluye ese bin y `mingw64/bin`; `CARGO_HOME` es `experiments/horizon-probe/runtime/launcher/cargo`; `RUSTC` y `RUSTDOC` se fijaron a rutas absolutas de bin. Cargo siempre con `--locked --offline`, un build a la vez y target raíz ignorado.

Casos cubiertos: rechazo de todas las variantes como cliente; mundo/jugador equivocados; posición y objetivo NaN/+inf/-inf; tick atrasado; metal bloqueado/reserva cero; contexto capturado y apagado de seguridad; ramas futuras sin mutación; apagado individual bloqueado/vacío; los ocho metales en orden; determinismo con efectos no vacíos y estado distinto del inicial; selección idempotente sin activar combustión; ticks nuevos/duplicados/atrasados y u64::MAX; eventos servidor/cliente y geometría; round-trip de IDs y reserva u64::MAX sin pérdida.

## SHA-256 del corte de revisión

| Archivo | SHA-256 |
| --- | --- |
| `crates/allomancy-core/src/lib.rs` | `f5e9cab3856e9663e41887eeef8e0f129485d3d0aaf952bd96b9c82f53d0dab5` |
| `crates/allomancy-core/src/types.rs` | `88a855816a280a11779326112f6af21df7f01ee12281df2167926d64fa08564e` |
| `crates/allomancy-core/src/state.rs` | `e765bd0b0daf449305f493d9bb322f2eeac52222c27e3ffeb66703c474ee5f6a` |
| `crates/allomancy-core/src/commands.rs` | `85e0f60efb5e756902301a0476c972e187e89f9b205b6fb68764f7e874491b95` |
| `crates/allomancy-core/tests/contracts.rs` | `e086a8a098f3df407a39da882bd22e8dd7139b43e917720c997cfecdee5e3e58` |
| `docs/contracts.md` | `bdb0305a83b92e0de018a53a20504cadb4b4243407fd16ddf9d5fa62ed3beb7d` |
| `Cargo.lock` (solo lectura) | `4515b7aa2d1a0c667a9d98b78c6e0b524a5025a5df09b0aaab90fc138cefa36b` |
| `p1-core-red.log` | `96cdd6db83c2cd1112acec20816e05f484b643e1c91adf850ccb3a9d90d8b15f` |
| `p1-core-green.log` | `ef85d82d0bd0cf2e4b64ac3b76fc61c72d2677f97599c638a8de5abd050096ae` |

## Límites y siguiente paso

Effect::Persist, AcceptedInMemory, serde y epoch transitorio no confirman aplicación al Host, fsync ni transacción crash-safe. Aplicación con copia previa/pendientes/rollback y gate P2 de raíz NeoForgeData corrupta están explícitos en contratos. Categorías de efectos futuras no tienen emisores P1 y deberán ampliar sus payloads antes de habilitarlas. Identidad persistente del mundo y codecs quedan en P2.

Siguiente paso: principal restaura rustdoc real, reejecuta suite completa y confirma el ejemplo compile_fail contra Deserialize; revisar este corte y compilar/probar el adaptador real con SDK fijado. Este implementador libera Cargo y no ejecutará nuevos builds sin coordinación.
