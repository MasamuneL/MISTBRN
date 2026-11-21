# Estado — Masamune's MISTBRN

Actualizado: 2026-11-21.

- Plan vigente: `docs/plan.md`. Legacy congelado: rama `legacy/windows-fdc87ae` (`930b96d`).
- Pin: Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` (`nightly-2026-11-20-0a875da`), Rust 1.99.1.
- Rama activa: `feat/p0b-repin`; base `main` `24b4b2c`.
- Fase: **P0b** — implementada y verificada (G1–G8 PASS en `60318a3`, `docs/evidence/p0b.md`). Pendiente: revisión independiente y merge a `main`.
- Operación en curso: ninguna. No hay juego ni build activo.
- Runtime (fuera del repo): `~/Library/Application Support/MISTBRN-dev/` (`staged/`, `logs/`, `runs/`). Instancia `mistbrn-dev`; build instalado `native-0a875da-d0e4f2016bd4`.

## Siguiente acción exacta

1. Push de `feat/p0b-repin`, PR a `main`, revisión independiente (Herdr `hc run`, modelo opus), merge con `--match-head-commit`.
2. Rama `feat/p2-core` desde el merge: portar `reserves` (legacy sin serde) y nuevo `persistence` (codec MBRN v1, `TransactionSeq`), pruebas `--test reserves` y `--test persistence`.

## Gates abiertos

| Gate | Estado |
|---|---|
| P0b G1–G8 | PASS (macOS arm64) |
| P0b revisión | pendiente |
| P2-core | no iniciado |
| P2-native S1–S10 | no iniciado |
| Windows x64 / Linux x64 | P10 |
