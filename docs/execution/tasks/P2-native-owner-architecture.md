# P2 — contrato mínimo owner integrado

Fecha 2026-11-15 America/Mexico_City. Agente /root/backend_quarantine.
Estado: ruling de arquitectura en investigación, READ ONLY salvo este informe.
Comprobación inicial: el informe no existía. Intención: comprobar barrera previa a adopción entre perfiles y todas las rutas save/autosave/owner, usando SDK exacto y mecanismo bundle firstclass. No fuentes/Cargo/build/juego/SDK/datos/commits/subagentes. No planner ejecutado ni compatibilidad de targets prometida.
Contexto: diagnóstico P2-native-identity-debug.md; identidad guest vs Level mezclada por SDK. Prueba con --name Player y 12 casos N0 aprobados por root es QA acotado, no solución general. No se reintenta Astra; sustitución Sol high indicada por root.
## Ruling mínimo

No aprobar un parche aislado a save_owner ni presentar --name Player como solución general. Recomiendo una barrera de perfil **antes de preparar/adoptar** y fijar el UUID del owner únicamente durante la creación demostrada de un mundo Allomancy nuevo. Con una sola identidad desde el primer Level::write, save_owner, store_owner/autosave y los snapshots guest pueden conservar sus serializadores actuales: todos reciben B, sin remapear PlayerId ni admitir alias.

Esto es una propuesta concreta de adaptación nativa del bundle, no una integración validada. Los targets y selectores nuevos necesitan planner y build sobre el pin; no se ha ejecutado ninguno. El mecanismo firstclass está documentado por el SDK: helpers inyectados en funciones del juego, módulos propios y HEAD/BEFORE_CALL/AFTER_ASSIGN con cancelación donde corresponde. No necesita fork permanente ni modificar upstream original. Si el planner no permite el punto/locals/tipos necesarios o la cobertura del bootstrap, se rechaza la adaptación; no se sustituye por login tardío ni edición textual ad hoc del SDK.

## Hechos de fuente que cierran los dos riesgos

SDK pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca, fuente local original en runtime/launcher/native/source/<pin>.

1. El autosave también mezcla identidades: game/lan.rs:1790–1813 store_owner toma Owner.uuid, busca el guest y llama playerdata::store(g, &mut self.level, vehicle). game.rs:15413–15421 hace store_owner y write_level_for_role; este último, :22324–22336, llama Level::write para IntegratedServer y write_without_player para DedicatedServer. playerdata::store:435 copia player_extra sin sustituir player_uuid. Cambiar sólo playerdata::save_owner:104 deja esta ruta abierta.
2. Rechazar en login es tarde para un mundo existente A abierto por perfil B. game/lan.rs:2856–2857 llama load_owner antes de LoadFromFile y placement/login. playerdata::load_owner:84–95 aplica el Level cuyo singleplayer_file esté presente, aunque UUID del guest sea B. native_player_compound genera UUID B desde el guest, pero guest.saved puede contener payload A. El backend devuelve Fault por PlayerMismatch; luego el SDK puede guardar guest B con payload A. La barrera debe ocurrir antes de esa adopción y de la primera escritura normal del mundo.
3. WorldRequest sólo transporta save_root/folder/name/seed/creative/create (world_loading.rs:4–11). Su prepare_world sí recibe &Options (:85–92); no recibe Launch/owner. El guard actual sólo enlaza request y no conoce perfil. No se debe añadir comparación fingiendo que el perfil ya está en WorldRequest.
4. open_integrated construye Launch.player desde auth.current().name o Options::player_name y fija Launch.owner=offline_uuid(player) (integrated.rs:401–415). Después clona Options (:429), fija ExecutionRole::IntegratedServer y pasa clones a serve/WorldLoader; serve:1231–1253 crea WorldRequest y arranca WorldLoader, cuyo start mueve options al worker world_loading.rs:289–312. Un thread_local del server no llega al worker: no usarlo como contexto implícito.
5. prepare_world HEAD precede upgrade_older_world_with, migrate_legacy, SessionLock y Level::read (:140–149). El bootstrap de un mundo nuevo crea Level, hace let mut level y termina guardando level.write si level.dat no existe (:264–266), antes de publicar PreparedWorld. Aquí se genera el UUID por defecto Player si no se fija antes.
6. Level.ensure_player_uuid conserva un Some y en None asigna offline_uuid(Player) (save.rs:3790–3799). Esto no equivale al perfil activo general. Copiar B sólo después del primer namespace/archivo ya no es bootstrap: es una migración de identidad.
7. El Level de metadatos es compartido entre dimensiones: game/levels.rs Parked guarda World/Streaming/SaveDir y campos por dimensión; no un save::Level alternativo. No se encontró setter público Host del UUID de ese Level. Esto respalda conservar la igualdad tras bootstrap, pero el lifecycle real, respawn y cambios de dimensión siguen requiriendo ensayo.

## Opción concreta para revisión: tres adaptaciones del bundle

Los nombres siguientes identifican candidatos encontrados en fuente. Son **propuestas pendientes de planner**, no contratos públicos API411 ni selectores ya resueltos.

| Punto candidato | Información/acción mínima | Contrato de fallo |
| --- | --- | --- |
| Game::open_integrated en integrated.rs, justo después de crear la copia mutable options | Copiar Launch.player exacto a Options.player_name de esa copia del servidor. Son el mismo perfil; no cambiar Launch.owner, sesión auth, UUID online ni el cliente. AFTER_ASSIGN es una forma candidata. | Planner debe demostrar ocurrencia única, Launch y options en scope y binding mutable válido. No volver a generar nombre en worker. |
| WorldRequest::prepare_world HEAD, guard actual ampliado con &Options | Para ExecutionRole::IntegratedServer, exigir nombre congelado, derivar B con el mismo offline_uuid del SDK y validar originals/perfil antes de continuar. Dedicated conserva N0 sin política owner. | Ausencia/contexto inesperado o identidad ambigua devuelve Break(Err) del Result<PreparedWorld>, antes de migrar/escribir/adoptar. Mantener rechazo transport/codec/quarantine y backups. |
| prepare_world después de crear mut level y antes de make_streaming/primer Level::write | Sólo en bootstrap nuevo demostrado por auditoría HEAD, asignar Level.player_uuid=uuid_ints(B). Un BEFORE_CALL de make_streaming o punto local equivalente es candidato; debe poder devolver Err antes del primer write. | No reidentificar un Level leído existente. Rechazar si no se puede demostrar bootstrap, si aparece snapshot/namespace/marca o si el contexto difiere. No restaurar archivos ni tocar UUIDs NBT de terceros. |

La congelación del nombre aprovecha el valor exacto ya usado por Launch, incluso en sesión autenticada: el servidor integrado actual también deriva offline_uuid de ese nombre, no del auth UUID. Recalcular auth/Options::player_name desde el worker es peor contrato: Options::player_name puede load_or_create horizon-player.txt y el guard debe seguir siendo de lectura. Un mapa global por ruta o entorno no es necesario; evitarlo y evitar una identidad persistente paralela.

No se requiere que la ficha de guard exponga struct nuevo público: el helper y módulo bundle pueden compartir una decisión privada mínima o recomprobar criterios ya auditados antes de asignar el Level nuevo. No inferir fresh sólo de created=Level::read(...).is_none(): Level::read también falla sobre originales corruptos. N0 debe haber rechazado cualquier original ilegible y la condición nueva debe incluir ausencia real de los snapshots.

## Política de admisión propuesta, sin migración de perfiles

B es el UUID que Launch ya asignó al perfil de esta apertura. El mundo se considera preparado para Allomancy por su marker válido; presencia de namespace/marca sin marker sigue rechazando conforme N0.

| Entrada real | Conducta propuesta |
| --- | --- |
| Mundo sin marker y sin namespace/marca Allomancy | Flujo vanilla actual, backend deshabilitado; no normalizar identidad ni cambiar política owner. |
| Mundo preparado nuevo: level.dat y level.dat_old realmente ausentes, sin snapshots de jugador principales/backups/legacy ni datos de jugador existentes | Bootstrap B en el Level antes de que el SDK escriba el primer snapshot. No existe A económico ni identidad de jugador ajeno que transferir. |
| Mundo preparado existente con owner seleccionado inequívoco A=B, snapshots íntegros y namespaces válidos | Permitir, manteniendo UUID, inventario, namespace, ledger y claves ajenas. Sin migración ni renombrado. |
| Mundo preparado existente con A distinto de B, aunque aún no tenga namespace | Rechazar antes de Level::read/adopción y antes de inicializar Allomancy: permitir el primer namespace aquí recrearía el alias. No reasignar vanilla A por ausencia de state. |
| Existe state y perfil B distinto de su owner A; existe storage_fault; original inválido o selección owner ambigua | Rechazo temprano conservador. Un backup no sirve para eludir una marca o adoptar otra identidad. |
| Datos existentes pero no se determina owner, falta contexto de perfil o la selección SDK/fallback no se puede probar | Rechazar apertura Allomancy y requerir diagnóstico/importación offline separada. No tratarlo como mundo nuevo. |

La comparación debe cubrir lo que el SDK podría adoptar: metadata principal y backup, archivo nombrado por singleplayer_uuid con su dat_old, legacy Data.Player y el fallback al propio archivo B de load_owner. N0 sigue auditando todos los snapshots y valida identidad del payload contra UUID NBT y nombre donde corresponde; **no** sustituir expected_player por payload.player_id, no permitir A como alias de B y no omitir el archivo del owner. Para caso existente A=B basta comparar contra el mismo B que usará la conexión; ante alternativas incompatibles se rechaza.

La rama bootstrap estricta no incluye un mundo existente con default Player y datos vanilla, aun sin Allomancy. Normalizar ese caso podría transferir inventario, estadísticas, advancements, adjuntos y llaves de otros mods a otro perfil. Eso sería importación/migración y queda fuera de la opción mínima. Un mundo realmente nuevo puede asignar B antes de existir esos datos; esa operación no cambia significado entre perfiles. Cambiar después de nombre/perfil se rechaza, no se convierte en otra identidad económica.

## Cobertura requerida de guardado

Con creación nueva fijada en B antes del primer write y apertura posterior admitida sólo con A=B:

- Logout/cierre save_owner conserva Level.player_uuid=B; store copia namespace B y Level.write_player escribe UUID/payload B. save(guest) también escribe B.
- Autosave/pausa/save explícito store_owner→write_level_for_role→Level.write conserva B, y el writer guest produce B con playerdata::player_tag. No se necesita parchear cada serializer para reetiquetar payloads.
- Load/respawn/acting/cambio de dimensión permanecen bajo el mismo B; deben verificarse con SDK real. El cierre world_fault/cuarentena del backend sigue siendo necesario y no demuestra esta nueva invariante.
- Shutdown antes de que el owner se coloque puede escribir metadata: el Level ya debe tener B desde bootstrap, no esperar callback login para fijarlo. Error de apertura no debe crear snapshot parcial.
- El caso A existente abierto por B debe terminar antes de load_owner, así no puede llegar a un guest B con namespace A ni al logout normal de ese guest.

Esto cubre conceptualmente todas las rutas owner conocidas sin reidentificación en save. No es garantía frente a otro mixin que cambie identidades privadas, manipulación concurrente de archivos o bootstrap fuera de los targets cubiertos. No esconder ese límite con setters tardíos en save_owner/store_owner.

## Riesgos, gates y alcance

Riesgos reales: selector interior frágil o más de una coincidencia; visibilidad y lifetimes de Options/Launch/private helpers; profile name no congelado en alguna entrada; marca/perfil evaluados sólo en principal omitiendo backup; clasificación de nuevo por parse fallido; escrituras iniciales antes del punto seleccionado; pérdida de evidencia al intentar migrar default Player existente. Planner/build deben fallar de forma cerrada si cambia pin/estructura. Mantener el guard original antes de toda adopción NBT y sin escrituras.

Validación necesaria, serial por root: planner exacto y revisión del parche staged; build SDK real; mundo nuevo preparado con perfil arbitrario distinto de Player, observar primer snapshot y reopen coherentes; autosave/pausa/save explícito/logout/cierre temprano/respawn/dimensión; mundo A coherente abierto como B rechazado antes de mutación con hashes intactos incluso sin primer namespace; backups/legacy y archivos B de fallback; mundo vanilla sin marker conserva conducta; dedicated sin cambio; repetir 12 casos N0 más cuarentena/gates económicos pendientes. --name Player actual sólo evita A≠B en ese fixture y no cubre estos casos.

La opción es mayor que el fix de cuarentena: añade una política explícita mundo/perfil y adapta bootstrap nativo mediante el bundle. Recomiendo que root revise/apruebe este contrato antes de implementar. Si la autorización vigente sólo cubre quarantine/P2 originalmente descrito, requiere consulta de alcance para esta política; si ya autoriza resolver identidad nativa, puede seguir con planner/implementación reversible sin repetir un permiso genérico. No hay una exigencia de SKILL ni un bloqueo de seguridad externo que obligue a pedir confirmación por sí solo. Una migración de mundos existentes o soporte de cambio de owner **sí** es otro alcance y no se incluye.

Dictamen final: vía bundle firstclass plausible localizada en fuente, sin API validada ni código cambiado. Normalización sólo antes del primer snapshot de un mundo realmente nuevo; mismatch existente se bloquea antes de adopción. La propuesta evita el parche incompleto de save_owner y mantiene identidad core estricta. Si los puntos no resuelven de forma única, conservar bloqueo del producto y QA acotado hasta diseñar otra adaptación; no degradar N0.