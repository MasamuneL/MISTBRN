# Storage nativo de Allomancy — P2

Estado: implementación WIP, pendiente de compilación contra SDK y ensayos nativos. Contrato: `docs/execution/tasks/P2-storage-review.md` y `P2-native.md`. Este documento no acredita N0–N9.

El único estado económico es `NeoForgeData["allomancy:state"]`: Compound con `format_version: Int(1)` y `payload: ByteArray(encode_save(SavedPlayer))`. SavedPlayer incluye ledger y revisión; no existe ledger secundario. El guard N0 debe validar originales antes de adopción, porque Host no distingue pérdida de archivo de jugador nuevo.

El backend valida snapshot bruto (raíz, UUID, Health finita y NeoForgeData Compound si existe), conserva la ausencia real y clona las claves ajenas sin convertir JavaString a UTF-8. El ciclo usa Ready/Busy/Fault; no sostiene Session Mutex durante llamadas Host. Mutadores de lifecycle rechazan centralmente un `world_fault` ya detectado. `test_world` compara rutas canonicalizadas y WorldId, rechazando errores de IO y diferencias; mantiene las firmas NativeTestConfig/CreditIntent.

El crédito requiere jugador vivo y slot exacto, prepara candidato completo, hace CAS de un slot y aplica el namespace inline. Recomprueba Host antes de commit core y recibo. Replay también inspecciona el snapshot actual antes de devolver el recibo. Un fallo de inspección previo a side effects fija Fault y bloquea, de modo que el cierre Busy→Ready no lo habilita. Rechazos demostrados antes de cambios pueden conservar Ready. La aceptación inline y el recibo no son fsync.

## Cuarentena por rollback no confirmado

La representación que produce esta versión es el sibling:

```text
NeoForgeData["allomancy:storage_fault"] = Compound {
    "format_version": Int(1),
    "reason": String("rollback_unverified")
}
```

Cualquier presencia bloquea con `StorageError::Quarantined`, independientemente de su tipo, versión, campos o valor, aun sin state. No se decodifica para habilitar uso ni se normaliza. El inspector público, login incluso con Entry cargada, replay y mutadores normales/acting comprueban presencia. Las comprobaciones estructurales privadas usadas al confirmar la marca retienen los Tags, sin pasar por el rechazo habilitante.

Si falla la comprobación del rollback, primero se retienen en RAM la preimagen y los stacks esperado/reemplazo y se fija Fault. En el mismo callback se intenta marcar a través del Host real: snapshot bruto del UUID correcto, Compound actual válido, clon y adición exclusiva de la marca si falta. Nunca se restaura una raíz antigua para marcar; se conservan exactamente el state actual y las claves ajenas actuales. Una marca existente, incluso desconocida, se conserva sin setter. Raíz ausente o inválida se rechaza; no se autoriza reparación desde un getter normalizado vacío.

Para una marca nueva se exige `set_player_persistent_data == true`; para toda confirmación se exige otra lectura estructural del jugador correcto y coincidencia exacta del Compound esperado. No se sostiene Mutex en este recorrido. No hay CAS económico adicional ni reintento automático después de marcar; siempre se devuelve error y Fault permanece. Los eventos separan `rollback_unverified`, `quarantine_confirmed accepted_in_memory=true durable=false` y `quarantine_unconfirmed reason=... durable=false`.

Si snapshot/setter/readback no es accesible o no coincide, la cuarentena queda **NO confirmada**: sólo se conserva Fault en RAM y diagnóstico. No se promete bloqueo de la siguiente apertura ni detención del writer. El ensayo debe detenerse controladamente y revisar/restaurar un snapshot completo offline; ese resultado no acredita N7 ni persistencia de marca. No hay Host::flush inventado, sidecar ni escritura manual de .dat live.

## Límites y recuperación

La marca aceptada se vuelve persistente cuando Horizon captura y guarda al jugador; el bool del setter no acredita ese writer ni fsync. La protección ante cortes depende del perfil sin snapshots nuevos reentrantes durante el intervalo parcial: antes de publicar la marca sólo se puede recuperar un snapshot completo previo; después del callback, los snapshots nuevos deben contener la marca. Esa exclusión y el comportamiento del writer siguen pendientes de ensayo real.

`AfterInventory`, `AfterInline` y `RollbackInventory` son fallos inducidos del adaptador. RollbackInventory retorna antes del CAS inverso y deja el par parcial para ensayar la ruta de marca; no demuestra fallo del CAS, setter, IO ni writer del SDK. Desconexiones/guardados/mutaciones de mods ajenos desde SaveToFile pueden romper el perfil y conducir a Side::File; los rechecks no eliminan esa carrera.

N0 debe rechazar cualquier marca en todos los snapshots que audita, incluidos backups; no se selecciona automáticamente un backup limpio. La integración del guard es responsabilidad de su tarea y no se verifica aquí. Recuperación sólo offline desde snapshot completo coherente del jugador/mundo y su identidad, preservando evidencia original. Quitar únicamente la marca no prueba coherencia inventario/ledger. Login y guard no limpian ni migran automáticamente.

Pendientes con SDK/Host real: presencia v1/scalar/vacía/futura/extra con y sin state; login existente, replay, seed/consumo y respawn acting; rollback exacto AfterInventory/AfterInline; rama artificial RollbackInventory y reapertura N0; rechazo real de CAS inverso y preservación de ajenas; marca realmente no confirmada; lifecycle tras identidad inválida; cortes N6–N8 en fronteras observadas. Compilar o pruebas de texto no sustituyen estos oráculos.
