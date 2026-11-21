# Estado — Masamune's MISTBRN

Actualizado el 2026-11-21.

## Integrado en main

| Fase | Resultado |
|---|---|
| P0b | SDK fijado y adaptador nativo. [PR #2](https://github.com/MasamuneL/MISTBRN/pull/2) |
| P2 | Reservas, guardado versionado y transacciones de jugador. [PR #3](https://github.com/MasamuneL/MISTBRN/pull/3) · [PR #4](https://github.com/MasamuneL/MISTBRN/pull/4) · [Evidencia](evidence/p2.md) |
| P3a | 25 ítems, 25 recetas, viales para ocho metales, fuentes de estaño y zinc en mundos existentes y traducciones. [PR #5](https://github.com/MasamuneL/MISTBRN/pull/5) · [Evidencia](evidence/p3.md) |

P3a se integró en `7f1d159` después de 24 corridas nativas aprobadas en macOS arm64 y dos revisiones independientes. C7 confirmó una sola carga por mundo y que la reserva de un jugador no se arrastra al cambiar de mundo en el mismo proceso.

El pin vigente es Horizon `nightly-2026-11-20-0a875da` (`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`), juego 26.3 y Rust 1.99.1. El build de la batería fue `native-0a875da-9de1682533b2` en `mistbrn-dev`. Sus recibos y corridas se guardaron fuera del repo, en `~/Library/Application Support/MISTBRN-dev/`. Windows x64 y Linux x64 quedan para P10. [Compatibilidad](compatibility.md)

## Reanudación

- Rama activa: `fix/docs-p3` desde `main` (`7f1d159`): documentación posterior a P3a y este estado.
- Fase y paso: P3a cerrada; P3b sin empezar.
- Operación en curso: ninguna. No hay juego ni build activo.
- Último resultado: PR #5 integrada; `tools/check.sh` exit 0 en `7f1d159`.

## Siguiente acción exacta

1. P3b en `feat/p3b-accessories` desde `main`: comprobar en `surface.json` los hooks de equipamiento (capa) y luego `clip`, `boxing`, `glass_dagger`, `dueling_cane`, `mistcloak`, `bronze_earring`, con receta, arte y traducciones.
2. Pendientes de la revisión post-merge de P3a, a cerrar en P3b:
   - Deduplicar la cola de bebidas por jugador y slot: `living_entity_use_item_finish` puede dispararse una vez por jugador del mundo y, con dos o más jugadores, los duplicados se rechazan con un aviso técnico en el chat.
   - Mensajes de chat con claves de idioma en lugar de códigos (`item_not_in_slot`, `CapacityExceeded`).
   - Documentar como límite aceptado que colocar menas sacadas con Toque de Seda en posiciones "afortunadas" garantiza la muestra.

## Próximos hitos

1. **P3b** incorpora los accesorios propuestos, entre ellos monedas y capa de bruma.
2. **P4 y P7a** incorporan tirones y empujones de hierro y acero, controles y un HUD mínimo. La primera demo debe permitir preparar metal, beber, usar los poderes y retomar la partida.
3. Las fases posteriores amplían el recorrido con práctica, investigación, árboles de técnicas, hitos y las otras tres parejas. El cooperativo y la publicación tienen sus propias pruebas.

[Diseño](design.md) · [Plan de ejecución](plan.md) · [Historial anterior al re-pin](history.md)
