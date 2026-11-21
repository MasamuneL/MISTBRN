# Historia anterior al re-pin (resumen)

Todo el trabajo previo está íntegro y congelado en la rama `legacy/windows-fdc87ae` (commit `930b96d`). No se hace merge de ella; se cita con `git show legacy/windows-fdc87ae:<ruta>`.

- **P0 (Windows x64, Horizon `fdc87ae`, API 411):** probe nativo con carga, ticks, entrada con chat/inventario, impulso con colisión, item propio, HUD, eventos de muerte/loot, mensajes servidor↔cliente, sonido encolado y un contador en `NeoForgeData` persistido entre procesos. Decisión C0: núcleo Rust independiente + adaptador nativo, sin fork.
- **P1:** contratos (`docs/contracts.md`), núcleo determinista con validación antes de mutación, 13 pruebas + doctest, adaptador mínimo con tres ticks reales. Revisión independiente aprobada; merge en `main` (`24b4b2c`).
- **P2 parcial (no integrado):** reservas con restos fraccionarios, codec JSON versionado con ledger, identidad de mundo, y una batería nativa extensa (transacciones, cortes, writer con GDB, ciclo de vida) que no llegó a cerrarse.

Motivo del re-pin (2026-11-21): en el equipo de desarrollo actual solo existen nightlies posteriores; el SDK `0a875da` cambió el formato de mod (proyecto plano `horizon-mod.toml` + `[native]`, `mod_api`) y añadió transacciones que cubren inventario y `persistent_data`. El adaptador legacy no compila contra él. El núcleo P2 legacy se porta (ver `docs/plan.md`).
