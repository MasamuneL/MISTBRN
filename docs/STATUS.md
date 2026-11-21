# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: `feat/p2-native` (worktree `~/Library/Application Support/MISTBRN-dev/wt-p2-core`), sobre `feat/p2-core`; `main` = `db93ca0` (P0b mergeado, PR #2).
- Fase: **P2** — P2-core en PR #3 (revisión independiente en curso). P2-native verificado: S1–S11 PASS en `8e91fd5` (`docs/evidence/p2.md`); pendiente PR y revisión.
- Operación en curso: ninguna. No hay juego ni build activo.
- Runtime (fuera del repo): `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). Instancia `mistbrn-dev`; build instalado `native-0a875da-0720e215dcea`.

## Siguiente acción exacta

1. Resolver la revisión de PR #3 (P2-core) y hacer merge.
2. Rebasar `feat/p2-native` sobre `main`, abrir su PR, revisión independiente y merge.
3. Siguiente fase: P3 (contenido: metales, viales, recetas) en `feat/p3-content`.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b | mergeado (PR #2) |
| P2-core | PR #3, revisión en curso |
| P2-native S1–S11 | PASS; PR pendiente |
| Windows x64 / Linux x64 | P10 |
