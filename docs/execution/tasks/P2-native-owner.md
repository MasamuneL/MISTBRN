# P2 — adaptación owner integrado

schema_version: 1
fecha: 2026-11-15 America/Mexico_City
agente: /root/backend_quarantine
estado: implementation_in_progress; no native verification
base: e55db8cab13cdb6870e11ff5615cc7f14e42d221
rama: codex/alomancia-p2-owner

Intención ANTES de editar: contrato architecture + owner-review y precisión Level.uuidSomeB leídos; CHECKPOINT schema11 y docs/execution/HANDOFF.md vigentes leídos. Root autoriza dentro P2 adaptación reversible del bundle, no migración. Propiedad: integration/horizon/{world_guard.rs,preflight.rs,profile_capture.rs,owner_bootstrap.rs}, esta ficha y docs/world-identity.md. Sin descriptor/build/manifests/Cargo/core/backend.save/testdriver/SDK/commits/subagentes ni Cargo/juego.
Preimágenes SHA256: world_guard 4F3AD21B863141D206051C980BEAF8CD594A3606A85B22EC68031A0124EFCEEC; preflight C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55; world-identity.md 5B64F0CEC5C44734505C8263B638055B7DDDD1FF9BABEE8FC7C7E9BFCE7D62AB. Helpers y ficha nuevos ausentes al inicio.
Implementación prevista: perfil exacto Launch→Options; HEAD con owner esperado opcional; originales mantienen strictNBT/quarantine/core; reconstruir identidad Level y selección SDK, exigir SomeB en existentes marcados; bootstrap reaudita sólo nueva ausencia real, sin writer. Vanilla sin marker sigue sin política owner, Dedicated no normaliza.
Precisión comunicada por root: AFTER_ASSIGN exige initializer call; let mut level=level no sirve. Proponer dos BEFORE_CALL al mismo helper idempotente: write_world_gen_settings_with (UI createSome antes primer writer) y make_streaming (CLI createNone y recheck UI). Planner y build exclusivos de root; no afirmar resolución. Primer RED nativo PlayerMismatch ya preservado; no pruebas mock/texto ni resultados nuevos alegados.

## Entrega congelada para planner

Estado: implemented; planner/build/native verification PENDIENTES de /root. Primer RED real PlayerMismatch ya preservado en P2-native-identity-debug.md y snapshots originales. No se ejecutó Cargo, build, juego, mock Host ni test de texto. La revisión de fuente y el scoped git diff --check exit0 sólo acreditan edición/whitespace; hubo avisos EOL. Root conserva descriptor, planner, staging y build serial. No más cambios Rust después de entrega; si root formatea, debe registrar hashes posteriores.

Cambios: perfil exacto Launch.player→Options.player_name; helper HEAD exige perfil sólo IntegratedServer, deriva UUID mediante funciones SDK reales y aporta expected_owner opcional. validate_world(&Path,Option<[i32;4]>)->Result<(),String> conserva auditoría strictNBT/transport/core/quarantine, archivo propio/legacy/backups y vanilla sin marker. El mapa transitorio sólo retiene UUID opcional de snapshots principales ya auditados, no SavedPlayer ni segundo ledger. En marcado existente exige reconstrucción de Level UUID Some(B) y selección load_owner B en principal y backup; UUID None con fallback propio B rechaza. No se reasigna un Level existente ni se arreglan aliases.

bootstrap_owner(&Path,[i32;4],&mut horizon_client::save::Level,bool)->Result<(),String> reaudita originales con owner B. Admisión nueva exige marker válido, ambos level realmente ausentes y cero snapshots de jugador auditados; created consistente y UUID local None/B. Asigna sólo level.player_uuid=Some(B) en memoria. Repetir la llamada con B es idempotente. En existente sólo verifica !created y UUID real Some(B). Sin marker no asigna; Dedicated retorna sin normalizar. Errores del helper de inyección devuelven ControlFlow::Break(Err), sin unwrap/panic/cancelación simulada. Logs de decisiones son constantes sin nombre/perfil/UUID/ruta privados.

## Contrato de targets y bindings para /root

SDK fuente original exacta pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca; ubicaciones cotejadas por lectura. Las selecciones siguientes son las propuestas por root y los anclajes encontrados en fuente. NO resueltas, compiladas ni aplicadas aquí. El planner debe exigir una coincidencia en el método correcto y revisar el parche staged. El helper de un mismo archivo usado por dos inyecciones requiere que el generador dé nombres independientes; root verificará que no produce definiciones duplicadas.

1. profile_capture.rs: target candidato horizon_client::game::integrated::Game<'_>::open_integrated. AFTER_ASSIGN, selector propuesto self.options.clone, ordinal=0, require=1. Ancla exacta SDK integrated.rs:433: let mut options = self.options.clone();. Hay otra línea igual en run_component_harness:1279; queda fuera de este target, no usar búsqueda textual global. Locales/bindings requeridos: launch referencia ordinal0 como &Launch y options mutable ordinal0 como &mut Options. Contrato de helper: fn allomancy_profile_capture(launch:&Launch,options:&mut Options), sin retorno ni nuevo cálculo de perfil. Debe ejecutarse después de esa asignación y antes de options.execution_role/WorldLoader.
2. preflight.rs: target actual horizon_client::game::world_loading::WorldRequest::prepare_world, HEAD cancellable. Binding request:&WorldRequest=&self; options:&Options=options (argumento original, no local generado). Contrato fn allomancy_preflight(request:&WorldRequest,options:&Options)->ControlFlow<Result<PreparedWorld>>. HEAD permanece antes de cualquier fallback/recovery/upgrade/migrate/Level::read y antes de adoptar guest. Roles se usan calificados crate::game::execution_role::ExecutionRole para no depender de import no presente en world_loading.
3. owner_bootstrap.rs UI: mismo target prepare_world, BEFORE_CALL, selector propuesto horizon_client::save::write_world_gen_settings_with, ordinal=0, require=1. Call único observado world_loading.rs:173, dentro de if created && let Some(o)=create, antes del primer writer UI. Binding world:&Path desde &save_root (local ref ordinal0); level:&mut Level desde la segunda declaración sombreada let mut level=level (mutable local ordinal1); options:&Options=options; created:bool desde local ordinal0. Helper fn allomancy_owner_bootstrap(world:&Path,level:&mut Level,options:&Options,created:bool)->ControlFlow<Result<PreparedWorld>>.
4. owner_bootstrap.rs CLI/recheck: mismo target prepare_world, BEFORE_CALL, selector propuesto make_streaming (función del módulo world_loading), ordinal=0, require=1. Call único observado world_loading.rs:229. Exactamente los mismos bindings y contrato. Cubre created sin create (CLI) y reaudita UI tras la primera llamada. En UI el primer sitio debe aceptar ANTES de write_world_gen_settings_with; no presentar el segundo como la primera barrera de escritura.

let mut level=level (world_loading.rs:167) NO es CALL initializer: AFTER_ASSIGN no lo selecciona según toolchain. De ahí dos sitios idempotentes; no se reemplaza por un bootstrap tardío sólo make_streaming que dejaría writer UI antes de la comprobación. El primer sitio queda después de ajustes de memoria de Level UI y antes de la escritura, que es el límite material de este contrato.

Root advirtió que interior hooks pueden rechazar funciones con macros/closures/? u otras fronteras del generador. Esta función sí tiene esas construcciones. Si planner los rechaza, queda un límite real del toolchain y la adaptación no se declara disponible: conservar bloqueo/evidencia; no edición textual del SDK, no relajar N0, no mover validación a login, no ocultar Err con unwrap. Una alternativa requiere contrato/revisión aparte. Estos descriptores no se han escrito por esta tarea.

## Verificación y límites

Revisión estática con fuente SDK real: pub UUID helpers en horizon-client/lan/playerdata; Level pub player_uuid y selección Level::read:3949–3959; load_owner:84–95; autosave store_owner y write_level_for_role conforme revisión. La comparación de owner conserva el fallback de Level UUID a named UUID cuando un snapshot seleccionado carece de UUID; nunca usa el payload para decidir identity. Tras N0 válido, bootstrap recompueba el Level real. Backup sin principal, dato corrupto, cuarentena o IO no se convierten en ausencia.

Pendientes /root: planner/selectores/locals/cancelación; revisión staged y compilación; perfil arbitrario fresh→primer save→reopen; UI y CLI; autosave/pausa/logout/cierre antes placement; existente A abierto por B con/sin namespace sin cambios; metadata sin UUID+fallback propio B rechazo; backup/legacy; vanilla/dedicated; regresiones N0 y gates restantes. Ningún fsync ni protección frente a editores externos concurrentes o mixins ajenos inferida. Guard y bootstrap sólo leen disco y sólo bootstrap nuevo cambia un campo del Level en memoria. No hay migración ni importación de mundo existente.

## Hashes de entrega

SHA256 de bytes de entrega antes de cualquier formato por /root:

| Archivo | SHA256 |
| --- | --- |
| integration/horizon/world_guard.rs | 174BCF9F87BA1CFBE1AB8A3DFE5264458FEF7C3ECEB899D698628D81F65C99D8 |
| integration/horizon/preflight.rs | 9BC8B91BED086526AC0306340B6270C7CA35973D66135E48AD3B3F2F86C1E4B1 |
| integration/horizon/profile_capture.rs | 1F43BE8EF716DB9902B49EB35A5A1D9B70FF17344942344D294D3BE2A0CDF5EA |
| integration/horizon/owner_bootstrap.rs | 8F0C4699826F07D62D4CA412A148A2D117D3BC5EBE17ED5528836752DA26E0F2 |
| docs/world-identity.md | 50AC0C8858203650772B7822ED5F80198BAF8590EC02B50137E2C22C05353AAB |

Delta combinado SHA256 UTF-8 LF: 62DB37D34A85E57EF09C537FD242956E17CBD96FBBFE91943BDE8A85F8B879C0. Construcción: git diff de world_guard/preflight/world-identity y git diff --no-index /dev/null de profile_capture/owner_bootstrap; no incluye esta ficha para evitar autorreferencia. Se preservaron WIP preexistente y backups; la ficha se actualiza mediante temporal único + backup antes del reemplazo atómico.

## Intención HEAD-only ANTES de editar — 2026-11-15
Root aprueba contrato final CHECKPOINT + heads.md, sustituyendo interiores que fallaron E_UNSUPPORTED_CONTROL_FLOW (archivados target/p2-owner-plan1). Implementar mismos cuatro archivos Rust propios y docs/fichas, sin descriptor/build/manifests/core/backend/driver/SDK/Cargo/juego/tests/commits/subagentes. HEAD serve registra launch.owner exacto sólo con presencia marker protegida; HEAD preflight audita/admite modo explícito; HEAD Level.write primerFresh fijaUUID RAM y posteriores sólo afirman SomeB, sin DAT scan. Contexto privado compartido en único módulo library, canonicalización DIR fuera Mutex en tres puntos, owner inmutable y estado monotónico/noreset. Vanilla ambos markersNotFound no crea política; Dedicated admisión explícita. Inicializado nofsync y errorbootstrap puede dejar worldgen previo.
Fuentes preHEAD preservadas por root con hashes guard174BCF9F..., preflight9BC8B91..., capture1F43BE8..., bootstrap8F0C469...; no reset ni borrado. Siguiente: congelar código pequeño para planner2/build/revisión de root, sin afirmar targets resueltos ni native pass.

## Entrega HEAD-only vigente — 2026-11-15

Estado: implemented/no native verification; supersede implementación interior anterior conservada en target/p2-owner-plan1. RUST CONGELADO entregado a root para planner2/build real/revisión. Sin Cargo/juego/pruebas/commits/subagentes ni fuentes fuera de propiedad. Scoped diffcheck exit0; aviso EOL sólo preflight. Fallo apply_patch de formato múltiple target terminó antes de aplicar; se corrigió formato de edición sin descartar WIP.
Funciones soporte: capture_owner(&Path,[i32;4])->Result<(),String>; validate_world(&Path,OwnerMode)->Result<(),String>; before_level_write(&Path,&mut crate::save::Level)->Result<(),String>. OwnerMode Integrated/Dedicated/Unsupported; helpers y bindings finales detallados en heads.md, sin locals/interiores. Contexto privado por canonicalroot, captura exacta launch.owner y monotonicidad; primera Fresh reaudita DAT y fijaB sólo RAM, Existing/posteriores sólo assertB. Canonicalización DIR y marker metadata fueraMutex. Vanilla ambosNotFound no crea/borra contexto ni compara owner; Dedicatedadm explícita; ausenciaContextprotegido/poison/rolinesperado/error devuelvenBreakErr.
Límites: initialized no confirma writer/fsync; worldgen UI previo puede existir tras fallo; reintento nuevo LevelNone puede requerir reinicio; no migración, alias, sidecar ni DATlive. Targets/planner/build/Host real pendientes de root.
- integration/horizon/world_guard.rs: 47B054C2E8453B93324CB4104F401CB4800D08C107717622BEAFF5F4BCAEAE21
- integration/horizon/preflight.rs: BF83AED639E898AFF0391E5BF1B2477518F4B9A05CF39B4F46A18FBD2AF58342
- integration/horizon/profile_capture.rs: B61B830FF878635CB263632E24D9BDDB9D38A97013C16ADF4232547B8EFD7B0B
- integration/horizon/owner_bootstrap.rs: 878A4E2C9115A837AFD8B184F3D115425CA4E881DC88C49AAFF2FEA67B5C0DF0
- docs/world-identity.md: 76FC32B31F47E1874CB78B50577A776D91A1C708D9756164A9317DF2234B7FDA
SHA256 heads.md posterior: 23D66E45E2056082D7CDE2D7DF9FE9E169E9EF88D23FE2915E070C1B510B17CA


Precisión documental final: dos frases históricas sobre ausencia de locks ahora distinguen SessionLock/IO del Mutex RAM owner. SHA256 final docs/world-identity.md: C8004F542BB221741E8BF05660597758E264493C79D8E3F3C78C22316D27370D. Hashes Rust congelados sin cambio.
