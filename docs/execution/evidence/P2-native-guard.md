# P2 — evidencia propia de marker y guard nativo

Fecha: 2026-11-13, America/Mexico_City. Agente `/root/p2_native_guard`. Base asignada `fed744b9252e14ae6fcd12b314fba61040250fe0`; única raíz de trabajo `mistbrn-p1/MISTBRN`. Sin commits, subagentes ni cambios del SDK. Estado: marker verificado; fuentes del guard entregadas; compilación/juego/N0 pendientes del principal.

## Contrato y lecturas

API marker congelada y comunicada al principal antes de escribir fuentes: `MAX_WORLD_IDENTITY_BYTES=4096`, `WorldIdentitySource::{Primary,Backup}`, `LoadedWorldIdentity {world_id:WorldId,source}`, `WorldIdentityError::{Parse,Schema,Future(u64),InvalidId,Conflict,Io,TooLarge}`, `decode_world_identity(&[u8])`, `load_world_identity(&Path world_root) -> Result<Option<LoadedWorldIdentity>,...>`. Principal avisado de reenviar al consumidor backend cuando esté activo; no existía agente backend activo en el inventario observado. Sin contrato alternativo de defaults ni writes.

Se leyó completo `tasks/P2-native.md`, `p2-storage-architecture.md` (primer volcado truncado, rangos restantes releídos), `docs/persistence.md`, `docs/reserves.md`, persistence/types/lib del núcleo y fuentes originales del pin `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`: horizon-world `nbt.rs`/`compression.rs`, horizon-client `save.rs`, `game/world_loading.rs`, `lan/playerdata.rs`, horizon-client-core `uuid.rs`. SDK original del runtime leído, nunca escrito.

`world_guard.rs` reutiliza `Compression::Gzip.decode`, `nbt::{read_named,read_network,Limits,Tag,JavaString}` y `horizon_client::save::{uuid_from_text,uuid_text}`, todos públicos en ese pin. El auditor raw estructural usa SDK String para igualdad UTF-16; no replica modified-UTF8 ni codec SavedPlayer y no añade dependencias. Se avisó al principal del diseño acotado antes de escribir guard. No se afirma detectar duplicados por Tag map.

## TDD marker y comandos ejecutados

Primero se escribieron 16 tests conductuales de API+filesystem y un stub compilable con `decode -> Schema`, `load -> None`. RED real `cargo test --offline --locked -p allomancy-core --test world_identity`, exit101: 12 fallaron por comportamiento faltante, 4 pasaron accidentalmente contra el stub (rechazo Schema/ausencia). No hubo fallo de compilación como sustituto de RED. Log `p2-identity-red.log` conserva aserciones y resultado.

Luego implementación mínima con mapa que rechaza duplicados, sondeo de versión previo a v1 y lecturas acotadas. GREEN mismo comando, exit0: 16/16. `p2-identity-green.log`. Cobertura: UUID exacto, espacios/orden, límites 4096/4097, futuro antes de forma incompatible, campos ausentes/desconocidos/duplicados y nombres escapados, arrays/null/tipos erróneos, UTF-8/JSON/truncado/basura final; ambos ausentes, root ausente sin creación, principal solo, backup solo explícito sin rewrite, conflicto/pair inválido sin mutación, IO real de directorio en posición de principal/backup.

Después rustfmt modificó solo dos fuentes propias (`src/world_identity.rs`, `tests/world_identity.rs`). Workspace `cargo test --workspace --offline --locked`, exit0: contracts13 + persistence26 + reserves16 + world_identity16 = 71 tests; 1 doctest compile_fail PASS. Sin warnings observados. `p2-identity-workspace.log`. `cargo fmt --all -- --check`, exit0, sin salida: `p2-identity-fmt.log` vacío es el resultado esperado. No modifica archivos ajenos.

Toolchain original portable GNU 1.99.1; PATH tc/bin+runtime/mingw64/bin; CARGO_HOME runtime/launcher/cargo; RUSTC tc/bin/rustc.exe; RUSTDOC del devtools extraído y RUSTDOCFLAGS --sysroot tc; CARGO_INCREMENTAL=0. Todos los Cargo con cwd explícito de esta raíz y elevación de sandbox para caches autorizados, sin red (offline/locked), seriales. Principal concedió exclusividad; se liberó Cargo inmediatamente tras workspace/fmt antes de guard. No ejecuté SDK compile, launcher, planificador ni juego.

`rustfmt --edition 2024` y su `--check` se ejecutaron además solo sobre `integration/horizon/preflight.rs` y `world_guard.rs`. Eso comprueba parsing/formato, no tipos del SDK ni funcionamiento en juego.

## Fuentes congeladas entregadas al principal

SHA256 antes del relevo para que principal pueda planificar/compilar:

| Archivo | SHA256 |
|---|---|
| crates/allomancy-core/src/world_identity.rs | A0B1E9ABBCB9A26F528C723ACD578653AA014D6D676B1CFC1DE90DB213422B1A |
| crates/allomancy-core/tests/world_identity.rs | 04010345B62114274157847686DB76631ACD6028C240A5DA6C8A928F74886E5C |
| crates/allomancy-core/src/lib.rs | 028148DC1C80FBFD30689649C8DB27A29DABCAD7445051B73A58D4FD8525DE73 |
| integration/horizon/preflight.rs | 3B095A7009E46238CE34C9806999BD5035E6B2C52D0EFBCA4B46100AD3467D9A |
| integration/horizon/world_guard.rs | CD26849BDD037979AC2A49FBC46B6CF3E87586EBBEDC71B9D87F5929280B34D4 |

`lib.rs` solo recibe module/reexport. No cambios propios en persistence/state/reserves/types/manifests/locks/descriptores/buildscripts/SDK. Un único helper privado seguro síncrono en preflight; soporte pub `validate_world(&Path)->Result<(),String>`. Error a `Break(Err(reason.into()))` y Continue solo tras validación. Parent prepara [[crate]] core alias allomancy_core y [[module]] allomancy_world_guard; sin doble core ni SDK fork.

## Política de guard y límites entregados

Audita originales level.dat/old y todos players/data/dat/old antes de permitir prepare_world; legacy Data.Player y referencia owner. Detecta duplicados NBT con pase raw previo, luego decoder SDK con consumo total. Valida raíz/NeoForgeData/namespace/transport Int1 y ByteArray, codec exacto con expected WorldId/PlayerId. Principal semánticamente corrupto/futuro jamás cae a backup. Sin marker+sin namespace permite vanilla, sin habilitar economía; namespace sin marker bloquea. Backup de marker válido se reporta sin rewrite. Los mensajes del guard no imprimen datos privados.

Política conservadora deliberada y comunicada: cualquier backup .dat inválido también bloquea; backup-only pide restaurar un snapshot completo offline. No se acredita recuperación automática N7/N8. Límite64MiB archivo/descomprimido,1M nodos,depth128,4096 archivos jugador sumando principales/backups; límite menor al SDK puede rechazar un mundo válido grande. Snapshots regulares y directorio de jugadores regular; errores IO no se ignoran. No audita .dat_new/chunks/entidades ni excluye edición externa entre check y read del motor. No detecta un jugador totalmente desaparecido sin referencia/backup observable. No modifica Tags ni bytes; no writer, lock, repair o progreso económico sidecar.

No se añadió `allomancy:storage_fault`: principal comunicó que sigue evaluación de protocolo Fault; el contrato de namespace sano conserva exactamente dos campos.

## Pendiente y siguiente acción exacta

Principal: functional RED guard/inyección, descriptor y core único, plan aplicado+fingerprint, compilación SDK exacto, N0 integrado sin wrapper. Probar payload futuro/corrupto, NBT duplicado (incluida colisión modified-UTF8), raíz/NS inválidos, IO, backup-only, missing marker con namespace y fixture sana con namespace ajeno; observar error previo a body/finish_open_world y hashes originales intactos. Ningún test marker ni rustfmt demuestra esos resultados. N1–N9 y revisión independiente siguen pendientes; no se marca P2 completa.

Revisión previa al build detectó que Windows puede abrir .DAT como .dat. Se corrigió el reconocimiento de extensiones a ASCII case insensitive, manteniendo UUID stem canónico; se avisó al principal antes/después de la breve descongelación. Hash anterior world_guard B6B7D7110560452671B157C5180C98290E76CE73732BFE7B6DCE028BB2817D7D reemplazado por el vigente de la tabla. Formato/check propio volvió a exit0; N0 variante .DAT solicitada al principal. Sin otro Cargo.

## Relevo autorizado storage_fault y trazas N0

El contrato posterior de `CHECKPOINT.md:43–46` sustituye la nota histórica de protocolo pendiente en este informe: cualquier presencia de `NeoForgeData["allomancy:storage_fault"]` rechaza, incluso forma inválida/futura y state ausente. Forma canónica de escritura Compound `{format_version:Int(1),reason:String("rollback_unverified")}`; el guard lector no interpreta el valor.

Intención registrada atómicamente en la ficha antes de fuentes. Cambios exclusivos: cinco líneas añadidas en `world_guard.rs:168–172`, delante del lookup state, y expansión puntual de ambos brazos del match de `preflight.rs` para logs constantes accept/info y reject/warn. No se añade escritura, reparación, lock ni decode alternativo. Error original continúa a Break/Err; Continue solo tras Ok. Mensajes nuevos no incluyen rutas, UUID, payload o reason interpolado.

El helper `validate_player` ya es común a jugadores principales/backups y `Data.Player` legacy en ambos level; por ello la comprobación está en el único punto compartido. El lookup por presencia es independiente del tipo/contenido del Tag y precede al early return por state ausente. La detección estructural de NBT inválido/duplicado y otros rechazos previos permanecen intactos.

| Fuente | SHA256 antes | SHA256 después |
| --- | --- | --- |
| `integration/horizon/world_guard.rs` | `CD26849BDD037979AC2A49FBC46B6CF3E87586EBBEDC71B9D87F5929280B34D4` | `0B250DB096D8250B9D57E7BE569D605CAD6017398B4900A289EC017428F25DAE` |
| `integration/horizon/preflight.rs` | `3B095A7009E46238CE34C9806999BD5035E6B2C52D0EFBCA4B46100AD3467D9A` | `C0B99DDCC0FEAE322755757374F948441C54D6D0113CC2A755341FCBFE21DB55` |

No Cargo, build, juego, rustfmt, tests de texto ni Host falso en este relevo. Verificación estática y de edición únicamente: contenido leído, callsites compartidos, hashes originales exigidos y exactamente una coincidencia para cada reemplazo. No se afirma tipo SDK válido, funcionamiento en juego ni N0 aprobado. El principal conserva compilación y pruebas conductuales nativas con marca canónica, primitiva/malformed, futura, sin state, en backups/legacy y jugador; observar accept/reject antes de adopción y snapshots originales intactos. N1–N9 continúan pendientes.

Auto-review rechazó el primer comando antes de ejecutarlo por reconstruir todo preflight y riesgo WIP. No hubo mutaciones de fuentes en ese intento. Un segundo comando distinto y aprobado conservó el contenido completo y reemplazó exclusivamente las anclas únicas verificadas por hash. Las únicas escrituras adicionales fueron esta evidencia y la ficha mediante temporal/reemplazo atómico con .bak.
