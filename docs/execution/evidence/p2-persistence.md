# P2B — entrega del núcleo de persistencia para revisión

Fecha: 2026-11-13. Agente `/root/p2_persistence`, Sol heredado. Base de implementación `5f679d06297cddef25031d239df95b4693fca5d0`, rama `codex/alomancia-p2`, worktree `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`. Al retomar el cierre documental, HEAD observado `1da2b11ced4f4a6b0503526019e6d2f7c9d3d38f`; los hashes de fuentes y logs coincidían con el corte probado antes de la interrupción. No se repitió Cargo ni se modificó fuente al reanudar.

## Alcance realizado

Contrato congelado antes de implementar y relevo explícito del principal después de P2A. Archivos propios nuevos: `crates/allomancy-core/src/persistence.rs`, `crates/allomancy-core/tests/persistence.rs`, `docs/persistence.md`, ficha exclusiva `docs/execution/tasks/P2-persistence.md` y esta evidencia. Cambios puntuales autorizados: módulo/reexport en lib.rs, `serde(deny_unknown_fields)` en PlayerState/MetalState y ReserveRules/ReserveAccounting, nota strictserde en docs/reserves.md. No se alteraron campos de PlayerState ni lógica de reservas.

Principal movió serde_json =1.0.151 de dev a dependencies de producción; manifiestos siguieron siendo su propiedad. Cargo.lock conserva su hash previo. Sin commits ni subagentes, SDK build, cambios de comandos/tipos, scripts, mundos, backend Horizon o contratos globales por este agente. **Estado: implementation_ready_for_review; P2 completa e integración Horizon no acreditadas.** Cargo y lib.rs se liberaron al principal después del gate final.

## API y comportamiento entregados

- `SavedPlayer::new`, getters inmutables, `validate_save`, `encode_save`, `decode_save(bytes, expected_world, expected_player)`. PlayerState P1 no se amplía; accounting/reglas/revisión/ledger comparten envelope por identidad de mundo/jugador suministrada por el adaptador confiable.
- JSON v1 estricto, bytes acotados, identidad esperada obligatoria al decode, validación de reglas/restos y ledger completa antes de instalar. No overwrite ante error/futuro.
- Ledger de créditos i64 por metal sin expulsión, máximo 4096; saturación bloquea nuevos IDs y conserva replay. Mismo ID/payload distinto rechaza intacto.
- `prepare_credit` devuelve Replay o Pending(PreparedCredit), con candidato completo codificable y original intacto. `commit_credit` verifica preimagen completa y revisión; contrato obliga a aplicarlo después del Host completo, pero el núcleo no puede probar esa aplicación. Descartar preparado no consume ID. No equivale a fsync ni recibo del Host.
- `update_player` clona/valida y protege identidad; `replace_rules`, `on_death`, `on_reconnect`. Toda mutación exitosa incrementa revisión, incluso no-op/ABA; errores revierten campos y revisión íntegros. Consumo P2A se invoca dentro de update_player.
- Muerte limpia reservas/restos y conserva desbloqueos/ledger; reconexión apaga/resetea tick y conserva fracciones, sin combustible gratis.
- `migrate_fixture_v0` artificial offline separada de producción; P1 no desplegó saves. Rechaza combustible previo porque no puede inventar restos/IDs. `write_export_atomic` y `read_export_with_backup` son export/prueba offline: no sidecar económico ni .dat live. El lector informa Primary/Backup sin writes.
- Canon Horizon NeoForgeData inline; identidad estable, merge ajeno, corrupción raíz, inventario exacto, rollback y cortes quedan como gates reales, sin promesas de atomicidad jugador/chunk o durabilidad.

## TDD y comandos ejecutados

| Comando / etapa | Exit | Resultado observado |
| --- | --- | --- |
| `cargo test --locked --offline -p allomancy-core --test persistence` RED | 101 | Compila y ejecuta 20 pruebas; 20 fallan por comportamiento ausente, sin errores de importación/sintaxis. |
| Mismo dedicado GREEN inicial | 0 | 20/20 aprobadas. |
| Mismo dedicado RED guardas | 101 | 20 aprobadas y 2 fallan: arrays posicionales de structs admitidos por serde y backup futuro que podía reemplazarse. |
| Mismo dedicado GREEN guardas | 0 | 22/22 aprobadas tras corrección mínima. |
| `rustfmt --edition 2024` solo fuentes autorizadas y test propio | 0 | Sin formato mutante global. |
| Dedicado final | 0 | 24/24 aprobadas; las dos últimas amplían verificación de ramas filesystem ya implementadas. |
| `cargo test --locked --offline --workspace` | 0 | 13 contratos P1 + 24 persistencia + 16 reservas + 1 doctest compile_fail; cero fallos. |
| `cargo fmt --all -- --check` | 0 | Formato de workspace aprobado; no salida. |
| `git diff --check` y lectura de diff acotado | 0 | Solo módulo/reexport y cuatro atributos strict en los archivos compartidos autorizados. |

Logs locales ignorados: `p2-persistence-{red,red-guards,green,workspace,fmt}.log`. GREEN guarda el gate final de 24 pruebas. El log fmt vacío corresponde al exit0 observado en la llamada que también terminó exit0; no se infiere exit0 solo del archivo vacío. La ficha conserva la secuencia previa y el cierre.

Toolchain original usado: runtime `C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime`, tc `launcher/rust/1.99.1-x86_64-pc-windows-gnu`, PATH tc/bin + runtime/mingw64/bin; CARGO_HOME runtime/launcher/cargo; RUSTC tc/bin/rustc.exe; RUSTDOC runtime/devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe; RUSTDOCFLAGS `--sysroot <tc>`; CARGO_INCREMENTAL=0. Comandos offline/locked, cwd explícito, sin Cargo paralelo. No se reinstaló toolchain ni se recreó engine al retomar el cierre.

Las pruebas usan implementación real y filesystem temporal, sin mocks SDK ni aserciones sobre texto fuente. Cubren round-trip no trivial/restos/ledger, versión antes de DTO, JSON estricto y forma objeto, corrupción/truncado/tamaño/identidad, reglas/capacidad/fracciones, crédito/replay/conflicto/saturación, rollback de mutadores, revisión overflow/ABA/no-op, muerte/reconexión, fixture v0 segura y rechazada, snapshot previo/posterior a commit y recuperación offline.

Rename real Windows: un handle sin FILE_SHARE_DELETE provoca error de sustitución; se conservan principal y backup anteriores completos, se limpia temporal propio y un retry después de cerrar handle guarda el nuevo estado. Un temporal parcial preexistente no se promueve ni sobrescribe. Backup futuro, principal futuro/corrupto, backup corrupto, destino bloqueado y principal ausente se verifican sin overwrite. Esto no es una terminación real del proceso Horizon ni ensayo de pérdida de energía.

## SHA-256 del corte verificado

| Archivo | SHA-256 |
| --- | --- |
| `crates/allomancy-core/src/persistence.rs` | `356f47c4e007975190ce72b16438f218bd567f9f62aede15d3ac99bdc19873fb` |
| `crates/allomancy-core/tests/persistence.rs` | `0bbc1ea7b8664c7b4c15b2af74a6c7d2240f61d5f67e91e3605a9319751322df` |
| `crates/allomancy-core/src/state.rs` | `6436c9f418822284662247c17e141bf592c387b66b373f6dfafa448c66a13dc2` |
| `crates/allomancy-core/src/reserves.rs` | `a0207224ae0f8797eb9adaff42df773e6933be3e063047a6d34a709246ce9b78` |
| `crates/allomancy-core/src/lib.rs` | `1d3e33a2f4b454fe954aa2d4171753d3eb4ffd67dc569952a783e9f56a99ed61` |
| `docs/reserves.md` | `28628ecb901bbd6357551794d0b68817d82aed290d134c7e4fdbd3f6b4c5c17e` |
| `Cargo.lock` (sin modificar) | `4515b7aa2d1a0c667a9d98b78c6e0b524a5025a5df09b0aaab90fc138cefa36b` |
| `p2-persistence-red.log` | `b4f11770249a93f290314fa20fffede403d9fec6aa22a41d9345448a75df79fe` |
| `p2-persistence-red-guards.log` | `d53c972bd3633fee067d2484ea43f57212f16200b516e995b9607d137fe764f3` |
| `p2-persistence-green.log` | `f60c870e4f11178f6d28a45a871088f16157bb52cc8c8fc11f6cc06f18cf05f7` |
| `p2-persistence-workspace.log` | `ac69e0c5a2a150d2f2c1122c70fa35c34c4b8896f88971218148d71632723926` |
| `p2-persistence-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Hashes de documentos finales se registran en la ficha después de su promoción atómica, evitando una autorreferencia del informe.

## Límites y siguiente acción

Ledger íntegro sin expulsión hasta 4096; al saturar bloquea nuevos créditos, conserva replay. Clones/inserción/serialización O(n), validación O(n log n), lookup O(log n). JSON limitado a 4 MiB y IDs a 256 bytes; no autenticación criptográfica. Crédito es la única operación transaccional implementada: no compras ni hitos/P3-P5 ni campos inventados de conocimiento.

La utilidad offline exige acceso exclusivo y valida principal/backup antes de reemplazar; no fsync, no garantía de power-loss, transacción jugador/chunk ni test de writer Horizon. Las simulaciones previas/posteriores a commit comprueban snapshots del núcleo, sin inventario Host. La seguridad de rollback y orden Host→commit sigue siendo responsabilidad explícita del adaptador.

Canon real pendiente: NeoForgeData inline namespaced, identidad estable resuelta por Host, player_compound fuera de acting para detectar raíz corrupta, merge ajeno, slot exacto, rollback y autosave/cortes nativos. No se habilitó beber ni se escribió .dat live. Revisión independiente del corte y posterior integración serial del principal son la siguiente acción; no declarar P2 completa.

Cargo y fuentes liberados; cierre documental recuperado de temporales propios, sin repetir verificaciones costosas ni tocar fuentes. Revisión independiente pendiente del principal.
