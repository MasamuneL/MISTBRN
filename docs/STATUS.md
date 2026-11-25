# Estado — Masamune's MISTBRN

Actualizado el 2026-11-24.

## Integrado en main (P6, al integrar la PR #18)

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

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P6) fue `p6-final3` (commit `8daa88b`, árbol limpio) en Windows x64, instancia `mistbrn-p6`: H1–H5, HV, M3 y la regresión de 19, 26/26 PASS. Recibos y corridas fuera del repo, en `%LOCALAPPDATA%\MISTBRN-p6-dev\` (Windows); los de fases anteriores, en `~/Library/Application Support/MISTBRN-dev/` (macOS). El gate completo de Windows x64 y Linux x64 sigue en P10. [Compatibilidad](compatibility.md)

## Reanudación

- Rama activa: `feat/p3c-alchemy` (P3c, alquimia: alambique, sal y la nueva cadena de viales), rebasada sobre `main` con P7b (PR #20). PR abierta, pendiente de la aprobación del usuario para integrar.
- Revisión: Codex APROBADO (ronda 2), Gemini APROBADO (ronda 2); guía APROBADO por Gemini (ronda 4); arte Claude ⇄ Codex AGREE en el juego.
- Batería: `p3c-final` 39/39 PASS (A1–A6, AV, L1–L6 y la regresión de 26, `ae5b9d4`, Windows x64, instancia `mistbrn-p3c`). [Evidencia](evidence/p3c.md) · [Guía](p3c-guide.md)
- Concluir una investigación pide ahora mirar a un alambique (P7b aceptaba también la mesa de crafteo).

## Siguiente acción exacta

1. P3c: revisión del usuario con `docs/p3c-guide.md`; merge con su aprobación y PR `docs/p3c-status`.
2. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3; horno en un nivel aparcado y rotura del alambique a media tanda (P3c).

## Próximos hitos

1. **P8** poderes de las tres parejas que los hitos ya desbloquean, sobre el esquema 3.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
