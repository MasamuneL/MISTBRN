# Estado — Masamune's MISTBRN

Actualizado el 2026-11-21.

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

## Reanudación

Estado previsto tras integrar la PR #12 (escrito en la propia PR).

- Rama activa: ninguna tras integrar `feat/p7a-hud`; `main` incluye P0b, P2, P3a, P3b, P4 y P7a.
- Fase y paso: P7a cerrada; demo V1 sin empezar.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: batería P7a en `63cf094` (ver arriba); Codex APROBADO en la 2.ª ronda y Gemini APROBADO en la 1.ª.

## Siguiente acción exacta

1. Demo V1 (validación, sin rama): preparar metal, beber, usar hierro y acero con las teclas, consultar la reserva en el HUD y continuar tras guardar; lista V1 y evaluación humana del usuario.
2. Pendientes abiertos: prueba de dos clientes de la cola de bebidas, las monedas y el presupuesto de peticiones (P9); mover las reglas de reserva y de fuerza a datos; márgenes estrechos de N1 (paso medio) y N3 (flotación).

## Próximos hitos

1. **V1**, la primera demo, debe permitir preparar metal, beber, usar hierro y acero, consultar la reserva y retomar la partida.
2. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
