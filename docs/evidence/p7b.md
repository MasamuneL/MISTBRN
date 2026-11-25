# P7b — Evidencia del libro (Windows x64)

Fecha: 2026-11-24. Pin de Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, juego 26.3, Rust 1.99.1, Horizon de Windows con Git Bash, instancia `mistbrn-p6`. Batería final `p7b-final`, compilada desde `2982d30` con el árbol limpio (`git_dirty=no`); `mistbrn.dll` SHA-256 `5c0db492636f9263c18063672517742b7dbebc4e45e8dafa86b63b9a6607de87`: **32/32 PASS**. Corridas en `%LOCALAPPDATA%\MISTBRN-p6-dev\runs\p7b-final-*`. Diseño y decisiones: [plan](../plan.md) (P7b) · [guía](../p7b-guide.md).

**Repaso tras revisar las capturas.** En `l3-refused` una caja tapaba la nota de Concluir y el botón. Era el tooltip del propio botón (el puntero se queda encima tras el clic), no la barra de acción: `64a2d9c` (quitar la barra) se revirtió en `2986346`, y `02f69d0` quita ese tooltip. L3, R1 y B1 en `p7b-final3` (`2986346`, árbol limpio, SHA-256 `8472b127…dacc3`): PASS. La nota pasó a dos líneas y más corta (`3fda7b0`, `69387a5`); L3 en `p7b-final5` (`69387a5`, árbol limpio): PASS, captura sin solapes. Esos cambios solo tocan el dibujo del diario y los textos; el resto de la batería final no se repitió. `p7b-final2` (`b16c001`): L3, R1, B1 PASS con el tooltip aún presente.

## Qué entra

- **Núcleo** (`core/src/control.rs`): op 8 «concluir» (arg = investigación ≠ 0); tests de ida y vuelta, rechazo de 0 y de todo byte desconocido. Catálogo, esquema 3, fixtures y `mistbrn:progress` sin cambios.
- **Servidor** (`controls.rs`, `bridge.rs`): op 8 concluye con `prepare_conclude` + `commit_credit` en `host.transaction`, tras comprobar input no capturado, supervivencia o aventura y que el primer bloque de la mirada a ≤ 5 bloques es `minecraft:crafting_table` o `mistbrn:alembic` (provisional hasta P3c). Se quitan la conclusión con el cuaderno en la mano y la receta del cuaderno (el ítem sigue registrado y abre el libro). `/mistbrn-test setting` (solo pruebas) guarda un ajuste como la página de opciones.
- **Cliente** (`book/`, `paint.rs`, `client.rs`, `hud.rs`): el libro (J) con pestañas Árboles, Diario, Hitos y Reservas; plantilla de árbol por pareja con scroll y siluetas; diario con lista y detalle, botón Concluir y nota si el servidor no concluye; tooltips; foco por teclado; ajustes «Reducir destellos», «Tamaño del libro» y «Tamaño de las barras de reserva». A 100 % se dibuja como antes; a 75/125 % por `gui_paint` con pose escalada. Tests del modelo del libro: 5.
- **Arte**: sin PNG nuevos (rellenos y las hojas existentes). Ronda visual Claude ⇄ Codex en `~/.local/share/MISTBRN-dev/review/p7b-assets/discussion.md`: AGREE, con tres ajustes aplicados (selector más ancho, texto de siluetas, icono bloqueado más claro).

## Resultados (build `p7b-final`)

| Escenario | Observado | Resultado |
|---|---|---|
| L1 vistas | Las cinco vistas del árbol, Diario, Hitos y Reservas; scroll por muesca hasta 56 y de vuelta a 0; tooltip de Alcance; ninguna compra por mirar | PASS |
| L2 teclado | Tab ×10 hasta `node.1`, Intro compra el rango 1; Intro mantenido (`repeat=true`) no pide nada; un segundo toque compra el rango 2; tras relanzar `nodes=1:2` | PASS |
| L3 concluir | Negativas: creativo (`GameMode`), mirando al cielo, a 5,5 bloques desde el botón del libro (nota naranja tras 20 ticks) y con vidrio en medio (`NoTable` ×3); junto a la mesa, una conclusión (`research=1:e6h0c1 knowledge=3 xp=300`); repetida `AlreadyConcluded` | PASS |
| L4 ajustes | Libro a 0,75 y a la mayor escala que cabe (1,08 en GUI 427×240); barras a 1,25 (`hud_box`); aviso de reserva baja: 12 intensidades sin el ajuste, una fija (84) con él | PASS |
| L5 reservas | Pestaña Reservas con hierro ardiendo y estaño desbloqueado | PASS |
| L6 receta | `recipe give` responde «Unknown recipe: mistbrn:notebook»; un cuaderno existente abre el libro | PASS |
| Regresión | h1–h5 hv m3 t1 x1 r1 b1 b2 b3 pd1 m1 q1 u1 v1 k1 k6 k9 s1 c2 c4 s10 n1 (T1: el cuaderno ya no se fabrica; R1/B1 concluyen con el botón; R1, Q1, B1 y HV pulsan por nombre) | PASS |

Capturas revisadas a mano: `l1-*`, `l2-focus`, `l3-refused`, `l3-concluded`, `l4-book-75`, `l4-hud-125`, `hv-milestones`. `l3-refused` mostró una caja encima de la nota y del botón: ver el repaso arriba.

Baterías anteriores: `p7b-1` (humo L1: nombres cortados, panel ensanchado), `p7b-2` (L2–L4 cortados por el arnés: tecla `page_down` sin nombre en horizon-ctl, `look-at block` fuera de alcance, sin python en Windows) y `p7b-3` (L3 caía del suelo, L4 esperaba 1,25 en una GUI donde no cabe).

## Revisión

- **Codex**: ronda 1 NO APROBADO (alta: Intro mantenido compraba rango tras rango; medias: tecla del libro reasignada, nodos recortados clicables, mutex en llamadas al host, rechazo de Concluir invisible, cobertura de L3/L4); ronda 2 **APROBADO** (medias atendidas: nota neutra, tooltip del foco, fallos de `book_at` en el shell del escenario, orden en L2).
- **Gemini**: ronda 1 **APROBADO** (medio y bajo ya resueltos en la ronda 1 de Codex; la guía pide supervivencia para el vial).

## Límites

- Sin mando: sus clics no llegan a pantallas de mod. Re Pág/Av Pág solo con test unitario (horizon-ctl no tiene nombre para esas teclas).
- «Reducir destellos» se comprueba por la intensidad que se pasa al relleno (log de prueba), no comparando capturas.
- Las notificaciones de vanilla (tutorial, recetas) se dibujan sobre la esquina superior derecha del libro mientras se ven.
- La mesa de crafteo vale solo hasta que P3c integre el alambique.
- Solo Windows x64 en juego.
