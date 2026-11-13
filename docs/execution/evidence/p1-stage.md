# P1-stage — informe

2026-11-13: implementado `tools/build-native.ps1` con validación previa, stage limitado, receipt JSON y registro de salida. El receipt guarda hashes de los archivos de entrada, argumentos y estado; el export existente se rechaza.

Validación local: parser AST PowerShell sin errores. Con el adaptador ausente, la ejecución rechazó `crates/allomancy-horizon/src/lib.rs` antes de crear `runtime/native-builds/p1-stage-precondition-check`. No se ejecutó launcher, Cargo, SDK falso ni compilación; la integración con SDK real queda para el principal.

2026-11-13 revisión: preflight incluye horizon-resources.json; receipt conserva .bak y registra staging antes de copiar y building antes del launcher. AST correcto; sin ejecutar helper, Cargo ni launcher.

2026-11-13 intención: crear smoke P1 aislado siguiendo CLI/export/resources de smoke.ps1; solo validar AST, sin ejecutar runtime.

2026-11-13 resultado: tools/smoke-native.ps1 usa export y recursos del build, game-dir único bajo runtime ignorado, logs y smoke-report locales, world-frames 80 y timeout mínimo 180 s. Espera al proceso propio; en timeout conserva PID para inspección. AST sin errores; no se ejecutó runtime, launcher ni Cargo.

2026-11-13 recuperación — intención: corregir el path staged de core para la copia del SDK, registrar transformación/hash y permitir RuntimeRoot existente; verificar AST sin ejecutar build ni runtime.
