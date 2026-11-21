# Estado — Masamune's MISTBRN

Actualizado el 2026-11-21.

## Integrado en main (P3b, al integrar la PR #7)

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |
| P3b | Monedas, daga, bastón, capa de bruma y pendiente; recetas sin choques con vanilla; pendientes de la revisión de P3a cerrados. [PR #7](https://github.com/MasamuneL/MISTBRN/pull/7) · [Evidencia](evidence/p3b.md) |

P3a se integró en `7f1d159` después de 24 corridas nativas aprobadas en macOS arm64 y dos revisiones independientes. C7 confirmó una sola carga por mundo y que la reserva de un jugador no se arrastra al cambiar de mundo en el mismo proceso.

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P3b) fue `native-0a875da-5aa538f1db8e` en `mistbrn-dev`. Sus recibos y corridas se guardaron fuera del repo, en `~/Library/Application Support/MISTBRN-dev/`. Windows x64 y Linux x64 quedan para P10. [Compatibilidad](compatibility.md)

## Reanudación

Estado previsto tras integrar la PR #7 (escrito en la propia PR).

- Rama activa: ninguna tras integrar la PR #7 (`feat/p3b-accessories`); `main` incluye P0b, P2, P3a y P3b.
- Fase y paso: P3 cerrada (P3a y P3b); P4 sin empezar.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: batería P3b 10/10 en `ca3b4e6` (`docs/evidence/p3b.md`); revisión de Codex y Gemini aprobada.

## Siguiente acción exacta

1. P4 en `feat/p4-iron-steel` desde `main` actualizado: comprobar en `surface.json` los hooks de física de entidad, raycast/búsqueda de bloques y entidades metálicas, y el envío de impulsos al jugador; si falta algo esencial, parar y presentar opciones.
2. Pendientes abiertos: prueba de dos clientes de la cola de bebidas (P9); mover las reglas de reserva a datos.

## Próximos hitos

1. **P4 y P7a** incorporan tirones y empujones de hierro y acero, controles y un HUD mínimo. La primera demo debe permitir preparar metal, beber, usar los poderes y retomar la partida.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
