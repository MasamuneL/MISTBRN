# P0 capabilities — revisión incremental independiente

## Estado vigente — fixround 1/5

**Único P2: ADDRESSED.** Re-revisión limitada a soundcategory. `p0-sound-fix-review.diff` contiene exactamente el cambio `"players"` → `"player"` en la llamada a `play_sound_to`; el archivo histórico `experiments/horizon-probe/runtime/review-v3-source.rs` conserva el SHA256 `F134E7BD2A7FAB977037D1C2407A681C8334B3BF6D2884FCA4CCD0A774C4C3EA` de la revisión inicial. No revisé la extensión P0-storage en curso ni el source vivo como si fuera el snapshot v4.

**Veredicto de especificación actualizado:** la corrección satisface el requisito de intentar encolar sonido vanilla y registrar su retorno; el único incumplimiento detectado en la revisión inicial está resuelto. **Veredicto de calidad actualizado:** cambio mínimo correcto, sin hallazgos de esa revisión pendientes. Estos veredictos se limitan al probe de capacidades y al fix revisado; no acreditan el mod final ni extensiones posteriores.

Evidencia local leída, sin repetir builds o pruebas:

- `p0-smoke-capabilities-baseline-stderr.log:25` registra `sound_queued=false`; líneas 27 y 30 registran ping/ack válidos. Su result JSON contiene `timed_out=false, exit_code=0`. Esta es la evidencia RED útil; el archivo llamado `p0-smoke-capabilities-red-result.json` contiene timeout y no se utiliza para acreditar el comportamiento de sonido.
- `p0-capabilities-fixed-build.log` termina con build e instalación completados y exportación `runtime/probe-engine-v4/horizon-engine.exe`; el ejecutable existe. El principal reportó exit0 del build.
- `p0-smoke-capabilities-green-stderr.log:24` registra `ping_sent=true sound_queued=true inventory_slots_occupied=Some((36, 0))`; líneas 26 y 29 registran `ping valid=true ack_sent=true` y `ack valid=true`. `p0-smoke-capabilities-green-result.json` contiene `timed_out=false, exit_code=0`.

El paso false→true con el cambio de literal y la evidencia v4 cierra el defecto de categoría. `sound_queued=true` acredita aceptación/encolado por el helper, no audición. El resto de este informe conserva la revisión inicial y sus límites históricos; esta re-revisión no pretende cerrar cada ensayo pendiente del probe.

## Revisión inicial — histórica

Alcance: `P0-capabilities.md`, informe de implementación, diff `p0-capabilities-review.diff` y `experiments/horizon-probe/bundle/api/src/lib.rs` (108 líneas). Base v2: `6e5f001da17a3bf58b12ad69badeccbc3f0a815c`. Fuente v3 revisada: SHA256 `F134E7BD2A7FAB977037D1C2407A681C8334B3BF6D2884FCA4CCD0A774C4C3EA`. Solo revisión estática: no ejecuté Cargo, launcher, juego ni pruebas y no modifiqué implementación.

**Veredicto de especificación: requiere una corrección menor antes de considerar preparado el ensayo completo.** HUD, protocolo fijo, observadores, lectura de inventario y gates están implementados con firmas públicas reales. La categoría de sonido inválida impide encolar el efecto pedido. X horizontal era opcional y su omisión está documentada. No se exige arquitectura ni funcionalidad del mod final.

**Veredicto de calidad: proporcional al probe, con un hallazgo P2 concreto.** El incremento mantiene un solo archivo, presupuesto de logs y payloads mínimos; no introduce mutaciones de daño, inventario o loot. Tras corregir la categoría de sonido no identifico otro bloqueo estático para compilar y ensayar v3. No es un veredicto de compilación ni de compatibilidad en ejecución.

## Hallazgo que corregir

**[P2] Categoría `players` rechazada por el helper de sonido — `bundle/api/src/lib.rs:93`.** `play_sound_to(..., "players", ...)` devuelve siempre `false`: la implementación busca igualdad exacta en `SOUND_SOURCES` y retorna antes de encolar si no encuentra la categoría. La fuente fijada `crates/horizon-client-core/src/sound_source.rs:4–6` contiene `"player"`, singular; `crates/horizon-client/src/game/native_hooks.rs:10054–10072` hace la validación. Cambiar únicamente `"players"` por `"player"` permite intentar el encolado solicitado. Mantener el registro del retorno y la limitación de que un export sin audio no demuestra audición. Hallazgo comunicado al principal; no lo corregí en esta revisión.

## Verificación estática del resto

Fuentes bajo `experiments/horizon-probe/runtime/launcher/native/`; API generada en `target/fast/build/horizon-native-game-4e31fe35dc1758a4/out/` y source bajo `source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca/`:

- **Entrada y persistencia:** `lib.rs:50–60` conserva sin cambio la selección de tecla, liberación, `HELD`, `focused && logged_in && Playing` y techo de tres impulsos. Solo cambia 0.05 a 0.30. El aporte máximo del probe pasa a aproximadamente +0.90 vertical por proceso, sin garantía de velocidad total, colisión o ausencia de daño por caída. Codec, login y ticks anteriores se conservan. Manifiesto y dependencia mantienen el pin.
- **HUD:** `HudEvent.batch` y `Host::fill` coinciden con `mod_api_generated.rs:4232–4235` y `_host.rs:375`. `native_hooks.rs:10779–10783` interpreta `[x,y,w,h]` y ARGB: los dos rellenos producen un marco cian de 10×10 con centro negro de 6×6 en coordenadas GUI, sin texto. El hook es HUD y la API omite F1; visibilidad real y posible solapamiento con otros HUD no se deducen de la firma.
- **Ping/ack:** `lib.rs:18–20,65–74,91–95` usa canal exacto, ping `[1,1]` y ack `[1,2]`. El cliente compara todos los bytes antes de emitir un ack constante; datos inválidos o canales ajenos no producen respuesta. El servidor solo compara y registra, por lo que no hay eco arbitrario ni bucle. Hooks y campos coinciden con `mod_api_generated.rs:359–362,4490–4502`; helpers con `_host.rs:502–507`. El ping nace únicamente en login y hay techo de diez mensajes por lado. `is_ok()` en envío significa aceptado/encolado, no recibido; el ack observado es la evidencia pendiente del recorrido completo. No hay correlación por solicitud o autenticación adicional: para este intercambio diagnóstico fijo se acepta ese límite, sin convertir `ack valid=true` en garantía de transacciones futuras.
- **Observadores:** `use_item`, `use_item_on`, `living_death` devuelven siempre `Flow::Continue` y no cambian el evento. `loot_drops` solo lee `len()`. Firmas reales: `mod_api_generated.rs:279,287,772,3019` y eventos en `4412–4423,4884–4888,7293`. Los eventos de uso son servidor; `ItemStack.count` es público (`crates/horizon-items/src/items.rs:333–338`). La muerte observada es intento de muerte de un ser vivo; loot tiene sus propias rutas y no equivale a todo tipo de drop.
- **Lecturas y efectos de login:** `player_logged_in` corre en servidor sin estado de jugador intercambiado (`mod_api_generated.rs:576–579`), contexto adecuado para `player_stacks`. `_host.rs:522–524` y `game/native_menus.rs:232–245` confirman snapshot opcional de 36 slots. Se registran total/ocupados, no identidad ni contenidos, y no hay concesión o consumo. El helper de sonido permite cualquier hook servidor y retorno de encolado (`_host.rs:1560–1569`), con la categoría que debe corregirse.

## Límites y siguiente validación

`EVENTS` comparte diez registros entre uso, muerte y loot: una ruta puede agotar el presupuesto antes de las otras. Planear esas observaciones temprano o en procesos nuevos; ausencia de log tras agotarlo no demuestra ausencia del hook. Mensajes inválidos del canal propio también consumen su presupuesto. Son techos aceptables para un probe desechable y están descritos en el informe.

Leí evidencia previa v2: controles de down/repeat/up, inventario y chat en `p0-smoke-controls3-controls.json`; el stderr correspondiente tiene un solo `event=impulse`, ticks y login. Los JSON de salida de controls3 y reopen1 registran exit0; el log reopen1 registra `previous_count=1`. Son resultados del binario v2, no de este source ampliado. No se transfieren al impulso nuevo, HUD, mensajes, sonido ni observadores v3. El foco perdido requiere ensayo propio si aún no está acreditado.

`git diff --check` y su variante limitada a lib.rs terminaron exit0; solo hubo aviso de conversión LF/CRLF en un checkpoint ajeno. No son pruebas de compilación. Después de la corrección: build serial real, comprobar HUD visual, ping/ack y rechazo de payload inválido, `sound_queued` sin inferir audición, lectura de inventario, observaciones de uso/muerte/loot y nueva física/gates en mundo desechable. Los registros no contienen nombres, UUID, direcciones ni rutas de cuenta.
