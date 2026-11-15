# P2 nativo — contratos de integración y propiedad
schema_version: 1
fecha: 2026-11-13 America/Mexico_City
estado: in_progress; implementación nativa aún no ejecutada
base: fed744b9252e14ae6fcd12b314fba61040250fe0
rama: codex/alomancia-p2

## Dependencias comprobadas
P1/C1 main24b4b2c; reservasP2A reviewed; codecP2B fix1 reviewed26+13+16+1doctest, commitd160197. Catálogo/validadorP3internos reviewed21, commitfed744b. Main no ha recibido P2/P3. Astra p2-storage-architecture.md prescribe N0–N9; es la referencia del protocolo.
Planner exploratorio en target/p2-guard-planner resolvió HEADcancellable WorldRequest::prepare_world y &self contraAPI411, exit0, sin mutarSDKlock. Helper exploratorio Continue no es guard funcional ni build aprobada.

## Contrato marker compartido (núcleo sin dependencia de juego)
Archivo `<world_root>/allomancy/world-identity.json` y `.bak`, JSON UTF8 estricto `{schema_version:1,world_id:"<32 hex minúsculas UUID N>"}`. UUID provisionado Guid.NewGuid().ToString('N') por herramienta offline; jamás epoch/semilla/nombre/ruta/fecha ni regenerar al error. Máximo4096bytes; objeto exacto, campos presentes, tipos correctos, duplicados/extra/futuro rechazados. Backup/principal si presentes deben coincidir semánticamente; principal inválido NO cae a backup. Principal ausente+backupválido retorna fuente Backup sin reescribir; ambos ausentes retorna None; cualquier IO distintoNotFound rechaza.
API a congelar por implementador guard: WorldIdentitySource::{Primary,Backup}; LoadedWorldIdentity{world_id:WorldId,source:WorldIdentitySource}; WorldIdentityError categorías Parse/Schema/Future/InvalidId/Conflict/Io/TooLarge; decode_world_identity(&[u8])->Result<WorldId,...>; load_world_identity(&Path)->Result<Option<LoadedWorldIdentity>,...>. No crea ni escribe marker desde core ni guard.
Mundo sin marker y sin namespace Allomancy puede abrirse como vanilla; backend Allomancy permanece deshabilitado. Namespace existente sin marker es error de identidad, no reset. Para ensayo P2 se provisiona mundo dedicado cerrado antes de abrir. La herramienta offline no promete auditoría NBT: el guard nativo la hace antes de carga. Si el mundo ya contiene progreso pero el marker falta, recuperar marker original, no asignar otro.

## NBT y mutación
NeoForgeData["allomancy:state"]=Compound{format_version:Int1,payload:ByteArray(encode_save(SavedPlayer))}. No serdeJSON lossy de JavaString; bytes codec exactos. Alias de core compartido en SDK debe ser único: plan raíz propone [[crate]]id=core→native-crates/allomancy/core y manifest API staged apunta al mismo path. No doble paquete core ni reimplementar codec en guard.
N0 lee originales antes de migración/Level::read, valida marker, namespace/transport/payload/identidad y rechaza datos inválidos/futuros/corrupción antes de abrir. Auditoría duplicados NBT debe ocurrir antes de normalización BTreeMap; reutilizar SDK cuando permita y documentar cualquier límite exacto, no fingirlo con player_compound. Sin escribir datos ni reparar silenciosamente. No usar callbacks trait tardíos como barrera ni wrapper como prueba de producto.
Backend usa Host real, fuera de acting para player_compound+slot exacto; snapshot de raíz no-Compound rechaza intacto; clonar/preservar claves ajenas. Ready/Busy/Fault por jugador; no mantener mutex a través de Host que pueda drenar callbacks. Guardar candidato completo(incluyeledger/revision) tras aplicar inventario exacto; commitcore solo al aceptar Host completo. Rollback o bloquear explícitamente si falla; ACK no fsync. No sidecar económico ni .dat live escrito manualmente.
Death/reconnect/loggingout según Astra: no asumir acceso a Host en player_died; colas/eventos y orden real deben trazarse y probarse antes de habilitar consumo. No nuevos comandos gameplay ni poderes: callbacks/escenarios nativos P2 de ensayo opt-in, restringidos a mundos preparados dentro runtime.

## Propiedad y coordinación
- Sol guard: core/src/world_identity.rs, tests/world_identity.rs, core/lib.rs solo module/reexport; integration/horizon/preflight.rs y world_guard.rs; docs/world-identity.md; fichas/evidenciaP2-native-guard. Nada de persistence/state/reserves/Cargo/manifests/buildscripts/SDK directo. Cargo exclusivo para tests marker, luego liberación.
- Sol backend: allomancy-horizon/src/save.rs y lib.rs; docs/horizon-storage.md y fichas/evidenciaP2-horizon-save. No core/manifests/guard/SDK. No Cargo hasta relevo root; tests de helpers reales SDK, sin Host falso.
- Terra preparador: tools/prepare-world.ps1 y tools/test-world-preparation.ps1; docs/world-preparation.md y fichas/evidenciaP2-world-preparation. Sin Rust/SDK/Cargo ni ejecutar mundos. Pruebas con directorios temporales ignorados.
- Principal: descriptor compartido, manifests/Cargo.lock, build/smoke/verificadores, CHECKPOINT y commits; planificador/builds/ensayos seriales. Antes de cada build medir espacio, conservar baseline y limpiar exports obsoletos tras verificar sustituto.

## Gates
TDD helper marker→tests significativos, sin mocksSDK. Compilar guard/backend contra SDK exacto; N0 real sin wrapper y N1–N9 conforme informe Astra con hashes previos/reapertura/cortes/rollback/errores. No marcar P2 completa por unit tests ni catálogos. La inyección es parte nativa first-class del bundle fijado, no fork ni modificación permanente upstream; incompatibilidad debe rechazar y preservar datos.