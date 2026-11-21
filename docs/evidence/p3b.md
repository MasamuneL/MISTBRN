# P3b — Evidencia de accesorios (macOS arm64)

Fecha: 2026-11-21. Commit probado: `ca3b4e645f763c193ac2d5c5e3356baff08122f9` (`git_dirty=no` en `mod/` y `tools/`). Build `p3b-6` → `native-0a875da-5aa538f1db8e`; `libmistbrn.dylib` SHA-256 `86cfc146f9e86e32d130f02e32e1a06bea79a7894e8d834b79349bbf441e6718`. Recibo con el hash de cada fichero empaquetado en `staged/p3b-6/receipt.txt`. Corridas `p3b6-<escenario>` en `~/Library/Application Support/MISTBRN-dev/runs/`.

## Superficie

El pin expone lo necesario sin fork: `minecraft:equippable` y `minecraft:attribute_modifiers` como componentes de contenido, recursos de equipamiento vanilla (`assets/<ns>/equipment/`, recargados por `client_reload`) y `Text::translate` para mensajes con claves de idioma.

## Entrega

- Seis ítems: `clip`, `boxing`, `glass_dagger`, `dueling_cane`, `mistcloak` (pecho, capa de equipamiento propia de 64 × 32) y `bronze_earring` (cabeza, modelo diminuto). Valores y recetas en `docs/plan.md` (P3b).
- Ninguna receta del mod coincide con una vanilla. La revisión encontró que desde P3a `iron_flakes` y `copper_flakes` usaban un lingote solo, como las pepitas vanilla; las virutas pasan a lingote + `#minecraft:stone_crafting_materials` (lingote + pedernal habría sido el mechero) y las monedas salen de una pepita.
- Pendientes de P3a cerrados: una bebida por jugador y slot en cada tick; el slot se comprueba antes que la capacidad (un vial movido se rechaza en silencio); los mensajes al jugador usan claves (`message.mistbrn.*`) en `en_us` y `es_es`.
- `tools/check.sh`: texturas de equipamiento, capas `humanoid` obligatorias, fichero de equipamiento para cada `asset_id` del mod y peso medido sobre los blobs del índice. El último punto se añadió tras la batería (solo `tools/check.sh`).

## Resultados

| Escenario | Corrida | Observado | Resultado |
|---|---|---|---|
| D1 fabricar | `p3b6-d1` | Daga, pendiente, clip y boxing en la rejilla 2 × 2; bastón y capa en una mesa. Un lingote solo de hierro, cobre y oro sigue dando 9 pepitas. Inventario exacto | PASS |
| D2 equipar | `p3b6-d2` | Capa en pecho y pendiente en cabeza tras guardar y relanzar, sin copia en el inventario. Captura en tercera persona en la corrida (no versionada) | PASS |
| D3 reserva llena | `p3b6-d3` | Cuatro créditos (100 000 milésimas) y un único `vial_refused … CapacityExceeded(Iron)`; chat con `message.mistbrn.vial.reserve_full`; quedan 1 lleno y 4 vacíos | PASS |
| C1 | `p3b6-c1` | Vidrio → viales, lingote + adoquín → virutas, vial + viruta → vial de hierro | PASS |
| C2, C2b | `p3b6-c2`, `p3b6-c2b` | Regresión de beber en ambas manos | PASS |
| C3, C4 | `p3b6-c3`, `p3b6-c4` | Cortes antes y después de guardar | PASS |
| C5 | `p3b6-c5` | Fallo inyectado revierte inventario y guardado | PASS |
| S9 | `p3b6-s9` | Guardado con CRC roto bloquea al jugador y los bytes siguen intactos. El mensaje con la clave `message.mistbrn.save_blocked` no se comprueba en esta corrida | PASS |

10/10 en el build final. Builds anteriores (`p3b-1` … `p3b-5`) encontraron el choque de recetas, el formato real del error de capacidad (`InvalidReserves(CapacityExceeded(Iron))`) y el pendiente flotando sobre la cabeza; sus corridas se descartaron.

## Límites

- Solo macOS arm64; Windows x64 y Linux x64 en P10.
- Deduplicación de bebidas sin probar con dos clientes: diferida a P9 por decisión del usuario. Los revisores no ven ruta de doble crédito: cada bebida revalida el stack exacto dentro de la transacción.
- `bronze_earring` ocupa el slot de casco. El render del pendiente no se verificó visualmente (queda diminuto junto a la cabeza).
- Las monedas y la daga no tienen todavía comportamiento alomántico ni arrojadizo (P4).
