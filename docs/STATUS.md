# Estado — Masamune's MISTBRN

Actualizado el 2026-11-22.

## Integrado en main (P7a, al integrar la PR #12)

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |
| P3b | Monedas, daga, bastón, capa de bruma y pendiente; texturas nuevas (PR #9) y Fortuna en las muestras (PR #10). [PR #7](https://github.com/MasamuneL/MISTBRN/pull/7) · [Evidencia](evidence/p3b.md) |
| P4 | Tirón de hierro y empujón de acero: objetivo por la mirada, anclas, salto con moneda, monedas como proyectil, topes, colisión del motor y cancelación por muerte, dimensión, objetivo inválido y agotamiento. [PR #11](https://github.com/MasamuneL/MISTBRN/pull/11) · [Evidencia](evidence/p4.md) |
| P7a | Teclas V/Z/X/R/G (pareja, alternar metales, avivar, apagar), paquete `mistbrn:control` validado y con presupuesto en el servidor, HUD de reservas sincronizado y bloqueo con chat, inventario o contenedor abiertos. `/mistbrn-test` queda para pruebas. [PR #12](https://github.com/MasamuneL/MISTBRN/pull/12) · [Evidencia](evidence/p7a.md) |

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la última batería (P7a) fue `native-0a875da-4870250b4b03` (`p7a-5`, commit `63cf094`) en `mistbrn-dev`: K1–K8, N3, N7 y S1 PASS; N1 falló una vez por 0,001 (dos movimientos del cliente en un tick) y pasó en dos repeticiones. Recibos y corridas fuera del repo, en `~/Library/Application Support/MISTBRN-dev/`. Windows x64 y Linux x64 quedan para P10. [Compatibilidad](compatibility.md)

## En curso: V1 (rama `feat/v1-unlock`)

Desbloqueo por ítem (decisión del usuario): la «Cartilla: hierro y acero» (libro + lingote de hierro + lingote de acero), provisional hasta P5/P6, desbloquea la pareja en la misma transacción que la consume. Escenarios U1–U5 y `v1` (recorrido completo sin comandos de prueba) y regresión C2, S1, K1, K6, N1 PASS con el build `v1-2` (`native-0a875da-c01af71c3d7a`, `a63f1a0`). Revisión: Codex y Gemini APROBADO en la 1.ª ronda; hallazgos medios y bajos atendidos. [Evidencia](evidence/v1.md) · [Guía para jugar el recorrido](v1-guide.md)

## Reanudación

- Rama activa: `feat/v1-unlock` (desde `main` `6d440f4`); worktree `../MISTBRN-v1` (`wip/v1-scenario`) solo para el escenario de Codex, ya integrado.
- Fase y paso: V1, PR #13 revisada; falta la evaluación humana y la aprobación del merge.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: batería V1 con `v1-2`, todo PASS (U1, U4 y v1 tras arreglar el harness o el foco de la ventana, ver evidencia); captura `runs/v1-2-v1b/v1-hud.png` revisada.

## Siguiente acción exacta

1. Evaluación humana del usuario con `docs/v1-guide.md`: criterio de cierre de V1. Merge solo con su aprobación tras ver la captura.
2. Pendientes abiertos: dos clientes (P9); reglas de reserva y fuerza a datos; márgenes estrechos de N1 y N3.

## Próximos hitos

1. **V1**, la primera demo, debe permitir preparar metal, beber, usar hierro y acero, consultar la reserva y retomar la partida.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
