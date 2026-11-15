# P2 — revisión de storage y cuarentena persistente

Fecha: 2026-11-14, America/Mexico_City. Agente: /root/storage_review.
Worktree: C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN.
Rama observada: codex/alomancia-p2-native. HEAD: fed744b9252e14ae6fcd12b314fba61040250fe0.
Alcance: revisión estática del WIP; no implementación, Cargo, build, juego, suites ni commits. Fuentes leídas: HANDOFF.md, tasks/P2-native.md, evidence/p2-storage-architecture.md, save.rs, lib.rs, world_guard.rs y SDK pin local. Este dictamen no acredita N0–N9.

## Dictamen

El protocolo mínimo conserva C0 y P2B: un único SavedPlayer/ledger inline, CAS de un slot, candidato completo, commit core después de aceptación Host y lectura de comprobación. No hace falta cambiar firmas públicas salvo añadir StorageError::Quarantined; la escritura de marca cabe en un helper privado. La cuarentena todavía no está implementada y el backend no está aprobado para ensayo económico hasta cerrar los hallazgos P1 siguientes. No se encontró otro bloqueante confirmado del crédito dentro del perfil de callbacks observacionales; compilación y ciclo de vida siguen pendientes de evidencia real.

## Hallazgos priorizados

1. **P1 — Fault se pierde al reiniciar y permite adoptar un par parcial.** save.rs:850–869 conserva preimagen y llama fault(), pero fault():317 solo escribe Session. El failpoint RollbackInventory deja inventario nuevo y namespace viejo; un autosave posterior puede persistirlos. world_guard.rs:167–178 solo busca allomancy:state y permite ese estado v1 válido. Cerrar con sibling de cuarentena, presencia bloqueante y lectura posterior síncrona antes de devolver del callback. No resolverlo con otro ledger.

2. **P1 — error de inspección vuelve a Ready.** apply_credit():751 usa raw_player(...)? antes de llamar fault; credit_for_native_test():730–736 convierte cualquier Busy restante en Ready. Root/UUID/Health/ModsRoot inválidos o Unavailable salen como error pero dejan una sesión operable; añadir Quarantined al inspector sin corregir esta salida conservaría el defecto. Diferenciar rechazo previo demostrado sin cambios (slot/CAS precondición, capacidad, etc.) de corrupción, cuarentena o contexto incierto: estos últimos quedan Fault/bloqueados. La marca persistente se exige si el par pudo cambiar y no se confirma rollback; un simple rechazo sin mutación no requiere inventar un fallo económico persistente.

3. **P1 — las comprobaciones de cuarentena deben cubrir todas las entradas.** inspect_player():105 es punto común fuera de acting; mutate():441 usa player_persistent_data durante respawn y necesita su propia comprobación por presencia. login():337 retorna temprano cuando ya existe Entry: comprobar antes de ese retorno o mantener una precondición equivalente documentada. Replay debe rechazar la marca antes de devolver recibo, aunque no requiera vial. seed/consumo/muerte/reconnect/respawn no pueden sustituir, borrar ni ignorar el sibling. No basta añadirlo solamente al guard N0.

4. **P2 — cierre por fallo de identidad no cubre mutadores de lifecycle.** world():260–280 fija world_fault y tick():593 deja de trabajar, pero mutate():412 no lo consulta; logging_out/respawned pueden seguir escribiendo desde callbacks. Escenario concreto: marker inválido o cambiado detectado durante sesión, seguido de muerte/respawn. Comprobar world_fault centralmente antes de mutar es suficiente para cerrar ese estado ya detectado; no se requiere nueva API pública. No se promete protección contra sustitución externa concurrente de archivos.

5. **P2 — RollbackInventory no demuestra fallo Host.** rollback():885 retorna Err antes de ejecutar CAS inverso. Sirve para entrar en la rama de cuarentena dejando un par parcial real en memoria; no es error del setter, CAS, IO ni writer. AfterInventory/AfterInline son también fallos del adaptador. Conservar nombres/evidencia honestos.

6. **Pendiente de integración, no descubrimiento nuevo:** descriptor sigue solo [api]; Cargo.toml carece de feature native-test y lib.rs la referencia. No existe prueba de N0 aplicado ni de compilación SDK de este corte. Los gates previos no se repitieron. No declarar esta revisión aprobación de integración.

## Contrato mínimo implementable

Representación producida por esta versión:

```text
NeoForgeData["allomancy:storage_fault"] = Compound {
    "format_version": Int(1),
    "reason": String("rollback_unverified")
}
```

Es un tombstone de bloqueo, no estado recuperable. No incluir saldos, ledger, copia de SavedPlayer, inventario, epoch, timestamp ni una segunda identidad. Diagnóstico detallado/preimagen quedan en RAM y evidencia de ensayo; su pérdida no desbloquea el tombstone. El motivo fijo identifica esta ruta, sin interpretarlo como prueba de causa SDK.

**Lectura:** cualquier presencia de esa clave bloquea, sin exigir Compound, versión, campos ni valor conocido. Byte(0), Compound vacío, versión futura y objeto malformado también bloquean. No confundir ausencia de allomancy:state con permiso cuando existe storage_fault. La comprobación no depende del marker de mundo ni de decode_save. Si NeoForgeData no es Compound, se rechaza igualmente por raíz inválida. No se normaliza ni elimina la marca. Una marca existente se conserva exactamente, incluso desconocida.

**Guard N0:** comprobar presencia después de validar NeoForgeData Compound y antes de la salida por ausencia de state. Aplicar al mismo conjunto que ya audita: players/data principales y .dat_old, level.dat/Data/Player y level.dat_old/Data/Player. Mantener auditoría de duplicados NBT antes de BTreeMap. Una marca en cualquier snapshot auditado bloquea apertura, aunque el principal esté limpio y el backup marcado; es la política conservadora actual. No seleccionar automáticamente un backup para evitarla. Sigue siendo guard nativo HEAD real, no wrapper ni login tardío.

**Escritura ante incertidumbre:** conservar Busy hasta resolver; cuando rollback no se confirma, fijar Fault/bloqueo antes de cualquier intento posterior y retener preimagen. En el mismo callback, sin await/save/Host que drene callbacks ni Mutex sostenido:
1. Leer el Compound de mods actual del jugador correcto. Para esta ruta de crédito usar el snapshot bruto fuera de acting; no tomar un getter normalizado vacío como autorización para reparar una raíz inválida.
2. Si ya hay storage_fault, preservar su Tag y confirmar presencia; no actualizarlo ni limpiar nada.
3. Si está ausente, clonar el Compound actual y añadir solo la marca. Preservar exactamente state (sea A o B) y todas las claves ajenas actuales, no restaurar la raíz capturada antes de callbacks.
4. Llamar set_player_persistent_data con esa raíz. Exigir true y una nueva lectura que coincida exactamente con la raíz esperada. El inspector usado para verificar no puede ocultar los Tags al devolver Quarantined: separar lectura estructural/confirmación de la comprobación habilitante.
5. Devolver error, jamás recibo de éxito ni Ready. Registrar por separado rollback no confirmado y marca confirmada/no confirmada.

No aplicar un CAS inverso adicional después de marcar ni reintentar automáticamente economía. Se puede conservar el rollback existente: exige raíz esperada completa, hace CAS inverso y vuelve a comprobar antes de restaurar. Si cambió una clave ajena, rechazar rollback es conservador; la marca debe mezclar sobre la raíz ACTUAL, nunca pisar esa clave.

Si lectura/setter/readback es imposible, **cuarentena no confirmada**: Fault solo RAM y evidencia explícita. No asumir que quedó persistida, que el writer se detuvo o que la próxima apertura será segura. En ensayo, detener el caso controladamente sin presentar un cierre normal como reparación y revisar/restaurar snapshot completo offline. No inventar un Host::flush, abortar como contrato de producto, escribir .dat live ni añadir sidecar para suplir esa limitación. Ese caso no acredita N7 ni el gate de persistencia de marca.

**Recuperación:** únicamente offline, partiendo de snapshot completo coherente del jugador/mundo y su identidad, preservando evidencias originales. Quitar solo la marca no demuestra coherencia de inventario/ledger. No migración o limpieza automática durante login/guard.

## API411 comprobada y límites

SDK S = C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.
G = runtime/launcher/native/target/fast/build/horizon-native-game-4e31fe35dc1758a4/out.

| Evidencia releída | Hecho comprobado |
| --- | --- |
| G/mod_api_generated_host.rs:524,1637,1685,2001 | Host público tiene player_stacks, player_compound, set_player_persistent_data y set_inventory_slot. No se deduce un inventario público con flag online. |
| S/crates/horizon-client/src/game/native_player_state.rs:70–94,110–135,386–407 | Getter normaliza raíz ausente/inválida a vacío; snapshot bruto no existe acting. Setter sustituye Compound en memoria, admite acting/guest accesible, no despacha callbacks ni escribe disco; Compound vacío retira raíz. |
| S/crates/horizon-client/src/game/native_menus.rs:232–245 | player_stacks entrega los 36 slots para jugador colocado/vivo/no acting; no expone exclusión online que sobreviva otra llamada. |
| S/crates/horizon-client/src/game/native_inventories.rs:164–186,336–351,386–416 | CAS exacto; drena autosaves antes de elegir Live/File y descarta error del drain. Si el jugador desapareció, puede escribir su archivo. No hay CAS de namespace ni online-only en esa firma. |
| S/crates/horizon-client/src/game/native_hooks.rs:8438–8473 | Login fuera de acting; respawn acting; SaveToFile notificación posterior, no cancelable ni recibo por operación. |
| S/crates/horizon-client/src/save.rs:3175–3209 | Ruta de writer normal puede usar fs::write y renombres; el bool de inline no acredita ese writer ni fsync. |

La marca queda aceptada en memoria tras setter+readback; persiste cuando Horizon captura y guarda ese jugador. No es una barrera durable por acción. Bajo el perfil SIN snapshots nuevos reentrantes durante el intervalo parcial, un corte antes de publicar marca solo permite recuperar un snapshot previo completo; después del callback, snapshots nuevos contienen la marca. Esta conclusión depende de la exclusión indicada y debe ensayarse; no es una garantía demostrada por compilar.

Un mod ajeno que desconecta, guarda o muta desde SaveToFile puede invalidar ese perfil y conducir a Side::File. Los rechecks no eliminan esa carrera. No prearmar cada operación en este alcance; tampoco atribuir al tombstone protección frente a setter inaccesible, pérdida de energía, guardados de otros jugadores/chunks o manipulación externa.

## Ensayos siguientes: todos pendientes

| Caso | Oráculo y clasificación honesta |
| --- | --- |
| Presencia N0 | Fixtures offline con marca v1, scalar, vacía, futura, campos extra; state presente/ausente y marker ausente. Rechazo del guard nativo antes de adopción, hashes intactos. Incluir backup, legacy y .DAT. |
| Login y mutadores | Marca presente antes de login; inserción controlada vía Host en fixture ya cargada, antes de replay/seed/consumo y respawn acting. Error y Fault; ningún cambio de slot/state/ajenas. No editar .dat live. |
| Rollback normal | AfterInventory y AfterInline: restaurar A exacto, sin recibo T ni marca, Ready solo con verificación completa. |
| Rama artificial | RollbackInventory deja par parcial, escribe marca con Host real y readback, Fault permanente. Guardado normal/cierre/reapertura N0 rechaza. Etiqueta: fallo inducido del adaptador, setter de marca real; NO fallo Host real. |
| CAS inverso real | Provocar una precondición real distinta en mundo dedicado y observar el Err real del helper; distinguir la intervención de ensayo del comportamiento espontáneo. Verificar preservación de ajenas y cuarentena. Si no se alcanza, pendiente. |
| Marca no confirmada | Host no alcanzable o setter/readback realmente rechazado: Fault y diagnóstico no confirmada, nunca éxito de cuarentena. Un failpoint que omite llamada solo cubre rama artificial. |
| Cortes y writer | N6–N8 mantienen sus oráculos A/B completos o snapshot marcado bloqueado. Capturar frontera real y fuente recuperada; fixture offline/corte aleatorio no sustituye breakpoint o estado de renombre observado. |
| Lifecycle/identidad | Muerte→logout, muerte→respawn→corte, reconnect y fallo de marker ya detectado. Trazar acceso efectivo de Host; sin repetir suites core como sustituto. |

Verificación de este informe: fuentes y ubicaciones inspeccionadas; rama/HEAD consultados. git status en el worktree devolvió error de worktree, por lo que no se certifica limpieza Git. Solo se escribe este documento mediante temporal del mismo directorio (backup si ya existiera). No se modificaron fuentes, manifests, SDK ni se ejecutaron tests.
