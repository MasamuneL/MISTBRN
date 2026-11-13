# P0 content fixture

## Checkpoint inicial

- Estado: intención registrada antes de crear el fixture.
- Brief: `docs/execution/tasks/P0-content.md`.
- Esquemas consultados en el pin `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`: `docs/mods.md`, `docs/modpacks.md` y `crates/cargo-horizon-mod/template/content/{horizon-mod.toml,item.json,recipe.json}`.
- Línea base de `experiments/horizon-probe/content/`: el directorio no existía; no hubo archivos previos que respaldar.
- Respaldo preescritura: estado vacío registrado arriba; escrituras finales publicadas con archivo temporal y renombre atómico.

## Intención

- Un mod `allomancy_probe_content` versión `0.0.1`, API `0.2`, lado `both`, capacidad `registries`.
- Un item `probe_metal`, plantilla `minecraft:iron_ingot`, stack máximo 16; receta shapeless de un lingote por una muestra.
- Asset/modelo visual propio con referencia a modelo y textura vanilla; no se copian recursos.
- Locales `en_us` / `es_es`: “Probe sample” / “Muestra de prueba”.
- Sin build, compilación, instalación o carga del juego; validación local de JSON solamente.

## Checkpoint final

- Contenido creado: 7 archivos (1 manifest TOML, 6 JSON); límite de 8 respetado.
- JSON: `Get-Content -Raw <archivo> | ConvertFrom-Json` para cada JSON — exit 0, 6/6 parseados.
- Compilación/carga real del juego: no ejecutada; queda al principal.
- Respaldo postescritura: hashes SHA-256 para restauración exacta:
- `experiments\horizon-probe\content\assets\allomancy_probe_content\items\probe_metal.json` — SHA-256 `E164E0F9ED643EED4B880E1B019F864ECC35E78B73EC661A0E547F24A14164F8`
- `experiments\horizon-probe\content\assets\allomancy_probe_content\lang\en_us.json` — SHA-256 `D7F36DD5680E0AE0973CB73EFCE2A1BC05DF4B20A896DEA87784195A54C78357`
- `experiments\horizon-probe\content\assets\allomancy_probe_content\lang\es_es.json` — SHA-256 `52DB6EF814F4CDC85FE87A761C18353B963649C29392DFC2BA3751FF27DF4761`
- `experiments\horizon-probe\content\assets\allomancy_probe_content\models\item\probe_metal.json` — SHA-256 `111B0FE4BAE45FD290791055B405B1D617668E1293CD20BC0EC26CD9F8DE124A`
- `experiments\horizon-probe\content\data\allomancy_probe_content\horizon\item\probe_metal.json` — SHA-256 `8BA15A038DC75438DCBD93592C21E36AE10EE355A696D3AD14FE3647DC72AC06`
- `experiments\horizon-probe\content\data\allomancy_probe_content\recipe\probe_metal.json` — SHA-256 `0DFBD70E8E794D579A061841890968377ED017B23438204B1AD888EFE819CB8A`
- `experiments\horizon-probe\content\horizon-mod.toml` — SHA-256 `C1C44A9909B038BD9A5709632F8AD6D433804F5610BE63F0C2E2FB18F03DB29F`
## Diagnóstico estático del timeout (2026-11-12)

- Observación de ejecución, comunicada por principal: el mismo binario v3 sin `--dev-mod` llegó a jugador/ticks/ping en 7.5 s (exit 0); con esta carpeta indicada, superó el timeout de 45 s antes de las tablas, y solo se vio el banner Horizon. Esto aísla el costo al camino activado por el mod, pero por sí solo no demuestra rechazo ni API incompatible.
- La forma estática del fixture coincide con el loader fijado: API 0.2 está en `horizon_mods::manifest::APIS` (`crates/horizon-mods/src/manifest.rs:20`); `ItemFile` acepta `template`, `max_stack_size` y `check_item` limita el stack a 1..99 (`crates/horizon-mods/src/content.rs:329,989`); la búsqueda lee `data/<id>/horizon/item/*.json` y declara su id desde nombre/ruta. La receta/idiomas/modelos quedan como recursos vanilla 26.3 según `docs/modpacks.md:194-206`. No se encontró un error evidente en el fixture por inspección estática.
- Camino de costo confirmado: `packs.rs:500` llama `content_root`; el loader pasa los paquetes aceptados a `mod_content::resource_root` (`game/mods.rs:98-121`). Con cualquier archivo de contenido no vacío la raíz de recursos requiere stamp/cache; en un cache miss llama `build` (`mod_content.rs:2742-2765`). `build` invoca `tables` antes de copiar los archivos (`:2473-2490`). `tables` vuelve a leer y serializar cada tabla JSON/NBT base existente, incluso para un mod pequeño; declara tres tablas en paralelo y advierte que cada hilo mantiene una tabla completa en memoria (`:2405-2449`). Por tanto el primer cache miss puede hacer trabajo voluminoso aunque el fixture solo tenga un item. La propia doc dice que el root se cachea después, y documenta una construcción de ~25 s y pico >3 GB para ATM10 (`mod_content.rs:2742-2749`); `docs/modpacks.md:1061-1068` también distingue el coste de la primera construcción del root.
- Esto es una explicación plausible y respaldada para demora de primera carga, no prueba suficiente para explicar >45 s: fixture/caché/host difieren de ATM10 y el principal aún debe observar baseline comparado con un cache miss/caché caliente o identificar el último log. No ejecutaré nada según la instrucción.
- Sin build ni juego en esta revisión. No se modificó el fixture; solo se agregó este diagnóstico a la evidencia.
