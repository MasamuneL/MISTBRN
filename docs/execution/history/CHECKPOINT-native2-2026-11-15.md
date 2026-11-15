# Checkpoint de ejecución — Alomancia Horizon
schema_version: 10
fecha: 2026-11-14 America/Mexico_City
estado: P0/P1 integrated; P2/P3 parciales, nativo WIP sin compilar
rama: codex/alomancia-p2
HEAD: fed744b9252e14ae6fcd12b314fba61040250fe0
main_remoto_conocido: 24b4b2c5759d746caab21b67f68f6766d688548e
repo: https://github.com/MasamuneL/MISTBRN.git

## Leer primero para continuar
HANDOFF.md en esta carpeta es el relevo técnico completo, con rutas, pruebas, WIP, problemas pendientes y próxima secuencia exacta. Diseño y plan aprobados siguen vigentes; no reiniciar P0.
Worktree activo: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN.
Original/runtime compartido: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN. Conservar WIP original; no copiar SDK/toolchain ni desarrollar allí por error.

## Estado comprobado
P1/C1 integrado main mediante PR1. P2A reservas reviewed16tests. Codec P2B reviewed26tests tras fixlast_tick; workspace13+26+16+1doctest/fmt0. Catálogos/validatorP3 internos reviewed21casos y publicados enfed744b, sin runtime. No poderes jugables todavía.
WIP markercore nuevo:16tests/workspace71+1doctest/fmt0, sin revisión independiente aún. Guard NBT/HEAD helper completos pero SIN SDKcompile/runs. Backend save.rs/lib.rs en borrador, SIN compilación; driver native_test.rs y features/descriptor/staging nuevo todavía faltan. Preparadoroffline pasó suite/AST; enlaces/IOreal pendientes.
No declarar P2/P3 completos. Next: revisaridentity/backend/quarantine, integrarcorecompartido/guard/staging/driver y ensayarN0–N9 reales, revisión antes demerge.

## Reglas permanentes
- Usuario aprobó diseño/plan/GitHub/autonomía; exige ramas nuevas, pruebas y revisión antes de merge a main.
- Principal es único escritor de CHECKPOINT/manifests/lock/commits compartidos; hijos con archivos exclusivos, un Cargo/build a la vez, modelos del plan según disponibilidad.
- Checkpoints antes/después de acciones relevantes; atomic .tmp + .bak, conservar parciales/evidencias; cotejar estado real antes de repetir operaciones interrumpidas.
- Leer MAINTENANCE.md antes de builds/cierre de fase. Limpieza autorizada de copias obsoletas ya completada:8.82GiB recuperados, original16.83GiB. No repetirla. Conservar actual+baseline necesario; no borrar mundos/backups/fuentes/evidencias/dependencias/target sin decisión explícita.
- No tocar mundos personales, no fork/ediciónupstream silenciosa, no paquete con recursosMinecraft/SDK. PinAPI411/Rust1.99.1 y runtime existentes se reutilizan.
- Preservar dos diferencias P0 preexistentes de CRLF, no stage automático de sourceP0.

## Recuperación y handoff
No agentes hijos activos al preparar relevo (list_agents solo root); no build/juego del principal activo conocido. No se lanzaron nuevos builds ni pruebas al preparar este handoff.
Las fuentes nativas aún SIN COMMIT se mantienen en este worktree: revisar git status, no reset/clean/checkout destructivo. Sus hashes/entregas están en HANDOFF e informes de tareas.
Problema abierto: cuarentena persistente tras rollback incierto (storage_fault), formato/test aún no cerrados; failpointRollbackInventory artificial no es falloHostreal. Estado FaultsoloRAM no debe acreditarse como protección trasrestart. Ver HANDOFF para mensaje arquitectónico y límites.
Historial completo anterior conservado en history/CHECKPOINT-before-handoff-2026-11-14.md y .bak. Informes P2-native/P2-horizon-save pueden estar parciales por cuota; no creer solo su estado textual si hay código ya escrito.
## Reanudación 2026-11-14 — P2 nativo, paso 2
Principal: modelo de sesión Codex; subagentes conforme plan, sin heredar historial.
Lecturas de los seis documentos requeridos realizadas; git status/branch/HEAD y artefactos inspeccionados. HEAD fed744b; codex/alomancia-p2; WIP nativo preservado. No procesos cargo/rustc/Horizon/java detectados. Sin temporales de checkpoint pendientes; native-builds conserva p1-bridge1/p1-bridge2. Sin builds/pruebas repetidos.
Intención ANTES: crear rama codex/alomancia-p2-native para el nuevo corte sin otro checkout, preservando todos los cambios. Delegar revisión estática identity y backend/cuarentena, solo informes exclusivos; ninguna mutación de fuentes hasta contratos cerrados. Principal posee CHECKPOINT/manifests/staging/driver. Ningún hijo puede ejecutar Cargo/build ni commits. Revisores no repiten suite marker vigente.
Acceso al worktree necesita exec require_escalated; autorización de usuario explícita, primera comprobación exit0. Runtime/SDK compartidos se reutilizan.
Siguiente: confirmar rama nueva, revisar informes y congelar contrato storage_fault antes de implementación.
## P2 nativo — rama y revisión identity confirmadas
Resultado: git switch -c codex/alomancia-p2-native exit0; HEAD sigue fed744b y WIP preservado. Revisión Sol high aprobada solo helper identity; informe P2-identity-review.md con hashes. Ningún test repetido. Astra high revisa backend/cuarentena; Terra medium prepara QA sin juego/Cargo. Archivos de informes exclusivos, sin escritores de fuentes concurrentes.
Mantenimiento previo: runtime 17,911,392,844 bytes / 266,127 archivos; disco C libre 141,633,712,128 bytes. Receipts P1 bridge1 failed1 y bridge2 succeeded0; bridge2/probev5 son baseline conservado. Sin exports nuevos ni limpieza necesaria.
Intención siguiente: una vez contrato cuarentena revisado, asignar backend y guard por archivos exclusivos; principal modifica descriptor/staging/features/driver. Nueva build p2-native1 (no existe al inspeccionar) guardará inputs/log/receipt antes de compilar. Toolchain/SDK compartidos, un Cargo a la vez. No merge hasta N0–N9 y revisión.
## Decisión P2 cuarentena — antes de implementación
Ruling: sibling NeoForgeData[allomancy:storage_fault] = Compound{format_version:Int(1),reason:String("rollback_unverified")}; no economía ni ledger adicional. Guard preadopción (incluidos backup/legacy), login y mutadores rechazan cualquier presencia sin interpretar formato, aunque state falte. Escritura tras rollback no confirmado clona mods actuales preservando state/claves ajenas, setter inline y relectura exacta fuera de Mutex; Fault de sesión siempre. Si setter/readback no disponible: cuarentena NO confirmada, detener ensayo y recuperar snapshot completo offline. No fsync ni protección universal ante Host unreachable/reentrada ajena. Ruling basado en revisión Astra actual y HANDOFF; costo si hipótesis falsa: N7 pendiente, nunca aprobación implícita.
Bloqueante adicional a corregir: raw_player al principio de apply_credit no debe propagar ? y permitir Busy→Ready ante cuarentena/lectura inválida; debe marcar Fault.
Intención: integrar contrato en guard/backend con archivos exclusivos y pruebas nativas que fallen antes de acreditar. Congelar API existente NativeTestConfig/CreditIntent; añadir StorageError::Quarantined si hace falta. Driver/feature default OFF producción. Toda prueba pendiente sigue pendiente.
## Integración parcial descriptor/staging
Principal editó solo descriptor, Cargo.toml bridge y build-native.ps1. Descriptor contiene core crate único, módulo guard y HEAD cancellable. Fuente bridge declara native-test default OFF. Staging copia/hash helpers, transforma dependencia API a ../../native-crates/allomancy/core, y -NativeTest solo modifica default staged. -StageOnly permite inspección sin SDKcompile/export. AST inicial exit0; revisión de diff eliminó validaciones helper duplicadas introducidas al sustituir texto. No ejecutado aún staging/build.
Terra recibió propiedad exclusiva native_test.rs y P2-native-driver.md (sin Cargo) con observe/roundtrip; sin configuración exacta no ejecuta driver ni grants. Guard Sol conserva helpers y fichas guard. Backend queda pendiente asignación tras informe Astra. APIs públicas previas congeladas; principal conserva lib.rs/manifests/CHECKPOINT.
Siguiente: implementar backend cuarentena y revisar todos los cambios antes del primer SDKcompile p2-native1; producción sigue sin poderes.
## Antes de staging/plan guard real
Revisión Astra entregada P2-storage-review.md. Sol high backend_quarantine asignado solo save.rs/docs propios; guard Sol ya implementó rechazo por presencia y trazas, hashes world_guard 0B250DB096D8250B9D57E7BE569D605CAD6017398B4900A289EC017428F25DAE y preflight C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55. Terra driver aún en curso. Intento inicial spawn backend rechazado por concurrencia; segundo tras cierre revisor aceptado, sin dos escritores.
Operación próxima: tools/build-native.ps1 -BuildName p2-stage1 -RuntimeRoot <ORIGINAL>/experiments/horizon-probe/runtime -NativeTest -StageOnly. No compila ni crea export, solo fuentes y receipt; evidencia staging, NO build. Luego horizon-mixins plan con guard real y core único. No usar p2-stage1 para ensayos: backend/driver aún no congelados; build p2-native1 capturará fuentes finales. Manifest/lock SDK deben seguir intactos.
## Staging y diagnóstico planificador
p2-stage1 -StageOnly exit0, receipt staged; sin export/SDKcompile. Primer plan directo exit1 E_MANIFEST por [api] launcher-only. Segundo plan2 exit1 E_CRATE porque --mods requiere carpeta PADRE de bundles (manifest.rs:240–290), no bundle raíz. Ambas causas verificadas en launcher/api.rs::strip y mixins manifest.rs; SDK Cargo.lock intacto 0F4FE7DE... en ambas. No defectos guard acreditados ni pruebas repetidas. Próxima: plan3 con copia pequeña del bundle bajo target/p2-real-guard-mods/allomancy, sin [api]; no SDK patch. Uso skill systematic-debugging para diagnosticar frontera antes de cambios.
Backend Sol en curso; Terra driver entregado sin compilar, nueva comparación canonicalize coordinada con backend. Principal añadirá observador lifecycle en lib.rs para N0, antes de compilar.
## Plan3 — primer defecto real de integración
Plan3 exit1 E_MODULE: support module source cannot have file attributes or a shebang. Causa identificada: dos comentarios //! iniciales world_guard se interpretan como atributos de archivo no admitidos por módulo SDK. Sol guard recibió fix mínimo //!→//, sin lógica nueva. No más cambios exploratorios indiscriminados. SDK lock conservado. Observador real Hooks::world_lifecycle (firma generada API411 verificada) añadido por principal a lib.rs solo cfg(native-test): phase/epoch/error_present, sin imprimir error_text privado.
## Plan4 aprobado — antes de formato/build nativo
horizon-mixins plan4 exit0: crates1/modules1/injections1, HEAD cancellable guard real. SDK Cargo.lock antes/después igual 0F4FE7DE576DBEC180EB501BA7687145E7B51AA04F53ABB15774227D9B66E154. Target plan JSON preservado y resultado curado evidence/p2-real-guard-plan4-result.json. Corrección de comentarios guard no altera lógica. Esto NO es N0 ni compilación.
Siguiente operación relevante: rustfmt --edition2024 sobre bridge lib/save/native_test y helpers reales; luego tools/build-native.ps1 -BuildName p2-native1 -NativeTest -RuntimeRoot runtime compartido, cwd worktree activo; log docs/execution/evidence/p2-native1-build.log y receipt runtime/native-builds/p2-native1/receipt.json antes de launcher. Backend/guard/driver fuentes congeladas al relevo; no Cargo de hijos. Gate esperado SDKcompile real pin, core único, artefacto engine+receipt succeeded. Si falla, conservar intento/log y diagnosticar causa; no sobrescribir BuildName.
## SDKcompile p2-native1 RED — antes de p2-native2
Launcher exit1 / Cargo101 E0505 en native_test.rs: cierre intent presta replacement y última aserción lo movía. Core/guard/backend llegaron a compilación real; no build aprobada. Receipt failed y log preservados. Sin procesos Cargo/Horizon restantes al inspeccionar; sin export p2-native1 verificado. Terra corrigió solo Some(replacement)→Some(replacement.clone()), SHA driver 0C813CE928721812DF218D330B0F00222C7F681E71E32C620F12B205398A032D. Revisión Astra de cuarentena aprobada estática sin bloqueantes perfil acordado (P2-quarantine-review.md), gates pendientes.
Runtime medido antes de nuevo build: 19077624823 bytes / 271872 archivos. No copias export nuevas válidas; conservar baseline P1/probev5 y fuentes stage de intentos. No limpieza de target ni dependencias.
Comando siguiente: tools/build-native.ps1 -BuildName p2-native2 -NativeTest -RuntimeRoot runtime compartido; cwd worktree. Log evidence/p2-native2-build.log, receipt native-builds/p2-native2/receipt.json, sin sobrescribir intento1. Después verificar receipt, engine/hash y package core único via cargo metadata serial antes N0.
## SDKcompile p2-native2 aprobado — antes de N0
Build launcher exit0, receipt succeeded; engine SHA256 0A51CCBF7D1D695B0467E41842214B9F4E3ABCFF7CD418CE7AE97F391DB9E0BB, bytes 189029624. Todos inputs receipt coinciden con fuentes actuales. Cargo metadata --offline --locked exit0 demuestra core_count1 (native-crates/allomancy/core) y feature native-test activa solo stage; evidence/p2-native2-package-check.json. Parse PowerShell inicial falló por keys USB/usb; se corrigió usando -AsHashtable sobre salida ya existente SIN repetir Cargo. N0–N9 siguen sin ejecutar.
Mantenimiento: build1 fallida sin export; nueva build2 y cache fc09e76b9eed se conservan hasta verificar. Baseline P1bridge2 + probev5 necesario P0 no se elimina prematuramente; ningún SDK/target copiado manual/reinstalado.
Intención antes de mundos: crear directorio vacío dedicado runtime/game-p2-base/saves/AllomancyP2, provisionar UUID con tools/prepare-world.ps1 -ProvisionNew, sin mundos personales. Después lanzar build2 directamente con tools/test-native-open.ps1 Expect accept; receipt/log/hash antes/después. Harness en última ampliación Terra, esperar congelación antes de ejecutarlo. Copia baseline completa cerrada para fixtures posteriores; no modificar DAT de mundo vivo.
## Preparación baseline bloqueada por chequeo de ancestro
SDKcompile2 sí succeeded0, inputs iguales y engine189029624 bytes; script de validación/provisión se interrumpió antes de escribir evidence build-check por tool prepare exit1. Directorio runtime/game-p2-base/saves/AllomancyP2 creado y sigue vacío; NO asignó UUID ni hay mundo arrancado. Causa real: Assert-NoLinks sube por encima de AllowedRoot y rechaza OneDrive/Documents (reparse Microsoft0x9000601a, Target/LinkType nulos). No se desactiva seguridad Windows ni se copia runtime.
Siguiente acción: definir límite explícito de validación en raíz autorizada runtime; rechazar reparse en AllowedRoot y TODOS descendientes, sin auditar ancestros fuera de la frontera permitida. Es corrección de preparador para raíz compartida explícitamente autorizada, no permitir junctions dentro de la zona de prueba. Prueba RED real preservada por error antes de provisión; cambiar helper/docs y ejecutar suite del preparador por cambio nuevo. Marker aún no existe, reutilizar directorio vacío tras inspección, no crear otra baseline por error.
## Preparador corregido y baseline provisionada — antes N0 accept
Boundary-root cambio testeado: primer suite tras límite falló por FileInfo sin Parent; revisor Astra detectó el mismo defecto. Fix usa FileInfo.Directory y DirectoryInfo.Parent, chequeo ReparsePoint antes igualdad OrdinalIgnoreCase, throw si nunca alcanza AllowedRoot. Segunda suite exit0 (p2-preparation-boundary-tests2.log), junction real passed, symlink unavailable por privilegio; no se marca symlink aprobado. Provisión baseline antes vacía exit0 UUID58f1d6d013a548e3be20db72d476dd7f. Build-check curado ya escrito input_hashes_match=true.
Siguiente comando: test-native-open.ps1 -BuildName p2-native2 -RuntimeRoot original/runtime -RunId n0-base1 -WorldRoot runtime/game-p2-base/saves/AllomancyP2 -Expect accept. SDK/Host reales, engine directo sin wrapper preflight. Logs/resultado bajo runtime/native-open-runs/n0-base1, todos snapshots y marker respaldados por hashes antes/después. N0 pendiente hasta resultado, no ejecutar otros juegos simultáneos.
## Primer juego nativo p2-native2 observado — harness falló
n0-base1 engine sí abrió mundo integrado: stderr WorldOpenReady/login/ready/WorldCloseFinal; no engine vivo al inspeccionar después. Harness falló por ReadAllText logs compartidos mientras redirección activa; result failed, exit code no capturado. NO gate pass por inferencia. Solicitado fix mínimo de lectura FileStream FileShare.ReadWrite|Delete a Terra. Guard trace se inspecciona en logs; próximo n0-base2 justificado por defecto harness, misma build/fuentes. Baseline ya tiene DAT reales y backup cerrado completo runtime/backups/p2-base-after-first-open; UUID original conservado, no reprovisionar. No se tocó mundo personal.
## Antes de fixtures N0 offline
Harness corregido (SHA283967A8...), reensayo n0-base2 en curso o recién finalizado; inspeccionar result antes de avanzar. Se crearán copias dedicadas de backup cerrado p2-base-after-first-open bajo runtime/game-p2-n0cases/saves/n0-KIND, nunca de mundo vivo. Tipos: foreign/fault/fault-scalar/fault-future/future-transport/future-json/corrupt-json/invalid-root/duplicate/truncated/dat-uppercase/backup-fault. fault ya representa marca canónica sin state, no repetir alias fault-without-state idéntico. Generador solo archivo playerUUID1 nuevo con flag wx; source backup y baseline preservados. Todos ensayos seriales con RunId único, antes/después hash/trace/exit en receipt; no repetir core suites vigentes. Próximos adicionales: IO real con sharing lock de fixture foreign; marker ausente con state presente en copia (mover marker a evidencia fuera world, sin reprovisionar).
## RED nativo real: reopen PlayerMismatch, detener N0 fixtures
n0-base2 Expect accept NO pasó: guard reject y WorldOpenFailed; error real Allomancy payload PlayerMismatch al reabrir snapshot escrito por backend en primer juego. PID25968 propio confirmado game-p2-base se terminó controladamente porque engine esperaba Enter en crash; harness terminará como failed. No source/snapshot reparado. Investigar representación UUID Host u128 vs save::uuid_text y raw IntArray, NO cambiar saved data silenciosamente ni desactivar guard.
Las 12 fixtures creadas desde backup cerrado aún NO se ejecutaron; ese baseline contiene mismatch y NO puede acreditar rechazo específico. Conservar como evidencia fallida, no borrar ni reciclar para fingir N0. Primera apertura vacía no prueba roundtrip; P2 sigue pendiente. Siguiente: analizar fuente SDK UUID y snapshot/payload reales, prueba native roundtrip corregida en nuevo mundo dedicado tras build3, luego regenerar fixtures desde baseline verificada con nuevos nombres. Revisores Sol/Astra para fix identidad según causa, sin Cargo concurrente.
## Diagnóstico causal confirmado, QA de perfil alineado separado
Sol inspeccionó NBT: named owner a01e… tiene UUID NBT a01e pero payload PlayerId18eb; propia18eb coincide. SDK Level::ensure_player_uuid usa offline_uuid("Player"), save_owner get_or_insert conserva ese identificador inicial al copiar namespace Guest. No endian. Datos fallidos y 12fixtures NO ensayadas conservados como evidencia; no reparación ni aceptación relajada. Astra evalúa bloqueo de producto y opción mínima; P2 no puede cerrarse por limitar QA.
Trabajo independiente autorizado: nueva baseline runtime/game-p2-aligned/saves/AlignedP2, desde vacío con nuevo UUID, lanzamiento EXPLÍCITO --name Player (perfil offline alineado con UUID inicial SDK). Es ensayo acotado, NO arreglo producto general. Harness ahora acepta -PlayerName registrado en arguments y termina PID propio ante rechazo inesperado en Expect accept; evita esperar crash Enter, sin convertirlo en pass. Misma build2 nativa, sin recompilar. Antes marker nuevo con prepare-world, luego n0-aligned1 y reapertura n0-aligned2; SOLO si ambas pasan usar backup nuevo para N0 específico. No repetir baseline fallida ni sobrescribir fixtures antiguas.
## Perfil alineado primera apertura pass — antes reapertura
n0-aligned1 pass=true, exit0, cierre espontáneo, guard accept/WorldOpenReady/backend Ready y WorldCloseFinal. UUID de mundo e0f1ae8032614d8795371acf11dac6d2, perfil EXPLÍCITO Player. Limitación owner de producto sigue ABIERTA. Siguiente test n0-aligned2 misma build+misma identidad+perfil; reabre DAT nativo previamente escrito para verificar roundtrip inicial y permiso guard, no repite nuevo mundo ni suites core. Si pasa crear backup cerrado alineado y proceder casos negativos desde él.
## Antes de lote N0 específico — baseline reabierta pass
n0-aligned2 pass/exit0; guard y backend cargaron snapshots escritos por Horizon en perfil Player. Backup cerrado inmutable runtime/backups/p2-aligned-after-reopen. Se crean nuevas fixtures (diferentes de fallidas previas) runtime/game-p2-n0aligned/saves/n0-KIND y se ensayan secuencialmente con build2 -PlayerName Player. Casos foreign/fault/scalar/future/future-transport/future-json/corrupt-json/invalid-root/duplicate/truncated-both/dat-uppercase/backup-fault; cada resultado before/after+PID/exit/trace. Truncated tendrá primary+old inválidos; ningún .dat de mundo vivo editado. Cada acción del lote guarda checkpoint antes/después y no da pass hasta hashes iguales en rechazo.
ANTES N0 foreign : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-foreign.

DESPUÉS N0 foreign : pass=True, exit=0, termination=spontaneous; receipt runtime/native-open-runs/n0a-foreign/result.json. Próximo tipo del lote solo si pass.

ANTES N0 fault : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-fault.

DESPUÉS N0 fault : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-fault/result.json. Próximo tipo del lote solo si pass.

ANTES N0 fault-scalar : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-fault-scalar.

DESPUÉS N0 fault-scalar : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-fault-scalar/result.json. Próximo tipo del lote solo si pass.

ANTES N0 fault-future : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-fault-future.

DESPUÉS N0 fault-future : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-fault-future/result.json. Próximo tipo del lote solo si pass.

ANTES N0 future-transport : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-future-transport.

DESPUÉS N0 future-transport : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-future-transport/result.json. Próximo tipo del lote solo si pass.

ANTES N0 future-json : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-future-json.

DESPUÉS N0 future-json : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-future-json/result.json. Próximo tipo del lote solo si pass.

ANTES N0 corrupt-json : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-corrupt-json.

DESPUÉS N0 corrupt-json : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-corrupt-json/result.json. Próximo tipo del lote solo si pass.

ANTES N0 invalid-root : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-invalid-root.

DESPUÉS N0 invalid-root : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-invalid-root/result.json. Próximo tipo del lote solo si pass.

ANTES N0 duplicate : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-duplicate.

DESPUÉS N0 duplicate : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-duplicate/result.json. Próximo tipo del lote solo si pass.

ANTES N0 truncated : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-truncated.

DESPUÉS N0 truncated : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-truncated/result.json. Próximo tipo del lote solo si pass.

ANTES N0 dat-uppercase : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-dat-uppercase.

DESPUÉS N0 dat-uppercase : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-dat-uppercase/result.json. Próximo tipo del lote solo si pass.

ANTES N0 backup-fault : copiar backup alineado cerrado y crear fixture exclusiva; después lanzar engine directo RunId n0a-backup-fault.

DESPUÉS N0 backup-fault : pass=True, exit=-1, termination=controlled_after_rejection; receipt runtime/native-open-runs/n0a-backup-fault/result.json. Próximo tipo del lote solo si pass.

## 2026-11-15 — lote N0 alineado completado
12/12 receipts pass (foreign accept exit0 y once rechazos controlados exit-1 con hash DAT/marker íntegro). Sesión lote exit0. Razones específicas comprobadas en log, ejemplo future-json UnsupportedVersion(999), backup-fault storage_fault; baseline alineada no tenía PlayerMismatch. Son N0 acotados perfil Player, no cierre general P2. Sin juegos/Cargo vivos al comprobar.
Astra cuota durante ruling owner; se registra sustitución Sol high backend_quarantine, sin reintentos indefinidos. Ruling parcial Astra: save_owner no es única ruta (Game::store_owner autosave también), login tardío no protege namespace previo al cambiar perfil; se necesita guard previo adopción/load_owner. Investigación continúa sin tocar SDK ni datos fallidos.
Intención lote siguiente offline+serial: tres nuevas copias de backup alineado con RunId n0a-io (archivo foreign exclusivamente bloqueado FileShare.None durante guard), n0a-marker-missing (mover ambas copias marker a backup dedicado, con state presente), n0a-marker-future (alterar solo marker de copia nueva con versión futura). Ningún wrapper preflight, misma build2, hashes antes/después y no overwrite. Preservar originales y todos receipts. Cada caso se registra antes/después.
ANTES N0 io : copia nueva y modificación fixture cerrada; lanzar n0a-io esperando reject.

DESPUÉS N0 io : pass=True,exit=-1,termination=controlled_after_rejection,IOlock=released_after_exit.

ANTES N0 marker-missing : copia nueva y modificación fixture cerrada; lanzar n0a-marker-missing esperando reject.

DESPUÉS N0 marker-missing : pass=True,exit=-1,termination=controlled_after_rejection,IOlock=not_requested.

ANTES N0 marker-future : copia nueva y modificación fixture cerrada; lanzar n0a-marker-future esperando reject.

DESPUÉS N0 marker-future : pass=True,exit=-1,termination=controlled_after_rejection,IOlock=not_requested.

## Antes de crédito/replay Host real — perfil alineado
Los tres casos adicionales N0 IO real/marker ausente/futuro terminaron pass, exit controlado; IO log snapshot open IO error y lock released_after_exit, hashes íntegros. No procesos juego vivos. N0 observado15casos del lote + dos aperturas alineadas, con limitación owner abierta. Preparar nuevo mundo runtime/game-p2-credit/saves/CreditP2 desde backup alineado cerrado; build2 feature native-test escenario roundtrip, perfil Player y control endpoint exclusivo runtime/native-credit-runs/roundtrip1/control.json.
Intención: lanzar engine directo, esperar backendReady; vía control TCP local autenticado enviar comando vanilla give @s minecraft:iron_ingot 3 (solo este mundo de ensayo); driver consume un objeto y acredita1000Iron, replay mismoID sin segundo gasto y comprueba revisión/ledger/inventario con Host real. Al PASS pedir quit, esperar exit y conservar archivos/logs/receipt, verificar offline snapshot. Helper Node control aún no probado: resultado real puede fallar; no fingir PASS ni atribuir durabilidad por driver. Sin fsync ni cierre N1 completo/restart replay todavía.
## Antes de roundtrip2 con harness persistente
roundtrip1 NO es PASS: controlador inline falló en log inicial vacío y framebudget offscreen terminó proceso sin give; receipt actualizado ended_without_credit_exit_uncaptured, datos preservados. No driver crédito ejecutado. Terra entregó test-native-credit.ps1 AST0, elimina límites frames y controla quit con deadline120; fallos sólo PID propio y nunca PASS al matar. Nuevo RunId roundtrip2, nueva copia cerrada alineada runtime/game-p2-credit2/saves/CreditP2. Comando tools/test-native-credit.ps1 -BuildName p2-native2 -RunId roundtrip2 -WorldRoot ... -PlayerName Player. Receipt/log/comandos Node bajo native-credit-runs/roundtrip2; verificar exit0+driverPASS+WorldCloseFinal y offline payload después. Mismo engine2/fuentes Rust, no rebuild/SDKcopias.