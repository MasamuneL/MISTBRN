# Persistencia P2B — contrato implementado del núcleo

Fecha: 2026-11-13. Contrato propio del proyecto, aprobado en su forma por jugador por el principal y congelado antes de implementar. Núcleo implementado y verificado con 24 pruebas de persistencia; integración/revisión quedan pendientes. Consume [contratos P1](contracts.md), [reservas P2A](reserves.md) y [análisis P0 del backend](execution/evidence/p0-persistence-api.md). Evidencia y límites en [entrega P2B](execution/evidence/p2-persistence.md).

## Alcance y representación

Una unidad canónica contiene **un jugador en un mundo estable**. No se modifica la estructura de `PlayerState` P1 ni se introduce un agregado `WorldSave` que sugiera commit entre jugadores. Las firmas `encode_save/decode_save(&WorldState)` de P1 eran reservas no exportadas: P2 las concreta para `SavedPlayer`.

`SavedPlayer` tiene `Clone, Debug, PartialEq, Eq, Serialize, Deserialize` y campos privados:

| Campo | Tipo / significado |
|---|---|
| `state` | `PlayerState` P1 completo |
| `accounting` | `ReserveAccounting`, guardado junto a reservas |
| `rules` | `ReserveRules`, parámetros explícitos que explican el saldo y los restos |
| `revision` | `u64`, comienza en 0, nunca se resta ni se reinicia al cargar |
| `ledger` | Colección ordenada por `TransactionId`, sin expulsión; representación wire descrita abajo |

Los getters son inmutables. Constructor confiable `SavedPlayer::new(PlayerId, WorldId, ReserveRules) -> Result<Self, SaveError>` crea `PlayerState::new`, contabilidad cero, revisión 0 y ledger vacío. Solo se usa para una ausencia real y validada del namespace; nunca para recuperarse silenciosamente de errores de decode. No convierte texto cliente en identidad autenticada.

Formato producción 1: JSON UTF-8, envelope exacto `{ "schema_version": 1, "saved_player": { ... } }`. `saved_player` contiene exactamente los cinco campos anteriores. `state`, `accounting` y `rules` conservan sus nombres serde P1/P2A; arrays tienen exactamente ocho elementos, orden `Metal::ALL`. En wire `ledger` es un array de entradas `{ "transaction": TransactionId, "payload": { "metal": Metal, "amount_milli": i64 }, "committed_revision": u64 }`, ordenado estrictamente por ID. Así se rechazan IDs duplicados sin la ambigüedad de deserializar claves repetidas de un mapa. En memoria se usa el mismo `Vec<LedgerEntry>` ordenado, sin segundo ledger autoritativo.

`CreditPayload { metal:Metal, amount_milli:i64 }` y `CreditReceipt { payload:CreditPayload, committed_revision:u64 }` son datos tipados con igualdad exacta. El ID está en la clave del ledger. No usar hash de JSON, reloj ni RNG para determinar igualdad o decisiones.

No se inventan campos de investigación/XP/hitos/nodos P3/P5 ni un JSON opaco para ellos: P1 aún no los posee. Muerte/reconexión preservan todos los campos persistentes que sí existen, incluidos desbloqueos y ledger. Al incorporarse conocimiento/nodos, la ampliación tendrá esquema y migración explícitos; no podrá borrarlos por reconstruir `PlayerState::new`.

## API del núcleo

```rust
encode_save(&SavedPlayer) -> Result<Vec<u8>, SaveError>
decode_save(bytes: &[u8], expected_world: &WorldId, expected_player: &PlayerId)
    -> Result<SavedPlayer, SaveError>
validate_save(&SavedPlayer) -> Result<(), SaveError>

update_player<F>(&mut SavedPlayer, update: F) -> Result<(), SaveError>
where F: FnOnce(&mut PlayerState, &mut ReserveAccounting) -> Result<(), ReserveError>
replace_rules(&mut SavedPlayer, ReserveRules) -> Result<(), SaveError>
on_death(&mut SavedPlayer) -> Result<(), SaveError>
on_reconnect(&mut SavedPlayer) -> Result<(), SaveError>

prepare_credit(&SavedPlayer, TransactionId, Metal, amount_milli: i64)
    -> Result<CreditPreparation, SaveError>
commit_credit(&mut SavedPlayer, PreparedCredit) -> Result<CreditReceipt, SaveError>

migrate_fixture_v0(&[u8], &WorldId, &PlayerId) -> Result<SavedPlayer, SaveError>
write_export_atomic(&Path, &SavedPlayer) -> Result<(), SaveError>
read_export_with_backup(&Path, &WorldId, &PlayerId)
    -> Result<RecoveredExport, SaveError>
```

`CreditPreparation` es `Replay(CreditReceipt)` o `Pending(PreparedCredit)`. `PreparedCredit` tiene campos privados, sin Serialize/Deserialize ni constructor público; ofrece `candidate() -> &SavedPlayer`, `receipt() -> &CreditReceipt`, `is_current(&SavedPlayer) -> bool`. Retiene el valor previo completo y el candidato. No es un token cliente ni recibo del Host. Helpers, mutadores y constructores son para lógica confiable; no aceptan un campo `Authority`, un `VerifiedEvent` deserializado ni evidencia cliente.

`SaveError` distinguirá al menos: `InputTooLarge`, `InvalidJson`, `UnsupportedVersion(u64)`, `InvalidSchema`, `InvalidIdentity`, `WorldMismatch`, `PlayerMismatch`, `InvalidReserves(ReserveError)`, `InvalidLedger`, `NegativeCredit`, `TransactionConflict`, `LedgerFull`, `RevisionOverflow`, `StalePreparation`, `Io`. El detalle interno de serde/IO puede acompañar la categoría; ningún error provoca writes o reinicialización. Categorías de parseo no prometen detectar modificación maliciosa que siga siendo un estado válido: JSON no lleva autenticación/checksum.

## Validación íntegra y límites del prototipo

1. Antes de parsear, rechazar más de `MAX_SAVE_BYTES = 4 * 1024 * 1024` bytes. Encoder también rechaza una salida superior. Solo un documento JSON completo; rechazar truncado, tokens extra y números fuera de sus tipos enteros. No admitir flotantes para cantidades.
2. Hacer primero un sondeo tipado únicamente de `schema_version`, que rechace su duplicación; su entero debe ser no negativo y representable en `u64`. Cualquier versión distinta de 1 produce `UnsupportedVersion` **antes** de intentar el DTO actual. Después deserializar envelope completo y estricto. No interpretar un esquema futuro con defaults actuales.
3. Rechazar campos ausentes, desconocidos o duplicados en todos los objetos: envelope, SavedPlayer, PlayerState, MetalState, ReserveAccounting, ReserveRules, entrada de ledger y payload. Variantes desconocidas y arrays de longitud incorrecta fallan. Los structs deben ser objetos JSON, no arrays posicionales aceptados por serde. Tras el sondeo se comprueba esa forma sobre Value acotado; luego el decode tipado original vuelve a detectar duplicados. Se usa `serde(deny_unknown_fields)` en las estructuras existentes P1/P2A y tipos nuevos, sin espejos de estado.
4. WorldId/PlayerId/TransactionId: no vacíos, máximo 256 bytes UTF-8 cada uno, sin caracteres de control. Son opacos: no normalizar, recortar, cambiar mayúsculas ni asignar nueva identidad durante carga. `decode_save` exige igualdad exacta de mundo/jugador con los valores confiables esperados. `encode_save` valida identidad también. Epoch de sesión P1 no es una identidad persistente válida para el adaptador.
5. Llamar a `validate_reserves(state, accounting, rules)` completo: reglas positivas y acotadas, capacidad/overflow, quemado/desbloqueo, reserva/resto y fracciones 0..1000. Guardar reglas no autoriza elegir balance desde un cliente. El adaptador decide si adopta las guardadas o usa `replace_rules` validado; no debe reinterpretar silenciosamente reservas con otras reglas ni truncarlas al bajar capacidad.
6. `MAX_LEDGER_ENTRIES = 4096`; rechazo de documentos con más entradas. IDs estrictamente ordenados y únicos, payload de crédito no negativo, recibos con revisión entre 1 y la revisión actual, revisiones de recibos distintas. Pueden tener huecos por consumo/ciclo de vida. Un ledger no vacío con revisión 0 es inválido. No intentar reconstruir saldos sumando recibos: puede haber consumo/muerte posteriores.

Los límites son decisiones del prototipo, no balance. Retener 4096 recibos cuesta O(n) espacio/serialización; clones/candidatos e inserción en Vec cuestan O(n), lookup O(log n), validación íntegra O(n log n). El decode hace sondeo, comprobación de forma y deserialización tipada sobre entrada acotada. Al llenarse se rechazan **nuevas** transacciones económicas; replays válidos siguen funcionando. No LRU, TTL, recorte, limpieza en muerte ni reemplazo automático. Aumentar el límite exige revisar límites de bytes/almacenamiento y pruebas; nunca habilitar replays olvidando IDs viejos.

Decode construye/valida un valor separado y no recibe `&mut` al estado live. Encode valida antes de producir bytes. Cualquier fallo deja intactos bytes originales, estado live, inventario y ledger. Ausente se distingue de corrupto; no hay autosave correctivo tras error.

## Revisión monotónica y mutación

Todo cambio a un SavedPlayer instalado pasa por sus mutadores; los campos privados impiden mutaciones externas que eludan revisión. `update_player` clona estado/contabilidad, ejecuta la closure confiable, exige identidad original, valida candidato completo y suma 1 a revisión con `checked_add`; solo entonces reemplaza el valor. `replace_rules` hace lo mismo con reglas. Error de closure, validación u overflow conserva el original íntegro. Una mutación exitosa incrementa revisión **incluso si el valor final de los campos resulta igual**. Consumo de cero ticks, apagado ya apagado o una secuencia A→B→A no reviven preparaciones previas. No incrementar por lectura, encode/decode, replay o rechazo.

`consume_reserves` se invoca dentro de `update_player`, con ticks transcurridos calculados por adaptador confiable; P2A no modifica `last_tick`. El integrador actualiza el ancla transitoria dentro de esa misma closure, evita cobrar ticks duplicados y no cobra tiempo offline. Mientras exista un PreparedCredit no se permite otra mutación entre la comprobación final y commit; no sostenerlo a través de await/reentrada.

`on_death` usa `clear_reserves_on_death`: cero reservas/restos, ocho combustiones Off; conserva identidad, pareja, desbloqueos, reglas y ledger. Puede limpiar reservas inconsistentes al preparar el candidato, pero un guardado corrupto no se instala primero para repararlo. Reinicia `last_tick = None` y suma revisión. `on_reconnect` apaga los ocho metales, pone `last_tick = None`, conserva reservas y **restos existentes**, desbloqueos, pareja, reglas y ledger; valida y suma revisión. No recrear accounting con Default al cargar. Decode no aplica automáticamente estas transformaciones: round-trip preserva el snapshot; el adaptador invoca reconexión antes de habilitar gameplay.

## Ledger y frontera prepare/commit

Orden de `prepare_credit`: validar SavedPlayer e ID; rechazar cuantía negativa; buscar ID. Mismo ID y payload exacto devuelve el recibo histórico `Replay`, sin recobrar inventario, acreditar, incrementar revisión ni exigir hueco nuevo. Mismo ID con otro metal/cuantía devuelve `TransactionConflict`, íntegro. Para ID nuevo comprobar límite/revisión, clonar el estado y llamar a `credit_reserve`, sumar revisión e insertar recibo en **ese mismo candidato**. Crédito cero es válido pero ocupa ID/revisión una vez. Capacidad/overflow/ledger lleno rechazan sin aceptar parcialmente.

El payload del ledger es solo el crédito tipado del núcleo. La resolución de slot, receta, stack esperado y cuantía viene del servidor; el adaptador debe vincular una intención estable y su ID a esa resolución. No generar un ID nuevo al reintentar una operación incierta. Si reutiliza un ID para otro slot que resuelve al mismo crédito, se trata de replay y no consume ese nuevo slot. Comprar/reclamar aún no son operaciones soportadas por este ledger; al ampliarlas se necesita un payload discriminado y revisión de esquema. Un EventId enviado por cliente no otorga recompensas.

Protocolo del adaptador:

1. Validar autoridad/contexto/identidad y observar slot/stack exacto fuera de acting. Si es replay, devolver el resultado anterior sin tocar inventario ni premio.
2. Para Pending, comprobar `is_current` inmediatamente antes de escribir. Codificar `candidate()` completo: incluye saldo, accounting, reglas, **revisión nueva y ledger nuevo**. Preparar merge namespaced y copias de rollback antes de mutar el Host.
3. En un callback servidor síncrono sin await, aplicar inventario exacto y el estado inline completo al Host. No publicar éxito/premio/efectos posteriores todavía. Ante fallo, deshacer todas las aplicaciones parciales y descartar PreparedCredit; el SavedPlayer previo no consumió ID. Si el rollback falla, bloquear operaciones y conservar pendiente/error explícito; no volver a consumir ni otorgar por una repetición ciega.
4. Solo tras aceptar ambas aplicaciones llamar a `commit_credit`, que vuelve a exigir igualdad íntegra con el valor previo, incluida revisión, y reemplaza con el candidato ya entregado al Host. Si falla por obsolescencia inesperada, restaurar Host o bloquear según el punto anterior; no presentar éxito. Un preparado antiguo falla aunque los campos económicos vuelvan a coincidir (ABA).
5. A partir de ese punto puede confirmarse aceptación en memoria. `commit_credit`, setter bool y callback de guardado **no son recibos durables**. Un reinicio restaura el último snapshot completo aceptado por Horizon; el ledger solo existe en snapshots donde también existen saldo e inventario correspondientes.

La API pura no prueba observación del Host ni impide que código confiable llame incorrectamente a commit. Es una frontera explícita para integración/test real, no una transacción distribuida. Simulaciones de caída entre prepare/commit comprueban snapshots completos viejos/nuevos y replay; no acreditan por sí solas resistencia a cortes del proceso real.

## Versiones antiguas y utilidades de archivo

P1 no tenía guardado desplegado. Producción `decode_save` acepta exclusivamente versión 1; no simula soporte de un formato histórico de Horizon. La prueba de migración usa una **fixture artificial v0, offline y nunca desplegada**: envelope `{schema_version:0,state:PlayerState,rules:ReserveRules}`, sin contabilidad ni ledger. `migrate_fixture_v0` es utilidad de test/export separada del decode de producción: acepta únicamente reservas cero, combustión Off, last_tick None y reglas/identidad válidas; crea revisión 0, restos cero y ledger vacío. Rechaza fixture con combustible o indicios de economía previa porque no puede inventar restos ni IDs perdidos. Devuelve v1 en memoria sin escribir el original.

El writer de exportación offline, separado del backend live, encode/valida antes de abrir destino; valida principal y backup existentes e impide bajar revisión o reemplazar un futuro/corrupto. Exige acceso exclusivo al archivo. Escribe `.tmp` con create_new, escribe el anterior válido a `.bak.tmp`, renombra a `.bak` y sustituye principal mediante rename en el mismo directorio. Nunca abre/trunca el principal antes de sustituirlo. Temporales preexistentes se conservan/rechazan para recuperación manual, sin promoverlos. Errores controlados limpian solo temporales que creó esta llamada.

El lector de recuperación no escribe y devuelve `RecoveredExport { saved:SavedPlayer, source:ExportSource }`, donde source es Primary o Backup; una identidad distinta o futuro principal produce error sin fallback. Puede recuperar backup ante principal ausente/corrupto, comunicándolo. Se probaron reemplazo real Windows, rename denegado por un handle sin FILE_SHARE_DELETE, destinos bloqueados, temporal parcial, backup inválido/futuro y principal ausente. No se simularon power loss ni terminación real en cada instrucción. No se llama a fsync; atomicidad del rename frente a los fallos ensayados no acredita durabilidad. No es sidecar económico, ni escribe `.dat` live, ni sustituye el writer de Horizon.

## Backend real y gates pendientes

Canon: subcompound namespaced/versionado en `NeoForgeData` del jugador, usando helpers del pin Horizon verificados en P0. Antes de consumo/writes, leer `player_compound` **fuera de acting** y distinguir NeoForgeData ausente de raíz existente no-Compound; el getter normalizado no basta. Rechazar raíz/namespace inválidos o futuros intactos. Leer/mezclar compound completo preservando otras claves; si se transporta el JSON como bytes/string dentro del namespace, documentar esa forma NBT concreta en `horizon/save.rs` antes de habilitarla.

WorldId se resuelve a una identidad estable por mundo; PlayerId viene de la identidad Host. `world_dir()` readonly no decide por sí solo esa identidad. Resolver y probar ambas antes de persistencia live: no guardar epoch, ruta efímera ni nuevo ID en cada carga. Hasta ese gate, el codec puro solo compara IDs suministrados.

No se acreditan todavía invocación inline, coherencia de autosave/inventario, crash tests nativos, fsync/pérdida de energía ni atomicidad jugador+chunk/entidad/otro jugador. No usar attachments como segunda autoridad, sidecar económico, callback SaveToFile del estado actual ni escritura manual de `.dat`. Beber/recompensas gameplay siguen cerrados hasta resolver los gates reales del adaptador.

## Verificación del núcleo y relevo al adaptador

TDD conductual en `tests/persistence.rs`: RED inicial 20/20, RED adicional de dos guardas, GREEN final 24/24 y workspace/formato aprobados. Las últimas dos pruebas amplían verificación de ramas de filesystem existentes. Un solo Cargo coordinado por principal; evidencia exacta en el informe P2B. Comportamientos cubiertos:

- Round-trip no trivial con varios metales, combustiones, tick, pareja, restos no cero, reglas y ledger; bytes deterministas por orden y mismos datos.
- Rechazar truncado, basura final, campos ausentes/desconocidos/duplicados anidados, enum inválido, arrays incorrectos, cantidad negativa o fuera de rango, exceso de bytes, IDs inválidos y mundo/jugador distinto; original/snapshot intactos.
- Futuro válido con forma incompatible devuelve UnsupportedVersion antes del DTO; versión 0 producción rechazada. Fixture v0 segura migra a v1; fixture con combustible o datos desconocidos se rechaza sin overwrite.
- Reglas mal acotadas, capacidad excedida, resto inválido/vacío, metal bloqueado/quemando y overflow; validar encode tanto como decode.
- Primer crédito, replay mismo ID tras encode/decode, payload cambiado, ID distinto, crédito cero, negativo/capacidad/overflow sin mutación; ledger lleno retiene replay antiguo y bloquea nuevo.
- Recibos duplicados, desordenados, revisiones cero/futuras/duplicadas; revisión cerca de u64::MAX bloquea mutadores/crédito íntegros.
- Prepare no modifica original; candidato ya contiene saldo/resto/revisión/ledger; descartar por fallo Host no consume ID. Commit obsoleto después de consumo/muerte/reconexión y de A→B→A/no-op falla; commit normal aplica exactamente una vez.
- Muerte mantiene desbloqueos/ledger y vacía reservas/restos; reconexión apaga/resetea tick conservando fracciones y saldo. Repetir ciclo de vida nunca reduce revisión.
- Recuperación de export tras fallos antes/después de temporal/reemplazo: principal/backup anterior o nuevo completos, corrupción rechazada, futuro intacto y elección de backup explícita. Pruebas reales del filesystem temporal, sin SDK falso ni aserciones sobre texto fuente.

Luego principal integra/revisa y ejecuta pruebas reales Horizon del análisis P0: merge ajeno, raíz corrupta, slot cambiado, fallo/rollback, recarga/replay, autosave en vuelo y cortes. Tests unitarios no sustituyen esos ensayos.

## Cambios compartidos coordinados con el principal

- Módulo/reexports de persistence en lib.rs añadidos serialmente después de P2A.
- Principal movió `serde_json = "=1.0.151"` de dev-dependencies a dependencies de allomancy-core. Pruebas offline/locked aprobadas, Cargo.lock intacto; sin crates/features nuevos.
- Atributos `serde(deny_unknown_fields)` añadidos exclusivamente en PlayerState, MetalState, ReserveRules y ReserveAccounting, sin alterar campos/firma ni lógica P2A. Tipos nuevos estrictos desde origen. Actualizar contrato global solo por principal.
- Coordinar todas las mutaciones persistentes del adaptador mediante SavedPlayer antes de habilitar gameplay; P1 `apply_command`/`advance_tick` no se cambian en este encargo documental.
