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
| macOS arm64 | Builds nativos y escenarios verificados en [P0b](evidence/p0b.md), [P2](evidence/p2.md) y [P3a](evidence/p3.md) |
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
| NBT crudo del jugador | `raw-root`, S11 | `player.entity().nbt()` incluye `NeoForgeData` (caso sano) y también una raíz corrupta escrita en el `.dat` con el juego cerrado (S11: `Int` visible): la guarda del adaptador la detecta y bloquea |
| Guardado transaccional | S1–S10 (`docs/evidence/p2.md`) | Inventario + `PlayerPersisted.mistbrn` en una `host.transaction`; cortes `kill -9` dan estado viejo o nuevo completo; datos ilegibles bloquean sin sobrescribir |
| Contenido `.hmod` | C1, C2 y C2b | Registro de ítems y recetas, fabricación y consumo en ambas manos. El crédito de reserva y el retorno del envase se guardan juntos |
| Fuentes en chunks existentes | C6 | Muestras al minar y vial al abrir un cofre de botín. Toque de Seda devolvió los bloques de mena sin añadir muestras |
| Cambio de mundo en el mismo proceso | C7 | Una carga por mundo. Al reabrir el segundo, la reserva, la transacción y los viales seguían en cero para el mismo jugador |
| Jugador sin cuenta | `horizon-ctl launch --game` | Juega offline con nombre generado; no usa la cuenta Microsoft |

## Límites conocidos

- Las muestras de mena dependen solo de la posición: colocar menas sacadas con Toque de Seda en posiciones elegibles y volver a minarlas con una herramienta que dé el mineral en bruto garantiza la muestra. Límite aceptado (cuesta una mena por muestra). La Fortuna multiplica la muestra como el bonus vanilla de menas (Fortuna III: 1–4, media 2,2), también determinista por posición. Solo se lee la Fortuna del parche de componentes de la herramienta (mesa, yunque, `/enchant`, botín): `block_drops` no da acceso a los componentes por defecto, así que la Fortuna que un datapack o mod ponga por defecto en una herramienta no cuenta.
- La cola de bebidas se deduplica por jugador y slot en cada tick porque la doc del hook de fin de uso indica que en el servidor puede dispararse para cada jugador del mundo. Sin probar con dos clientes (P9).
- El comando de prueba `/mistbrn-test` (P4) existe solo con `MISTBRN_TEST_SCENARIO=p4` exacto y para operadores de nivel 2. Su `credit` desbloquea un metal y acredita reservas sin vial en el guardado real del mundo abierto: el SDK no expone el nombre del mundo para limitarlo a los mundos de prueba `mistbrn-*`. Límite aceptado por el usuario (2026-11-21): es código de prueba tras una variable que un jugador no define, como los escenarios de P2, y un operador ya puede darse viales con `/give`. P10 comprueba que sin la variable el comando no existe.
- El tope de velocidad de P4 limita la velocidad que produce la fuerza: no frena una caída más rápida (la gravedad no se limita). El servidor recibe a veces dos movimientos del cliente en un tick, por eso los escenarios miden velocidad y paso medio en 4 ticks.
- Una moneda empujada daña según su desplazamiento real entre ticks. Los teletransportes que `entity_teleport` avisa (comandos dentro del mismo nivel por la ruta de Horizon) no se barren: el tramo previo al salto se evalúa y el vuelo sigue desde el destino. Uno que el SDK no avisa y que no supera la velocidad de la moneda al acabar el tick anterior + 0,1 bloques no es distinguible de un vuelo y podría dañar lo que cruza.
- Hierro y acero actúan en `level_tick_post`, por nivel: en `ServerTickEnd` solo un nivel está en contexto y los rayos y lecturas de entidades de los demás responden `None` (el Nether o el End perderían ítems y golpes de moneda). El consumo sigue en `ServerTickEnd`, después de la fuerza de ese tick.
- `bronze_earring` ocupa el slot de casco: no se lleva a la vez que un casco.
- La batería P3a se ejecutó en el servidor integrado de macOS arm64. El cooperativo y el servidor dedicado corresponden a una fase posterior. [Evidencia de P3a](evidence/p3.md)
- Las pruebas de corte de P3a cubren el inventario del jugador cuando tiene espacio para el envase. La durabilidad conjunta de la reserva y un envase soltado por desbordamiento sigue por verificar.
- `player.persistent_data()` convierte un `NeoForgeData` no-compound en vacío y `set_persistent_data` lo sobrescribe (fuente `native_player_persistent_data.rs`). El adaptador lee primero `entity().nbt()`, que sí muestra una raíz corrupta (S11), y bloquea al jugador sin escribir.
- En muerte y en la primera salida del End solo sobrevive `NeoForgeData.PlayerPersisted`.
- `tools/fetch-sdk.sh` verifica SHA256 contra `SHA256SUMS` y el de `api-source.tar.xz` contra un valor fijado en `tools/lib.sh`; la firma de la release la verifica el launcher al compilar.
- El empaquetado nativo solo admite `Cargo.toml`/`Cargo.lock` raíz, `build.rs` y `src/**` como fuentes (`horizon-mod-package/src/manifest.rs::source_file`); por eso el núcleo vive en `mod/src/core/`.
