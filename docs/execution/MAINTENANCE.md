# Mantenimiento continuo del espacio de desarrollo

Regla solicitada por el usuario el 2026-11-13: revisar y limpiar regularmente los artefactos de desarrollo para evitar acumular copias completas del juego.

## En cada build y cierre de fase

1. Antes de un build grande, consultar procesos/receipts pendientes y medir el runtime. No duplicar una build válida sin una razón registrada.
2. Después de verificar la build nueva, revisar exports y launcher/native/server-builds. Conservar la build actual validada y la anterior necesaria como baseline; justificar cualquier copia adicional en CHECKPOINT.
3. Eliminar de forma rutinaria los exports obsoletos y cachés de engines duplicados ya sustituidos, dentro del runtime y con manifests verificados. Esta limpieza de copias prescindibles está autorizada por el usuario; no requiere volver a preguntar por cada copia equivalente.
4. Registrar antes/después: destinos exactos, motivo, bytes recuperados, resultado y archivos conservados. Si hay una interrupción, revisar receipt/estado real antes de repetir.

## Siempre conservar

Fuentes, manifests, Cargo.lock, .git, documentación, checkpoints, evidencia/receipts, mundos y backups. Mantener SDK, toolchain y cachés de dependencias útiles para el siguiente paso. En este entorno rustdoc se ejecuta desde devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe: no borrar devtools indiscriminadamente.

La autorización de rutina NO incluye borrar launcher/native/target, dependencias, mundos, backups, fuentes, evidencia ni un runtime completo. La limpieza opcional de target requiere una decisión explícita porque obliga a recompilar el juego.

## Controles de cada eliminación

- No limpiar mientras algún proceso propio de build/juego pueda usar esos artefactos.
- Lista explícita de destinos, rutas absolutas resueltas y contenidas estrictamente en runtime; rechazar links/reparse inesperados y manifests que indiquen una build distinta de la obsoleta prevista.
- Un único shell PowerShell y Remove-Item -LiteralPath; no borrar con glob genérico ni encadenar otro shell.
- Verificar que siguen presentes la build retenida, toolchain, mundos/backups y evidencia. El .gitignore excluye artefactos de Git, pero no limita su espacio en disco.