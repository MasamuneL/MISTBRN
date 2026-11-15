# Identidad estable de mundo y guard de apertura

Contrato P2, 2026-11-13. La identidad se provisiona offline una sola vez. El núcleo y el guard solo leen; no fabrican un UUID por semilla, ruta, nombre, reloj ni error de carga. Provisión: [world-preparation.md](world-preparation.md). Política nativa completa: [p2-storage-architecture.md](execution/evidence/p2-storage-architecture.md).

## Marcador y API compartida

Ruta relativa a la raíz del mundo: `allomancy/world-identity.json`, copia `allomancy/world-identity.json.bak`. UTF-8 JSON, máximo 4096 bytes:

```json
{"schema_version":1,"world_id":"00112233445566778899aabbccddeeff"}
```

El objeto v1 tiene exactamente esos dos campos, una vez cada uno; `schema_version` es entero 1 y `world_id` tiene 32 dígitos hexadecimales ASCII minúsculos. Se permiten espacios y otro orden. Arrays, null, tipos distintos, nombres extra/duplicados, JSON truncado, UTF-8 inválido y basura final se rechazan. La versión futura se informa antes de interpretar los campos v1; nunca se transforma en v1 por defaults. La unicidad de nombres del objeto se comprueba antes del sondeo de versión. No es un DTO posicional ni un `Value` que borre duplicados del objeto marcador.

Exportado por `allomancy_core`:

```rust
pub const MAX_WORLD_IDENTITY_BYTES: usize = 4096;
pub enum WorldIdentitySource { Primary, Backup }
pub struct LoadedWorldIdentity {
    pub world_id: WorldId,
    pub source: WorldIdentitySource,
}
pub enum WorldIdentityError {
    Parse, Schema, Future(u64), InvalidId, Conflict, Io, TooLarge,
}
pub fn decode_world_identity(bytes: &[u8]) -> Result<WorldId, WorldIdentityError>;
pub fn load_world_identity(world_root: &Path)
    -> Result<Option<LoadedWorldIdentity>, WorldIdentityError>;
```

`load_world_identity` recibe **la raíz del mundo**. Principal inválido/futuro/ilegible no cae a backup. Si ambos existen, ambos deben ser válidos e iguales semánticamente; un backup inválido también rechaza. Principal ausente con backup válido devuelve `Some(... Backup)` explícito sin crear el principal; ambos ausentes devuelven `None`. Los errores IO distintos de `NotFound` se propagan como `Io`; las lecturas de archivo se acotan a 4097 bytes para detectar exceso. El API no escribe ni repara.

Mover o copiar un mundo cerrado junto con el marcador conserva `WorldId`; cambiar dimensión también. Un marcador perdido con progreso existente se recupera de la identidad original: asignar otro ID no es una reparación. El UUID Host, normalizado a 32 hex minúsculas por el adaptador, decide `PlayerId`; no lo decide el nombre del jugador ni texto cliente.

## Guard del bundle

`integration/horizon/preflight.rs` contiene una función privada segura y síncrona, propuesta HEAD cancellable en `WorldRequest::prepare_world`, con `request: &WorldRequest` enlazado a `&self` y `options: &Options` al argumento del SDK. Llama `allomancy_world_guard::validate_world(&request.save_root, mode)` y únicamente continúa tras `Ok(())`; el error devuelve `ControlFlow::Break(Err(reason.into()))` antes del cuerpo original. No usa panic, Host ni un `PreparedWorld` vacío; el registro privado usa únicamente locks breves en RAM sin FS/Host sostenidos. La ampliación owner y sus nuevos bindings/selectores están implementados en el bundle, pendientes de planner/build/ensayos del pin; el descriptor lo integra principal.

El módulo soporte valida los originales `level.dat`, `level.dat_old`, todos los `players/data/*.dat` y `*.dat_old` (extensión reconocida sin distinguir mayúsculas en Windows), el `Data.Player` legacy de ambos level y la referencia `Data.singleplayer_uuid` del principal. Exige raíz/`Data` Compound y comprueba UUID de archivo frente al UUID de snapshot cuando está presente. Para el payload usa `decode_save` del mismo core, con WorldId del marcador y PlayerId esperado. `UUID` acepta las formas IntArray de cuatro enteros y String lossless reconocidas por el SDK; los nombres de archivos de jugador exigen UUID canónico con guiones minúsculos, que es el writer del pin.

El namespace es exactamente `NeoForgeData["allomancy:state"] = Compound { format_version: Int(1), payload: ByteArray(bytes_codec) }`. El transporte futuro se rechaza antes de layout v1. No vuelve a serializar el estado como Tags ni usa conversión UTF-16 lossy para JSON. Una raíz `NeoForgeData` presente no-Compound también rechaza. Sin marcador y sin namespace se permite abrir vanilla; el backend debe conservar deshabilitada la economía Allomancy. Un namespace existente sin marcador rechaza la apertura, incluso si solo aparece en un backup auditado. No hay inicialización por error de decode.

Política inicial conservadora: todos los `.dat` y `.dat_old` presentes deben ser válidos; un backup inválido bloquea aunque haya principal válido. Backup sin principal bloquea y solicita restauración offline de un snapshot completo. El guard no elige automáticamente `.dat_old`, no copia solo el namespace y no cambia inventario, economía ni bytes originales. Si `level.dat` falta pero existen snapshots de jugadores también bloquea. La referencia owner del principal exige el archivo principal nombrado o `Data.Player` legacy; no adopta otro jugador silenciosamente. Este perfil **no acredita recuperación automática N7/N8**. La recuperación de marcador `Backup` es distinta: identifica explícitamente el mismo mundo sin reparar el archivo.

## Owner integrado: tres HEAD y primer snapshot

Adaptación HEAD-only P2 implementada, pendiente de planner2/build/ensayo. La vía interior anterior falló realmente E_UNSUPPORTED_CONTROL_FLOW y se conserva en target/p2-owner-plan1. No se reintentan selectores interiores equivalentes. Principal integra tres HEAD cancellable: integrated::serve, WorldRequest::prepare_world y horizon_client_lib::save::Level::write.

profile_capture.rs recibe &Launch al entrar serve. Calcula la misma raíz current_dir/saves/launch.folder que WorldRequest::new y registra el UUID launch.owner ya decidido, usando uuid_ints SDK. No recalcula auth/perfil ni cambia Options. Si ambos markers están ausentes por NotFound, no crea contexto ni impone conflictos a vanilla. Presencia o IO distinto de NotFound cuenta como protegido; preflight aplica después el lector canónico.

El único módulo soporte vive público en save.rs, compilado por horizon-client-lib y reexportado por gameplay. Usa crate::save para Level/helpers y el único core staged como dependencia de horizon-client-lib; no hay ciclo hacia Game. Mantiene un registro privado Mutex/BTreeMap por raíz marcada canonicalizada: owner inmutable, admisión e initialized monotónicos en RAM. No es ledger, snapshot, recibo ni identidad persistente. Canonicalización y metadata de directorio/marker ocurren fuera del lock en los tres puntos; nunca hay FS/Host con Mutex sostenido. Capturar de nuevo el mismo owner conserva todos los campos; otro owner protegido rechaza, no sobrescribe. No se borra el registro.

preflight.rs audita primero originales strictNBT/core/quarantine. Vanilla sin marker ni namespace/marca continúa sin pedir contexto. Para marcado distingue Integrated, Dedicated y rol no soportado: Integrated exige captura y audita owner esperado; Dedicated registra admisión explícita sin owner, también necesaria para el Level::write inicial que el SDK usa al preparar un servidor dedicado. Rol inesperado o contexto protegido ausente/poison rechaza. En integrado existente, principal/backup deben reconstruir Level UUID ya Some(B) y selección load_owner B; metadata sin UUID/legacy con fallback propio B se rechaza sin migrar.

owner_bootstrap.rs recibe &mut Level y &Path en HEAD Level::write, antes de write_internal/ensure_player_uuid. Ambos markers NotFound permiten vanilla sin resetear contexto viejo. En protegido exige admisión; Dedicated continúa sin normalizar. Fresh no inicializado reaudita por última vez la ausencia real de ambos level y todos los snapshots de jugador, y permite sólo UUID local None/B; entonces fija Some(B) en memoria y marca initialized bajo lock sin FS. Existing y writes posteriores sólo afirman UUID Some(B), además de metadata ligera de marker/directorio, sin DAT scan ni parse de marker mientras el writer puede renombrar archivos.

initialized significa asignación/verificación en RAM, no fsync ni confirmación del writer. Si la escritura real falla después, no se resetea y un nuevo Level None puede requerir reinicio; no se reidentifica para ocultar el error. La creación UI puede escribir world_gen_settings antes de HEAD Level::write: esa configuración no contiene identidad de jugador, pero un fallo tardío de bootstrap puede dejar esos archivos. No se promete rollback de archivos ni ausencia de toda escritura en ese caso.

Con B desde el primer snapshot y admisión previa en existente, save_owner, store_owner→Level::write y writer guest deben conservar B. La llamada directa save_owner→Level::write_player no es interceptada por el HEAD de write y depende de esa invariante de preadopción; no reetiqueta payloads. Reopen/arbitrary profile, autosave, logout, cierre temprano, Dedicated/Vanilla, legacy/backups y restantes gates siguen pendientes de SDK real. Los snapshots del defecto previo permanecen intactos. Cambios externos del marker/ruta o mixins ajenos quedan bajo el mismo límite de TOCTOU de N0. Contratos, hashes y evidencia: P2-native-owner.md y P2-native-owner-heads.md.

## Auditoría NBT y límites exactos

El decoder SDK convierte Compound a BTreeMap y pierde claves duplicadas. Por eso el guard hace primero un pase estructural sobre NBT descomprimido: recorre tipos/tamaños, guarda nombres por compound y rechaza duplicados **antes** del mapa. Decodifica nombres mediante el SDK público `nbt::read_network` sobre un String sintético; compara JavaString UTF-16, incluyendo dos codificaciones modified-UTF8 aceptadas que den el mismo nombre. No copia el decoder modified-UTF8. Después ejecuta el SDK `nbt::read_named` real, exige consumo total y valida Tags/codec.

Usa `Compression::Gzip.decode` pública; si hay cabecera Gzip y falla, rechaza sin reinterpretar el contenido. También acepta NBT crudo como el lector upstream. No usa `read_dat` upstream porque oculta IO, consumo final y duplicados. Los nombres del root del archivo se saltan como upstream; no forman claves de un Compound. Strings de payload ajenas se validan después por el SDK. Los duplicados se rechazan también en compounds ajenos y elementos de listas; no se modifica ningún Tag ajeno.

Límites del perfil: 64 MiB por archivo comprimido/crudo y por resultado descomprimido, 1 000 000 nodos, profundidad máxima 128 y 4096 entradas de archivos `.dat`/`.dat_old` de jugador **sumadas**. Son más conservadores que `Limits::saved()` upstream y pueden rechazar mundos grandes válidos; ampliar exige revisión. El pase solo salta bytes/estructuras y no construye un segundo árbol de guardado ni otro `SavedPlayer`.

Archivos `.dat_new`, datos de chunks, entidades, cofres y otros saved-data no se auditan en este gate: el lector de jugadores/Level observado no los adopta como snapshots de jugador. No hay escaneo recursivo arbitrario. Snapshots deben ser archivos regulares; la ruta `players/data`, si existe, debe ser directorio regular. IO de metadata/open/read/read_dir/entries falla cerrado; desaparición de un archivo enumerado también falla. El guard no excluye cambios externos posteriores: el lock del motor y la operación con mundo cerrado siguen siendo necesarios. No promete detectar pérdida de un jugador sin archivo, backup ni referencia histórica observable.

Los diagnósticos propios contienen categorías, fuente de marcador y política de snapshot; no imprimen UUID, payload, rutas personales ni errores IO con rutas. El guard no adquiere SessionLock del mundo ni escribe disco; el registro owner usa sólo el Mutex RAM descrito arriba. Está pensado antes de migración/adopción del preparado; la lectura de resumen de la ventana anterior observada en upstream no queda cubierta como primera lectura absoluta del proceso.

## Evidencia y gates

El marcador pasó TDD conductual y workspace del núcleo. Ver [P2-native-guard.md](execution/evidence/P2-native-guard.md). Esos tests **no prueban ejecución del guard en el juego**. Principal conserva propiedad de descriptor/crate compartido, plan aplicado, build del pin y N0 integrado sin wrapper, con hashes previos/posteriores y error de apertura. N1–N9, rollback y ciclo de vida siguen siendo gates nativos independientes. P2 no se declara completa por estos resultados.
