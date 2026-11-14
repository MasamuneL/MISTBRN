# P2B — contrato de codec y ledger
schema_version: 1
fecha: 2026-11-13T16:19:48-06:00
estado: implementation_ready_for_review (núcleo verificado; integración pendiente)
agente: /root/p2_persistence; modelo heredado Sol
base: 24b4b2c5759d746caab21b67f68f6766d688548e
rama: codex/alomancia-p2
worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN
propiedad: docs/persistence.md, docs/execution/tasks/P2-persistence.md, docs/execution/evidence/p2-persistence.md

## Intención antes de editar
Preparar contrato acotado de SavedPlayer, JSON versionado estricto, revisión monotónica, ledger persistente sin expulsión y prepare/commit. Leer diseño/plan P2, contratos P1, reservas P2A congeladas y análisis P0 de persistencia. No implementar Rust, tocar manifiestos/lock/SDK/contratos compartidos ni ejecutar Cargo, commits o subagentes.

Estado previo: los tres documentos propios no existían; no hay temporales propios anteriores. Workspace compartido con P2A y principal, cuyas modificaciones se preservan. Resultado esperado: contrato congelado y lista de cambios de integración/pruebas para relevo serial explícito.

Dependencias: P2A posee reserves.rs/tests y lib.rs; principal posee manifiestos, lock, contratos compartidos y CHECKPOINT.md. SavedPlayer por jugador aprobado por principal; backend real NeoForgeData inline. Revisión/ledger forman parte del candidato antes de encode y aplicación al Host. Mutaciones exitosas incluso no-op incrementarán revisión para invalidar pendientes por ABA.

Pruebas actuales: lecturas y git status/branch/HEAD exitosos; sin Cargo ni runtime. Limitación: documentación no prueba codec, rollback, writer ni durabilidad.

Siguiente acción exacta: escribir docs/persistence.md y evidencia mediante temporales del mismo directorio; validar coherencia documental y git diff --check; sustituir esta ficha atómicamente conservando .bak; informar al principal y esperar relevo explícito para implementación.

## Relevo de implementación explícito
fecha: 2026-11-13T16:33:00-06:00
base: 5f679d06297cddef25031d239df95b4693fca5d0
estado: in_progress; contrato congelado, Cargo reservado por este agente
Validación documental y git diff --check exit0; temporales promovidos, referencias presentes. Principal autorizó persistence.rs/tests, lib.rs solo módulo/reexport, state.rs solo strictserde PlayerState/MetalState, reserves.rs solo strictserde Rules/Accounting y nota doc. Cargo.toml movido por principal a serde_json producción; no tocar manifiestos/lock.
Antes de editar: escribir pruebas conductuales reales y scaffold temporal compilable con comportamiento ausente; ejecutar RED dedicado con logs p2-persistence-red.log. No atributos strict ni implementación antes de RED. Toolchain/cache original autorizado, workdir explícito; no Cargo paralelo. Después de RED, implementación mínima, formato solo archivos propios, GREEN, workspace y fmtcheck. Sin commits/subagentes/SDK/integración Horizon.
## RED observado; antes de implementación
cargo test --locked --offline -p allomancy-core --test persistence exit101: compila y ejecuta 20 tests, 20 fallan por comportamiento ausente, sin errores de importación/sintaxis. Log docs/execution/evidence/p2-persistence-red.log. Scaffold temporal de datos/funciones exportado solo para RED; no attrs strict todavía.
Siguiente acción: sustituir scaffold persistence.rs por codec/validación/ledger/mutadores/export mínimo, añadir atributos strict únicamente autorizados, y nota de contrato reservas. Luego rustfmt mutante solo esos archivos y tests propios, GREEN dedicado. No mutar manifiestos/lock ni P2A lógica.
## GREEN inicial; revisión de bordes antes del gate
rustfmt solo persistence.rs/tests, state.rs/reserves.rs autorizados exit0; test persistence GREEN exit0, 20/20. Código real, sin warnings. Antes del gate revisar formas JSON de structs (serde también acepta secuencias) y backup de esquema futuro: agregar tests de regresión, observar RED si confirman brecha, corregir mínimo y repetir dedicado. No ampliar integración SDK.
## RED guardas observado
Suite dedicada22: exit101,20 pasan/2 fallan; log p2-persistence-red-guards.log. Causas reproducidas: serde derive acepta structs como arrays posicionales aunque deny_unknown_fields; writer valida principal pero no backup antes de reemplazarlo. Corrección mínima: comprobar forma objeto en JSON acotado antes del decode tipado (que aún detecta duplicados) y validar ambos snapshots existentes antes de writes. No DTO espejo/SDK/failpoints artificiales.
## GREEN guardas; antes de verificación final
Suite dedicada22/22 exit0; forma objeto verificada antes de typeddecode y ambos snapshots existentes validados antes de export. Sin DTO espejo. Añadir verificación filesystem real de rename Windows con handle que impide delete y temporal interrumpido existente; son ramas de error ya implementadas, sin mocks/failpoints SDK. Después dedicado final, workspace y fmtcheck con logs requeridos. Cargo reservado aún.
## Recuperación documental antes de promoción
fecha: 2026-11-13T21:16:57-06:00
HEAD observado: 1da2b11ced4f4a6b0503526019e6d2f7c9d3d38f; rama/cwd correctos. Se leyeron ficha, temporales, logs reales y hashes. Fuentes y logs coinciden con el corte probado; no hay trabajo de código incompleto. Fuente/Cargo/lib.rs liberados desde el gate previo.
Estado antes de modificar: persistence.md.tmp contiene actualización completa a implementado; evidence/p2-persistence.md.tmp recuperado y completado con resultados, exitcodes, API, límites y hashes. Ambos originales aún describían solo contrato; se conservarán como .bak. Orden vigente: SOLO cerrar estos documentos y ficha; NO Cargo/Rust/manifiestos/lock/SDK/scripts/commits/subagentes. No reinstalar toolchain ni recrear engines eliminados.
Siguiente acción exacta: validar temporales de documentación, promover con File.Replace mismo directorio y backup, registrar sus hashes y resultado de diffcheck. No repetir pruebas ya acreditadas. Revisión independiente del principal pendiente; no declarar P2 completa.
## Cierre verificado para revisión independiente
estado: implementation_ready_for_review; Cargo/lib.rs/fuentes liberados
Resultado del gate observado antes de la interrupción: dedicado final24/24 exit0; workspace13contratos+24persistencia+16reservas+1doctest exit0; cargo fmt --all -- --check exit0. Sin warnings de compilación. RED inicial exit101 (0/20) y RED guardas exit101 (20/22); logs reales y SHA-256 en evidence/p2-persistence.md. Últimas dos pruebas verifican ramas filesystem existentes: rename real Windows denegado y temporal parcial interrumpido.
Cierre documental retomado: fuentes y logs rehashados coinciden con corte; cero Cargo/nuevas pruebas/cambios de fuente tras reanudar. docs/persistence.md y evidence/p2-persistence.md promovidos mediante File.Replace con .bak válido; no temporales documentales pendientes. git diff --check exit0; solo avisos de normalización EOL de Git en workspace compartido.
SHA-256 docs/persistence.md: c4ede1c8286e641fd5e3995925da495499b28a97222ef16a41faabf5351e2e8d
SHA-256 docs/execution/evidence/p2-persistence.md: e1de3cc0416e24270e52994259e2c1304471136c7f245434ec628899ac9f8623
SHA-256 fuente persistence.rs: 356f47c4e007975190ce72b16438f218bd567f9f62aede15d3ac99bdc19873fb
SHA-256 tests/persistence.rs: 0bbc1ea7b8664c7b4c15b2af74a6c7d2240f61d5f67e91e3605a9319751322df
Archivos del corte P2B: src/persistence.rs, tests/persistence.rs, src/lib.rs (solo módulo/reexport), src/state.rs (solo dos attrs), src/reserves.rs (solo dos attrs), docs/reserves.md (nota), docs/persistence.md, esta ficha, evidence/p2-persistence.md. Hashes de esos fuentes compartidos y Cargo.lock intacto en evidencia. Cargo.toml fue cambio del principal, no de este agente.
Límites: export offline exclusivo con rename/backup, sin fsync ni power-loss acreditado; sin aplicación inventario/Host, identidad mundo resuelta o backend NeoForgeData integrado. Ledger4096 íntegro sin expulsión bloquea nuevos IDs al saturar. No compras/hitos/XP inventados. Muerte/reconnect preservan desbloqueos/ledger; reconnect conserva restos. No P2 completa ni SDK integración declarados.
Siguiente acción exacta: principal revisar el corte SHA-256 y despachar revisión independiente; después integrar serialmente adaptador real con gates NeoForgeData, inventario/rollback, autosave y cortes nativos. Este agente no reabre Cargo ni modifica fuente sin nuevo relevo explícito.