# P2 — contrato HEAD-only, revisión de firmas y cobertura

Fecha: 2026-11-15 America/Mexico_City. Agente: /root/backend_quarantine.
Estado: propuesta READ ONLY para aprobación de root; no implementación ni planner/build nuevos. Informe ausente al inicio. Única escritura: esta ficha mediante temporal/renombre. Fuentes owner plan1 congeladas y archivadas; no Cargo/juego/SDKedit/commits/subagentes. No duplicar revisión identity_review.

## Límite real y ruling

Planner1 exit1 E_UNSUPPORTED_CONTROL_FLOW en capture_profile/open_integrated/self.options.clone quedó preservado por root. Se abandona la familia interior equivalente. La alternativa HEAD es plausible en fuente, con módulo único compartido y admisión privada en RAM; no se declara resuelta por planner ni compilada.

La nueva barrera inicial sigue siendo HEAD de WorldRequest::prepare_world antes de migración/adopción. Captura HEAD de serve transmite el UUID exacto ya decidido en Launch.owner, sin auth/nombre recalculados ni Options mutables. HEAD de Level::write puede fijar el UUID antes del primer ensure_player_uuid/player snapshot. Cambia el límite bootstrap: puede haber escrito configuración de worldgen, pero todavía no identidad de jugador. No afirmar ausencia de toda escritura ante cualquier fallo Fresh posterior a N0.

## Tres firmas HEAD observadas

SDK: runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.

1. integrated.rs:1213:

```rust
fn serve(
    res: &Arc<Resources>, pack: PackResources, options: Options,
    settings: Settings, launch: Launch, flags: Arc<Flags>,
    tasks: Receiver<Task>, ready: &Sender<Ready>,
) -> Result<()>
```

Target candidato `horizon_client::game::integrated::serve`, HEAD cancellable. Argumentos directos disponibles, sin capturar locals: `launch: &Launch` desde `&launch`, y opcionalmente `options: &Options` desde `&options` para comprobar IntegratedServer. Helper puede calcular raíz exactamente como WorldRequest::new: `current_dir()?.join("saves").join(&launch.folder)`; registra Launch.owner ya decidido y devuelve `ControlFlow<Result<()>>`. Error antes de WorldLoader::start (serve:1227–1246), no panic ni unwrap. No se necesita copiar Launch.player a Options en esta variante.

2. world_loading.rs:85:

```rust
fn prepare_world(self, res: &Resources, pack: &PackResources,
    options: &Options, settings: &Settings,
    upgrade: &horizon_world::datafix::progress::UpgradeProgress,
) -> Result<PreparedWorld>
```

HEAD cancellable; bindings request=&self y options=options, retorno `ControlFlow<Result<PreparedWorld>>`. Resuelve rol desde Options, obtiene contexto capturado para Integrated y audita. Dedicated es modo explícito sin owner; UnknownRole no es bypass. Publica admisión sólo después de auditoría completa y después de revalidar el contexto bajo lock.

3. save.rs:4323:

```rust
pub fn write(&mut self, dir: &Path) -> Result<()> {
    self.write_internal(dir, true)
}
```

Target real candidato **`horizon_client_lib::save::Level::write`**, HEAD cancellable; bindings level=&mut *self y world=dir, retorno `ControlFlow<Result<()>>`. No confundir el reexport `horizon_client::save` con el crate que define el AST. Hay que comprobarlo con planner antes de afirmar el target. Helper se inserta en módulo save y llama al módulo soporte local.

## Ubicación única, visibilidad y dependencia

`crates/horizon-client-lib/Cargo.toml:12–13` define `[lib] path="../horizon-client/src/client_lib.rs"`; client_lib.rs:123 declara `pub mod save`. Gameplay horizon-client/src/lib.rs reexporta `horizon_client_lib::*`. Por tanto save.rs pertenece al crate library, que no depende del gameplay: no puede llamar a game::world_loading::allomancy_world_guard.

Propuesta cotejada: mover la declaración del **único** módulo soporte a `file="crates/horizon-client/src/save.rs"`, `name="allomancy_world_guard"`, `visibility="pub"`. No instalar otra copia del mismo static en gameplay. Helpers gameplay llaman `horizon_client::save::allomancy_world_guard`; HEAD de Level::write llama al módulo local `allomancy_world_guard`. Dentro del soporte usar `crate::save::{uuid_from_text,uuid_text,Level}` y dependencias library existentes; nunca importar Game/Options/Launch ni añadir dependencia inversa a horizon-client. El estado y el mapa siguen privados; sólo funciones de registro/auditoría/writegate quedan accesibles entre crates.

Mixins pin 5a64af938ec86b00d53d16a408aabc0038fb3e57 local cotejado: `horizon-mixins-core/src/manifest.rs:72–94` admite Module.visibility `pub(crate)` o `pub`; transform/src/lib.rs:329–339 emite ese `pub mod`. `manifest.rs:32–40` admite [[crate]].dependent; native.rs:300–307 exige workspace member real, :371–374 guarda dependent_manifest y :737+ añade la dependencia. `horizon-client-lib` existe como package workspace. Root debe poner el único core `dependent="horizon-client-lib"` y mantener la dependencia API al mismo path staged native-crates/allomancy/core; no duplicar package core ni hacer library→gameplay. Cotejo de fuente, no validación aplicada de manifest/planner.

## Contexto efímero y admisión: contrato para implementar tras aprobación

Un solo registro privado Mutex<BTreeMap<PathBuf,Context>> en library. Conserva contexto de arranque y estado de admisión; no saldos, inventario, ledger, recibos, copia de SavedPlayer ni identidad económica paralela. No archivos/env/thread_local. Registration y consultas cortas: liberar Mutex antes de FS o cualquier Host y comprobar que contexto no cambió al volver a adquirirlo.

- Captura correlacionada por la misma raíz que usa WorldRequest; para la ruta normal SDK el worker y Level::write reutilizan ese PathBuf. Si se elige canonicalizar, hacerlo antes del lock y definir la misma clave en los tres puntos, sin FS en autosaves posteriores. No adivinar por nombre de directorio. Alias de ruta entre registros concurrentes deben rechazarse o resolverse antes de publicar la reserva; no permitir dos propietarios del mismo mundo por diferencias de spelling.
- Owner esperado de un mundo marcado es inmutable en proceso. Registrar de nuevo el mismo perfil no resetea admisión ni initialized; perfil distinto rechaza, no sobrescribe. Reopen permitido sólo mediante auditoría fresca coherente con el contexto ya existente; error deja estado cerrado, no borra evidencia ni revierte initialized.
- Admisión explícita `Vanilla` (sin marker y sin namespace/marca), `Dedicated` (N0 vigente, sin owner), `Fresh(B)` o `Existing(B)`. UnknownRole y ausencia de contexto/admisión en un write marcado rechazan; no usar ausencia de contexto como Dedicated/Vanilla. Vanilla admitido sigue original sin asignar/comparar UUID; Dedicated también permite el Level::write inicial con su comportamiento existente, sin normalizar owner.
- Precisión vanilla: una reserva de perfil no debe imponer política A≠B a mundos sin marker antes de clasificarlos. Mantener Vanilla explícito sin comparación de owner; si un mundo pasa a marcado dentro del proceso, no reinterpretar una reserva antigua ni resetear a Fresh, sino exigir nuevo contexto válido/proceso conforme ruling. Root debe mantener esta distinción al implementar los conflictos del registro.
- HEAD prepare audita todos originales y cuarentena. Existing exige owner seleccionado B y Level UUID resultante Some(B), incluyendo backup/legacy/fallback; metadata None+archivo propio B rechaza. Fresh sólo si marcador válido y level principal/backup y todos player snapshots realmente ausentes. La auditoría no se evita por tener registro previo.
- Primer HEAD write de Fresh realiza la última comprobación de ausencia/contexto fuera del lock, valida Level UUID None/B, asigna únicamente Some(B) y avanza monotonía a initialized. No incluir writer en el lock. Si el writer falla después, no marcar durable ni resetear para volver a adoptar otro Level None; un reintento sólo puede afirmar B, o rechazar.
- Existing y todo write posterior de Fresh inicializado sólo consultan registro y afirman Level.player_uuid==Some(B), sin DAT scan/marker read ni inspección FS mientras autosave/writer puede estar entre renombres. Un cambio inesperado rechaza antes de write_internal; no se corrige en silencio. Vanilla/Dedicated conocidos son ramas explícitas sin normalización.

## Cobertura de escrituras y frontera honesta

world_loading.rs:173 llama write_world_gen_settings_with en UI createSome. save.rs:3542–3670 crea directorios y escribe exclusivamente data/minecraft/world_gen_settings.dat (seed/generador/dimensiones); no UUID/player/namespace ni Level.ensure_player_uuid. Después world_loading.rs:264–266 hace level.write(&save_root) si falta level.dat **antes** de devolver PreparedWorld. Así HEAD Level::write cubre bootstrap CLI/UI antes del primer snapshot de jugador y antes de publicar/adoptar el mundo.

Level::write_internal:4344 crea dir/restaura metadata, :4391 llama ensure_player_uuid, :4468 incorpora singleplayer_uuid, :4568 escribe player y :4570 level.dat. HEAD está antes de todo eso. Dedicated también pasa por level.write en creación, aunque sus autosaves usan write_without_player; por eso necesita Dedicated admission explícita.

Autosave/pausa/save explícito game.rs:15416–15421 store_owner→write_level_for_role→Level::write pasan por el assert sin FS. Logout/cierre **también tiene una ruta directa que no pasa por Level::write**: playerdata::save_owner:101–110 llama Level::write_player y save(guest). Su seguridad aquí depende de la invariante anterior: Level ya Some(B) y adopción guest B admitida. No presentar el tercer HEAD como interceptación de todos los writers; si se exige comprobación en cada writer directo haría falta otro punto/contrato. No se encontró ruta del preparado que llame write_player/ensure_player_uuid antes del primer Level::write; las rutas gameplay posteriores, otros mixins y herramientas fuera del flujo admitido siguen fuera de esta observación.

## Pendiente antes de implementación/éxito

Revisión identity_review y aprobación root del contrato/admisión/transiciones. Root planner único para tres HEAD, módulo público sobre archivo perteneciente a library y dependent core correcto, después revisar staged, compilar y ensayar perfil arbitrario fresh→save/autosave→reopen; Dedicated/Vanilla; mismatch existente y LevelNone+fallbackB con hashes; cierre antes placement/reentry; réplica exacta de primer fallo. No se ejecutó ninguna de estas verificaciones ni se cambia fuente en esta tarea.
## Precisión final de root — presencia de marker y vanilla

Esta precisión sustituye las frases anteriores que exigían contexto también al write vanilla o cero FS en autosaves. Root la define como contrato mínimo a cotejar con identity_review antes de implementar:

1. HEAD serve sólo reserva/impone owner inmutable cuando el marker principal o backup está presente según metadata. Cualquier resultado metadata distinto de ambos NotFound cuenta como protegido, incluidos errores IO: el preflight posterior los rechazará mediante el lector canónico. Si ambos son NotFound, no se crea/modifica contexto ni se compara propietario con una reserva antigua. No se parsea marker aquí.
2. HEAD prepare siempre ejecuta primero auditoría canónica strictNBT/core/quarantine y marker existente. Si es Vanilla (sin marker y sin namespace/marca), continúa sin pedir contexto ni crear política de owner. Namespace/marca sin marker sigue rechazando. Si el mundo es marcado exige contexto capturado para Integrated o admisión Dedicated explícita; no ausenciaContext=bypass.
3. HEAD Level::write comprueba sólo presencia de ambos markers por metadata fuera del lock. Ambos NotFound permiten conducta vanilla sin assert de owner y sin resetear contexto previo. Si es protegido exige admisión validada explícita Integrated/Dedicated. Integrated Existing y Fresh ya inicializado sólo afirman UUID B en memoria, sin reauditar DAT ni decodificar marker en autosave. La consulta metadata ligera por write está permitida; no sostener lock durante ella.
4. La rama Fresh primera escritura conserva comprobación única de ausencia de snapshots antes de asignar B y de avanzar initialized. Toda la auditoría de snapshots queda fuera del Mutex. Reservas protegidas no se sobreescriben ni se resetean por reabrir mismo perfil; conflicto de propietario protegido rechaza. Las ramas Vanilla no borran esas reservas.
5. Una transición externa del marker mientras el mundo está abierto mantiene el límite de TOCTOU ya declarado para N0; esta adaptación no promete reparación ni barrera contra manipulación externa. No inventar otra persistencia de contexto para suplirlo. Falta contexto con marker presente sigue cerrado.

Así se evita imponer A≠B a vanilla y se mantiene explícito Dedicated incluso en su Level::write inicial. Presence es una sola comprobación compartida en soporte, no otra implementación del parser de identidad. La revisión independiente de identity_review y el planner/build root siguen pendientes; esta tarea sólo registra fuente y contrato, sin código final.

## Implementación HEAD-only entregada — 2026-11-15

Root aprobó CHECKPOINT final y precisiones de revisión antes de esta edición. Se implementaron las cuatro fuentes propias, sin descriptor/build/manifests/SDK/core/backend/driver. Se sustituyen los interiores fallidos por tres helpers HEAD: capture(&Launch)->ControlFlow<Result<()>>, preflight(&WorldRequest,&Options)->ControlFlow<Result<PreparedWorld>>, owner_bootstrap(&mut Level,&Path)->ControlFlow<Result<()>>. Módulo soporte importa crate::save y se instalará único/público en save.rs de horizon-client-lib; pública sólo la entrada OwnerMode y funciones capture_owner/validate_world/before_level_write, registro privado.

La captura registra launch.owner exacto por canonicalroot sólo protegido; mismo owner no resetea, conflicto rechaza. Preflight audita antes de requerir contexto; Existing integrado realiza una segunda auditoría con expected B para reconstruir Level UUID/selección. Fresh se reaudita únicamente antes de su primer write; después initialized o Existing sólo assert B, sin DAT scans de autosave. Dedicated es admisión explícita y Vanilla no tiene política nueva de owner. Marcador/padre canonical metadata fuera Mutex en cada punto, no parsing duplicado del marker para presencia. Bootstrap setterRAM+initialized monotónicos; no writer/fsync inferido. Error tardío puede dejar worldgen previo y retry de un Level None requiere reinicio, sin reset automático.

Scoped git diff --check terminó exit0 con aviso EOL de preflight. Sólo revisión estática; ningún Cargo/build/juego/test/mock. Fuentes congeladas para planner2 de root; targets nuevos no acreditados. Hashes al congelar:

- integration/horizon/world_guard.rs: 47B054C2E8453B93324CB4104F401CB4800D08C107717622BEAFF5F4BCAEAE21
- integration/horizon/preflight.rs: BF83AED639E898AFF0391E5BF1B2477518F4B9A05CF39B4F46A18FBD2AF58342
- integration/horizon/profile_capture.rs: B61B830FF878635CB263632E24D9BDDB9D38A97013C16ADF4232547B8EFD7B0B
- integration/horizon/owner_bootstrap.rs: 878A4E2C9115A837AFD8B184F3D115425CA4E881DC88C49AAFF2FEA67B5C0DF0
- docs/world-identity.md: 76FC32B31F47E1874CB78B50577A776D91A1C708D9756164A9317DF2234B7FDA


Precisión documental final: dos frases históricas sobre ausencia de locks ahora distinguen SessionLock/IO del Mutex RAM owner. SHA256 final docs/world-identity.md: C8004F542BB221741E8BF05660597758E264493C79D8E3F3C78C22316D27370D. Hashes Rust congelados sin cambio.
