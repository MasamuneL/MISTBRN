# P3c — Evidencia de alquimia (Windows x64)

Fecha: 2026-11-25. Pin de Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, juego 26.3, Rust 1.99.1, Horizon de Windows con Git Bash, instancia `mistbrn-p3c`. Batería final `p3c-final`, compilada desde `ae5b9d4` (rebase sobre `main` con P7b, `917b40d`) con el árbol limpio (`git_dirty=no`); `mistbrn.dll` SHA-256 `7d1f044ba41f540b8a0e35833d4b074ea586d8295de3ee1de61d0209bf3fa959`. Recibo en `%LOCALAPPDATA%\MISTBRN-p3c-dev\staged\p3c-final\receipt.txt`; corridas `p3c-final-*` en `%LOCALAPPDATA%\MISTBRN-p3c-dev\runs\`: **39/39 PASS**. Diseño y decisiones: [plan](../plan.md) (P3c — Alquimia) · [guía](../p3c-guide.md).

## Qué entra

- **Núcleo** (`core/src/persistence.rs`, `content.rs`): `prepare_basic_vial` acredita +25 000 a cada metal desbloqueado hasta su capacidad, una vez (`commit_credit`); sin sitio, `NothingToCredit` y no se gasta. `brew_result` (sal + agua → salmuera; viruta + salmuera → vial de ese metal), `BREW_TICKS = 600` y `beach_salt` (1 de cada 20, determinista por posición y tiempo). 6 tests nuevos en `tests/alchemy.rs`.
- **Adaptador** (`alchemy.rs`, `bridge.rs`, `lib.rs`, `content.rs`):
  - Alambique: block entity con 3 huecos de vial (1 por hueco) y 1 de ingrediente, sin combustible ni caras para tolvas. Se abre con la mano vacía (`use_without_item` → `PlayerRef::open_menu`). El progreso va en el nbt; la barra, en un data slot. La tanda es `ingrediente@huecos`: cambiarlos la reinicia.
  - Vial de agua: clic derecho mirando una fuente; el rayo usa `fluid_state` por pasos, porque `clip` ignora fluidos.
  - Sal de la arena de playa: la decide `level_tick_post`, fuera del mutex, porque `block_drops` no lee bioma ni tiempo.
  - Horno: vial de agua → sal, y el vial vacío vuelve con `spawn_item` encima del horno.
  - Vial de básicos por la misma transacción que los demás viales.
- **Datos 26.3**: bloques `salt_rock` y `alembic`, menú `alembic` (4 huecos, 1 data slot), botín con `condition`/`modifier`, feature `simple_block` + placed feature en arena + biome modifier `#minecraft:is_beach`, recetas (alambique; vial de básicos; sal en el horno), recetas `vial_<metal>` eliminadas.
- **Concluir** (P7b): solo mirando a un alambique (`RESEARCH_TABLES`, textos, R1 y L3).
- **Arte**: 7 PNG 16×16 (`tools/gen-alchemy.py`). Rondas Claude ⇄ Codex en `~/.local/share/MISTBRN-dev/review/p3c-art/discussion.md`; las dos partes aprueban en el juego (ronda 4: banda ocre del vial de básicos).

Tests: 147 en `allomancy-core` (2 ignorados) y 11 en el adaptador.

## Resultados (build `p3c-final`)

| Escenario | Observado | Resultado |
|---|---|---|
| A1 agua | Un vial vacío mirando una fuente da 1 vial de agua; mirando piedra, nada; 1 `vial_filled` | PASS |
| A2 sal | Piedra de sal da 1–5; con Toque de Seda, el bloque; 100 de arena de playa dan sal entre 1 y 15 veces, 40 de llanura ninguna; el horno da sal y devuelve el vial (`furnace_vial dropped=true`) | PASS (`a2b`) |
| A3 alambique | Agua → salmuera ×3 con la tanda partida por un guardado y relanzamiento; hierro ×3 a los 600 ticks y no a los 590; etapa equivocada no hace nada; un vial tardío reinicia la tanda; roto suelta exactamente alambique, 1 vial de hierro y 2 de estaño | PASS |
| A4 básicos | Sin metales: `NothingToCredit`; tras hierro: `credited=[25000, 25000, 0, …]`; con fallo forzado el vial, las reservas y `last_transaction=2` no cambian | PASS |
| A5 receta vieja | Vial vacío + viruta ya no fabrica nada; un vial de hierro antiguo se bebe (`reserve_iron=25000`) | PASS |
| A6 worldgen | Playa localizada; piedras de sal en la arena de chunks nuevos | PASS |
| AV capturas | Bloques en el mundo, pantalla del alambique a mitad de tanda, ítems en el inventario | PASS |
| L1–L6 (P7b) | El libro, con Concluir mirando a un alambique (`l3-concluded`, `l3-far`) | PASS |
| Regresión | h1–h5 hv m3 t1 x1 r1 b1 b2 b3 pd1 m1 q1 u1 v1 k1 k6 k9 s1 c2 c4 s10 n1. V1 destila su vial de hierro en el alambique; C1 ya no fabrica viales | PASS |

A2 se repitió como `p3c-final-a2b` porque la carpeta `p3c-final-a2` quedó de una batería cortada; mismo build. Baterías anteriores: `p3c-1`…`p3c-5` (formatos 26.3, apertura del menú, escenarios ajustados y arreglos de revisión).

## Revisión

Informes en `~/.local/share/MISTBRN-dev/review/p3c-final/` (WSL).
- **Codex**: ronda 1 CAMBIOS (alto: reentrada de la sal de playa; medios: horno, cola al cerrar, composición de la tanda, fallo del vial de básicos); ronda 2 **APROBADO**.
- **Gemini**: ronda 1 CAMBIOS (bloqueante rechazado: el menú sí abre en el juego; horno en otra dimensión y tanda, arreglados); ronda 2 **APROBADO**. Guía: rondas 1–3 CAMBIOS aplicados, ronda 4 **APROBADO**.

## Límites

- Solo Windows x64 en el juego para P3c.
- Si el horno no está en el nivel en curso, se busca en las tres dimensiones vanilla; un nivel aparcado no recibe el vial.
- Cambiar un vial por otro igual en el mismo hueco no reinicia la tanda. Romper el alambique con una tanda a medias no tiene escenario.
- Si otro mod cancela los drops de la arena después de MISTBRN, la sal sale igual. La cola de arena tiene un tope de 256 entre dimensiones.
- La piedra de sal solo aparece en chunks nuevos; el 5 % de la arena es determinista por posición y tiempo.
