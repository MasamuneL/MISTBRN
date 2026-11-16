# Auditoría de recibos nuevos de P2

Fecha: 2026-11-16. Revisión de solo lectura del worktree `C:\Users\alanj.VIKINGO\.codex\worktrees\mistbrn-p1\MISTBRN`.

## Dictamen

Se verificaron diez recibos nuevos PASS, adicionales a los 47 observados anteriores: siete checks `gates-*`, `gates-partial-provision-1.json`, `gates-second-new-1-state.json` y `rich-reconnect-3-state.json`. Esto respalda evidencia adicional para N1/N2 y el reconnect enriquecido; no consolida automáticamente un total de 57 en manifiesto alguno ni significa que P2 esté completo. El intento `rich-dead-login-3` no es PASS: `result.json` reporta timeout y `termination=left_running`; se excluye del conteo.

Fuente previa: `evidence/p2-cuts-checks.json` declara build `p2-cuts1`, engine SHA256 `8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70`, 8 recibos nuevos y 47 previos (cuatro cortes N6 más cuatro reaperturas). Su propio alcance marca `p2_complete=false`, `n7=false`, `fsync=false`; no reinterpreté esos cuatro casos como cortes de writer.

## Recibos revisados

| Caso | Resultado observado | Evidencia/checks |
|---|---|---|
| `gates-both-owner-invalid-1` | PASS, rechazo controlado (`exit=-1`), archivos rechazados sin cambio | check SHA256 `720572D0E46A4BA0F33A00FF335D42E8C5646ADE274EEB04E841838EA42414B6`; clone y backup presentes |
| `gates-future-owner-primary-valid-old-1` | PASS, rechazo controlado, archivos sin cambio | `5E83ED2A1E7D2B9779E125866EFCA84D5AC89B79D3C4A557BA04EC6E1EDF4A8D`; clone/backup |
| `gates-marker-conflict-1` | PASS, rechazo controlado, archivos sin cambio | `DC0379E0237D7B5D3DF78E94C1009F2BAED35BC576567F0CD718C717B601207B`; clone/backup |
| `gates-marker-corrupt-1` | PASS, rechazo controlado, archivos sin cambio | `23BE610E5818A8BB793AC658082C02CC2CD8D6EEE473176E10AA86DCD1BAE756`; clone/backup |
| `gates-rename-1` | PASS, aceptación (`exit=0`, espontáneo), mundo renombrado | `0B78F5F4E99D32FB89AD671B4954043C8552F4A419779EE49610A65E91673A67`; clone/backup |
| `gates-restored-complete-1` | PASS, aceptación (`exit=0`, espontáneo) | `3B53819F2633212E336D3653333D6966B9C6551E99083A3816934115FC5BD8D6`; clone/backup y `*-state.json` |
| `gates-world-mismatch-1` | PASS, rechazo controlado, archivos sin cambio | `47135CE5BFE5284151B64D9D416AC42D058082A73929E4190B6C37BE214E36D9`; clone/backup |
| `gates-partial-provision-1` | Rechazo esperado; `.tmp` quedó preservado, hash `90AF0A90721947F556B57A9316DB4A0EC0109020C9815291DA32191A5C7A72AA` | JSON declara `reject=true`; backend deshabilitado consta en el recibo. No hay pareja check/clone/backup en el inventario de este caso |
| `gates-second-new-1` | Oráculo PASS de identidad; primary y `_old` coinciden `4D8A829C3AB59F126DEDB2C160D2DAE90EDB7F204E84E66FAEE3C61B8AF0B3E3`; `_new` ausente | `second-new-1-state.json`, `second-new-1-provision.txt` |
| `rich-reconnect-3` | Oráculo PASS de reconnect; primary y `_old` ambos `A04BF96ACC85FAE229A3CC3EC930E07A6F2A82725D8CCB5669BE8768001DD693`; `_new` ausente. Baseline `B543D7634C6418FABC9B399C6916079D6AAC02B8CC45EE32717B228AA129EE38` | state/fixture/clone; clone dice `closed=true`, `equal=true`, 22 entradas copiadas iguales |

Rutas principales: todos los recibos están en `docs/execution/evidence/`; cada check liga el caso al mundo de ensayo `C:\Users\alanj.VIKINGO\OneDrive\Documents\ChatGPT\MISTBRN\experiments\horizon-probe\runtime\game-p2-gates\saves\<caso>` y a su backup bajo `runtime\backups\gates-<caso>-complete`. El caso reconnect referencia baseline bajo `runtime\native-gate-runs\rich-reconnect-3\baseline\...dat`. No se recalcularon todos los hashes de árboles externos; se cotejaron los hashes/manifiestos consignados en recibos.

## N1/N2 y reconnect rico: cobertura y faltantes

La pareja `both-owner-invalid` cubre rechazo cuando ambas opciones de owner están inválidas; `future-owner-primary-valid-old` cubre que un primary inválido no se rescate desde `_old`; `world-mismatch`, marker conflict/corrupt y rename/restored-complete añaden casos de identidad/marker y ubicación/restauración. El segundo mundo nuevo agrega un estado de identidad generado independiente. Son evidencias valiosas de N1/N2, sin demostrar toda la matriz ni completar sus gates.

El fixture `rich-reconnect-3` parte de clon completo del mundo `game-p2-owner-econ\saves\credit`, reporta 22 archivos iguales, y su estado final cambia el player snapshot. Sin embargo, en la vista inspectada el estado por sí solo no documenta valores esperados/no triviales de fracciones, desbloqueos, reglas de juego o ledger ni sus checks campo por campo. No afirmo esas dimensiones PASS sólo por el título “rich”. El check/inspector de campos de ese caso no aparece entre los archivos localizados. Falta vincular evidencia que demuestre explícitamente esos valores y su persistencia/reapertura.

Los `*-check.json` de gates contienen resultados agregados (expect, exit/termination, ruta, y sin-cambio para rechazos), no campos `checks` anidados; los inspectores dedicados por caso tampoco aparecen junto a estos diez recibos. Así que se acepta PASS en el nivel que esos recibos expresan, pero no se atribuyen verificaciones más granulares no registradas allí.

## Intento de muerte fallido, excluido

`rich-dead-login-3-fixture.json` acredita fixture de jugador muerto (primary y `_old` seeded a `997f43...` desde `990502...`); `rich-dead-login-3-clone.json` confirma clon cerrado y 22 archivos iguales. Eso sólo acredita preparación, no comportamiento de muerte.

El log `runtime\native-open-runs\rich-dead-login-3\stderr.log` muestra aceptación de identidad, `WorldOpenReady`, login, `logged_out`, varios `save_observed durable=false`, `WorldCloseFinal`, seguido de `Horizon: frame-limited run timed out`, crash report y `Press Enter to close`. `stdout.log` registra login y mensaje “died”. La secuencia muestra que el harness sí alcanzó login/muerte/cierre del mundo y luego se atascó en la condición de término/limitador; no prueba que la operación de muerte-respawn concluyera correctamente. `result.json` (señalado en el relevo como `pass=false`, timeout, `left_running`, PID 29208) queda excluido. Los logs observados no identifican por sí mismos si la causa raíz fue el harness, la pantalla de muerte o el frame limiter; no existe diagnóstico concluyente aquí.

## Bloqueantes restantes

- Sin evidencia explícita de checks por campo para fracciones, desbloqueos, reglas y ledger en `rich-reconnect-3`.
- `rich-dead-login-3` fallido; no reutilizarlo como éxito ni ejecutar death-respawn como completado.
- Persisten gates N3/N4/N5/N7/N8/N9 indicados por el relevo; N7 y fsync no están cubiertos por cortes N6.
- El archivo de checks base declara expresamente que el bloque no completa P2. No actualizar manifiestos raíz ni checkpoints aquí.

## Addendum de validación de inspectores y lifecycle (revisión ampliada)

Corrección de precisión a la primera versión: `tools/inspect-native-gate-state.cjs` SÍ existe y fue leído junto con `tools/make-native-gate-fixture.cjs`. El inspector extrae el NBT completo, exige UUID del jugador/WorldId/schema, parsea el `saved_player` completo, valida ocho metales y ledger/revision, y compara deep-exact también `Inventory`, todos los campos foráneos de `NeoForgeData`, metadata del state y campos externos del envelope. Tolera el NBT list vacío legal (`child=0`, `items=[]`). El fixture crea estados deliberadamente ricos (ocho metales desbloqueados, reservas y burn variados, fracciones 123..130, reglas no triviales, ledger y revision); por lo tanto el PASS de reconnect sí cubre esos campos en su totalidad, aunque el JSON resumido sólo imprime hashes. Retiro la observación anterior de que faltaba evidencia por campo para reconnect.

Oráculos ejecutados localmente, sólo lectura, sin guardar sus stdout como evidencia ni modificar archivos:

| Run | Modo solicitado | Salida/hash primary y `_old` | Evaluación |
|---|---|---|---|
| `rich-reconnect-3` | `reconnect`, `old-expect=same` | PASS. Baseline `B543D7634C6418FABC9B399C6916079D6AAC02B8CC45EE32717B228AA129EE38`; ambos snapshots `A04BF96ACC85FAE229A3CC3EC930E07A6F2A82725D8CCB5669BE8768001DD693`; `.dat_new` ausente | PASS nativo en `native-open-runs/reconnect-3/result.json`, exit 0 espontáneo; completo deep-exact del modelo esperado. |
| `rich-death-respawn-4` | `death-respawn`, `old-expect=same` | El inspector entregado devuelve PASS por su contrato: baseline `E812513DB0907728E63B5833D5CF54580C55D5ADC69390176042A499D4C67A66`, ambos `FD1394E185EEF1C998BDB6A5FFEB3C21C1BB6B9F1BD49F8FBE69A620B1EE7BC0`; `.dat_new` ausente | **No aprobar como PASS P2**. El result nativo exit 0/harness `status=passed`, pero `offline_oracle` queda literalmente `pending: root compares closed-world snapshot with credit B1000/ledger1 baseline`. El inspector genérico define `death-respawn` como esperado con inventario vacío; el criterio de negocio/root conserva el stack de 2 lingotes de hierro. Los replies del harness muestran `held.count=2` antes, se mata al jugador y luego `held=null` al respawn. El snapshot con inventario vacío confirma la pérdida 2→0. Es fallo semántico aunque el oráculo con expectativa incorrecta lo rotule PASS. Contarlo entre los dos intentos fallidos/excluidos, no en los 13 PASS.
| `rich-dimension-4` | `unchanged`, `old-expect=same` | PASS. Baseline `E812513DB0907728E63B5833D5CF54580C55D5ADC69390176042A499D4C67A66`; ambos `D823AA0E82A59C5F8EA376DF7B729E9C1CD88779E523AF6A9BCD474544848981`; `.dat_new` ausente | PASS. Recibo nativo `status=passed`, exit 0, `spontaneous_after_quit`; `dimension_changed=1`, offline_oracle marcado pending por root y ahora cotejado. |
| `rich-save-pause-4` | `unchanged`, `old-expect=same` | PASS. Baseline y ambos snapshots `E812513DB0907728E63B5833D5CF54580C55D5ADC69390176042A499D4C67A66`; `.dat_new` ausente | PASS. Recibo nativo `status=passed`, exit 0, `spontaneous_after_quit`; 10 `save_observed`; offline_oracle marcado pending por root y ahora cotejado. |

El archivo `result.json` de death-respawn registra 1 `death_queued`? En realidad son 2; 1 `death_applied`, 1 `respawn`, 1 `logged_out`, 6 `save_observed`, y `death_logout_before_apply=null`. No da evidencia de logout ordenado antes de aplicar muerte. Su estado `pass=true` significa que el harness completó el guion interactivo, no que la semántica de persistencia satisfaga la comparación de root. La inspección prescrita por el usuario produce el hash del snapshot, pero no debe desplazar la expectativa correcta de conservar el inventario normal.

El total revisado queda: 13 PASS nuevos verificables sobre 47 observados previos (los diez gates de la tabla anterior más reconnect-3, dimension-4 y save-pause-4). Se excluyen dos fallos: `rich-dead-login-3` (timeout, `left_running`, PID 29208) y `rich-death-respawn-4` (harness exit 0 pero pérdida de 2 lingotes de hierro). No se afirma consolidación local ni P2 completo; root consolida el manifiesto.

### Diagnóstico SDK confirmado para dead-login-3

En el SDK fuente compartido `runtime/launcher/native/stage/crates/horizon-client/src/game.rs`, alrededor de línea 18772, el cliente comprueba `self.options.frames.is_some() && self.started.elapsed() > self.options.timeout`, emite `frame-limited run timed out` y sale del event loop. La rama de timeout sólo se activa cuando se pasa `--frames`; el argumento de dead3 era `--frames 1000000` junto a `--world-frames 80`. El harness de apertura usa el contador de mundo para la prueba y el SDK tiene además este deadline global de frames. El stderr demuestra que se llegó a `WorldCloseFinal`, y luego venció el deadline global mientras la app no acababa la condición de terminación esperada; se escribió crash report y quedó el prompt `Press Enter to close`, coherente con proceso retenido/timeout del harness. Esto confirma la interacción frame-limiter/terminación como diagnóstico del timeout; no convierte death login en PASS ni demuestra por sí solo si hubo un defecto secundario de pantalla de muerte.
