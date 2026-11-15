# P2 owner HEAD-only — revisión independiente de implementación

Fecha: 2026-11-15 America/Mexico_City. Revisor `/root/identity_review`, Sol high. Worktree `C:/Users/alanj.VIKINGO/.codex/worktrees/mistbrn-p1/MISTBRN`; rama comprobada `codex/alomancia-p2-owner`; base HEAD `e55db8cab13cdb6870e11ff5615cc7f14e42d221`.

**Veredicto spec: APROBADO en el alcance estático asignado. Veredicto calidad: APROBADO, sin hallazgos accionables.** Apto para continuar compilación y ensayos experimentales coordinados por root; no se declara native verified ni P2 completo, ni se aprueba merge por este resultado.

## Alcance y versión

Lectura de los cuatro helpers owner, descriptor `integration/horizon/horizon-mixin.toml` y contrato final `P2-native-owner-heads.md`, incluidas precisiones de presencia de marker/vanilla y admisión Dedicated. Se consultaron también ficha de entrega y cabecera del staging, sin ejecutarlos; el manifest probe histórico no se usa como evidencia de la API staged actual. No se modificó fuente, SDK, manifiesto o datos. Sin Cargo, build, juego, tests, commits ni subagentes.

La primera lectura fue sobre guard `47B054C2…`; durante la revisión root comunicó rustfmt de los cuatro helpers, exclusivamente formato. Se releyeron regiones y se usan abajo líneas/hashes posteriores a ese formato, guard `D42457D1…`. Los otros tres hashes Rust no cambiaron. Root comunica planner2 exit0 con tres HEAD, un módulo y un core, y lock SDK igual: es evidencia recibida, no ejecución ni validación independiente del planner por este revisor. El código generado/staged y el grafo compilado conservan gate propio de root.

## Cumplimiento y calidad comprobados

Referencias `guard` = `integration/horizon/world_guard.rs`; demás nombres relativos a `integration/horizon/`.

| Punto | Comprobación estática |
| --- | --- |
| Un solo registro sin dependencia inversa | `guard:3–14,48–55` usa Level/UUID del crate library, core existente y un único static privado. Descriptor `:11–21` agrega core a horizon-client-lib y declara exactamente un módulo público en save.rs. Helpers gameplay acceden al mismo módulo vía reexport; owner_bootstrap llama al módulo local. No se introduce import de Game/Options/Launch en library ni segundo ledger/static. La unicidad del paquete final debe conservarse en staging/build. |
| Captura del owner real | `profile_capture:1–9` usa Launch.owner y ruta con current_dir/saves/folder, sin auth, nombre regenerado ni UUID del payload. El contrato HEAD final autoriza capturar directamente Launch.owner en lugar de copiar Launch.player a Options. Descriptor `:23–29` enlaza &launch en HEAD serve. |
| Reserva inmutable e idempotencia | `guard:83–105` sólo crea contexto si es protegido; mismo owner conserva admission/initialized, otro owner rechaza. No hay remove/clear/reset del mapa. `world_key:63–71` unifica canonicalroot en captura, prepare y write antes de adquirir mutex, evitando registros por spelling distinto. |
| Presencia y vanilla | `guard:57–60` considera protegido cualquier metadata distinta de ambos NotFound, sin parser de marker paralelo. `:110–113` audita antes de pedir contexto y retorna para Vanilla; `:167–169` permite write sin política owner si markers ausentes, sin borrar reserva antigua. Namespace sin marker y quarantine siguen rechazando dentro del audit. La presencia metadata/canonicalización por write está expresamente permitida en la entrega final; no es un DAT scan. |
| Roles y Dedicated inicial | `preflight:5–11` pasa modo explícito desde ExecutionRole. `guard:115–134` publica Dedicated sólo tras auditoría y rechaza contexto incompatible/rol inesperado. `:172–175` permite Dedicated con owner None y exige admisión para writes protegidos. Así no se usa contexto ausente como bypass y se cubre el primer Level.write dedicado. |
| Existing y fallback con Level UUID ausente | `guard:140–143,308–310` reaudita existentes con expected B, incluidos ambos level. `validate_owner:329–355` distingue named file, legacy, UUID que conserva Level y selección load_owner; exige ambos Some(B). Metadata sin UUID/legacy con archivo propio B deja level_uuid=None y rechaza. No usa payload.player_id como identidad ni reasigna un Level existente. |
| Fresh y monotonía | `guard:302–314` sólo clasifica Fresh con marker válido, ambos level ausentes y cero snapshots auditados. `:152–159` admite inicial o Fresh→Existing, conserva Existing y no permite reset de Fresh inicializado a Fresh vacío. `:180–201` deja de auditar DAT al estar initialized/Existing; exige Some(B). Primera Fresh reaudita fuera del lock, acepta UUID None/B, compara de nuevo el contexto, asigna sólo UUID en memoria y marca initialized. Ningún error de writer posterior revierte ese estado. |
| Mutex y errores | `guard:74–80,88–103,121–129,145–161,193–202`: guardias cortas; no FS, writer ni Host bajo mutex. Tras auditoría se compara la copia previa con el contexto actual antes de publicar transición. Poison, missing context, conflicto y errores devuelven Err. Los helpers convierten Err en Break(Err) y sólo Ok en Continue; no panic/unwrap ni cancelación fingida. |
| Conservación de N0 | `guard:207+` conserva marker canónico, strictNBT antes de mapa, backups, límites, UUID/filename, transporte y codec compartido. `:390–395` rechaza storage_fault por presencia antes de state. El módulo no escribe disco ni repara snapshot; el único setter nuevo es Level.player_uuid en primera Fresh (`:200`). |

No se detectaron branches que puedan cambiar un owner registrado, volver initialized de true a false o admitir un write protegido sin la admisión correspondiente bajo el flujo nativo contratado. Una lectura fallida no se convierte en fresh. La auditoría doble de Existing responde al orden aprobado «auditar primero, contexto después»; no se propone abstraerla o añadir persistencia.

## Límites materiales y gates pendientes

El HEAD de Level.write no intercepta la ruta directa save_owner→write_player/save(guest); su cobertura depende de conservar Some(B) desde bootstrap/admisión y del guest B ya validado. Esa frontera está declarada en el contrato final y no se convierte aquí en un gate cubierto. No se revisó nuevamente el SDK completo ni mutadores de terceros.

initialized es aceptación en RAM, anterior al writer real: no significa persistencia/fsync ni rollback confirmado. Un fallo Fresh puede dejar configuración worldgen previa y exigir reinicio antes de reconstruir otro Level None. El contexto vive todo el proceso, conserva owner y puede bloquear reciclaje de ruta/perfil hasta reiniciar; no es migración. Cambios externos de markers/archivos entre comprobaciones permanecen fuera de garantía, conforme precisión final; no se propone cambiar ese contrato.

Pendientes de root: revisar parche staged/target real de library y dependencia core única, compilación del pin, nuevo mundo con perfil distinto de Player y primer snapshot coherente, save/autosave/cierre/reopen, mismatch A/B sin mutación, LevelNone+fallbackB rechazado, Dedicated marcado inicial y posterior, Vanilla sin marker, conflicto de owner al reabrir/ruta alias, error de writer sin reset de initialized, regresiones N0/quarantine y gates económicos/lifecycle restantes. El planner reportado y esta aprobación estática no sustituyen esas verificaciones.

## SHA256 finales revisados

| Entrada | SHA256 |
| --- | --- |
| `integration/horizon/world_guard.rs` | `D42457D1F5DDC664D7C2C8C7A9E5EEC0FD0488D22732453CFDCC884FBB41003C` |
| `integration/horizon/preflight.rs` | `BF83AED639E898AFF0391E5BF1B2477518F4B9A05CF39B4F46A18FBD2AF58342` |
| `integration/horizon/profile_capture.rs` | `B61B830FF878635CB263632E24D9BDDB9D38A97013C16ADF4232547B8EFD7B0B` |
| `integration/horizon/owner_bootstrap.rs` | `878A4E2C9115A837AFD8B184F3D115425CA4E881DC88C49AAFF2FEA67B5C0DF0` |
| `integration/horizon/horizon-mixin.toml` | `F60CD9D06859F34B1E8FA56E13D2581EF0C053654ED1D4BC0D1AE6D1AD699CD5` |
| `docs/execution/tasks/P2-native-owner-heads.md` | `0D6AA89F1654110B271AE7468CD03CFD234205732C70FEEDE3BE9B25C0F6E52F` |

Única escritura de esta revisión: este informe mediante temporal/promoción atómica. Las referencias y aprobación corresponden a los hashes de esta tabla.
