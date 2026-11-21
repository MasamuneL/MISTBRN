# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: ninguna; `main` incluye P0b y P2.
- Fase: **P2** cerrado — P2-core (PR #3) y P2-native (PR #4: revisión ronda 2 approve; S1–S11 + S5b PASS en `70308f9`, `docs/evidence/p2.md`).
- Operación en curso: ninguna. No hay juego ni build activo.
- Runtime (fuera del repo): `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). Instancia `mistbrn-dev`; build instalado `native-0a875da-482083add929`.

## Siguiente acción exacta

1. P3 (contenido) en `feat/p3-content` desde `main`: metales, viales vacío/lleno, recetas, fuentes de muestra, assets originales, `es_es`/`en_us`; beber vial con `Session::credit_from_slot` (sustituye la pepita de prueba). Antes: comprobar en `surface.json` los hooks de uso de item.
2. Pendiente menor de la revisión P2 (R3): doble carga al abrir un segundo mundo en el mismo proceso (inocua); cerrar en P3.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b | mergeado (PR #2) |
| P2-core | mergeado (PR #3) |
| P2-native | mergeado (PR #4) |
| Windows x64 / Linux x64 | P10 |
