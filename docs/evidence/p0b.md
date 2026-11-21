# Evidencia P0b — re-pin a Horizon 0a875da (macOS arm64)

Fecha: 2026-11-21. Commit probado: `60318a3df63dfcab1d41fb7eb0a6a25840d28a57` (`git_dirty=no` en el receipt).
Equipo: macOS arm64 (Apple M3 Max), launcher 0.2.93, Rust del launcher 1.99.1. Instancia `mistbrn-dev` fijada a `nightly-2026-11-20-0a875da`.

## Build (G1)

`tools/build-native.sh p0b-4` → exit 0, `native-0a875da-d0e4f2016bd4 built and selected for its instance` (20 s; el primer build, `p0b-3`, tardó 137 s porque descargó 186 MB de herramientas del SDK; `p0b-2` falló por timeout de esa descarga, sin relación con el código).

| Artefacto | SHA256 |
|---|---|
| paquete `mistbrn` 0.2.0 (según `native-mods.json`) | `f70501eb50d171a4cf08168b1019671130c1a7916666c61ae42c3b66741cd2b2` |
| `libmistbrn.dylib` | `f6d6a8f78b62288623e5825e04e9d306261b33a19adbb0185eb70a4b39d2e0fc` |
| `horizon-client` (SDK sin cambios) | `012bf6dbfecb095bfb27f9b84452105639501cc5444aca15517360cd90547935` |

Hallazgo de empaquetado: el launcher solo acepta como fuentes `Cargo.toml`/`Cargo.lock` raíz, `build.rs` y `src/**` (`p0b-1`: `unexpected package path: crates/allomancy-core/Cargo.toml`). El núcleo se movió a `mod/src/core/`.

## Escenarios en juego

Lanzados con `tools/native-test.sh <escenario> <escenario>-p0b4`; cada corrida usa su propia carpeta de juego bajo `~/Library/Application Support/MISTBRN-dev/runs/`.

| Gate | Corrida | Línea observada | Resultado |
|---|---|---|---|
| G2 carga | pd-roundtrip-p0b4 l1 | `MISTBRN event=loaded version=0.2.0` | PASS |
| G3 tick | pd-roundtrip-p0b4 l1 | 3 × `MISTBRN event=player_tick accepted=true tick_recorded=true` | PASS |
| G4 aislamiento | todas | saves solo en `runs/<run>/saves/mistbrn-*`; la home propia de Horizon no tiene carpeta `saves` antes ni después | PASS |
| G5 persistent_data | pd-roundtrip-p0b4 | l1 `phase=write ok=true`; l2 (tras `quit` y relanzar) `phase=read value=1,2,3` | PASS |
| G6 journal | tx-abort-p0b4 | `result=deliberate_abort count_before=3 inside_count=2 count_after=3 pd_unchanged=true` | PASS |
| G7 raíz cruda | raw-root-p0b4 | `raw_nbt=some has_neoforgedata=true type=compound` | PASS → **sí**: P2 valida la raíz con `entity().nbt()` |
| G8 core | `tools/check.sh` | fmt, clippy `-D warnings` (núcleo y adaptador), 13 pruebas + 1 doctest, peso | PASS |

El juego corre offline con un nombre generado (`Player21872`): las pruebas no usan la cuenta Microsoft.

## Peso del repositorio

Árbol versionado tras P0b: 29 ficheros, ~99 KB (antes ~300 KB en `main`, 2.4 MB en la rama P2 legacy). `.git` empaquetado: 1.04 MiB (historia previa; no se reescribe).

## Límites

- Solo macOS arm64. Windows x64 y Linux x64 quedan como gates de P10.
- La firma `SHA256SUMS.sig` del SDK la verifica el launcher; `tools/fetch-sdk.sh` solo SHA256.
- G6 prueba el rollback en memoria del journal; la coherencia en disco ante cortes es P2 (S3/S4).
