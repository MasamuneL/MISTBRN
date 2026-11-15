# P2 — diagnóstico identidad backend/snapshot

Fecha 2026-11-14 America/Mexico_City. Agente /root/backend_quarantine.
Estado: investigación en progreso; fuentes y datos congelados.
Intención previa: diagnosticar PlayerMismatch del reopen n0-base2 mediante SDK exacto, logs n0-base1/n0-base2 y snapshots offline del mundo dedicado/backup inmutable. Sólo se modifica este informe; scripts temporales únicamente en target; sin Cargo/build/juego/commits/subagentes, sin modificar guard/backend/datos.
Hipótesis abiertas: representación UUID, transferencia owner entre perfiles y selección del guard. Root revisa guard en paralelo; esta tarea sigue login y snapshots Host.

## Dictamen causal

El rechazo PlayerMismatch corresponde a una inconsistencia real del snapshot persistido por el SDK del owner integrado, no a endian ni a selección errónea demostrada del guard. En el archivo a01e3843-e521-3998-958a-f459800e4d11.dat (y dat_old), UUID NBT=a01e3843e5213998958af459800e4d11 pero payload SavedPlayer.state.player_id=18ebd4a646ae3c618bbe17928f9084b6. El archivo 18ebd4a6-46ae-3c61-8bbe-17928f9084b6.dat (y dat_old) tiene UUID y payload ambos 18ebd4a646ae3c618bbe17928f9084b6. level.dat y level.dat_old nombran a01e… como singleplayer_uuid y no contienen Data.Player.

La combinación procede de dos identidades que Horizon mantiene para un owner integrado: el guest de la conexión tiene offline_uuid del nombre de perfil, mientras Level conserva su player_uuid previo. La ruta save_owner conserva player_uuid ya existente mediante get_or_insert, copia guest.saved (incluyendo NeoForgeData) a Level, escribe el player_tag de Level bajo ese UUID conservado y además escribe player_tag del guest bajo el UUID de conexión. Eso produce exactamente los dos archivos observados: uno coherente con el UUID del guest y otro con UUID NBT del Level más namespace del guest. El backend inicializa SavedPlayer para el UUID de conexión validado por Host; no controla la segunda serialización del SDK.

## Evidencia observada y secuencia

1. n0-base1/stderr.log:13–14 acepta preflight; :18 WorldOpenReady; :21–23 login, identidad Primary, ready accepted_in_memory; :24 logged_out; :25–28 save_observed; :29 WorldCloseFinal. stdout.log:1 identifica Player86652. smoke.json tiene error=null. El fallo de sharing del harness está en result.json y no invalida esas señales nativas; tampoco convierte el ensayo completo en gate aprobado.
2. n0-base2/stderr.log:13 reject; :16 WorldOpenFailed; :17–18 PlayerMismatch; no login/ready. smoke.json tiene el mismo error y player=null. Por tanto el snapshot inconsistente ya existía antes del segundo login.
3. Lectura offline Node stdlib de NBT gzip de mundo actual y backup inmutable posterior al primer open, ejecutada dos veces sin escribir datos: los seis archivos auditados de cada uno coinciden por SHA256. No hay Data.Player legacy en ninguno de los level snapshots; no storage_fault en ninguno de los cuatro player snapshots. Todos los payloads presentes tienen world_id=58f1d6d013a548e3be20db72d476dd7f, igual al marker. El problema es identidad de jugador, no de mundo.
4. Cálculo observacional con el algoritmo exacto SDK (MD5 OfflinePlayer:<name>, versión 3, variante IETF, sin Host falso): offline_uuid(Player)=a01e3843e5213998958af459800e4d11; offline_uuid(Player86652)=18ebd4a646ae3c618bbe17928f9084b6. Coincide con el perfil del primer log y con el UUID por defecto que ensure_player_uuid crea para Player. No se trazó en ejecución la primera llamada que insertó ese valor por defecto; la atribución de esa inserción concreta es inferencia de fuente+valor, no breakpoint capturado.

## Trazado del SDK exacto y backend

SDK raíz: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.

- horizon-protocol/src/crypto.rs:162–180: offline_uuid termina en u128::from_be_bytes; uuid_string formatea u128 como 32 hex y añade guiones. horizon-client-core/src/uuid.rs:3–13 concatena cada IntArray word como u32 de ocho hex en el mismo orden.
- horizon-client/src/lan/playerdata.rs:37–49: uuid_ints usa shifts 96,64,32,0; ints_uuid hace fold acc<<32 | word as u32. allomancy-horizon/src/save.rs:49–50 y :118–128 usan exactamente esa representación. No existe inversión de endian entre estas funciones.
- horizon-client/src/game/native_player_state.rs:110–136 encuentra guest por g.uuid y devuelve playerdata::player_tag(guest). playerdata.rs:221–227 construye un Level temporal con player_uuid=uuid_ints(guest.uuid). Por eso el snapshot bruto visto por backend puede validar correctamente UUID=18eb… aunque la serialización posterior del owner a otro Level produzca a01e….
- save.rs backend:355–390 hace inspección raw para el UUID de login; namespace ausente crea SavedPlayer::new(player_id(uuid)); setter inline :387 sustituye NeoForgeData del guest correcto y comprueba readback. native_player_state.rs:398–410 escribe guest.saved fuera de acting; no el UUID global del Level. Este recorrido permite el ready observado y no acredita coherencia del segundo snapshot que save_owner fabricará después.
- horizon-client/src/game/integrated.rs:415 fija launch.owner=offline_uuid(player); :813 y :860–863 fijan Owner.uuid a ese owner. game/lan.rs:2856–2857 carga owner mediante load_owner y :1556–1557 guarda mediante save_owner. load_owner playerdata.rs:84–95 puede aplicar el Level de la identidad singleplayer_uuid al guest owner actual.
- playerdata.rs:101–110 save_owner usa level.player_uuid.get_or_insert(uuid_ints(guest.uuid)), después store, level.write_player y save guest. store :435 copia level.player_extra=guest.saved.clone() sin reemplazar level.player_uuid.
- horizon-client/src/save.rs:3790–3799 ensure_player_uuid conserva un Some anterior y crea offline_uuid(Player) si falta. :4096 clona player_extra, :4169–4172 reescribe UUID desde Level.player_uuid; :4334–4341 nombra el archivo con ese mismo UUID; :4468 lo publica como singleplayer_uuid. Es el punto exacto en que namespace y UUID del snapshot pueden quedar separados.
- allomancy-core/src/persistence.rs:185–186 rechaza saved.state.player_id distinto del esperado. Para el archivo a01e… ese error es correcto con los bytes observados. La revisión de selección owner del guard queda a /root, en paralelo; este informe no modifica ni debilita su contrato.

## IDs y hashes de los snapshots

Las filas siguientes son del backup inmutable; las seis filas equivalentes del mundo actual tienen exactamente el mismo hash. player_id se muestra sin guiones, como lo serializa core.

| Archivo | UUID NBT o singleplayer_uuid | payload player_id | SHA256 |
| --- | --- | --- | --- |
| level.dat | a01e3843e5213998958af459800e4d11 | — | DB9EA9625219933596C931BFE278A45489ED8C3798C034722DD63B7FC0A3148F |
| level.dat_old | a01e3843e5213998958af459800e4d11 | — | 39BD7F53DF78BAA068002FC8A112D8509987E14BC5A2DB9B146F4C753DD07832 |
| players/data/18ebd4a6-46ae-3c61-8bbe-17928f9084b6.dat | 18ebd4a646ae3c618bbe17928f9084b6 | 18ebd4a646ae3c618bbe17928f9084b6 | 09449A9577E0457DC08E7331AF5AE9FA687EBABE4C217F963A4D09FDEDCF7822 |
| players/data/18ebd4a6-46ae-3c61-8bbe-17928f9084b6.dat_old | 18ebd4a646ae3c618bbe17928f9084b6 | 18ebd4a646ae3c618bbe17928f9084b6 | 09449A9577E0457DC08E7331AF5AE9FA687EBABE4C217F963A4D09FDEDCF7822 |
| players/data/a01e3843-e521-3998-958a-f459800e4d11.dat | a01e3843e5213998958af459800e4d11 | 18ebd4a646ae3c618bbe17928f9084b6 | A7365A5D11CE8738B9768E7AC204CF2F2AFD5566C8111971870651EAE1813450 |
| players/data/a01e3843-e521-3998-958a-f459800e4d11.dat_old | a01e3843e5213998958af459800e4d11 | 18ebd4a646ae3c618bbe17928f9084b6 | A7365A5D11CE8738B9768E7AC204CF2F2AFD5566C8111971870651EAE1813450 |

Hashes logs: n0-base1/stderr=73BD2FA91BFBF13E41C0609C02A398EA643DF1B367FF3831073052FF715D1754; n0-base1/stdout=AFDCA654D589CA7FAF558FECCA95450BA2B9C12D1E08E9ACD14034E7394EE91C; n0-base2/stderr=623086602C2D294F5047E20F44EF809A27C2C1F4D5834EA9895BC74BC00973B3; stdout n0-base2 vacío SHA256=E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855.

Script temporal diagnóstico target/p2-native-identity-inspect.cjs SHA256=563885696733D974A4FEED6F535D39B1B894C2547AB050A36E523230D671A7B3. Sólo usa fs/path/zlib/crypto de Node; auditoría estructural con bounds, tipos NBT, profundidad, duplicados y fin exacto; imprime únicamente IDs/metadatos/hashes del mundo dedicado. No valida core ni reemplaza el guard. Fuente save.rs sigue congelada SHA256=00043AF15B0516BBA1CD7EB7E30568D95C031C932FF695F7304C3019866E3BD0.

## Implicación y límite del diagnóstico

La compatibilidad persistente del owner integrado queda bloqueada: aceptar el primer login/compilar no acredita reopen. La resolución necesita una regla explícita para conservar identidad económica cuando SDK duplica el owner entre UUID de Level y UUID de conexión. No hay autorización en este diagnóstico para corregir SDK, adoptar el payload como otra identidad, omitir el snapshot inconsistente o reparar datos. No se propone un cambio de endian. Conservar originales y backup para el ruling; no quitar namespaces/marca ni restaurar sólo fragmentos para hacer pasar el gate.

Estado final del diagnóstico: causa SDK localizada con fuente exacta y bytes reales; no se capturó breakpoint de inserción inicial de Player ni se ensayó corrección. Sin Cargo/build/juego/commits/subagentes. Sólo informe y script temporal target; guard/backend y datos intactos.
