# Guardado — contrato del núcleo (P2-core)

Fuente: `mod/src/core/src/persistence.rs`. Sustituye al codec JSON + ledger de la P2B legacy.

## Unidad guardada

`SavedPlayer` (campos privados, getters): `state: PlayerState`, `accounting: ReserveAccounting`, `rules: ReserveRules`, `revision: u64`, `last_transaction: u64`. Una unidad = un jugador. `WorldId` es `local` en el adaptador: el aislamiento entre mundos lo dan los ficheros de jugador de cada mundo. `last_tick` no se guarda (los ticks del servidor reinician al abrir el mundo); un jugador cargado empieza con `None`.

Invariantes (`validate_save`): ids no vacíos, ≤ 256 bytes, sin caracteres de control; `last_transaction ≤ revision`; `validate_reserves` completo.

## Codec, esquema 1

```text
"MBRN" | schema:u16 | payload_len:u32 | payload | crc32:u32   (LE; CRC-32/IEEE de todo lo anterior)
payload = player_id:str | world_id:str | selected_pair:u8
        | 8 × (unlocked:u8 | reserve_milli:u64 | burn:u8) | 8 × remainder:u16
        | 8 × consumo:u64 | 8 × capacidad_base:u64 | reducción:u16 | bonus:u16
        | revision:u64 | last_transaction:u64
str = len:u16 | UTF-8
```

Enums: pareja 0..3 en orden `Pair`, burn 0 Off / 1 Burning / 2 Flared, unlocked 0/1. Máximo 64 KiB.

`decode_save(bytes, world, player)` rechaza, en este orden: tamaño, `Truncated`/`BadMagic`, `UnsupportedVersion(schema)` para cualquier esquema ≠ 1 (antes de leer el payload), longitud (`Truncated`/`TrailingBytes`), `ChecksumMismatch`, enums desconocidos, invariantes, `WorldMismatch`, `PlayerMismatch`. Nunca repara ni pone valores por defecto: ante error el adaptador bloquea al jugador y no reescribe los bytes. `save_schema(bytes)` lee solo el esquema para diagnóstico.

Migraciones: el esquema 1 es el primero. `tests/fixtures/saved-player-v1.bin` (300 bytes) queda versionado; P5 crea el esquema 2 y debe cargar ese fichero.

## Transacciones exactamente-una-vez

1. `prepare_credit(&saved, metal, amount) -> PreparedCredit`: `seq = last_transaction + 1`, base = `revision` actual; calcula el `candidate()` sin tocar `saved`.
2. El adaptador codifica el candidato y lo escribe junto con el cambio de inventario en un solo `host.transaction`.
3. Solo si ese commit del juego tiene éxito: `commit_credit(&mut saved, prepared)`. Rechaza `Duplicate` si `seq ≤ last_transaction` y `StalePreparation` si la revisión cambió. En ambos casos `saved` no cambia.

Retención O(1): no se guarda una lista de ids; cualquier transacción con secuencia ya confirmada se rechaza siempre. Corte antes del guardado del juego → el fichero viejo no tiene el crédito ni la secuencia; corte después → el nuevo tiene ambos una sola vez (probado en `cut_before_or_after_commit_yields_old_or_new_never_both`).

## Ciclo de vida

`on_death`: limpia reservas/restos/combustiones, conserva desbloqueos y contadores. `on_reconnect`: apaga combustiones, conserva reservas y restos. `update_player` y `replace_rules` validan el resultado completo y no dejan cambios parciales. Toda modificación aceptada incrementa `revision`.
