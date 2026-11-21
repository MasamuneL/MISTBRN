# Evidencia P0b — re-pin a Horizon 0a875da (macOS arm64)

Fecha: 2026-11-21. Commit probado: `c35d89144dc9a94fd34dbdc58ab9fe5dba526654` (`git_dirty=no` en receipt y report). Primera pasada en `60318a3`; se repitió completa tras corregir la revisión.
Equipo: macOS arm64 (Apple M3 Max), launcher 0.2.93, Rust del launcher 1.99.1. Instancia `mistbrn-dev` fijada a `nightly-2026-11-20-0a875da`.

## Build (G1)

`tools/build-native.sh p0b-5` → exit 0, `native-0a875da-e93082d7c828 built and selected for its instance`. Historial: `p0b-1` rechazado por layout (`unexpected package path: crates/allomancy-core/Cargo.toml`: el packager solo acepta `Cargo.toml`/`Cargo.lock` raíz, `build.rs` y `src/**`; el núcleo pasó a `mod/src/core/`); `p0b-2` falló por timeout de la descarga de 186 MB de herramientas del SDK (red, no código); `p0b-3`/`p0b-4` exit 0.

| Artefacto (p0b-5) | SHA256 |
|---|---|
| `libmistbrn.dylib` | `8218e4349655e1c2c6547f6136c94542debd9d1c43b8119cd378840c9571c382` |
| `horizon-client` (SDK sin cambios) | `012bf6dbfecb095bfb27f9b84452105639501cc5444aca15517360cd90547935` |

## Escenarios en juego

`tools/native-test.sh <escenario> <escenario>-p0b5` (y de nuevo `-p0b5b` con el arnés de la ronda 2: G3 exige `accepted=true tick_recorded=true`; las 3 PASS). Cada corrida usa `runs/<run>/` como carpeta de juego; `report.txt` registra build, SHA256 de la librería y commit. G2–G4 los comprueba el script en cada lanzamiento.

| Gate | Corrida | Observado | Resultado |
|---|---|---|---|
| G2 carga | las 3 corridas, cada lanzamiento | `MISTBRN event=loaded version=0.2.0` | PASS |
| G3 tick | las 3 corridas, cada lanzamiento | 3 × `MISTBRN event=player_tick accepted=true tick_recorded=true` | PASS |
| G4 aislamiento | las 3 corridas | `runs/<run>/saves` = solo `mistbrn-<escenario>`; listado de la home propia de Horizon idéntico antes y después (`diff` vacío) | PASS |
| G5 persistent_data | pd-roundtrip-p0b5 | l1 `phase=write ok=true`; l2 (tras `quit` y relanzar) `phase=read value=1,2,3` | PASS |
| G6 journal | tx-abort-p0b5 | `result=deliberate_abort count_before=3 inside_count=2 inside_pd=1 count_after=3 pd_unchanged=true` | PASS |
| G7 raíz visible | raw-root-p0b5 | `raw_nbt=some has_neoforgedata=true type=compound` | PASS (alcance abajo) |
| G8 calidad | `tools/check.sh` | fmt, clippy `--all-targets -D warnings` núcleo y adaptador (con el lock del SDK), 13 pruebas + doctest `compile_fail,E0451`, peso | PASS |

**Alcance de G7:** prueba que `entity().nbt()` expone `NeoForgeData` en el caso sano. No prueba que una raíz corrupta sea visible: `nbt()` serializa el estado en memoria. Lo decide S11 en P2-native.

El juego corre offline con un nombre generado: las pruebas no usan la cuenta Microsoft. El fichero del jugador está en `saves/<mundo>/players/data/<uuid>.dat` (gzip NBT, con `NeoForgeData` en la raíz).

## Peso del repositorio

Árbol versionado en `c35d891`: 31 ficheros, 104 187 B (antes ~300 KB en `main` y 2.4 MB en la rama P2 legacy). `.git` empaquetado: 1.04 MiB (historia previa, no reescrita).

## Revisión

Revisión independiente (opus vía Herdr) sobre `218c455`: changes requested con B1 (alcance de G7) y B2 (plan desalineado). Ambos corregidos en `c35d891`, junto con los no bloqueantes 1, 3, 4, 6–8, 10 y 11, 13 (sha fijado), 14–16, 18 y 19. El 2 queda parcial: se corrigió la variable vacía, y la sonda permanente pasa a comprobarse en P10. El 9 se completó tras la ronda 2: G3 exige `accepted=true tick_recorded=true`. El 12 usa el lock del SDK sin `--locked`, por diseño, porque cargo añade `mistbrn` y `allomancy-core`. Los relativos a Windows (5, 10, 17) están implementados pero sin probar y quedan para P10. Ronda 2 (`a40d8c2`): **approved**; sus no bloqueantes 1–3 se aplicaron y el 4 (herramienta de S11) es de P2.

## Límites

- Solo macOS arm64. Windows x64 y Linux x64 son gates de P10.
- La firma del SDK la verifica el launcher; los scripts verifican SHA256 (incluido un valor fijado para la fuente de la API).
- G6 prueba el rollback en memoria del journal; la coherencia en disco ante cortes es P2 (S3/S4).
