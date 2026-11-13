# P0 — persistencia del jugador, revisión acotada

Fecha: 2026-11-12T23:37:17-06:00. Agente `/root/p0_api`; modelo asignado Sol high. Análisis estático de **fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca**, la revisión instalada. No se ejecutó Cargo, juego ni ensayo de fallo en este encargo; los resultados runtime previos los obtuvo el principal. No se modificaron código, source, checkpoint global ni informe API anterior.

## Decisión provisional

**Puede admitirse el adaptador individual sin fork en C0**, con una elección explícita: estado económico canónico en `Host::player_persistent_data` / `set_player_persistent_data`, compound `NeoForgeData` del propio jugador, y cambios de inventario del mismo jugador en una sola operación síncrona del servidor. Esa vía está implementada y expuesta por el Host generado de la build existente; todavía falta invocarla y ensayar cortes en P2. No hay garantía de commit durable por cada acción ni de atomicidad global de mundo/jugadores.

**No usar Player Attachments como único estado canónico de consumo en esta revisión**: existe una ventana de autosave del dueño individual que puede juntar inventario nuevo y attachments anteriores. Compartir fichero NBT no elimina ese problema de construcción del snapshot. La alternativa `NeoForgeData` evita esa copia separada. Attachments pueden servir como réplica de presentación reconstruible desde el estado canónico; no mantener dos autoridades.

## Convención de rutas

`S = experiments/horizon-probe/runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca/`.

`C = S/crates/horizon-client/src/`; `W = S/crates/horizon-world/src/`.

`B = experiments/horizon-probe/runtime/launcher/native/target/fast/build/horizon-client-d2e354e11d8bfc14/out/` (artefacto ya existente inspeccionado, no generado por este encargo).

Fuente pública: [commit instalado](https://github.com/horizon-rust/horizon/tree/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca).

## Evidencia de snapshot y orden

| Hecho | Ruta y líneas exactas | Implicación |
|---|---|---|
| `player_tag(&Guest) -> Tag` crea un Level temporal, llama `store` y `Level::player_tag`. | C/lan/playerdata.rs:219–227 | Logout/autosave comparten el serializador; no extraen inventario en el writer. |
| `store` clona `Guest.saved`, incorpora `guest.attachments.to_nbt()` y copia inventario/componentes. `Level::player_tag` conserva extras y reescribe `Inventory`. | C/lan/playerdata.rs:435–447, 510–531; C/save.rs:4095–4113, 4207–4209 | Inventario y attachments pueden formar un único Tag, siempre que ambos campos del Guest sean actuales. |
| Snapshot inmutable contiene el Tag completo antes de mandarlo por canal FIFO al writer. | C/lan/playerdata.rs:246–252, 275–300, 309–313; writer:260–266 | El writer no lee el Game vivo y no puede cortar por sí mismo entre dos mutaciones de un hook. Puede estar escribiendo un snapshot anterior. |
| El callback del jugador se ejecuta dentro de `as_guest`; entra, llama closure y sale en orden. Inventario/estado se intercambian; `level.player_extra` se intercambia con `g.saved`. | C/game/lan_actions.rs:350–375; extras:630 y 1021; `native_player_tick`:4239–4250 | Un hook síncrono ordinario, sin reentrada que guarde, no cede un punto de autosave entre dos instrucciones. Eso no convierte una secuencia de helpers falibles en transacción automática. |
| Attachments vivos del jugador están en `Game.player_attachments` / `guest_attachments`, separados de Guest.attachments. | C/game/mod_api_host.rs:52–57, 135–142; C/game/lan_actions.rs:502–505, 990–994 | Para guardarlos hay que copiar expresamente el mapa actual al Guest. |
| Autosave: drena writer, `store_owner`, escribe Level; solo después copia attachments a cada Guest y encola sus snapshots. `store_owner` serializa el Guest sin refrescarlos. | C/game.rs:15413–15421, 15431–15453; C/game/lan.rs:1790–1805 | El primer write del dueño puede contener inventario actual + copia vieja de attachments; un snapshot posterior lo corrige. Un corte entre ambos conserva estado desparejado. |
| IntegratedServer escribe el fichero de dueño; no evita esa primera escritura. | C/game/execution_role.rs:18–22; C/game.rs:22324–22332; C/save.rs:4567–4570 | La ventana anterior afecta al objetivo individual, no solamente a un camino obsoleto. El thread integrado se configura en C/game/integrated.rs:436–437. |
| Logout refresca attachments antes de esperar autosaves y guardar. | C/game/lan.rs:1546–1562; C/lan/playerdata.rs:98–109 | Un cierre normal que funciona no reproduce necesariamente la ventana de autosave. El contador persistido entre procesos no la descarta. |

La ventana anterior es una deducción concreta del orden del código, **no un fallo reproducido**. Secuencia mínima para ensayarla: attachment vivo cambia A0→A1 y el vial desaparece de inventario; autosave del dueño escribe `(vial ausente,A0)`; antes del snapshot posterior `(vial ausente,A1)` termina el proceso. Si el último Guest.attachments ya fue refrescado por otro camino, usar una nueva mutación para hacer visible el desfase.

## Alternativa inline disponible

- `native_player_persistent_data(&self, uuid: u128) -> Option<Tag>` y `native_set_player_persistent_data(&mut self, uuid: u128, data: Tag) -> bool`: **C/game/native_player_state.rs:381–407**. En la acción del propio jugador leen/escriben `self.level.player_extra`; fuera de ella, `host.hub.guests[i].saved`. `native_place` comprueba jugador alcanzable:110–120.
- El setter reemplaza **el compound completo de mods**, clave `NeoForgeData`: **mismo archivo:15–17,68–94**. El adaptador debe leer/mezclar su subclave namespaced y conservar los datos de otros mods. Validar previamente schema/codec y no sobrescribir un esquema futuro o datos inválidos. No sincroniza automáticamente al cliente.
- No hay copia separada para ese compound: `store_owner` toma `g.saved` actual y el inventario del mismo Guest; al estar swapped-in, `leave_guest` devuelve extras e inventario al registro antes del siguiente snapshot. `store` no borra `NeoForgeData` al incorporar attachments.
- Registro del helper: **S/tools/native-hook-fragments/player_persistent_data.json:1–10**. El generador incorpora los helpers de fragments además de native-hooks.json: **S/crates/horizon-mod-api-gen/src/lib.rs:1010–1033**. Por tanto, su ausencia del catálogo raíz no significa indisponibilidad. Esta evidencia amplía el alcance del análisis anterior basado en snapshot parcial.
- Host generado de la build: **B/mod_api_generated_host.rs:1680,1685** declara ambos métodos; **B/mod_api_host_generated.rs:309–310** llama a la implementación concreta. `player_compound` es lectura de todo el jugador, **C/game/native_player_state.rs:123–135**, y por sí solo no es una vía de escritura.
- Inventario exacto: `native_set_inventory_slot(uuid, kind, slot, expected, new) -> Result<Option<ItemStack>,String>` y `native_take_stacks(uuid, slots) -> Result<Vec<ItemStack>,String>`: **C/game/native_inventories.rs:329–379**; wrappers **B/mod_api_host_generated.rs:354–355**. `take_stacks` valida todas las ofertas antes de retirar. Estos helpers rechazan una acción mientras hay un jugador swapped-in (**native_inventory_side:164–178**); en `use_item` emplear el stack/slot del acting player, o encolar un comando servidor y aplicarlo fuera de esa acción.
- Los helpers de inventario para jugadores desconectados escriben un fichero directamente (**native_edit_slots:404–415**). No mezclar esa ruta offline con progreso online. La ruta live cambia el Guest en memoria (:395–402), apropiada para juntar progreso inline e inventario del mismo jugador en el siguiente snapshot.

Integración mínima propuesta: un subcompound propio con `schema_version`, estado, versión monotónica y TransactionIds; precomputar y validar el resultado sin mutar, verificar slot/stack exactos, aplicar ambas mutaciones en un callback server sin `await`, trabajo diferido ni guardado intermedio, y deshacer en memoria si el segundo paso falla. Revisar expresamente las rutas de rollback/reentrada: `native_edit_slots` drena callbacks de saves previos **antes** de editar (:392–394). No emitir recompensa efectiva antes de que la operación completa sea aceptada. Repeticiones con mismo ID no vuelven a gastar ni otorgar. Todo ello es responsabilidad del adaptador, no una propiedad automática del Host.

## Fichero, backup y durabilidad

`save::write_dat` llama `write_dat_as(..., false)` (**C/save.rs:3130–3133**). Se serializa/comprime el Tag entero antes de tocar destino; escribe `<uuid>.dat_new`, renombra el fichero actual a `.dat_old` y luego el nuevo a `.dat` (**3175–3209**). Es sustitución con backup y una ventana en la que no hay fichero principal; no es una operación de dos renombres indivisible. La carga prueba `.dat`, después `.dat_old` (**C/lan/playerdata.rs:56–73** y dueño **C/save.rs:3946–3959**). Hay un test upstream que simula el hueco quitando el principal: **C/lan/playerdata.rs:798–825**; solo se leyó, no se ejecutó.

El autosave normal y logout usan esa ruta **sin `sync_all`** (**C/lan/playerdata.rs:231–243,260–266**). `completed(wait=true)` espera que el writer termine; pese a su comentario de “durability barrier”, no cambia `write_dat` por `write_dat_durable`. “SaveToFile” puede llegar después de WorldSaved y su estado live puede ser más nuevo que el snapshot (**C/game/native_hooks.rs:8465–8473**); no usar ese callback para escribir un sidecar del estado actual ni como recibo de una transacción específica.

La variante durable sí sincroniza el archivo antes de renombrar (**C/save.rs:3135–3139,3156–3160,3193–3196**) y carpeta en Unix; en Windows `sync_folder` es no-op (**3163–3172**). Existe `playerdata::save_now` para handover, pero no se encontró un helper Host transaccional de “persistir ahora el jugador” en la interfaz inspeccionada. No proponer escribir el `.dat` manualmente: competiría con el FIFO existente y sus backups.

Por tanto: ante terminación del proceso, una ruta con snapshot inline coherente puede recuperar una versión vieja o nueva; no se acredita durabilidad frente a pérdida de energía/errores de almacenamiento. Un ACK de comando aceptado tampoco significa fsync. La transacción entre jugador y chunk, entidad, cofre, fichero de otro jugador o estado externo sigue fuera de un único snapshot.

## Ensayos necesarios en P2

1. Invocar ambos helpers inline en la build fijada: subcompound propio + inventario exacto, guardar y reabrir; preservar datos de otro namespace y rechazar esquema futuro sin sobrescribir. Esto aún no se ejecutó.
2. Capturar un autosave del dueño antes de que termine el writer: comprobar que `NeoForgeData` e inventario corresponden a la misma versión. Contrastar con la ventana de attachments señalada, sin usar cierre normal como sustituto.
3. Cortes controlados: antes de mutar, entre las dos mutaciones sin save, después de ambas antes del snapshot, durante `.dat_new`, después de `.dat→.dat_old` y después del reemplazo antes del callback. Recuperación válida: estado anterior completo o nuevo completo; nunca vial ausente con recompensa vieja ni el inverso.
4. Fallo de codec/setter/stack cambiado/inventario lleno: observar rollback y no emitir premio. Repetir TransactionId tras recarga, desconexión y cambio de dimensión; muerte conserva conocimiento y vacía reservas según diseño (el compound inline no tiene `copy_on_death` automático de AttachmentType).
5. Error de escritura, backup corrupto, writer pendiente y logout: verificar que no se declara commit durable por callback tardío, y que un snapshot viejo no pisa otro más nuevo. FIFO y barrera ayudan, pero probar la secuencia del adaptador.
6. Para loot/hitos, cortar entre guardar jugador y guardar chunk/muerte del jefe; esos ficheros son independientes. Idempotencia debe usar identidad estable del evento, no un ID nuevo generado al reintentar.

No hay carencia esencial demostrada que obligue a fork para **coherencia del snapshot del mismo jugador** usando NeoForgeData. Si se exige confirmar cada acción durablemente antes de responder, la extensión mínima pendiente sería un helper de commit del jugador que refresque estado canónico, drene el FIFO y escriba el snapshot con retorno de error y semántica durable; no basta exponer `save_now` sin coordinar writer/owner. Esa extensión no es necesaria para iniciar P1 bajo aceptación explícita del modelo de autosave y los gates P2 anteriores.

## Verificación del encargo

Lecturas selectivas y búsquedas de firmas/source/generated: exitosas. Algunas búsquedas iniciales a nombres de archivo supuestos no existentes devolvieron exit 1; se localizaron las rutas reales con `rg --files` antes de concluir. No ensayos de corte, tests upstream, Cargo ni juego ejecutados por este agente. La aceptación runtime previa de codec/ticks/net/input/item/HUD/sonido/impulso fue reportada por el principal y no acredita aquí la vía inline ni atomicidad frente a fallo.