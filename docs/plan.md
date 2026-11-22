# Masamune's MISTBRN — plan de ejecución

Aprobado el 2026-11-21. P0b, P2 y P3a están integrados. El plan de 2026-11-12 queda en `legacy/windows-fdc87ae`. [Estado vigente](STATUS.md)

## Contexto

El plan de 2026-11-12 y todo lo ejecutado (P0, P1 en main; P2 parcial en `codex/alomancia-p2-gates`) están fijados a Horizon `fdc87ae` en Windows. En esta Mac:
- Solo están disponibles nightlies 11-13 → 11-20; instalado y preparado `nightly-2026-11-20-0a875da` (commit `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`, juego 26.3, launcher 0.2.93). Su SDK firmado `sdk-0a875da…` existe (api-source, slices `aarch64-apple-darwin`, `package_format` 6, Rust 1.99.1, `content_api` 0.2, mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`).
- El formato de mod cambió: proyecto plano `horizon-mod.toml` + `[native]`, dependencia `horizon-mod-api` por path `.horizon/sdk/api/crates/horizon-mod-api`, `#![forbid(unsafe_code)]` rechazado, `Hooks`/`ServerHost`/`#[mod_api::horizon_mod]`. El adaptador P1 y el P2 nativo legacy (save.rs, world_guard, native_test, gates GDB) no compilan contra este SDK.
- Novedad útil: `host.transaction` registra en un solo journal inventario de jugador + `persistent_data` con rollback (docs `concepts/transactions.md`). `horizon-ctl` automatiza cliente/servidor integrado (launch, give, command, save, quit, inventory, events).
- Riesgo que persiste (leído en fuente `native_player_persistent_data.rs`): el getter convierte `NeoForgeData` no-compound en compound vacío y el setter lo sobrescribe; en muerte/primera salida del End solo sobrevive `NeoForgeData.PlayerPersisted`.
- La rama gates pesa 2.4 MB de árbol (169 evidencias, zip 484 KB, historiales de checkpoint). Main pesa ~300 KB, la mayoría evidencia Windows obsoleta.

Decisiones del usuario: congelar gates como legacy y borrar `codex/*`; desarrollo en macOS arm64; portar núcleo P2 legacy, reescribir adaptador. Nombre del mod **Masamune's MISTBRN**, id `mistbrn`.

**Multiplataforma (requisito):** el mod funciona en todo SO que soporte el SDK fijado (macOS arm64, Windows x64, Linux x64). Reglas: el código del mod y del núcleo no usa APIs, rutas ni comportamiento específicos de un SO; sin `cfg(target_os)` salvo justificación escrita en la PR; sin dependencias nativas; rutas solo vía la API del juego. Las herramientas (`tools/*.sh`) son bash portable (macOS, Linux, Git Bash en Windows) y resuelven rutas del launcher por SO en `tools/lib.sh`. Verificación: macOS arm64 en cada fase con build nativo; Windows x64 y Linux x64 son gates obligatorios de P10 (build + S1/S4/S5).

---

## Paso 0 — Congelar legacy y limpiar ramas (sin código nuevo)

1. Verificar que los dos diffs locales (`docs/execution/evidence/p0-initial-probe.rs`, `experiments/horizon-probe/bundle/api/src/lib.rs`) son solo CRLF: `git diff --ignore-cr-at-eol --stat` vacío (ya comprobado). Descartarlos con `git restore` (no son contenido; los ficheros desaparecen de main en P0b igualmente).
2. `git fetch origin` y comprobar `git merge-base --is-ancestor X origin/codex/alomancia-p2-gates` para `main`, `origin/codex/alomancia-p1-resume`, `origin/codex/alomancia-p2`. Si alguna NO es ancestro: detenerse y avisar, no borrar nada.
3. Crear `legacy/windows-fdc87ae` en `930b96d` (HEAD de gates), `git push origin legacy/windows-fdc87ae`, verificar con `git ls-remote` que el SHA remoto es `930b96d…`.
4. Borrar remotas `codex/alomancia-p1-resume`, `codex/alomancia-p2`, `codex/alomancia-p2-gates` (`git push origin --delete …`). Antes, `gh pr list --state open` debe no tener PRs con esas ramas como head; si los hay, parar y avisar.
5. `legacy/windows-fdc87ae` es de solo lectura: nunca se hace merge de ella ni se commitea en ella. Se cita como fuente con `git show legacy/windows-fdc87ae:<ruta>`.

## Reglas permanentes

### Git
- `main` solo recibe merges de PR. Una rama por fase: `feat/p0b-repin`, `feat/p2-core`, `feat/p2-native`, `feat/p3-content`, … Nombre `feat/<fase>-<tema>`; arreglos post-merge `fix/<tema>`.
- Condiciones de merge (todas): (a) `tools/check.sh` exit 0; (b) pruebas nativas de la fase PASS cuando aplique, con evidencia; (c) revisión independiente sin hallazgos bloqueantes; (d) PR con resumen, gates y limitaciones. Merge commit (no squash), con `gh pr merge --merge --match-head-commit <sha>`.
- Revisión independiente: al menos dos proveedores distintos, en sesiones de solo lectura por Herdr. Incluir ambos informes en el cuerpo de la PR y cerrar las sesiones al terminar.
- Commits con la línea `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Nunca `git add .`; staging explícito.

### Peso del repo (aplicado por `tools/check.sh`)
- Prohibido versionar: `.horizon/`, `target/`, `mod/Cargo.lock` (el mod no tiene dependencias de registro **propias**; las del SDK se resuelven con el `Cargo.lock` del SDK, que `tools/check.sh` copia temporalmente), logs, capturas, zips, `.dat/.mca`, binarios (`.dylib .so .dll .exe`), mundos, SDK, recursos de Minecraft.
- Imágenes: solo texturas originales del mod en `mod/assets/mistbrn/textures/**.png`, ≤ 16 KiB cada una (`.gitignore` las excluye del ignore de `*.png`; `tools/check.sh` rechaza cualquier otro PNG o uno mayor).
- `tools/check.sh` falla si: algún fichero versionado > 64 KiB; extensión prohibida (`zip jpg jpeg gif log dat dat_old mca dylib so dll exe jar tar xz gz`); PNG fuera de texturas o > 16 KiB; árbol versionado (index) > 1 MiB.
- Evidencia: un `docs/evidence/<fase>.md` por fase, ≤ 16 KiB, con comandos, SHA del commit probado, SHA256 del artefacto, resultados por escenario y límites. Logs crudos quedan fuera del repo en el runtime root.
- Estado de reanudación: `docs/STATUS.md` ≤ 60 líneas (sustituye CHECKPOINT.md, progress.md, HANDOFF, PLAN-ADJUSTMENTS). Contiene: rama, commit base, fase y paso, operación en curso, último resultado, siguiente acción exacta.

### Rutas fuera del repo
- Runtime root (por SO en `tools/lib.sh`; macOS: `~/Library/Application Support/MISTBRN-dev/`, Linux: `$XDG_DATA_HOME/MISTBRN-dev`, Windows: `%LOCALAPPDATA%\MISTBRN-dev`) con `staged/<build>/` y `logs/<build>/` (builds) y `runs/<run>/` (una carpeta de juego por corrida). Nunca dentro del repo (`runtime_root` lo rechaza).
- Instancia dedicada del launcher: `mistbrn-dev`, fijada a `nightly-2026-11-20-0a875da`. La instancia propia de Horizon y sus mundos no se tocan. Cada corrida usa como carpeta de juego su `runs/<run>/` (el juego usa el directorio de trabajo como home): saves, logs y cachés quedan ahí. Mundo de prueba: `mistbrn-<escenario>`, uno por corrida.

### Toolchain
- `rust-toolchain.toml`: `channel = "1.99.1"`, `profile = "minimal"`, `components = ["rustfmt","clippy"]`, sin `targets`. Rustup descarga 1.99.1 (fuera del repo). Builds nativos usan el Rust del launcher (`horizon-launcher rust install`; en macOS arm64 queda en `…/horizon-launcher/rust/1.99.1-aarch64-apple-darwin`).
- El núcleo y el mod no tienen dependencias de registro propias (ni serde ni serde_json). Motivo: el launcher exige aprobación de build scripts/proc macros de registro ("Registry dependency compilation was not approved") sin flag CLI documentado, y los tipos de registro no son intercambiables con los del SDK. Cualquier codec es propio.

---

## P0b — Re-pin a 0a875da y esqueleto nativo (rama `feat/p0b-repin`)

### Layout nuevo
```text
Cargo.toml                    # workspace: members ["mod/src/core"], exclude ["mod"]
rust-toolchain.toml
mod/                          # el paquete .hmod, plano
  horizon-mod.toml            # id "mistbrn", name "Masamune's MISTBRN", api "0.2", [native] game "26.3", horizon 0a875da…, mixins e59da909…
  Cargo.toml                  # package mistbrn; deps: horizon-mod-api (path .horizon/sdk/…), allomancy-core (path src/core)
  src/lib.rs                  # #![deny(unsafe_code)], #[mod_api::horizon_mod] impl Hooks
  src/{bridge,scenario}.rs    # P0b; src/save.rs llega en P2-native
  src/core/                   # crate allomancy-core; el launcher solo empaqueta Cargo.toml/Cargo.lock raíz, build.rs y src/** (manifest.rs::source_file)
  assets/mistbrn/ data/mistbrn/   # vacíos hasta P3
tools/{fetch-sdk.sh,build-native.sh,native-test.sh,check.sh}
docs/{design.md,plan.md,contracts.md,compatibility.md,STATUS.md,history.md,evidence/}
```

### Pasos
1. `git mv crates/allomancy-core mod/src/core`; quitar serde: derives `Serialize/Deserialize` fuera de types/state/commands; doctest `compile_fail` de `VerifiedEvent` se reescribe sin serde_json (construcción por literal de campos privados). Tests P1 deben seguir 13/13.
2. Eliminar de main lo exclusivo de Windows/legacy: `experiments/`, `tools/*.ps1`, `integration/`, `crates/allomancy-horizon/`, `docs/execution/`, `docs/superpowers/`. Crear `docs/history.md` (≤ 3 KiB): qué se probó en P0/P1 y que todo está en `legacy/windows-fdc87ae`.
3. `docs/design.md` = spec actual con plataforma cambiada a "macOS arm64 primero; Windows x64 verificado antes de publicar". `docs/plan.md` = este plan. `docs/compatibility.md` reescrito para 0a875da. `docs/contracts.md` actualizado (sin serde; tipos P2 añadidos en P2-core).
4. `tools/fetch-sdk.sh`: descarga `sdk.json`, `SHA256SUMS`, `api-source.tar.xz`, `surface.json` de la release `sdk-<commit>`, verifica SHA256 contra `SHA256SUMS` y además el de `api-source.tar.xz` contra el valor fijado en `tools/lib.sh` (`SDK_API_SOURCE_SHA256`), desempaqueta en `mod/.horizon/sdk/api/`, escribe `mod/.horizon/sdk/build` al final. Límite documentado: la firma `SHA256SUMS.sig` la verifica el launcher en `native-build`, no este script.
5. `tools/build-native.sh <build-name>`: copia limpia a `staged/<build-name>/mistbrn/` de `horizon-mod.toml Cargo.toml src assets data` (excluye `.horizon target Cargo.lock`); registra SHA256 de cada input en `staged/<build-name>/receipt.txt`; ejecuta `horizon-launcher native-build <staged> --instance mistbrn-dev --trust-native-code` con log en `logs/<build-name>/build.log`; rechaza si `<build-name>` ya existe.
6. Crear instancia: `horizon-launcher instance create mistbrn-dev` + `instance version mistbrn-dev nightly-2026-11-20-0a875da`. `native-build --instance mistbrn-dev` instala `native-0a875da-<clave>` y lo selecciona para esa instancia; `native-test.sh` usa exactamente el build que la instancia juega.
7. `tools/native-test.sh <escenario> [run]`: crea `runs/<run>/` y lanza desde ahí `horizon-ctl launch --game <cliente del build> --resources <recursos preparados de la versión base> --world mistbrn-<escenario> --creative --wait`, con `MISTBRN_TEST_SCENARIO=<escenario>` cuando el paso lo pide. Sockets en una carpeta temporal corta (límite ~104 bytes de las rutas de socket Unix); los logs se copian a `runs/<run>/`. Registra en `report.txt` build, SHA256 de la librería del mod y commit. Comprueba en cada lanzamiento G2 y G3, al final G4, y las líneas `MISTBRN_TEST key=value` esperadas; exit 0 solo si todo coincide. Cortes duros con `kill -9` (Windows: `taskkill //F`).
8. Adaptador mínimo: solo `server_tick` (fase `ServerTickEnd { world_running: true }`, autoridad `IntegratedServer` o `DedicatedServer`; ambos son servidores autoritativos) llama `advance_tick` y el apagado de seguridad. Los hooks de ciclo de vida (`player_logged_in/out`, `player_died`) llegan en P2-native. Escenarios de prueba activados solo por la variable de entorno `MISTBRN_TEST_SCENARIO` (no hay features de Cargo: `E_NATIVE_FEATURES`). Sin la variable, el código de escenarios no hace nada; P10 lo verifica.

### Gates P0b (todos PASS para merge)
| Gate | Criterio exacto |
|---|---|
| G1 build | `build-native.sh` exit 0; log contiene compilación de `mistbrn`; SHA256 del ejecutable registrado |
| G2 carga | Log del juego contiene `MISTBRN event=loaded version=0.2.0` |
| G3 tick | ≥ 3 líneas `MISTBRN event=player_tick` por lanzamiento, comprobado por `native-test.sh` (en P0b `accepted=true tick_recorded=true`; desde P2-native `state=active` o `state=blocked`) |
| G4 aislamiento | `runs/<run>/saves/` contiene exactamente `mistbrn-<escenario>` (comprobado por `native-test.sh`), y la home propia de Horizon (`horizon-launcher --json instance list`, instancia `horizon`) no gana carpetas de mundos (comprobado a mano antes/después y anotado en la evidencia) |
| G5 persistent_data | Escenario `pd-roundtrip`: escribe `NeoForgeData.PlayerPersisted.mistbrn_probe = ByteArray[1,2,3]`, `quit`, relanzar, lee mismo valor |
| G6 journal | Escenario `tx-abort`: dentro de `host.transaction`, quita 1 `minecraft:iron_nugget` del slot 0 y escribe persistent_data, y lee ambos (`inside_count=2 inside_pd=1`); devuelve `Err`; tras el cierre inventario y persistent_data idénticos a antes |
| G7 raíz cruda | Escenario `raw-root`: registra si `player.entity().nbt(host)` contiene `NeoForgeData` (caso sano). Solo prueba que la clave es visible; si una raíz **corrupta** también lo es se prueba en P2-native (S11) |
| G8 core | `tools/check.sh` exit 0: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, fmt y clippy `--all-targets -D warnings` del adaptador contra el SDK, y control de peso |

Si G1 falla por exigencias del launcher (lock, formato, refusal `E_*`): corregir según el código de error documentado; tras dos intentos fallidos con la misma causa, parar y reportar. Si G2–G7 revela que una API no existe o no se comporta como dice la doc: parar y reportar al usuario antes de cambiar backend. Nunca fork de Horizon sin aprobación.

---

## P2-core — Reservas y guardado en el núcleo (rama `feat/p2-core`, tras merge de P0b)

Fuente a portar: `legacy/windows-fdc87ae:crates/allomancy-core/src/{reserves,persistence}.rs`, `tests/{reserves,persistence}.rs`, `docs/{reserves,persistence}.md`. Se porta la semántica y las pruebas; se quita serde y se cambian los puntos marcados abajo.

### Reservas (`reserves.rs`) — semántica legacy sin cambios
- `ReserveRules { consumption_milli_per_tick:[u64;8], base_capacity_milli:[u64;8], reduction_milli:u16 (0..=200), capacity_bonus_milli:u16 (0..=500) }`; exceso se rechaza, no se recorta. `capacity = floor(base × (1000+bonus) / 1000)`.
- `ReserveAccounting { remainder:[u16;8] }`, restos en 0..1000.
- `consume_reserves(state, acc, rules, elapsed_ticks)`: numerador `u128 = consumo × (1000−reducción) × mult × ticks + resto`; mult 1 encendido, 3 avivado; consume `numerador/1000`, guarda `numerador%1000`; al agotarse deja 0, apaga, borra resto, devuelve metales agotados en orden `Metal::ALL`; overflow rechaza el lote entero sin tocar estado. Agrupar o separar ticks da el mismo resultado.
- `credit_reserve`: rechaza negativo, overflow y exceder capacidad (sin crédito parcial). Cero es idempotente.
- `clear_reserves_on_death`: reservas y restos a cero, todo apagado, desbloqueos intactos.
- Reglas por defecto del adaptador P2 (ensayo, no balance): consumo 10 milli/tick en los 8 metales, capacidad base 100 000 milli, reducción 0, bonus 0.

### Guardado (`persistence.rs`) — cambios respecto a legacy
- `SavedPlayer { state, accounting, rules, revision:u64, last_transaction:u64 }`. Se elimina el ledger de 4096 entradas (bloqueaba al llenarse).
- Transacciones: `TransactionSeq(u64)` emitida por el núcleo, `prepare_credit(&saved, metal, amount) -> PreparedCredit` usa `seq = last_transaction + 1` y `base_revision = revision`; `commit_credit(&mut saved, prepared)` exige `revision == base_revision` (si no, `StalePreparation`) y fija `last_transaction = seq`, `revision += 1`. Una seq ≤ `last_transaction` se rechaza siempre (`Duplicate`). Retención O(1): no existe forma de reaplicar una transacción antigua. Los `TransactionId` de `Command` quedan reservados para P7 (peticiones del cliente) y nunca autorizan un crédito.
- Codec binario propio, esquema 1:
  `b"MBRN"` | `u16 LE schema` | `u32 LE payload_len` | payload | `u32 LE CRC-32/IEEE` sobre todo lo anterior. Payload: campos en orden fijo, enteros LE, enums como `u8` con valores explícitos, ids como `u16 len` + UTF-8. Máximo 64 KiB.
  Decode estricto: magia, longitud exacta, CRC, sin bytes sobrantes, enums conocidos, `validate_reserves` completo, identidad de jugador igual a la esperada. `schema > 1` → `UnsupportedVersion` sin escritura; `schema < 1` → `UnsupportedVersion`.
- Migraciones: esquema 1 es el primero real (el `v0` legacy era sintético y no se porta). Se versiona `tests/fixtures/saved-player-v1.bin` (< 1 KiB, generado por un test con `--ignored` y revisado). P5, al añadir conocimiento/nodos, crea esquema 2 y debe probar migración 1→2 con ese fixture.
- `on_death(saved)` = `clear_reserves_on_death` + revisión+1; `on_reconnect(saved)` = apagar combustiones, restos intactos. Ambos idempotentes en estado.
- Identidad: `PlayerId` = UUID en hex de 32 caracteres; `WorldId` constante `local` en P2. El aislamiento entre mundos lo dan los ficheros de jugador por mundo de Minecraft; trasplantar un `.dat` entre mundos queda fuera de alcance (documentado).
- Se elimina `write_export_atomic`/`read_export_with_backup` y `world_identity.rs`: el núcleo no escribe ficheros; la durabilidad la da el writer del juego (prueba en P2-native).

### Pruebas core (ejecutar por separado)
`cargo test -p allomancy-core --test reserves` y `--test persistence`, cubriendo como mínimo: agotamiento exacto, ticks agrupados = separados, capacidad y bonus en límites (200/201, 500/501), avivado ×3, varias combustiones a la vez, crédito negativo/overflow/capacidad, round-trip, cada byte truncado (0..len-1) rechazado, CRC alterado, bytes sobrantes, esquema 2 y 0 rechazados, identidad ajena, commit de preparación obsoleta, seq duplicada, corte simulado antes de commit (estado previo intacto) y después (exactamente una aplicación), muerte y reconexión.

Merge con G8 + revisión.

---

## P2-native — Guardado real en Horizon (rama `feat/p2-native`, tras merge de P2-core)

### Backend (decidido)
Bytes del codec en `NeoForgeData.PlayerPersisted.mistbrn` como `ByteArray`.
Motivos: (1) `set_persistent_data` e inventario de jugador están en el mismo journal de `host.transaction`; los attachments de jugador no aparecen en esa tabla; (2) `PlayerPersisted` sobrevive a la muerte y a la primera salida del End; (3) inventario y NeoForgeData están en el mismo `.dat`, que el juego reemplaza entero.

Reglas de lectura/escritura en `mod/src/save.rs`:
- Leer siempre el compound completo, modificar solo la clave `mistbrn` dentro de `PlayerPersisted`, conservar las demás claves y namespaces, escribir el compound completo.
- `PlayerPersisted` presente pero no compound, o `mistbrn` presente pero no `ByteArray`, o decode con error → jugador en estado **Bloqueado**: sin escrituras del mod para ese jugador en toda la sesión, poderes desactivados, mensaje en chat `Alomancia: datos guardados ilegibles; no se modificarán.` y línea `MISTBRN event=save_blocked reason=<…>`.
- Raíz `NeoForgeData` no-compound: antes de cualquier escritura se lee `entity().nbt()`; si contiene `NeoForgeData` y no es compound → Bloqueado. Que esta guarda funcione se prueba con S11. Si S11 muestra que el juego normaliza la raíz al cargar (`nbt()` la ve compound), la guarda no detecta nada: se mantiene como defensa barata, la evidencia lo declara **límite aceptado** (solo una edición externa del fichero puede producir esa raíz) y ningún documento afirma detección.
- Ausencia real de `mistbrn` → `SavedPlayer::new` y escritura inicial.
- Toda mutación económica = `host.transaction { modificar inventario (si aplica) + set_persistent_data }`; `Err` en cualquier paso aborta ambos; el estado en memoria se actualiza solo tras `Ok` del commit.
- Consumo por ticks: `elapsed = 1` por cada `ServerTickEnd{world_running:true}`; no se cobra tiempo offline ni pausado. Estado sucio se persiste cada 20 ticks, al agotarse un metal, en muerte y tras cada transacción. En `player_logged_out` se intenta escribir solo si el handle del jugador sigue resolviendo (al cerrar el servidor sí; en una desconexión normal el jugador ya salió de la lista). Pérdida máxima ante corte o desconexión: 20 ticks de consumo (a favor del jugador).
- Muerte: en `player_died` → `on_death` en memoria e intento de escritura (el juego suele rechazarla con `Dead`; se reintenta cada 20 ticks). La garantía la da `player_clone` con `was_death`: al reaparecer, también tras salir desde la pantalla de muerte o reiniciar el servidor, se carga el guardado que `PlayerPersisted` trajo, se aplica `on_death` (idempotente) y se escribe en el jugador nuevo. Reconexión: en `player_logged_in` (o en el primer tick con el jugador presente) → leer, `on_reconnect`, escribir. Un corte del proceso entre la muerte y esa escritura devuelve al jugador al último guardado del juego: posterior a la muerte → reaparece muerto y se reaplica; anterior → todo el jugador, inventario y reservas, vuelve a ese momento.

### Escenarios nativos (via `tools/native-test.sh`; cada uno en mundo nuevo)
Disparador de prueba "beber": con `MISTBRN_TEST_SCENARIO=drink`, en el primer tick con el jugador cargado y una `minecraft:iron_nugget` en el slot 0, desbloquear hierro y acreditar 25 000 milli de hierro por esa pepita en una transacción (sustituto del vial de P3, solo en escenarios). `MISTBRN_TEST_SCENARIO=report` registra el estado cargado, las pepitas y un hash de los bytes guardados. Invariante comprobada en todos: `pepitas × 25 000 + reserva_hierro` constante salvo consumo por quema, y `revision`/`last_transaction` coherentes con ella.

| Escenario | Pasos | Esperado |
|---|---|---|
| S1 roundtrip | give 3 pepitas, beber, `save`, `quit`, relanzar | 2 pepitas, reserva 25 000, last_transaction 1 |
| S2 abort | beber con fallo inyectado tras escribir inventario y guardado (`drink-fail`), `save`, `quit`, relanzar | 3 pepitas, reserva 0, last_transaction 0 |
| S3 corte antes de guardar | lanzamiento 1: give 3, `save`, `quit` (estado A en disco); lanzamiento 2: beber y `kill -9` < 30 s tras abrir (antes del autosave de 6000 ticks); lanzamiento 3: report | estado A completo: 3 pepitas, reserva 0, last_transaction 0 (nunca mezcla) |
| S4 corte después de guardar | como S3 pero `save` antes del `kill -9` | estado nuevo completo: 2 pepitas, 25 000, last_transaction 1 |
| S5 muerte | beber, `command kill @s`, `respawn` | reserva 0, metales desbloqueados intactos, last_transaction 1 |
| S5b salir muerto | beber, `kill @s`, `quit` sin reaparecer; relanzar, `respawn`, `save`, `quit`; report | `event=respawn_death_applied`; reserva 0, desbloqueos intactos, last_transaction 1 |
| S6 reconexión | escenario fija hierro Burning, `quit`, relanzar | combustión Off, reserva y resto conservados |
| S7 dimensión | beber, `command execute in minecraft:the_nether run tp @s 0 100 0` | datos idénticos |
| S8 agotamiento | crédito 1 000, Burning, esperar 100 ticks (`wait 100`) | reserva 0, Off, `event=exhausted metal=iron` |
| S9 corrupto | escenario escribe `mistbrn` con CRC inválido, `quit`, relanzar, `save`, `quit` | `save_blocked reason=crc`; bytes idénticos tras el segundo guardado |
| S10 esquema futuro | igual con `schema=2` | `save_blocked reason=unsupported_version:2`; bytes intactos |
| S11 raíz corrupta | con el juego cerrado, `tools/nbt-poke.py` (Python 3, solo biblioteca estándar, versionado, < 8 KiB; única dependencia de pruebas fuera de bash/jq) reemplaza `NeoForgeData` del jugador por un `Int` en el fichero de jugador del mundo; relanzar con `report` | `state=blocked reason=root_not_compound` para el UUID que `report` registró (el fichero editado es exactamente `players/data/<uuid>.dat` de ese jugador) |

S3 y S4 se repiten 3 veces cada uno; los 6 deben pasar. Evidencia en `docs/evidence/p2.md`. Merge de P2 completo cuando S1–S10 y S5b PASS, S11 ejecutado con su resultado documentado, G8 y revisión. P2 queda cerrado; no se añaden gates de writer/GDB (el reemplazo del `.dat` lo garantiza el juego; S3/S4 prueban coherencia viejo/nuevo).

---

## Fases siguientes (cada una: rama propia, pruebas, revisión, merge)

Regla común: al empezar una fase, comprobar en `surface.json` del SDK que existen los hooks/métodos que necesita; si falta algo esencial, parar y presentar opciones al usuario (nunca fork sin aprobación). Si Horizon publica un nightly nuevo, no se re-pina salvo decisión del usuario; el re-pin es una fase propia con G1–G8.

| Fase | Rama | Depende | Entrega | Criterio de cierre |
|---|---|---|---|---|
| P3a contenido | `feat/p3-content` | P2 | metales, viales, recetas, muestras en mundos existentes, arte original y traducciones | Integrado en `7f1d159` con 24 corridas y dos revisiones. [Evidencia](evidence/p3.md) |
| P3b accesorios | `feat/p3b-accessories` | P3a | monedas, capa de bruma y accesorios (ver abajo) | Validador de contenido; en juego: fabricar cada accesorio y equipar la capa; pendientes de la revisión de P3a cerrados |
| P4 hierro/acero | `feat/p4-iron-steel` | P2, P3a | tirón/empujón sobre metales y anclajes, límites de velocidad, colisión del motor, cancelación por muerte/dimensión/objetivo inválido | Escenarios horizon-ctl con posiciones medidas: sin atravesar bloques, velocidad ≤ tope, apagado al agotarse |
| P7a controles/HUD mínimo | `feat/p7a-hud` | P4 | V selecciona pareja, Z/X activa, R aviva, G apaga; HUD de reservas; sin activación con chat/menú | Escenario con chat e inventario abiertos sin activación; captura local revisada (no versionada) |
| V1 demo | — (validación) | P3a, P4, P7a | preparar metal, beber, usar hierro y acero, consultar la reserva y continuar tras guardar | Lista V1 de legacy PLAN-ADJUSTMENTS y evaluación humana del usuario |
| P5 habilidades/investigación | `feat/p5-progress` | V1 | árboles, experiencia por práctica, investigación con pistas; **esquema de guardado 2 + migración 1→2** | Tests core + fixture v1 migrado; escenario en juego |
| P6 hitos | `feat/p6-milestones` | P5 | fragmentos ligados a bastión, guardián, ciudad antigua, dragona; reclamo único por jugador | Duplicados rechazados en core y en juego |
| P7b UI completa | `feat/p7b-ui` | P5 | libro, árboles, pistas | Revisión visual del usuario |
| P8 tres parejas restantes | `feat/p8-<pareja>` | P7b | estaño/peltre, zinc/latón, cobre/bronce | Pruebas por pareja |
| P9 cooperativo | `feat/p9-coop` | P8 | autoridad servidor con 2 clientes, server export dedicado | Dos clientes reales; si no se logra, se entrega individual y se documenta |
| P10 publicación | `feat/p10-release` | P8 (P9 opcional) | `.hmod`, verificación **Windows x64 y Linux x64** con el mismo pin (incluye `tools/*.sh` en Git Bash: `cygpath`, `tasklist`/`taskkill`, `jq` instalado), código de escenarios inactivo sin variable y sin sondas `mistbrn_probe*` en un mundo jugado sin ella, licencias de assets | Build y S1/S4/S5 en Windows x64 y Linux x64 PASS; paquete sin recursos de terceros |

### P3a — Cadena de metales

- 25 ítems. `vial_empty`, ocho `vial_<metal>`, ocho `<metal>_flakes`, seis lingotes de estaño, zinc, acero, peltre, latón y bronce, y `raw_tin` / `raw_zinc`. Hierro y cobre usan lingotes vanilla. Texturas propias de 16 × 16, nombres en inglés y español y pestaña creativa Alomancia.
- 25 recetas. Dos vidrios en vertical rinden dos viales vacíos. Cada lingote con un bloque de piedra (`#minecraft:stone_crafting_materials`) rinde dos virutas (desde P3b; un lingote solo chocaba con las pepitas vanilla y lingote + pedernal con el mechero) y cada viruta llena un vial. Un hierro y dos carbones rinden un acero. Tres estaños y un cobre rinden cuatro peltres. Dos cobres y un zinc rinden tres latones. Tres cobres y un estaño rinden cuatro bronces. Estaño y zinc en bruto se refinan en horno y alto horno.
- En chunks existentes, la minería de cobre da un estaño adicional en 1 de 4 posiciones, y la de hierro da un zinc en 1 de 3. Incluye pizarra profunda. Se exige el mineral en bruto vanilla en el botín, por lo que Toque de Seda y herramientas inadecuadas no dan muestras. Al abrir por primera vez determinados cofres de mazmorra, mina o ruina submarina, 1 de 8 posiciones añade un vial de descubrimiento. Las tasas son deterministas por dimensión y posición.
- En supervivencia o aventura, el uso completo de 32 ticks intenta acreditar 25 000 milésimas. El servidor comprueba el mismo slot y stack y consume un vial, devuelve el envase y escribe el guardado en una transacción. El desbloqueo de poderes corresponde al sistema de progresión. Las menas propias quedan para una fase posterior.
- Gates: `tools/check.sh`, C1 fabricación, C2 beber y persistir, C2b mano secundaria, C3/C4 cortes antes/después de guardar (3 corridas por caso), C5 rollback, C6 minería y botín, C7 cambio de mundo en el mismo proceso, regresión S1–S11 + S5b y dos revisiones independientes.

### P3b — Accesorios

Seis ítems con receta, arte original provisional y traducciones. Valores provisionales, ajustables con pruebas:

| Ítem | Base | Valores | Receta |
|---|---|---|---|
| `clip` | moneda | pila 64 | 1 pepita de cobre → 1 |
| `boxing` | moneda | pila 64 | 1 pepita de oro → 1 |
| `glass_dagger` | espada | daño 4, velocidad 2,4, 48 usos, sin reparación | vidrio sobre palo (2 × 2) |
| `dueling_cane` | espada | daño 3, velocidad 2,0, 131 usos | tronco, palo, palo en columna (mesa) |
| `mistcloak` | equipable en pecho, sin armadura | pila 1, capa de equipamiento propia | lana gris con forma de peto (mesa) |
| `bronze_earring` | equipable en cabeza, sin armadura | pila 1, modelo diminuto junto a la cabeza | lingote de bronce + hilo |

Ninguna receta del mod coincide con una vanilla: un lingote solo sigue dando pepitas (D1 lo comprueba). También cierra los pendientes de la revisión de P3a: una bebida por jugador y slot en cada tick, mensajes con claves de idioma y el límite de Toque de Seda documentado. P4 incorporará la interacción de las monedas con los empujones de acero y los cuchillos arrojadizos (necesitan entidad de proyectil propia). Escenarios: D1 fabrica los seis, D2 equipa capa y pendiente y los conserva tras relanzar, D3 llena la reserva y rechaza el quinto vial con mensaje traducido; regresión C2, C2b, C5 y S9.

### P4 — Hierro y acero

Aprobado el 2026-11-21. Superficie del pin sin fork: `PlayerRef::set_velocity` (el cliente mueve al jugador con su colisión), `EntityRef::{set_velocity,push,velocity,position,eye_position,view_vector,on_ground,reset_fall_distance,hurt}`, `LevelRef::{clip,block,ray_trace_entities,entities_in}`, `host.item_entity`, `register_commands`/`run_mod_command`, `player_died`, `player_changed_dimension`. No hay lectura de tags de bloque: la lista de metales es explícita.

- **Metal** (`core/src/metals.rs`, ids exactos): bloques de hierro, oro y cobre (incluidas variantes oxidadas, enceradas y cortadas), raíles, calderos, yunques, tolva, campana, faroles, placas de presión de peso, `netherite_block`; ítems en el suelo de hierro, oro, cobre, malla y netherita, y del mod (lingotes, virutas, `clip`, `boxing`). Fuera de P4: mobs con armadura, gólems y minecarts.
- **Física** (`core/src/physics.rs`, pura): `dir` jugador→objetivo; hierro tira (+dir), acero empuja (−dir). `a(d) = A·(1 − d/R)`, A = 0,12 bloques/tick² (0,24 avivado), R = 16 (24 avivado). Bloque = ancla (todo el Δv al jugador); ítem = masa 0,05 (Δv al ítem) salvo ítem `on_ground` empujado hacia abajo (`dir.y < −0,5`): ancla (salto con moneda). Topes: la fuerza nunca lleva al jugador por encima de 1,0 bloque/tick (1,4 avivado), pero tampoco frena una caída más rápida que eso (la gravedad no se limita); ítem ≤ 2,0 absoluto. Moneda empujada: se sigue su desplazamiento real entre ticks (ya detenido por el motor en los bloques) hasta 40 ticks tras el último empujón, resuelto al inicio de cada tick antes de los empujones nuevos (un teletransporte por comando, avisado por `entity_teleport`, reinicia el tramo en el destino; como defensa ante otros, un tramo más largo que la velocidad de la moneda al acabar el tick anterior + 0,1 no daña); si a > 0,5 bloques/tick cruza un ser vivo, daño `2 + 4·|v|` (`minecraft:thrown`) aplicado tras liberar los locks del tick, la moneda se para y no vuelve a ser objetivo. Mientras la fuerza levanta al jugador se resta 0,15 × vy (amortiguación cerca de la crítica: sin ella N3 oscilaba 8 bloques por el retraso de la velocidad reportada por el cliente) y se llama `reset_fall_distance`.
- **Objetivo**: al empezar a arder o sin objetivo, rayo desde los ojos según la mirada hasta R; el impacto metálico más cercano (bloque por `clip`, entidad por `ray_trace_entities` con margen 0,3); no atraviesa bloques. Se mantiene mientras sea válido; si no, `target_cleared` y la combustión sigue.
- **Núcleo**: `Activate`/`Flare` habilitados solo para Iron y Steel. Hasta P7a, arder = actuar; si arden ambos, gana el último activado. Consumo de P2 (10 milésimas/tick, ×3 avivado), con o sin objetivo. Muerte limpia objetivo y reserva; cambio de dimensión limpia objetivo y apaga hierro y acero.
- **Activación de prueba**: comando `/mistbrn-test burn <iron|steel> [flare]`, `/mistbrn-test off`, `/mistbrn-test credit <metal> <milli>` (desbloquea y acredita sin vial). Se registra solo con `MISTBRN_TEST_SCENARIO` definida (valor `p4` en estos escenarios).

Contrato de log (lo leen los escenarios):
```text
MISTBRN event=burn metal=<iron|steel> burn=<burning|flared|off> result=<ok|<RuleError>>
MISTBRN event=target_acquired kind=<block|item> id=<namespace:path> at=<x>,<y>,<z>
MISTBRN event=target_cleared reason=<broken|gone|range|dimension|death|off|exhausted>
MISTBRN p4 tick=<n> metal=<iron|steel> pos=<x>,<y>,<z> vel=<x>,<y>,<z> target=<block|item|none> dist=<d> dv=<x>,<y>,<z> anchor=<true|false>
MISTBRN p4item tick=<n> entity=<id> pos=<x>,<y>,<z> vel=<x>,<y>,<z> speed=<v>
MISTBRN p4coin tick=<n> entity=<id> pos=<x>,<y>,<z>
MISTBRN event=coin_hit entity=<namespace:path> speed=<v> damage=<f>
MISTBRN event=exhausted metal=<iron|steel>
```
Flotantes con 3 decimales; `pos`/`vel` del jugador son las del servidor en ese tick.

| Escenario | Montaje (plataforma de piedra en y = 100, creativo salvo N3) | Esperado |
|---|---|---|
| N1 tirón | `iron_block` a 12 bloques en +x, mirándolo, `burn iron` | x crece; velocidad ≤ 1,05 y Δx medio en ventanas de 4 ticks ≤ 1,05 (el servidor recibe a veces dos movimientos del cliente en un tick); x final ≤ cara − 0,25; nunca dentro del bloque |
| N2 muro | `iron_block` delante, vidrio 4 bloques detrás, `burn steel` | x se para contra el vidrio (cara + 0,3 ± 0,05) |
| N3 moneda | supervivencia, `clip` bajo los pies, mirando abajo, `burn steel` | y sube y flota (oscilación ≤ 1); vy ≤ tope; al apagar cae |
| N4 ítem | `iron_ingot` a 6 bloques, `burn iron` | el ítem llega y se recoge; el jugador se mueve ≤ 0,05 |
| N5 proyectil | zombi `NoAI` a 8 bloques, `clip` 1,5 delante, `burn steel` | moneda ≤ 2,0; salud del zombi baja (log del mod); un `coin_hit` |
| N5w muro | N5 con vidrio en x = 516 y el zombi en 517,5 | la moneda avanza ≥ 1 y no pasa del vidrio; ningún `coin_hit` |
| N11 Nether | N4 en el Nether | el lingote llega y se recoge; el jugador sigue en el Nether |
| N6 avivado | N1 a 20 bloques con `flare` | velocidad y Δx medio en 4 ticks ≤ 1,45; consumo ×3 |
| N7 agotamiento | `credit iron 200`, N1 | `exhausted metal=iron` a los 20 ticks; sin `p4` después |
| N8 muerte | N1, `kill @s` a mitad, `respawn` | `target_cleared reason=death`; reserva 0; sin `p4` después |
| N9 dimensión | N1, `execute in minecraft:the_nether run tp @s 0 100 0` | `reason=dimension`; hierro Off |
| N10 inválido | N1, `setblock` del bloque a aire a mitad | `reason=broken`; ningún `p4` con `target=block` después |

Gates: `tools/check.sh`, N1–N10 en un build con árbol limpio, regresión S1, S5, S8, S9 y C2, evidencia `docs/evidence/p4.md` y dos revisiones. Valores de ensayo, no de balance.

---

## Archivos críticos
- Nuevos/movidos: `mod/horizon-mod.toml`, `mod/Cargo.toml`, `mod/src/{lib,bridge,save,scenario}.rs`, `mod/src/core/src/{reserves,persistence}.rs`, `mod/src/core/tests/{reserves,persistence}.rs`, `tools/{fetch-sdk,build-native,native-test,check}.sh`, `docs/{plan,design,compatibility,contracts,STATUS,history}.md`, `docs/evidence/{p0b,p2}.md`.
- Reutilizados del núcleo actual: `types.rs`, `state.rs`, `commands.rs` (`apply_command`, `advance_tick`, `VerifiedEvent::from_server_observation`), `tests/contracts.rs`.
- Fuente legacy portada: `legacy/windows-fdc87ae:crates/allomancy-core/src/{reserves,persistence}.rs` y sus tests/docs.
- Referencia SDK (fuera del repo): `mod/.horizon/sdk/api/crates/horizon-mod-vocab/generated/mod_api_generated{,_host}.rs` (PlayerRef, eventos), `…/src/item/transaction.rs` (HostTransaction), `…/horizon-core/src/nbt.rs` (Compound, Tag).

## Verificación
1. Paso 0: `git ls-remote origin` muestra `legacy/windows-fdc87ae` = `930b96d` y ninguna `codex/*`.
2. Cada PR: `tools/check.sh` (fmt, clippy -D warnings, tests, tamaños) exit 0.
3. P0b: G1–G8 con `tools/build-native.sh p0b-1` y `tools/native-test.sh {pd-roundtrip,tx-abort,raw-root}`.
4. P2: `cargo test -p allomancy-core --test reserves`, `--test persistence`; `tools/native-test.sh s1 … s10` (S3/S4 ×3).
5. Tras cada merge: `docs/STATUS.md` actualizado, rama de fase borrada en remoto, `git count-objects -vH` registrado en la evidencia (control de peso).
