# Estado — Masamune's MISTBRN

Actualizado el 2026-11-23.

## Integrado en main (V1, al integrar la PR #13)

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

- Rama activa: ninguna tras integrar `feat/v1-unlock` (PR #13, merge `6c1f4ba`); `main` incluye P0b, P2, P3a, P3b, P4, P7a y V1.
- Fase y paso: V1 integrada con la aprobación del usuario (2026-11-23). Sus notas de evaluación (claridad del control, respuesta del movimiento, utilidad del HUD) no se registraron por escrito; se pueden añadir a `evidence/v1.md` cuando las dé.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: batería final de V1 con `v1-12`, 18/18 PASS; Codex y Gemini APROBADO en las tres rondas de revisión; assets aprobados por Claude y Codex.

## Siguiente acción exacta

1. P5 habilidades e investigación (`feat/p5-progress`): árboles, experiencia por práctica, investigación con pistas, **esquema de guardado 2 + migración 1→2** con el fixture v1, y la forma definitiva de desbloquear parejas, que sustituye a la cartilla provisional.
2. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3; límites del HUD y la rueda en `compatibility.md` (chat encima, mando, nombre de tecla en los avisos).

## Próximos hitos

1. **P5**, la primera entrega vertical: una investigación y un árbol sobre la demo V1.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
