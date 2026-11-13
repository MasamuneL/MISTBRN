# P2A — reservas puras
schema_version: 1
fecha: 2026-11-13 America/Mexico_City
estado: in_progress
base: 24b4b2c5759d746caab21b67f68f6766d688548e
rama: codex/alomancia-p2
worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN
propiedad: reserves.rs, tests/reserves.rs, lib.rs solo módulo/reexport, docs/reserves.md, esta ficha y evidence/p2-reserves.md

## Registro
- Inicio: diseño aprobado, sección P2, P2.md, contratos P1 y fuentes leídos. Sin cambios de comandos/estado/tipos/manifiestos/lock/adaptador.
- Antes de contrato y RED: fijar API pura con reglas por tick, denominador fijo 1000, restos serializables separados de PlayerState. Preparar firmas temporales y pruebas conductuales para RED compilable; no consumidores antes de aviso.
- Cargo reservado por este agente; principal informado por despacho. Sin commits ni subagentes.
- Contrato docs/reserves.md congelado y comunicado al principal; 16 pruebas conductuales escritas, scaffold temporal compilable listo. Antes de RED: cargo test --locked --offline -p allomancy-core --test reserves; log p2-reserves-red.log. Reglas por tick evitan introducir frecuencia y denominadores mutables; serde ya existente, sin dependencias nuevas.

- RED observado: cargo test --locked --offline -p allomancy-core --test reserves exit101; compiló y ejecutó 16/16, todas fallaron por comportamiento ausente (assertions), sin errores de import/sintaxis. Log docs/execution/evidence/p2-reserves-red.log. lib.rs liberado al principal tras módulo/reexport, no se volverá a editar/formatear por este agente.
- Antes de GREEN: sustituir exclusivamente scaffold de reserves.rs por validación completa y cálculo local antes de aplicar. Después formatear solo reserves.rs/tests/reserves.rs y ejecutar test dedicado.

- GREEN dedicado observado: rustfmt --edition2024 reserves.rs tests/reserves.rs exit0; cargo test --locked --offline -p allomancy-core --test reserves exit0, 16/16 pasan, cero advertencias. Log p2-reserves-green.log. Sin cambios de API.
- Antes de gate: principal confirma que persistence.rs y su mod/reexport aún no existen; workspace libre. Ejecutar cargo test --locked --offline --workspace y cargo fmt --all -- --check con logs separados. No SDK build ni cambios de archivos ajenos.

- Gate observado: cargo test --locked --offline --workspace exit0: 13 contratos P1 + 16 reservas + 1 doctest compile_fail ejecutados y aprobados. cargo fmt --all -- --check exit0. Logs p2-reserves-workspace.log y p2-reserves-fmt.log. Cero advertencias.
- Antes de cierre: revisar diff y hashes exclusivos, redactar evidence/p2-reserves.md con límites y traspaso; no nuevas acciones Cargo necesarias. No declarar P2 completa: falta codec/adaptador/persistencia/inventario.

- Cierre P2A: diff revisado (lib sólo module/reexport), hashes e informe docs/execution/evidence/p2-reserves.md escritos. Reglas/estado/arithmetic validan antes de mutación; restantes límites documentados. Fuente estable para revisión, sin commits; P2 no completa. Cargo liberado al principal. Estado: implementation_ready_for_review.
