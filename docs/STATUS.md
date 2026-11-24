# Estado — Masamune's MISTBRN

Actualizado el 2026-11-23.

## Integrado en main (V1, al integrar la PR #13); P5 en la rama `feat/p5-progress`

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |
| P3b | Monedas, daga, bastón, capa de bruma y pendiente; texturas nuevas (PR #9) y Fortuna en las muestras (PR #10). [PR #7](https://github.com/MasamuneL/MISTBRN/pull/7) · [Evidencia](evidence/p3b.md) |
| P4 | Tirón de hierro y empujón de acero: objetivo por la mirada, anclas, salto con moneda, monedas como proyectil, topes, colisión del motor y cancelación por muerte, dimensión, objetivo inválido y agotamiento. [PR #11](https://github.com/MasamuneL/MISTBRN/pull/11) · [Evidencia](evidence/p4.md) |
| P7a | Teclas V/Z/X/R/G (pareja, alternar metales, avivar, apagar), paquete `mistbrn:control` validado y con presupuesto en el servidor, HUD de reservas sincronizado y bloqueo con chat, inventario o contenedor abiertos. `/mistbrn-test` queda para pruebas. [PR #12](https://github.com/MasamuneL/MISTBRN/pull/12) · [Evidencia](evidence/p7a.md) |
| V1 | Demo de hierro y acero: «Cartilla: hierro y acero» (provisional hasta P5/P6) que desbloquea y selecciona la pareja; barras de la pareja actual junto al hotbar, rueda de parejas en V (op 5 «elegir pareja») y avisos sobre el hotbar cuando una tecla no puede actuar; hoja de sprites original. Escenarios U1–U5, v1 y K9. [PR #13](https://github.com/MasamuneL/MISTBRN/pull/13) · [Evidencia](evidence/v1.md) · [Guía](v1-guide.md) |

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (V1) fue `v1-12` (commit `3544cca`, árbol limpio) en `mistbrn-dev`: U1–U5, v1, K1–K9, C2, S1 y N1, 18/18 PASS a la primera. Recibos y corridas fuera del repo, en `~/Library/Application Support/MISTBRN-dev/`. Windows x64 y Linux x64 quedan para P10. [Compatibilidad](compatibility.md)

## Reanudación

- Rama activa: `feat/p5-progress` (desde `main` `e592a48`). P5: práctica, investigación «principio del ancla», árbol de hierro/acero + general reducido, cuaderno (ítem y tecla J), tutorial del vial (sustituye a la cartilla), esquema 2 con migración 1→2. [Evidencia](evidence/p5.md) · [Guía](p5-guide.md)
- Fase y paso: P5 implementada en la PR #15; batería final `p5-5` (`4a1916b`, árbol limpio) 19/19 PASS; Codex APROBADO (ronda 4) y Gemini APROBADO (ronda 3); assets aprobados por Claude y Codex en juego. Falta tu evaluación con la guía y tu aprobación del merge.
- Operación en curso: ninguna. No hay juego ni build activo.
- V1: sus notas de evaluación no se registraron por escrito; se pueden añadir a `evidence/v1.md`.

## Siguiente acción exacta

1. P5: evaluación del usuario con `docs/p5-guide.md` (build `p5-5` ya instalado en `mistbrn-dev`) y merge de la PR #15 con su aprobación (`gh pr merge 15 --merge --match-head-commit <sha>`).
2. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3; límites del HUD y la rueda en `compatibility.md` (chat encima, mando, nombre de tecla en los avisos).

## Próximos hitos

1. **P6** hitos (fragmentos y reclamo único) y **P7b** UI completa (libro, árboles, pistas), sobre el esquema 2.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
