# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: `feat/p3-content` desde `314df92`.
- Fase: **P2** cerrado — P2-core (PR #3) y P2-native (PR #4: revisión ronda 2 approve; S1–S11 + S5b PASS en `70308f9`, `docs/evidence/p2.md`).
- P3a: prueba de contenido aprobada. `p3-vial-smoke-1` fabricó dos `mistbrn:vial_empty` con dos bloques de vidrio en supervivencia.
- Runtime (fuera del repo): `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). Instancia `mistbrn-dev`; build de prueba `native-0a875da-27d62f8f5a96`.

## Siguiente acción exacta

1. Completar P3a: metales, viales llenos, recetas, fuentes en mundos existentes, recursos originales y traducciones. Conectar el uso de los viales a la transacción de P2.
2. Pendiente menor de la revisión P2 (R3): doble carga al abrir un segundo mundo en el mismo proceso (inocua); cerrar en P3.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b | mergeado (PR #2) |
| P2-core | mergeado (PR #3) |
| P2-native | mergeado (PR #4) |
| Windows x64 / Linux x64 | P10 |
