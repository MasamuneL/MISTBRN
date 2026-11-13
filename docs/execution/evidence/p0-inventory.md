# P0 — inventario local (2026-11-12, America/Mexico_City)

Alcance: metadatos y existencia únicamente. No se leyeron credenciales, cuentas ni partidas; no se inició juego, instalación ni compilación. Los directorios personales fuera de las rutas explícitas no se exploraron exhaustivamente.

## Instalación y recursos

Raíz de versión `<USER_HOME>/AppData/Roaming/horizon-launcher/versions/nightly-2026-11-08-fdc87ae` (nombre de carpeta, no SHA completo verificado). Metadatos PE mediante `Get-ChildItem -File -Filter '*.exe' | Select-Object Name,Length,@{n='FileVersion';e={$_.VersionInfo.FileVersion}},@{n='ProductVersion';e={$_.VersionInfo.ProductVersion}}`:

| Archivo | Bytes | FileVersion / ProductVersion |
|---|---:|---|
| horizon-client.exe | 92381184 | 0.1.0 / 0.1.0 |
| horizon-server.exe | 8074752 | vacíos |
| horizon-ctl.exe | 884736 | vacíos |
| horizon-prepare.exe | 2339840 | vacíos |

Existe `crates/horizon-prepare` con `Cargo.toml` (715 bytes), `src` y `tests`; `mods/horizon/src/test`; `exporters/client` y `exporters/server`; `tools` contiene scripts de pruebas y preparación. Esto no acredita un SDK completo ni sus firmas. El CLI está en la raíz de versión, no en PATH. No se ejecutaron estos binarios.

Recursos preparados visibles por metadatos: `.local/horizon-client/26.3/asset-index.json` 597035 bytes, `attributes.json` 131846, `block-behaviours.json` 1982156 y `block-tags.json` 119841; `.local/minecraft/26.3/prepared.json` 12693 y `manifest.json` 44992; directorios jars, libraries y reports presentes. `<USER_HOME>/AppData/Roaming/horizon-launcher/server-jars/26.3.jar` existe, 62294556 bytes. Existencia no prueba integridad, licencia, compatibilidad ni que la preparación esté completa; no copiar ni distribuir recursos de terceros.

## Herramientas

Consulta `Get-Command <nombre> -ErrorAction SilentlyContinue`: rustc, cargo, javac, cl y horizon-ctl ausentes de PATH; `.cargo/bin` no existe. Ausencia de `vswhere.exe` en las dos rutas convencionales y de los dos directorios convencionales Microsoft Visual Studio; esto no excluye instalaciones alternativas. Python en PATH apunta a `<USER_HOME>/AppData/Local/Microsoft/WindowsApps/python.exe`; `python --version` no pudo iniciar (error del sistema al acceder al archivo; no hubo proceso ni exit code). Java en PATH apunta al shim `C:/Program Files (x86)/Common Files/Oracle/Java/java8path/java.exe`; hay JRE `C:/Program Files/Java/jre1.8.0_503/bin/java.exe`, versión de archivo 8.0.5030.1, 311480 bytes. `javac.exe` ausente en `latest/bin` y ese JRE. Un intento de `java -version` no produjo versión verificable; no se infiere JDK funcional. `git --version` produjo `git version 2.56.0.windows.4`, exit code 0.

## Comandos y límites de evidencia

- `Get-ChildItem -LiteralPath <ruta> -Force | Select-Object Name,Length,PSIsContainer` sobre raíces y subdirectorios seleccionados; cmdlets PowerShell, sin exit code de proceso propio; ejecución del comando de inventario terminada exit 0, salida mostrada.
- `Test-Path -LiteralPath <ruta>` y `Get-Command <nombre> -ErrorAction SilentlyContinue`: cmdlets, sin exit code propio; la búsqueda es limitada a ubicaciones indicadas/PATH.
- `git rev-parse --is-inside-work-tree`: `true`, exit 0. `git status --short --branch`: `No commits yet on master`, `?? docs/`; `git diff --stat` sin salida. No se asigna un código individual a esos dos procesos porque no se capturó.
- `python --version`: fallo al iniciar, sin exit code de proceso. `git --version`: exit 0. No se afirma éxito de Java ni de herramienta de compilación.

Siguiente paso: contrastar revisión y SDK con fuentes oficiales y verificar herramientas reales; pruebas de capacidad y compatibilidad permanecen no ejecutadas.
