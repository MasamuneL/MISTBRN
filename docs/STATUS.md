# Estado — Masamune's MISTBRN

Actualizado el 2026-11-27.

## Integrado en main (P8 parte 3, al integrar la PR #28)

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |
| P3b | Monedas, daga, bastón, capa de bruma y pendiente; texturas nuevas (PR #9) y Fortuna en las muestras (PR #10). [PR #7](https://github.com/MasamuneL/MISTBRN/pull/7) · [Evidencia](evidence/p3b.md) |
| P4 | Tirón de hierro y empujón de acero: objetivo por la mirada, anclas, salto con moneda, monedas como proyectil, topes, colisión del motor y cancelación por muerte, dimensión, objetivo inválido y agotamiento. [PR #11](https://github.com/MasamuneL/MISTBRN/pull/11) · [Evidencia](evidence/p4.md) |
| P7a | Teclas V/Z/X/R/G (pareja, alternar metales, avivar, apagar), paquete `mistbrn:control` validado y con presupuesto en el servidor, HUD de reservas sincronizado y bloqueo con chat, inventario o contenedor abiertos. `/mistbrn-test` queda para pruebas. [PR #12](https://github.com/MasamuneL/MISTBRN/pull/12) · [Evidencia](evidence/p7a.md) |
| V1 | Demo de hierro y acero: «Cartilla: hierro y acero» (provisional hasta P5/P6) que desbloquea y selecciona la pareja; barras de la pareja actual junto al hotbar, rueda de parejas en V (op 5 «elegir pareja») y avisos sobre el hotbar cuando una tecla no puede actuar; hoja de sprites original. Escenarios U1–U5, v1 y K9. [PR #13](https://github.com/MasamuneL/MISTBRN/pull/13) · [Evidencia](evidence/v1.md) · [Guía](v1-guide.md) |
| P5 | Práctica con topes por ventana, investigación «El principio del ancla» (2 de 3 evidencias, pistas opcionales, se concluye en la mesa de crafteo), árbol de hierro/acero (alcance, aceleración, salto con moneda) + general reducido (economía, capacidad), cuaderno (ítem y tecla J), el primer vial desbloquea la pareja (sustituye a la cartilla), esquema de guardado 2 con migración 1→2. [PR #15](https://github.com/MasamuneL/MISTBRN/pull/15) · [Evidencia](evidence/p5.md) · [Guía](p5-guide.md) |
| P6 | Cuatro hitos, cada uno una vez por jugador: bastión (estaño/peltre), guardián anciano (zinc/latón), ciudad antigua (cobre/bronce) y dragona (+3 de conocimiento); la pareja se desbloquea sin seleccionarla; jefes compartidos por quienes los dañaron en 30 s; pestaña Hitos del cuaderno; esquema de guardado 3 con migración 2→3. Verificada en Windows x64. [PR #18](https://github.com/MasamuneL/MISTBRN/pull/18) · [Evidencia](evidence/p6.md) · [Guía](p6-guide.md) |
| P7b | El libro (tecla J; el ítem cuaderno sin receta): pestañas Árboles (plantilla de las 4 parejas + General con scroll; solo los nodos del catálogo son reales), Diario (lista y detalle; Concluir mirando a una mesa de crafteo o un alambique a ≤ 5 bloques, op 8), Hitos y Reservas; tooltips, foco por teclado y ajustes «Reducir destellos» y tamaño del libro y de las barras. Sin cambio de esquema. Verificada en Windows x64. [PR #20](https://github.com/MasamuneL/MISTBRN/pull/20) · [Evidencia](evidence/p7b.md) · [Guía](p7b-guide.md) |
| P3c | Alquimia: alambique (3 viales + 1 ingrediente, tandas de 600 ticks, dos etapas: agua + sal → solución salina, solución + viruta → vial), sal de piedra de sal en playas, arena de playa y horno, vial de agua, vial de metales básicos; se quita la receta vial vacío + viruta y Concluir pide el alambique. Verificada en Windows x64. [PR #22](https://github.com/MasamuneL/MISTBRN/pull/22) · [Evidencia](evidence/p3c.md) · [Guía](p3c-guide.md) |
| P8 (1 de 4) | Mapa radial de habilidades en la pestaña Árboles: General en el centro, cuatro sectores como en la tabla alomántica (tirón, empujón y técnica combinada al fondo), puentes entre vecinos, sectores bloqueados en silueta con su hito; arrastre, rueda y teclado. Nodos generales coste de avivar y dosificación; op 9 para códigos 64..=127; esquema de guardado 4 con migración 1/2/3→4. Límite del árbol a 1,5 MiB. Verificada en Windows x64. [PR #24](https://github.com/MasamuneL/MISTBRN/pull/24) · [Evidencia](evidence/p8.md) · [Guía](p8-guide.md) |
| P8 (2 de 4) | Estaño y peltre: sentidos (visión nocturna, niebla, deslumbramiento), oído por movimiento con contorno, cuerpo de peltre (ataque, velocidad, aguante, caída), salto, arrastre tras avivar; Reflejos (esquiva con C, op 10); combustión simultánea y transición; puente Cuerpo de acero; investigación «el principio del umbral». Verificada en Windows x64. [PR #26](https://github.com/MasamuneL/MISTBRN/pull/26) · [Evidencia](evidence/p8.md) · [Guía](p8-guide.md) |
| P8 (3 de 4) | Zinc y latón: provocar y apaciguar a la criatura que miras (nunca a jugadores; jefes solo 2 s y 30 s de inmunidad; sin mobs con «brain» ni dragona), Infundir valor en tu lobo o un gólem construido, Retirada, Influencia selectiva; puente general Alcance externo (+2 bloques a los cuatro metales externos); investigación «el principio de la emoción». Verificada en Windows x64. [PR #28](https://github.com/MasamuneL/MISTBRN/pull/28) · [Evidencia](evidence/p8.md) · [Guía](p8-guide.md) |

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P8 parte 3) fue `native-0a875da-8d0e289e9da3` (commit `3081be1`, árbol limpio; `ddf989d` da el mismo hash) en Windows x64, instancia `mistbrn-p8`: ZB1–ZB6 y los anteriores afectados, 30/30 PASS (`p8-zb-final`, `p8-zb-final2-g3`). Desde esta rama, cada batería corre sus escenarios y los afectados; la regresión completa queda para el cierre de P8. Recibos y corridas fuera del repo, en `%LOCALAPPDATA%\MISTBRN-p8-dev\` y `%LOCALAPPDATA%\MISTBRN-p3c-dev\` (Windows); los de fases anteriores, en `~/Library/Application Support/MISTBRN-dev/` (macOS). El gate completo de Windows x64 y Linux x64 sigue en P10. [Compatibilidad](compatibility.md)

## Reanudación

- P8 en curso, en cuatro ramas: mapa (PR #24), estaño/peltre (PR #26) y zinc/latón (PR #28) integradas; queda `feat/p8-copper-bronze`. Plan aprobado fuera del repo.
- Toda revisión (Codex, Gemini), build y batería se lanza en una terminal herdr visible (`hc run --bg --name …`), nunca como proceso oculto en segundo plano.

## Siguiente acción exacta

1. `feat/p8-copper-bronze`: empieza con una prueba de `entity_part` (guardián alomántico), luego emisor, pulsos, nube de cobre, puentes 25 y 26 e investigación 4.
2. La regresión completa al cerrar P8, en esa rama.
3. P7b: notas de la revisión visual del usuario con `docs/p7b-guide.md`, si las hay, a `evidence/p7b.md`.
4. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3; horno en un nivel aparcado y rotura del alambique a media tanda (P3c).

## Próximos hitos

1. **P8** poderes de las tres parejas que los hitos ya desbloquean, sobre el mapa y el esquema 4.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
