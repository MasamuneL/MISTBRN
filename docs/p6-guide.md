# P6 — Guía para probar los hitos

Cuatro hitos, cada uno reclamable **una sola vez por jugador**. Un duplicado no desbloquea nada más. El aviso sale en el chat y el estado se ve en el cuaderno, pestaña **Hitos** (tecla J o el ítem cuaderno).

| Hito | Cómo se cumple | Qué da |
|---|---|---|
| Bastión | Entrar en una pieza de un bastión en ruinas del Nether | Desbloquea estaño y peltre |
| Guardián anciano | Matarlo, o haberle hecho daño en los 30 s antes de su muerte | Desbloquea zinc y latón |
| Ciudad antigua | Abrir un cofre de ciudad antigua que nadie había abierto | Desbloquea cobre y bronce |
| Dragona del End | Haberle hecho daño en los 30 s antes de que su vida llegue a 0 | +3 de conocimiento para el árbol general |

Desbloquear una pareja **no la selecciona**: sigues con la que tenías (cámbiala con la rueda, manteniendo V). Sus viales ya llenan la reserva y el HUD la muestra al seleccionarla, pero sus poderes llegan en P8. Los hitos se conservan al morir y al reconectar.

## Prueba rápida en creativo (con trucos)

Usa un mundo nuevo. Tu mundo de siempre conserva su estado, y un hito ya reclamado no se repite.

1. **Guardián anciano.** Escribe `/summon minecraft:elder_guardian ~3 ~ ~ {NoAI:1b}` y mátalo con la espada (o con `/damage @e[type=elder_guardian,limit=1] 2000 minecraft:player_attack by @s`). Debe salir el aviso del hito, y la pestaña Hitos lo marca reclamado. Invoca otro y mátalo: no debe salir ningún aviso.
2. **Ciudad antigua.** Escribe `/setblock ~2 ~ ~ chest{LootTable:"minecraft:chests/ancient_city"}` y abre el cofre. Debe salir el aviso, y la pestaña Hitos lo marca reclamado. Un segundo cofre igual no da nada, y un cofre con otra tabla tampoco.
3. **Dragona.** Escribe `/summon minecraft:ender_dragon ~5 ~ ~ {NoAI:1b}` y abátela al momento con `/damage @e[type=ender_dragon,limit=1] 2000 minecraft:player_attack by @s` (si tardas, echa a volar). Debe salir el aviso, la pestaña Hitos lo marca reclamado y en la pestaña Árbol los puntos de conocimiento suben en 3.
4. **Bastión.** Ve al Nether y escribe `/locate structure minecraft:bastion_remnant`. Teletranspórtate allí y camina hasta entrar en el bastión. El aviso sale antes de un segundo, y la pestaña Hitos lo marca reclamado.
5. **Guardado.** Guarda y sal del mundo, y vuelve a entrar. Los cuatro hitos siguen reclamados, las parejas siguen desbloqueadas y los puntos de conocimiento se conservan.

En supervivencia es igual; solo cuesta llegar.

## Grupo (sin probar con dos clientes)

Reclaman el jefe todos los jugadores que le hicieron daño en los 30 s antes de su muerte. Quien solo mira no reclama. El cooperativo real se prueba en P9.

## Qué anotar

- Si un aviso no sale, o sale dos veces para el mismo hito.
- Si la pestaña Hitos no coincide con lo que hiciste, o se ve mal (texto cortado o encima de otro).
- Si al reclamar cambió tu pareja seleccionada, tu reserva o algo del árbol que no debía.
- Los mundos de P5 cargan sin perder nada; sus hitos empiezan vacíos.

Envía las notas en el chat; se añaden a `docs/evidence/p6.md`.
