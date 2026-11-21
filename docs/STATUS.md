# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: `feat/p2-native` (worktree `~/Library/Application Support/MISTBRN-dev/wt-p2-core`); `main` = `6b981ab`.
- Fase: **P2** — P2-core mergeado (PR #3, `6b981ab`). P2-native PR #4: revisión ronda 1 changes requested (B1 salir muerto) corregida; S1–S11 + S5b PASS en `70308f9` (`docs/evidence/p2.md`). Pendiente: revisión ronda 2 y merge.
- Operación en curso: ninguna. No hay juego ni build activo.
- Runtime (fuera del repo): `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). Instancia `mistbrn-dev`; build instalado `native-0a875da-482083add929`.

## Siguiente acción exacta

1. Revisión ronda 2 de PR #4 y merge con `--match-head-commit`.
2. Borrar ramas `feat/p0b-repin`, `feat/p2-core`, `feat/p2-native` en remoto y el worktree.
3. Siguiente fase: P3 (contenido: metales, viales, recetas) en `feat/p3-content`.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b | mergeado (PR #2) |
| P2-core | mergeado (PR #3) |
| P2-native S1–S11 + S5b | PASS; PR #4 en revisión ronda 2 |
| Windows x64 / Linux x64 | P10 |
