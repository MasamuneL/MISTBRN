# P2A — reservas puras: entrega para revisión

Fecha: 2026-11-13 America/Mexico_City. Base `24b4b2c5759d746caab21b67f68f6766d688548e`, rama `codex/alomancia-p2`. Workspace exclusivo usado: `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`.

Estado: implementación P2A y verificaciones de núcleo terminadas; pendiente revisión/integración por principal. **No declara P2 completa.** Sin commits, subagentes, SDK build, dependencias nuevas ni cambios de manifiestos/lock/comandos/tipos/estado/contratos globales/adaptador. Archivos ajenos concurrentes vistos en status se conservaron intactos.

## Contrato entregado

`docs/reserves.md` se congeló y anunció antes de consumidores. `lib.rs` añadió únicamente módulo y reexport de reservas; se liberó al principal antes del gate y este agente no volvió a editarlo.

- `ReserveRules`: tasas base por metal en milésimas/tick, capacidades base, reducción global máxima 200/1000 y bonus máximo 500/1000. Valores requeridos explícitamente; tasas/capacidades positivas. No hay balance por defecto. Valida todos los metales, límites y overflow de capacidad; capacidad redondeada hacia abajo.
- `ReserveAccounting { remainder:[u16;8] }`: serializable, separado de PlayerState; resto en fracciones de milésima con denominador fijo 1000. Se valida incluso tras deserializar. Guardar junto al jugador; apagar/reconectar conserva resto, muerte lo limpia. Cambiar reducción no cambia su unidad.
- `validate_reserves`, `credit_reserve`, `consume_reserves`, `clear_reserves_on_death`, `ReserveRules::{validate,capacity}` y `ReserveError` reexportados desde el núcleo. Crédito firmado i64 rechaza negativos, suma desbordada y exceso de capacidad sin aceptación parcial.
- Consumo normal x1/avivado x3 por ticks. Todos los resultados se calculan en arrays locales antes de aplicar; fallo en un metal no modifica otros. Agotamiento exacto o sobrepasado apaga y borra resto, con lista de metales agotados en orden Metal::ALL. No toca last_tick ni integra comandos P1.

## Evidencia TDD y comprobaciones

| Comando | Exit | Resultado observado |
| --- | --- | --- |
| `cargo test --locked --offline -p allomancy-core --test reserves` RED | 101 | 16 pruebas compilan, se ejecutan y fallan por assertions conductuales ante scaffold temporal sin comportamiento; sin errores de importación/sintaxis. |
| `rustfmt --edition 2024 crates/allomancy-core/src/reserves.rs crates/allomancy-core/tests/reserves.rs` | 0 | Formateo mutante limitado a esos dos archivos propios. |
| `cargo test --locked --offline -p allomancy-core --test reserves` GREEN | 0 | 16/16 reservas aprobadas. |
| `cargo test --locked --offline --workspace` | 0 | 13 contratos P1 + 16 reservas + 1 doctest compile_fail aprobados; cero fallos/advertencias. |
| `cargo fmt --all -- --check` | 0 | Formato de workspace aprobado. |

Logs locales ignorados en esta carpeta: `p2-reserves-{red,green,workspace,fmt}.log`. El log fmt está vacío porque el formato correcto no emite salida; exit0 se observó en ejecución y se registró en la ficha exclusiva.

Las 16 pruebas ejercitan código real, sin mocks del SDK ni lecturas de fuente: ticks agrupados vs separados con fracciones; pausa/cero dt sin borrar resto; agotamiento exacto/sobrepasado; avivado x3 frente normal con reducción; ocho metales simultáneos en orden fijo; máximo20%/+50% operativo y capacidad con redondeo; reglas inválidas en último metal; estado inválido/deserializado y resto fuera de rango; overflow de numerador u128 y de cociente u64 tardío sin consumo parcial; overflow de capacidad incluso con metal apagado; crédito negativo/i64::MIN, overflow y capacidad excedida sin mutación; validación global antes de crédito; crédito exacto/cero sin activar ni desbloquear; serialización con resto y pausa de reconexión; cambio de reducción conservando unidad; muerte idempotente preservando identidad/desbloqueos/otros campos.

Toolchain existente original: runtime `C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime`; tc `launcher/rust/1.99.1-x86_64-pc-windows-gnu`; PATH tc/bin + runtime/mingw64/bin; CARGO_HOME runtime/launcher/cargo; RUSTC tc/bin/rustc.exe; RUSTDOC runtime/devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe; RUSTDOCFLAGS `--sysroot <tc>`; CARGO_INCREMENTAL=0. Escalación limitada al uso autorizado del toolchain/caches original. Comandos siempre con workdir explícito, Cargo offline/locked y sin Cargo paralelo.

## SHA-256 del corte

| Archivo | SHA-256 |
| --- | --- |
| `crates/allomancy-core/src/reserves.rs` | `01144899660dcc2617206c8437b1aa11030ccdec5e3cdb95dd2f0de084341627` |
| `crates/allomancy-core/tests/reserves.rs` | `415d76d5cf3a7d5a76f128b5666336d58ac40c3f53adb12b2f53d76d84070608` |
| `docs/reserves.md` | `321ccaeaf557ca6dc56a46042bdf75c7e2e9e7d66478b40dfb6edfd61b1794cf` |
| `Cargo.lock` (sin modificar) | `4515b7aa2d1a0c667a9d98b78c6e0b524a5025a5df09b0aaab90fc138cefa36b` |
| `p2-reserves-red.log` | `7f8892e1e74a6d12a90b357b33aed213827f4fde403b8935dadd58e52c88a002` |
| `p2-reserves-green.log` | `5b960f106d2ed19e2c726f78ad86f1499ac213495b2be9103061e3e13a9f5b9f` |
| `p2-reserves-workspace.log` | `a3b43c5e6a35e8b0164ed32d3344e8cb08816356e0e4e512b5557c75826945db` |
| `p2-reserves-fmt.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Límites y traspaso

Helpers internos no autentican clientes, inventario ni TransactionId, y no afirman aplicar al Host ni guardar durablemente. Crédito no deduplica transacciones: cada llamada válida suma. Integración debe verificar contexto/identidad/ticks y aplicar idempotencia/rollback. Serde no es un codec de guardado ni validación de esquema. El codec futuro debe guardar accounting, validar estado completo, apagar al reconectar sin consumo offline y limpiar al morir. P1 sigue rechazando Activate/Flare/Drink como antes; advance_tick no consume.

Se rechaza el lote si el numerador comprobado u128 o su cociente u64 desborda, incluso si la reserva finita terminaría agotada: límite explícito conservador, sin saturación silenciosa. Agrupar ticks conserva resultado para aritmética representable y sin transiciones intermedias. Las reglas no admiten tasa/capacidad base cero; apagado corresponde a BurnState::Off. Bonus de potencia del avivado no pertenece a este módulo.

Principal propuso evaluar `deny_unknown_fields` para DTO de guardado. No se incorporó a reservas por instrucción expresa de no cambiar contrato adicional antes de esta entrega; deberá coordinarse si el codec estricto lo exige. Faltan codec/ledger/versiones, integración servidor/guardado real, inventario, fault injection y revisión independiente para cerrar P2.

Cargo liberado al principal al entregar. Fuente estable; siguiente acción: revisión del corte y consumidor de persistencia contra `docs/reserves.md`.
