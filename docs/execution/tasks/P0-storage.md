# P0-storage — brief previo
schema_version:1
estado:pending dispatch
modelo:gpt-6.1-terra medium
base:6e5f001da17a3bf58b12ad69badeccbc3f0a815c / codex/alomancia-p0
propiedad exclusiva: experiments/horizon-probe/bundle/api/src/lib.rs, docs/execution/tasks/P0-storage.md, docs/execution/evidence/p0-storage.md
Objetivo: probar helper persistente alternativo existente para evitar ventana de autosave stale de attachments. No alterar inventario ni implementar transacciones P2.
Firma real Host::player_persistent_data(uuid)->Option<Tag> y set_player_persistent_data(uuid,Tag)->bool disponibles en generated_host. Source pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca/game/native_player_state.rs386/398: NeoForgeData inline Guest.saved/acting level.player_extra. Source bajo experiments/horizon-probe/runtime/launcher/native/source/<pin>/crates/horizon-client/src/.
Extensión mínima en player_logged_in manteniendo todos los hooks actuales: leer compound completo; preservar claves de otros mods; namespace propio allomancy_probe con compound schema:Int1 y login_count:Int no negativo. Si falta namespace,iniciar contador0; si tipo/schemafuturo/counterinválido,rechazar sin sobrescribir datos. Si válido,incrementar saturado y setcompletecompound,registrar ALLOMANCY_PROBE event=inline_storage previous_count=N write_ok=BOOL. SinUUID/nombre/rutas en logs. None de helper,error tipo o rechazo deben ser observables y no romperlogin. Adjuntosactuales se conservan para comparación,pero no presentar como almacén canónico seguro.
Checkpoint atómico antes/después con backup. No Cargo/launcher,juego,commits,otrosarchivos ni subagentes. Principal compilará v5 y verificará roundtrip entre dos procesos; no declarar atomicidadporunitmock. Mantener lógica pequeña,usar solo Tag/Compound reales.
Siguiente acción: leer source y firma;registrar intención y añadir helper/callback mínimo.

## Intención
Añadir lectura y actualización versionada del compound completo al login; rechazar tipos y esquemas inválidos sin escritura. Sin compilar ni ejecutar.

## Resultado
Implementación estática lista para revisión y compilación serial; ver evidence/p0-storage.md. Pendientes v5 y dos reaperturas reales. No se ejecutó Cargo ni juego.
