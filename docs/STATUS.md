# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: `feat/p3-content` desde `314df92`.
- Fase: **P2** cerrado — P2-core (PR #3) y P2-native (PR #4: revisión ronda 2 approve; S1–S11 + S5b PASS en `70308f9`, `docs/evidence/p2.md`).
- P3a: prueba de contenido aprobada. `p3-vial-smoke-1` fabricó dos `mistbrn:vial_empty` con dos bloques de vidrio en supervivencia.
- P3a incluye 25 ítems y 25 recetas. Pasaron los escenarios de contenido y la regresión de P2, 24 corridas en total. C7 verifica una sola carga por mundo y cierra R3. Evidencia en `docs/evidence/p3.md`.
- El runtime vive en `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). La instancia `mistbrn-dev` tiene seleccionado `native-0a875da-9de1682533b2`.

## Siguiente acción exacta

1. Incorporar en la PR los informes de las dos revisiones independientes de P3a.
2. P3b queda descrito en `docs/plan.md` con monedas, capa y accesorios.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b | mergeado (PR #2) |
| P2-core | mergeado (PR #3) |
| P2-native | mergeado (PR #4) |
| P3a | batería aprobada, revisión en la PR |
| Windows x64 / Linux x64 | P10 |
