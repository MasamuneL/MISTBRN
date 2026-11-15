# Handoff — Alomancia Horizon
schema_version: 1
fecha: 2026-11-14 America/Mexico_City
motivo: usuario pide prompt para continuar en otro chat

## Rutas y Git
Worktree ACTIVO y writable: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN.
Original preservado: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN. No desarrollar sobre su WIP antiguo. Su CHECKPOINT tiene aviso que apunta al activo.
Runtime/SDK/toolchain compartidos: <ORIGINAL>/experiments/horizon-probe/runtime. Operaciones que escriban allí o en .git compartido pueden necesitar require_escalated; usuario ya autorizó desarrollo/GitHub/ensayos dedicados. No copiar/reinstalar todo al worktree.
Repo: https://github.com/MasamuneL/MISTBRN.git.
Rama actual: codex/alomancia-p2. HEAD local y último push: fed744b9252e14ae6fcd12b314fba61040250fe0.
Main remoto conocido: 24b4b2c5759d746caab21b67f68f6766d688548e (P1 integrado mediante PR1). Comprobar remoto antes del próximo merge, sin reset/clean ni sobrescribir WIP.
Dos diferencias P0 preexistentes de CRLF en evidence/p0-initial-probe.rs y experiments/horizon-probe/bundle/api/src/lib.rs no son trabajo nuevo; conservar sin stage.

## Verificado/revisado y publicado
- P0/C0 y P1/C1 integrados. Pin Horizon fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca, nightly-2026-11-08-fdc87ae, API411; Rust1.99.1 WindowsGNU. No reabrir P0 ni repetir gates vigentes.
- P2A reservas reviewed, commit5f679d0: 16 pruebas; consumo entero con restos, reducción20%, capacidad+50%, flarex3, agotamiento sin mutaciones parciales.
- P2B codec/ledger reviewed tras fix1, commitd160197: 26 pruebas persistencia +13 contratos +16 reservas +1 doctest; fmt0. SavedPlayer versionado por jugador/mundo, revision/ledger, prepare/candidate/commit, strict JSON, lifecycle y export offline. No es backend nativo ni recibo durable.
- P3 parcial interno reviewed, commitfed744b: 8 metales,4parejas,32items,26recetas,21casos validador. Booleanos/DAG/grants corregidos; fuentes loot/trade propuestas NO verificadas. No paquete hmod/bridge/fabricación/beber todavía.
- No hay poderes jugables ni P2 nativa completada. No merge de P2 a main.

## Fuentes NATIVAS WIP sin commit: no recompilar ciegamente ni perderlas
1. Core world_identity.rs + tests/world_identity.rs y reexport lib.rs: 16 tests marker pasaron, workspace71 pruebas +1doctest yfmt0. Falta revisión independiente de ese nuevo módulo. API en docs/world-identity.md.
2. integration/horizon/preflight.rs y world_guard.rs: fuentes completas según agente, NO compiladas contraSDK ni ensayadas. Guard HEAD cancellable protege WorldRequest::prepare_world antes de migración/carga. Auditoría rawNBT antes de BTreeMap, decoder SDK, transport/payload/schema/UUID/marker. Extensión .DAT Windows corregida; hash FINAL world_guard CD26849BDD037979AC2A49FBC46B6CF3E87586EBBEDC71B9D87F5929280B34D4. Evidencia docs/execution/evidence/P2-native-guard.md (P mayúscula), ficha P2-native-guard.md.
3. allomancy-horizon/src/save.rs y lib.rs: código borrador real Host Ready/Busy/Fault, lifecycle, rollback y primitivas de ensayo. NO Cargo/build/juego ejecutados sobre él; ficha P2-horizon-save.md todavía parcial/desactualizada. Revisar fuente y completar informe antes de confiar en estado. No source docs/horizon-storage.md final encontrado al handoff.
4. tools/prepare-world.ps1 y test-world-preparation.ps1, docs/world-preparation.md: suite+AST pasados en fixtures temporales, no juego. API WorldRoot existente, AllowedRoot existente y ProvisionNew explícito solo para directorio totalmente vacío. Enlaces/junction y fallo IO real no probados por permisos; no marcarlos pass.

## Integración pendiente concreta
Leer docs/execution/tasks/P2-native.md y p2-storage-architecture.md completos.
Descriptor integration/horizon/horizon-mixin.toml AÚN SOLO tiene [api] P1: no ha incorporado guard/inyección/módulo/core compartido. tools/build-native.ps1 AÚN usa nesting core dentro API (P1). Cambiar solo staging/manifests compartidos como principal:
- [[crate]] id=core path=crates/allomancy-core, alias generado allomancy_core. SDK lo coloca en native-crates/allomancy/core.
- [[module]] file=crates/horizon-client/src/game/world_loading.rs, name=allomancy_world_guard, source=world_guard.rs.
- HEAD cancellable target=horizon_client::game::world_loading::WorldRequest::prepare_world, helper=preflight.rs, binding request:&WorldRequest=&self.
- Copiar/hash ambos helpers al bundle; API staged dependency core debe apuntar a ../../native-crates/allomancy/core para que API y mixin usen UN paquete core. No duplicar igual nombre/version en dos paths ni tocar upstream. Conservar source Cargo.toml hermano del repo; transformación solo staged y receipt.
Planner exploratorio ya resolvió target/binding (exit0, SDKCargo.lock intacto) en target/p2-guard-planner/plan.json. Helper de ese PROBE devuelve Continue y no es guard funcional. Usar fuentes WIP reales, no el probe, al construir.

## Driver de ensayos todavía falta
lib.rs bridge ya declara cfg(feature="native-test") mod native_test y rutea hooks a native_test::make_hooks. No existen src/native_test.rs ni [features] native-test en Cargo.toml todavía. No activar feature faltante y fingir compilación. Principal debe integrarlos, default OFF producción.
API materializada: hooks_without_native_test(); hooks_with_native_test(config, fn(&SaveBackend,&mut dyn Host)) -> (BoxHooks,ArcBackend). NativeTestConfig ata root/worldID exactos; native_test_state(uuid) -> (SavedPlayer,generation); seed/credit_for_native_test e intent slot<36. Ver fuente exacta. Host411 NO tiene native_inventory online público: usar player_stacks + server_players + player_compound/recheck y documentar límite de mod ajeno reentrante.
No Host falso ni tests sobre texto fuente; ensayos con SDK/Host real y mundos dedicados preparados. Ninguna prueba kernel sustituye N0–N9.

## Asunto de rollback pendiente de cerrar antes de afirmar N7
Astra dio ruling en mensaje al interrumpirse por cuota: sibling NeoForgeData[allomancy:storage_fault] como cuarentena escrita/verificada sincrónicamente si rollback no se confirma; guard N0+login+mutadores rechazan presencia incluso malformed/futura y preservan claves ajenas. Dentro del perfil sin snapshots reentrantes no recomendó prearmar cada operación; snapshot previo completo cubre crash antes de marca. No equivale a fsync ni cubre setter unreachable/offline/mod externo fuera del perfil. Este ruling NO quedó completamente documentado ni implementado; confirmar formato/test/alcance antes de añadirlo.
Ni guard ni backend tienen storage_fault todavía. CreditFailpoint::RollbackInventory actual devuelve Err artificial; eso NO acredita fallo real de Host/IO. Debe probarse el error real o etiquetar el límite, jamás simular éxito. No crear un segundo ledger/sidecar económico ni escribir .dat live.

## Toolchain para continuar
<RUNTIME>/launcher/rust/1.99.1-x86_64-pc-windows-gnu; PATH bin + mingw64/bin; CARGO_HOME=<RUNTIME>/launcher/cargo; RUSTC=tc/bin/rustc.exe; CARGO_INCREMENTAL=0.
RUSTDOC=<RUNTIME>/devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe y RUSTDOCFLAGS=--sysroot <tc>. Oficial extraído funciona con DLLs del bin enPATH; no restaurar en bucle ni cambiar seguridad. No borrar devtools entero.
Un Cargo/build a la vez. El planificador mixins invoca cargo metadata --offline --locked: también necesita env correcto. Tool existente <RUNTIME>/launcher/native/tools/5a64af938ec8/bin/horizon-mixins.exe.

## Retención y modelos
Se limpiaron8.82GiB de exports/cachesP0 obsoletos; carpeta original16.83GiB medidos. Conservar P1bridge2, probev5, cacheactual250494..., native/target, SDK/toolchain/cargo, mundos/backups/evidencias. Leer MAINTENANCE.md antes de builds. Usuario autoriza limpieza rutinaria de duplicados sustituidos, no borrar target/fuentes/evidencia/guardados. No reinstalar SDK ni recrear builds eliminadas sin motivo.
Modelos según plan: gpt-6.1-sol high para contratos/autoridad/integración; gpt-6.1-terra medium para helpers/QA; gpt-6-luna medium datos/docs; gpt-6-astra high revisión compleja puntual. fork_turns:none y propiedad de archivos disjunta. No depender de agentes/historia del chat anterior; al handoff list_agents muestra solo root, no hijos ni builds activos conocidos. Cuotas interrumpieron algunos informes; verificar archivos antes de reintentar.

## Siguiente secuencia exacta
1. Leer checkpoint+estehandoff+diseño/plan; git status/HEAD/branch y procesos/receipts, conservar WIP.
2. Revisar identity module y completar documentación/estática del backend, resolver cuarentena de rollback; congelar fuentes/APIs y permisos de archivos antes de delegar.
3. Integrar descriptor/core único/staging/feature+driver, planificar guard real y compilar SDK pin con nuevo BuildName sin sobrescribir intentos anteriores. Guardar comando/log/receipt ANTES, comprobar resultados DESPUÉS.
4. N0 nativo integrado SIN wrapper de preflight; después N1–N9 de Astra (namespace ajeno, futura/corrupción/IO/duplicados/.DAT, roundtrip/replay, rollback/cortes/autosave/lifecycle). No tocar mundos personales ni prometer atomicidad jugador/chunk/fsync.
5. Revisión independiente del nuevo corte y gates reales. Solo entonces cerrar P2/merge a main según autorización y continuar siguiente tarea del plan. Ramas nuevas y pruebas antes de cada merge.