# Guardado — contrato del núcleo (P2-core)

Fuente: `mod/src/core/src/persistence.rs`. Sustituye al codec JSON + ledger de la P2B legacy.

## Unidad guardada

`SavedPlayer` (campos privados, getters): `state: PlayerState`, `accounting: ReserveAccounting`, `rules: ReserveRules`, `revision: u64`, `last_transaction: u64`. Una unidad = un jugador. `WorldId` es `local` en el adaptador: el aislamiento entre mundos lo dan los ficheros de jugador de cada mundo. `last_tick` no se guarda (los ticks del servidor reinician al abrir el mundo); un jugador cargado empieza con `None`.

Invariantes (`validate_save`): ids no vacíos, ≤ 256 bytes, sin caracteres de control; `last_transaction ≤ revision`; `validate_reserves` completo. El formato concreto de los ids (UUID hex de 32 caracteres, mundo `local`) lo impone el adaptador; el núcleo exige además que el decode coincida exactamente con la identidad esperada.

## Codec, esquema 2 (P5)

```text
"MBRN" | schema:u16 | payload_len:u32 | payload | crc32:u32   (LE; CRC-32/IEEE de todo lo anterior)
payload = player_id:str | world_id:str | selected_pair:u8
        | 8 × (unlocked:u8 | reserve_milli:u64 | burn:u8) | 8 × remainder:u16
        | 8 × consumo:u64 | 8 × capacidad_base:u64 | reducción:u16 | bonus:u16
        | revision:u64 | last_transaction:u64
str = len:u16 | UTF-8
esquema 2 añade al payload del 1:
        | 4 × pair_xp:u32 | n_nodes:u8 | n × (code:u16 | rank:u8)
        | n_research:u8 | n × (code:u16 | evidence:u8 | hints:u8 | concluded:u8)
        | 3 × (window_start:u64 | window_count:u8)
```

Lo gastado, el conocimiento y el saldo no se guardan: se derivan de los nodos y de las investigaciones concluidas (`progress.rs`). Las listas van en orden estricto de código.

Enums: pareja 0..3 en orden `Pair`, burn 0 Off / 1 Burning / 2 Flared, unlocked 0/1. Máximo 64 KiB.

`decode_save(bytes, world, player)` rechaza, en este orden: tamaño (`TooLarge`), `Truncated`/`BadMagic`, `UnsupportedVersion(schema)` para cualquier esquema fuera de 1..=2 (antes de leer el payload), longitud declarada (`Truncated`/`TrailingBytes`), `ChecksumMismatch`; después, al leer los campos en orden: ids con UTF-8 inválido (`InvalidIdentity`), enums desconocidos (`InvalidEnum`), campos que no caben en el payload (`Truncated`) o payload más largo que los campos (`LengthMismatch`); por último invariantes (también las del progreso, `Progress(..)`: códigos, rangos, requisitos, gastado ≤ ganado, evidencias y pistas, y reglas = nodos), `WorldMismatch`, `PlayerMismatch`. Nunca repara ni pone valores por defecto: ante error el adaptador bloquea al jugador y no reescribe los bytes. `save_schema(bytes)` lee solo el esquema para diagnóstico.

Migraciones: el esquema 1 se lee y se migra a un progreso vacío con todo lo demás idéntico; la siguiente escritura guarda el 2. Fixtures versionados: `tests/fixtures/saved-player-v1.bin` (300 bytes; `schema_1_fixture_migrates_to_schema_2_keeping_every_field`) y `saved-player-v2.bin` (356 bytes, con progreso en todos los campos), que el esquema siguiente debe cargar.

## Progreso (P5)

`observe(&mut saved, &VerifiedEvent, game_time)` registra práctica o evidencia (sin revisión si no añade nada). `prepare_vial`, `prepare_purchase(code, rank)`, `prepare_conclude(code)` y `prepare_hint(code, index)` devuelven un `PreparedCredit` que se confirma con `commit_credit`, como un crédito. La muerte y la reconexión no tocan el progreso.

## Transacciones exactamente-una-vez

1. `prepare_credit(&saved, metal, amount) -> PreparedCredit`: `seq = last_transaction + 1`, base = `revision` actual; calcula el `candidate()` sin tocar `saved`.
2. El adaptador codifica el candidato y lo escribe junto con el cambio de inventario en un solo `host.transaction`.
3. Solo si ese commit del juego tiene éxito: `commit_credit(&mut saved, prepared)`. Rechaza `Duplicate` si `seq ≤ last_transaction` y `StalePreparation` si la revisión o la identidad cambió. En ambos casos `saved` no cambia.

`Duplicate` significa **este crédito no se aplicó** (otra transacción ya ocupa esa secuencia); nunca debe tratarse como éxito idempotente.

Contrato del adaptador: los pasos 1–3 son una sola operación sin ningún otro cambio del mismo jugador intercalado (mismo callback de servidor, sin tick ni `update_player` entre medias). Si el juego confirmó la transacción y aun así `commit_credit` falla, el disco ya tiene el candidato: el adaptador debe dejar de escribir a ese jugador (bloquearlo) y nunca seguir con el estado anterior, que borraría el crédito en la siguiente escritura.

Retención O(1): no se guarda una lista de ids; cualquier transacción con secuencia ya confirmada se rechaza siempre. Corte antes del guardado del juego → el fichero viejo no tiene el crédito ni la secuencia; corte después → el nuevo tiene ambos una sola vez (probado en `cut_before_or_after_commit_yields_old_or_new_never_both`).

## Ciclo de vida

`on_death`: limpia reservas/restos/combustiones, conserva desbloqueos y contadores. `on_reconnect`: apaga combustiones, conserva reservas y restos. `update_player` y `replace_rules` validan el resultado completo y no dejan cambios parciales. Toda modificación aceptada incrementa `revision`.
