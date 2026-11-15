# P2 native guard — checkpoint propio
schema_version: 1
estado: in_progress
base: fed744b9252e14ae6fcd12b314fba61040250fe0
propiedad: core world_identity + module/reexport; integration/horizon/preflight.rs + world_guard.rs; docs/world-identity.md; esta ficha/evidencia

API congelada y comunicada al principal antes de fuentes: WorldIdentitySource Primary/Backup, LoadedWorldIdentity {world_id,source}, WorldIdentityError Parse/Schema/Future(u64)/InvalidId/Conflict/Io/TooLarge. decode_world_identity(bytes), load_world_identity(world_root). No crea ni escribe archivos.
Intención: tests conductuales marker, stub y RED pertinente, implementación GREEN, workspace/fmt; luego guard readonly NBT con SDK decoder y auditoría raw de duplicados. Sin SDK build ni commits; principal ejecuta plan/build/N0.
Preservación: documentos nuevos mediante temporal/promoción en mismo directorio; actualizaciones conservan .bak mediante reemplazo atómico. No tocar archivos ajenos.
Siguiente acción: escribir tests y stub del módulo; Cargo exclusivo ya concedido por principal.

## Checkpoint de entrega
estado: handed_off (marker verified; guard source frozen; native gates pending)
Resultado: 16 tests marker GREEN; RED12/16 conductual; workspace71 +1doctest PASS; fmt check0. Cargo libre comunicado al principal. Fuentes guard/helper completas y congeladas, sin SDK compile ni ejecución de juego. API/documentación en docs/world-identity.md; hashes/evidencia en docs/execution/evidence/P2-native-guard.md.
Validación raw antes del decoder SDK detecta duplicados semánticos UTF-16; límites64MiB/1M nodos/depth128/4096 archivos jugador. No recuperación automática .dat_old; error pide restauración completa offline. Source marker Backup explícito sin rewrite. Sin marcador y sin namespace permite vanilla; backend mantiene economía apagada. Namespace sin marker, futuro/corrupto/IO/raíces inválidas bloquean.
Siguiente acción exacta: principal aplica descriptor/crate core único, functional RED/plan/build SDK y N0 integrado sin wrapper, incluyendo extensión .DAT Windows. P2 y N1–N9 no completados por estos tests. No storage_fault añadido; protocolo en evaluación por principal.
Fuentes entregadas por SHA256; world_guard final CD26849BDD037979AC2A49FBC46B6CF3E87586EBBEDC71B9D87F5929280B34D4. No más fuentes propias previstas. Cualquier revisión de build requiere relevo del principal para editar.
## Relevo acotado storage_fault — intención previa

estado: in_progress; nuevo contrato congelado por principal, sustituye la nota histórica de protocolo aún en evaluación.

Propiedad en este relevo: solo `integration/horizon/world_guard.rs`, `preflight.rs`, esta ficha y su evidencia. No core/manifests/backend/SDK, Cargo/build/juego, tests de texto, Host falso, commits ni subagentes.

Intención antes de fuentes: en `validate_player`, después de exigir `NeoForgeData` Compound y antes de buscar `allomancy:state`, rechazar cualquier presencia de `allomancy:storage_fault` sin interpretar tipo, versión o contenido. La marca canónica del writer es Compound `{format_version:Int(1),reason:String("rollback_unverified")}`, pero el lector no depende de esa forma. El helper compartido cubre snapshots de jugador principales/backups y `Data.Player` legacy en level principal/backup; también debe rechazar cuando falta state o marker.

Preflight hoy propaga Err sin emitir una traza propia de rechazo. Añadir dos logs constantes en el punto de inyección: `Allomancy preflight: accept` antes de Continue y `Allomancy preflight: reject` antes de Break. Sin UUID, rutas, payload ni interpolación de reason. Se mantiene la propagación del error original.

Plan de comprobación propia: releer diff y hashes; comprobar estáticamente orden y todos los callers. Principal conserva la prueba conductual SDK/N0 real (incluidas marcas malformed/futuras/state ausente, backups y legacy) y cualquier check de compilación/formato. Ningún resultado anterior de tests acredita este cambio nuevo.

## Relevo storage_fault — resultado y handoff

estado: handed_off_pending_native; fuentes acotadas entregadas, no ejecución SDK/N0 acreditada.

Resultado: `world_guard.rs:168–172` rechaza cualquier presencia de `allomancy:storage_fault` antes de buscar state en `:173`. No inspecciona el valor: Tag primitivo, Compound canónico, forma inválida/futura y state ausente se rechazan por presencia. El recorrido existente llama el mismo helper para `Data.Player` de level principal/backup (`:46–55`) y cada archivo jugador principal/backup (`:89–107`). Un UUID/raíz previamente inválido también sigue rechazando.

`preflight.rs:3–10` ahora emite logs constantes `Allomancy preflight: accept` (info) y `Allomancy preflight: reject` (warn) en los brazos del resultado antes de Continue/Break; no interpola reason ni datos privados. Se conserva exactamente la propagación de reason a Err y la validación síncrona previa al cuerpo original.

SHA256 finales: world_guard `0B250DB096D8250B9D57E7BE569D605CAD6017398B4900A289EC017428F25DAE`; preflight `C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55`. Sustituyen los hashes históricos anteriores del guard y preflight.

Verificación propia limitada a lectura estática de cambios/callers, hashes y validaciones de edición (hash original y una sola coincidencia por ancla). Sin Cargo/build/juego, rustfmt, tests de texto, Host falso, commits ni subagentes; ningún test anterior se atribuye al cambio nuevo. Principal debe compilar el pin y ejecutar N0 real con marca canónica/malformed/futura/state ausente, en principales/backups/legacy, fixture sana y logs accept/reject; comprobar hashes de snapshots antes/después. N1–N9 siguen pendientes.

Incidencia de edición resuelta: auto-review rechazó antes de ejecutar un primer comando que reconstruía preflight completo por riesgo de sobrescribir WIP; ninguna fuente cambió en ese intento. La alternativa aprobada cambió solo las líneas únicas de los dos brazos, con hashes previos, y solo insertó el rechazo en el guard. No hubo aprobación eludida ni escritura ajena.

## Plan3 SDK real — intención de corrección mínima

El log real `docs/execution/evidence/p2-real-guard-plan3.log` leído contiene `E_MODULE`, `support module source cannot have file attributes or a shebang`, mod_id allomancy. Principal informa que target plan no se produjo. `world_guard.rs` comienza con dos comentarios inner-doc `//!`, que el parser del módulo soporte trata como atributos del archivo.

Intención previa autorizada: convertir solo esos dos prefijos iniciales a comentarios ordinarios `//`, exigiendo hash previo `0B250DB096D8250B9D57E7BE569D605CAD6017398B4900A289EC017428F25DAE`, ubicación al inicio y exactamente una coincidencia por línea. Sin cambio funcional adicional, Cargo, build ni test de fuente. Preflight congelado en `C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55`. Principal ejecutará plan4 tras actualizar la copia.

Resultado plan3-fix: exclusivamente líneas 1 y 2 de `world_guard.rs`, `//!` → `//`; ambos textos conservados. Hash previo y posición/unicidad de anclas verificados antes de la escritura. SHA256 final world_guard `4F3AD21B863141D206051C980BEAF8CD594A3606A85B22EC68031A0124EFCEEC`; preflight permanece `C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55`. No Cargo/build ni prueba de texto; solo lectura final y hashes. El fallo plan3 sigue siendo fallo observado, no se afirma resolución SDK hasta que principal ejecute plan4 sobre copia actualizada. Fuente congelada y propiedad temporal cerrada.
