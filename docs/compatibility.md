# Compatibilidad con Horizon

## Pin vigente

| Elemento | Valor |
|---|---|
| Horizon | `nightly-2026-11-20-0a875da` = `0a875da91dbb0fdcb7ba8c42c11341aa229af51a` |
| Juego | Minecraft: Java Edition 26.3 |
| SDK | release `sdk-0a875da…`, `package_format` 6, `content_api` 0.2 |
| horizon-mixins | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |
| Rust | 1.99.1 (el del launcher para builds nativos; `rust-toolchain.toml` para el núcleo) |
| Launcher usado | 0.2.93 |

El mod usa solo la API (`horizon-mod-api`): sin `[[native.*]]`, sin internals, sin dependencias de registro. Con un `uses.json` (que escribe `cargo horizon-mod build`), el launcher podría compilarlo en builds posteriores donde nada de lo usado cambió; este repo no re-pina sin decisión explícita.

## Plataformas

El código del mod y del núcleo no depende del sistema operativo. El SDK fijado publica slices para `aarch64-apple-darwin`, `x86_64-pc-windows-gnu` y `x86_64-unknown-linux-gnu` (más servidor dedicado Linux).

| Plataforma | Estado |
|---|---|
| macOS arm64 | Build nativo y escenarios P0b verificados (`docs/evidence/p0b.md`) |
| Windows x64 | Pendiente: gate obligatorio de P10 |
| Linux x64 | Pendiente: gate obligatorio de P10 |

## Capacidades verificadas en este pin

| Capacidad | Prueba | Resultado |
|---|---|---|
| Build nativo | `tools/build-native.sh` → `native-build --instance mistbrn-dev` | Compila y se instala como `native-0a875da-<clave>` |
| Carga | log del juego | `MISTBRN event=loaded version=0.2.0` |
| Tick servidor → núcleo | `server_tick` (ServerTickEnd, world_running) | `accepted=true tick_recorded=true` |
| Aislamiento de pruebas | `horizon-ctl launch` con cwd por corrida | Saves, logs y cachés en la carpeta de la corrida; instancia propia de Horizon intacta |
| `persistent_data` entre procesos | `pd-roundtrip` | `PlayerPersisted.mistbrn_probe` escrito, juego cerrado, leído igual tras relanzar |
| Journal de transacción | `tx-abort` | Inventario (slot 0) y `persistent_data` escritos dentro de `host.transaction` y restaurados al abortar |
| NBT crudo del jugador | `raw-root` | `player.entity().nbt()` incluye `NeoForgeData` (compound): P2 puede detectar una raíz corrupta que el getter normaliza |
| Jugador sin cuenta | `horizon-ctl launch --game` | Juega offline con nombre generado; no usa la cuenta Microsoft |

## Límites conocidos

- `player.persistent_data()` convierte un `NeoForgeData` no-compound en vacío y `set_persistent_data` lo sobrescribe (fuente `native_player_persistent_data.rs`). P2 lee primero el NBT crudo y bloquea al jugador si la raíz no es compound.
- En muerte y en la primera salida del End solo sobrevive `NeoForgeData.PlayerPersisted`.
- `tools/fetch-sdk.sh` verifica SHA256 contra `SHA256SUMS`; la firma de la release la verifica el launcher al compilar.
- El empaquetado nativo solo admite `Cargo.toml`/`Cargo.lock` raíz, `build.rs` y `src/**` como fuentes (`horizon-mod-package/src/manifest.rs::source_file`); por eso el núcleo vive en `mod/src/core/`.
