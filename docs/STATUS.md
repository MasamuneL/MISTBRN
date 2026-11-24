# Estado — Masamune's MISTBRN

Actualizado el 2026-11-24.

## Integrado en main (P5, al integrar la PR #15)

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

## En revisión (rama `feat/p6-milestones`)

| Fase | Resultado |
|---|---|
| P6 | Cuatro hitos, cada uno una vez por jugador: bastión (estaño/peltre), guardián anciano (zinc/latón), ciudad antigua (cobre/bronce) y dragona (+3 de conocimiento); la pareja se desbloquea sin seleccionarla; jefes compartidos por quienes los dañaron en 30 s; pestaña Hitos del cuaderno; esquema de guardado 3 con migración 2→3. Verificada en Windows x64. [Evidencia](evidence/p6.md) · [Guía](p6-guide.md) |

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P6) fue `p6-final3` (commit `8daa88b`, árbol limpio) en Windows x64, instancia `mistbrn-p6`: H1–H5, HV, M3 y la regresión de 19, 26/26 PASS. Recibos y corridas fuera del repo, en `%LOCALAPPDATA%\MISTBRN-p6-dev\` (Windows); los de fases anteriores, en `~/Library/Application Support/MISTBRN-dev/` (macOS). El gate completo de Windows x64 y Linux x64 sigue en P10. [Compatibilidad](compatibility.md)

## Reanudación

- Rama activa: `feat/p6-milestones` (worktree aparte; otra sesión trabaja en `main`). Codex APROBADO (ronda 2), Gemini APROBADO (ronda 1); batería final 26/26.
- Fase y paso: P6 en PR, esperando la evaluación del usuario con `docs/p6-guide.md` y su aprobación para integrar.
- Operación en curso: ninguna. No hay juego ni build activo.
- V1: sus notas de evaluación no se registraron por escrito; se pueden añadir a `evidence/v1.md`.

## Siguiente acción exacta

1. P6: evaluación del usuario con `docs/p6-guide.md` (build `p6-final3` en la instancia `mistbrn-p6` de Windows); notas a `evidence/p6.md`; merge con su aprobación y PR `docs/p6-status`.
2. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3; límites del HUD y la rueda en `compatibility.md` (chat encima, mando, nombre de tecla en los avisos).

## Próximos hitos

1. **P7b** UI completa (libro, árboles, pistas) o **P8** poderes de las tres parejas que los hitos ya desbloquean, sobre el esquema 3.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
