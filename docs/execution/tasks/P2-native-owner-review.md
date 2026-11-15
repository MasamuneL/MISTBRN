# P2 — revisión independiente del contrato owner

Fecha: 2026-11-15 America/Mexico_City. Revisor `/root/identity_review`, Sol high, sustitución indicada por principal. Revisión conceptual y estática; no implementación.

**Veredicto: apto para planner experimental con la precisión de admisión existente descrita abajo. NO compilado ni native verified.** Hay una ambigüedad relevante de contrato que debe cerrarse antes de implementar/admitir mundos; no impide investigar los tres puntos del planner con el contrato reforzado. No se aprueba un parche aislado de save_owner, un alias de PlayerId ni migración de owner.

## Alcance y autorización

Leídos completos `P2-native-owner-architecture.md` y `P2-native-identity-debug.md`. Contraste acotado de los puntos concretos en SDK pin `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`, original/runtime/launcher/native/source del runtime compartido documentado. No reauditoría completa del SDK ni lectura de datos del mundo en este encargo.

La autorización transmitida por principal cubre resolver identidad mediante adaptación reversible del bundle, bootstrap de mundo realmente nuevo y rechazo de mismatch existente. Este informe no exige otro permiso genérico. No incluye importar un mundo existente, cambiar owner, mover inventario/adjuntos/economía ni alterar upstream original silenciosamente.

## Hallazgo que debe cerrar la implementación

**[P1] Diferenciar owner adoptado de UUID conservado en Level.** Referencias del contrato: arquitectura `:47`, `:52` y la invariante `:58–61`. La frase «owner seleccionado inequívoco A=B» debe exigir también que el `Level.player_uuid` que el SDK conservará sea **ya Some(uuid_ints(B))**, además de que la selección de `load_owner` corresponda a B. No basta deducir B del archivo propio de fallback.

Contraprueba de fuente: mundo existente preparado, level.dat válido sin `singleplayer_uuid` ni `Data.Player`, pero archivo propio B íntegro. `save.rs:3949–3959` no selecciona ese archivo para construir Level y deja `Level.player_uuid=None`; `playerdata.rs:89–90` sí carga el propio B en el guest. Después `game/lan.rs:1803–1805` llama store, que copia namespace B (`playerdata.rs:435`) sin fijar Level.player_uuid; `save.rs:4391` llama ensure_player_uuid y `:3790–3799` inserta offline_uuid("Player"). Para B distinto de Player reaparece el defecto que se quiere impedir, aunque la adopción del guest sea inequívoca. No es un fallo de endian ni del codec.

Regla mínima: en mundos existentes preparados, reconstruir de manera conservadora la identidad que Level::read conservará desde los originales auditados y exigir Some(B); exigir asimismo que ninguna selección/fallback que pueda adoptar load_owner contradiga B. Si falta UUID de Level, incluso con fallback B íntegro, **rechazar sin asignar B al Level existente**. Mantener la comparación de principal/backup/legacy y quarantine. Así el caso cae explícitamente en rechazo de la fila `:50`, sin migración. Una prueba nativa adicional deberá cubrir ese caso concreto y comprobar ausencia de mutación.

## Evaluación de los tres puntos

1. **Congelar Launch.player exacto en la copia de Options antes del worker: correcto.** `integrated.rs:403–416` calcula player y Launch.owner a partir del mismo texto; `:433` crea la copia mutable, antes de los cambios de rol/opciones y de serve. `serve:1227–1241` pasa options.clone a WorldLoader; `world_loading.rs:289–302` lo mueve al worker. Copiar a `Some(launch.player.clone())` evita depender de thread_local, auth leída otra vez o nombres regenerados. El guard debe leer el campo ya presente y rechazar ausencia, sin llamar al fallback `Options::player_name()` (`game.rs:637–646` puede crear el nombre offline). Planner debe demostrar unicidad, scope y binding mutable; no se afirma que lo admita todavía.

2. **Guard HEAD con el perfil exacto: correcto con la precisión P1.** `WorldRequest` no transporta owner, pero prepare_world recibe &Options (`world_loading.rs:85–100`). HEAD antecede a fallback/recovery, upgrade/migrate y lectura/adopción de Level (`:108–149`). Esto permite rechazar mismatch antes de que load_owner mezcle guest/Level. Dedicated mantiene la auditoría N0 existente sin imponer owner integrado; vanilla sin marker continúa con backend deshabilitado. No reemplazar expected_player por el payload, no omitir backups, ni convertir un perfil distinto en alias. El rol y presencia del perfil congelado deben validarse, sin asumirlos por nombre de directorio.

3. **Bootstrap de Level.uuid solo nuevo antes del primer guardado: correcto.** `world_loading.rs:149–167` construye el Level local; `:264–265` hace el primer Level.write en mundo nuevo. Asignar Some(uuid_ints(B)) antes evita el default Player de ensure_player_uuid. La ausencia real de snapshots debe quedar demostrada por N0; created=Level::read(...).is_none() solo no basta. Ningún Level leído existente se reidentifica. Para un punto de inyección que pueda rechazar/recomprobar, preferir inmediatamente después de `let mut level` (`:167`): el candidato BEFORE_CALL make_streaming (`:229`) está después de `write_world_gen_settings_with` (`:173–179`) cuando create está presente. Ese writer no escribe UUID de jugador, pero ya puede mutar archivos; no prometer rechazo tardío con hashes intactos sin revisar ese orden. La arquitectura ya identifica este riesgo en `:70`.

La invariante conjunta reforzada es: toda apertura Allomancy integrada que continúa tiene guest.owner=B y Level.player_uuid=Some(B) desde antes del primer guardado; en existente esto se verifica, en nuevo se inicializa. Store/autosave/logout no necesitan reetiquetar payloads. Respawn/dimensiones/cierre temprano siguen siendo gates de ejecución, no hechos acreditados por esta revisión.

## Gates y límites

Planner debe resolver los tres puntos de forma única, bindings y cancelación real, y root debe revisar el parche staged sobre el pin. Si falla el acceso al contexto o un target, se conserva el bloqueo; no se sustituye por login tardío ni edición textual upstream. Esta revisión no ejecutó planner, Cargo, build, juego, tests de fuente ni mocks; no modificó fuentes, SDK, snapshots o commits y no lanzó subagentes.

Además de la matriz de arquitectura `:72`: ensayar existentes con metadata sin UUID y fallback propio B (rechazo), nuevo con perfil arbitrario distinto de Player (primer write/reopen coherentes), mismatch A/B sin namespace (rechazo sin escrituras), autosave antes de logout, cierre antes de placement, backups/legacy y vanilla/dedicated. Las lecturas de resumen/preload anteriores al HEAD no son adopción del guest ni quedan convertidas aquí en primera lectura absoluta del proceso. TOCTOU de editores externos y mixins ajenos permanecen fuera de garantía.

## Hashes de entradas revisadas

SHA256 de bytes completos de los archivos; SDK relativo a la raíz del pin indicada arriba:

| Entrada | SHA256 |
| --- | --- |
| `docs/execution/tasks/P2-native-owner-architecture.md` | `F83A7EC4FAAB7C5E89B8F75B1BABEA2E28619AE96AAAC06C3A9E22A7F5F3A05A` |
| `docs/execution/tasks/P2-native-identity-debug.md` | `259D155313914A87D3618CF2BA4BED27826EDB242BD505A1165BA5BC32439A63` |
| `crates/horizon-client/src/game/integrated.rs` | `91DB0D97BB46C085D05CE41B875CA23848677D9D3770A135B8B477E152ECAD4E` |
| `crates/horizon-client/src/game/world_loading.rs` | `DD00D259B4B8115FFD7C07C7C2F0AC6923CA6A6CBC882FE72E53F8AE33C9FEE9` |
| `crates/horizon-client/src/save.rs` | `03A2CEB43C3D3D1341546FF4B49F5022279247A1739B396D185B619B109CF071` |
| `crates/horizon-client/src/lan/playerdata.rs` | `E65B5FC94B53DEF9C9C43112C6000915D48AF300E895DF38D45EC813026C61B6` |
| `crates/horizon-client/src/game.rs` | `419C2BDD785F2FEF499AE460C41F0BFC125B19FB12A8A8C3CEF54B27DE4687DF` |
| `crates/horizon-client/src/game/lan.rs` | `3A563D8BF332148A99947CBEF319286383B168A12B11E73C6F7F4A80DCA0DC35` |

Consulta incidental `horizon-command/src/args.rs:323` corresponde a Selector, no a Options; no sustenta el contrato. El campo Options.player_name correcto se localizó en `horizon-client/src/game.rs:610`, con método :637.

## Apéndice — variante HEAD-only tras fallo real planner1

Revisión conceptual puntual, 2026-11-15. Leído último CHECKPOINT, SHA256 `75226DCBBDE1758C32CC4B583FECCC148F0E81E9C1972194761AA49A90E912B7`. Según ese registro, planner1 falló realmente con E_UNSUPPORTED_CONTROL_FLOW en captura interior de open_integrated; el plan no está aprobado y no deben repetirse variantes interiores equivalentes. No ejecuté ni revalidé ese planner en esta revisión.

**Dictamen: apta para implementación/planner experimentales al incorporar la admisión Dedicated explícita indicada abajo.** El registro efímero por ruta es una adaptación proporcionada al límite del planner observado. No es identidad persistente paralela, economía ni ledger. No demuestra compilación ni funcionamiento nativo y no autoriza relajar el rechazo P1 de la revisión anterior.

**[P2] El primer Level::write de Dedicated también necesita una admisión explícita.** El contrato «Dedicated usa write_without_player» describe guardados posteriores, pero no basta para excluir el bootstrap: el prepare_world ya contrastado en esta revisión llama a `level.write(&save_root)` en `world_loading.rs:264–265` sin condición de rol. Un mundo dedicado nuevo marcado no pasó por integrated::serve; si HEAD Level::write rechaza todo missing-context marcado, lo bloqueará. Corrección mínima de contrato: HEAD prepare_world, tras N0 válido, registra una admisión Dedicated explícita que el hook reconoce para no imponer owner/bootstrap integrado. No inferir Dedicated de missing context ni permitir missing context marcado general. Así un bypass de preflight sigue fallando cerrado y el backend dedicado conserva la política anterior. Sin esta precisión no se cumple «Dedicated sin cambio».

Con esa corrección, los tres puntos son coherentes:

- HEAD integrated::serve registra el Launch real y la ruta calculada con las mismas entradas que WorldRequest::new. El UUID esperado debe corresponder al mismo Launch.player/owner, no derivarse de un nombre regenerado en worker. El par ruta/perfil queda inmutable durante el proceso: conflicto devuelve Err; el caso idempotente conserva admisión e initialized, sin resetearlos ni sobrescribir un worker anterior.
- HEAD prepare_world, en rol integrado, exige ese contexto por ruta y audita originales con el expected UUID exacto antes de fallback/migración/adopción. Existing exige selección compatible y Level.player_uuid resultante Some(B), incluso si el archivo propio B sería un fallback válido; Fresh requiere ausencia real de snapshots y no un parse que devolvió None. Sólo después de completar N0 se publica admisión válida. Vanilla no habilita normalización. Missing admission, IO/poison/conflicto o contexto incoherente rechazan de forma cerrada.
- HEAD Level::write en admisión integrada Fresh comprueba ausencia real de originales antes de la primera inicialización y admite sólo UUID en memoria None/B, fijándolo a Some(B) antes de ensure_player_uuid/serialización. El estado initialized se publica al completar la comprobación y asignación en memoria, no después del IO ni como confirmación de persistencia. Si el writer posterior falla, no resetear a Fresh para aceptar otro Level con None. Existing/initialized sólo afirma Some(B), sin abrir DAT en cada autosave. Liberar mutex antes de FS/Host y revalidar que la admisión sigue vigente al publicar la transición; ningún lock se mantiene a través del writer.

La escritura de worldgen settings anterior a Level::write no contiene UUID de jugador; mover el setter al HEAD del writer puede mantener la invariante del primer snapshot. HEAD prepare sigue siendo la barrera del mismatch existente antes de esas escrituras. No se debe afirmar que cualquier fallo tardío de bootstrap deja todos los archivos sin cambios: puede existir worldgen parcial aunque ningún snapshot de jugador se haya escrito. Eso debe quedar como error de ensayo, sin marcar recuperación automática.

El registro no se elimina y rechaza otro owner en la misma ruta, incluso tras cerrar. Reinicio para reutilizar la ruta es un límite deliberado aceptable en esta variante. No se promete detectar ediciones externas de snapshots, reemplazo de ruta con el mismo perfil o un mixin ajeno que eluda las rutas cubiertas; no añadir canonicalización/epochs persistentes, remapeos ni migraciones para resolver esos escenarios en este corte.

Gates mínimos adicionales: planner de los tres HEAD y revisión staged; nuevo integrado B distinto de Player/primer write/reopen; existentes sin UUID de Level con fallback B (rechazo); conflicto de owner en la misma ruta con worker y al reabrir; retries tras fallo de writer sin reset de initialized; dedicated marcado nuevo y posterior guardado; vanilla sin marker; mismatch A/B antes de mutación. Todos siguen pendientes de root, sin native claim. Única escritura de este encargo: este apéndice por temporal/reemplazo con .bak; sin fuentes, Cargo, juego, edición SDK, commits o subagentes.
