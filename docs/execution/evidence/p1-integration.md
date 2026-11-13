# P1 — verificación de integración

Fecha: 2026-11-13, America/Mexico_City. Rama: `codex/alomancia-p1-resume`, base `54469d6`. Estado: verificación técnica aprobada, revisión independiente pendiente. No merge a main ni C1 cerrado.

## Evidencia ejecutada

- `cargo test --locked --offline --workspace`: exit 0; 13 contratos y un doctest compile-fail aprobados. El doctest comprueba que VerifiedEvent no admite Deserialize.
- `cargo fmt --all -- --check`: exit 0. `rustfmt --edition 2024 --check crates/allomancy-horizon/src/lib.rs`: exit 0.
- `tools/build-native.ps1 -BuildName p1-bridge2 -RuntimeRoot <RUNTIME>`: exit 0 y receipt succeeded; compiló core, adaptador y juego real contra Horizon fdc87ae/API411, Rust 1.99.1. Registro exportado contiene únicamente allomancy 0.1.0.
- `tools/smoke-native.ps1 -BuildName p1-bridge2 -RunId bridge2 -RuntimeRoot <RUNTIME>`: exit 0, sin timeout ni error de smoke. Mundo nuevo AllomancyP1, modo offscreen, nueve chunks generados.
- `tools/verify-native-bridge.ps1 -LogPath <RUNTIME>/game-p1-bridge2/smoke-stderr.log`: exit 0; tres marcadores `accepted=true tick_recorded=true` desde callbacks reales del servidor integrado.

Hashes de inputs, engine, manifiesto staged, log y resultado curado: [p1-integration.json](p1-integration.json). Los ocho inputs de código/descriptor se cotejaron después del build con sus hashes del receipt; no cambiaron. Logs y runtime permanecen locales, fuera de Git.

## Corrección de staging

RED real: p1-bridge1 falló antes de compilar porque Horizon copia solo la carpeta `[api]`; la dependencia hermana `../allomancy-core` no existía en el stage del SDK. `[[crate]]` corresponde a helpers de mixins y no resuelve esa dependencia.

El build corregido anida una copia del núcleo dentro de la carpeta API y transforma únicamente el path de la copia del manifiesto. Exige exactamente una coincidencia y registra el SHA del resultado. Los manifiestos canónicos siguen separando el núcleo del juego. Se retiró `[[crate]]`; el SDK compiló el adaptador real sin necesitar inyecciones del probe.

## Reproducir núcleo

Con Rust 1.99.1 instalado normalmente, ejecutar los comandos Cargo anteriores desde la raíz. El workspace puro excluye el adaptador; su gate es la compilación nativa separada.

La toolchain portable del launcher no tenía rustdoc en bin. El ejecutable del paquete oficial extraído sí respondió correctamente al incluir las DLL de la toolchain en PATH. Para esta verificación se usó `RUSTDOC=<RUNTIME>/devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe`, `RUSTDOCFLAGS=--sysroot <TOOLCHAIN>`, `RUSTC=<TOOLCHAIN>/bin/rustc.exe`, `CARGO_HOME=<RUNTIME>/launcher/cargo`, `CARGO_INCREMENTAL=0`, y PATH con bin y mingw64/bin. SHA de rustdoc en el JSON. No se modificó la protección del sistema ni se repitió su restauración. La causa de desaparición anterior sigue sin confirmarse.

`RuntimeRoot` permite reutilizar una instalación preparada en otro checkout, con permisos de escritura sobre ese runtime. Los helpers rechazan nombres/destinos existentes; inspeccionar receipt y proceso antes de reanudar un intento interrumpido.

## Límites

Solo estado en memoria y contrato de autoridad. No hay poderes, consumo, guardado, recompensas ni interfaz final. Epoch identifica una sesión transitoria, nunca un mundo persistente. P2 conserva los gates de corrupción, rollback y replay del diseño.

Terra y Sol devolvieron límite de uso al intentar corregir/revisar esta recuperación. El principal implementó el fix y ejecutó las pruebas; esto no sustituye la revisión independiente. Siguiente acción: revisión P1 completa del diff y evidencias; corregir hallazgos y verificar antes de merge a main y P2.
