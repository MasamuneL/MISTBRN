# Estado — Masamune's MISTBRN

Actualizado el 2026-11-21.

## Integrado en main (P4, al integrar la PR #11)

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |
| P3b | Monedas, daga, bastón, capa de bruma y pendiente; texturas nuevas (PR #9) y Fortuna en las muestras (PR #10). [PR #7](https://github.com/MasamuneL/MISTBRN/pull/7) · [Evidencia](evidence/p3b.md) |
| P4 | Tirón de hierro y empujón de acero: objetivo por la mirada, anclas, salto con moneda, monedas como proyectil, topes, colisión del motor y cancelación por muerte, dimensión, objetivo inválido y agotamiento. Activación solo con `/mistbrn-test` hasta P7a. [PR #11](https://github.com/MasamuneL/MISTBRN/pull/11) · [Evidencia](evidence/p4.md) |

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P4) fue `native-0a875da-7fc1871a8f7f` (`p4-22`, commit `554e97c`) en `mistbrn-dev`: 17/17 (N1–N11, S1, S5, S8, S9, C2). Recibos y corridas fuera del repo, en `~/Library/Application Support/MISTBRN-dev/`. Windows x64 y Linux x64 quedan para P10. [Compatibilidad](compatibility.md)

## Reanudación

Estado previsto tras integrar la PR #11 (escrito en la propia PR).

- Rama activa: ninguna tras integrar `feat/p4-iron-steel`; `main` incluye P0b, P2, P3a, P3b y P4.
- Fase y paso: P4 cerrada; P7a sin empezar.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: batería P4 17/17 en `554e97c`; Codex (11 rondas) y Gemini (3 rondas) APROBADO en la última.

## Siguiente acción exacta

1. P7a en `feat/p7a-hud` desde `main` actualizado: teclas V/Z/X/R/G en el cliente, paquete al servidor que sustituye a `/mistbrn-test`, HUD de reservas y bloqueo con chat o menú abiertos. La API (teclas, canal cliente→servidor, HUD, `playing()`/pantallas) se investigó con Gemini durante P4; comprobarla contra `surface.json` antes del plan.
2. Pendientes abiertos: prueba de dos clientes de la cola de bebidas y de las monedas (P9); mover las reglas de reserva y de fuerza a datos; la flotación de N3 tarda en asentarse (margen estrecho frente al criterio de 1,0).

## Próximos hitos

1. **P7a** incorpora controles y un HUD mínimo. La primera demo (V1) debe permitir preparar metal, beber, usar hierro y acero, consultar la reserva y retomar la partida.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
