# Limpieza de copias obsoletas — completada

Fecha: 2026-11-13. Usuario autorizó la limpieza propuesta y pidió mantenimiento continuo. Los siete destinos siguientes se eliminaron tras preflight; resultado cleanup-summary.json: 8.82 GiB y 210385 archivos retirados, 60 hashes de archivos protegidos coincidentes.

Raíz de runtime original: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN/experiments/horizon-probe/runtime.

## Limpieza conservadora ejecutada: 8.82 GiB

Exports P0 antiguos, aproximadamente 5.04 GiB total:
- server-engine
- probe-engine-v2
- probe-engine-v3
- probe-engine-v4

Cachés de engines P0, aproximadamente 3.78 GiB total bajo launcher/native/server-builds:
- native-fdc87ae-0c8f7a9b8f47 (allomancy_probe, sha137aadde...)
- native-fdc87ae-9d6935b62822 (allomancy_probe, sha6b214374...)
- native-fdc87ae-a3af68e0ff84 (allomancy_probe, sha366a666c...; duplicado del export v5 retenido)

Se verificaron manifests reales. Conservar native-fdc87ae-250494cffe9b (allomancy actual sha e66284fb...) y p1-bridge2-engine. Conservar probe-engine-v5 (último probe verificado). Al eliminar exports viejos se pierde ejecución inmediata de esas versiones; conservar fuentes/logs/receipts/resultados y recompilar si se necesita repetirlas.

## Opcional con coste de recompilación

launcher/native/target: aproximadamente 6.79 GiB, productos reconstruibles de Cargo; no contiene fuentes propias canónicas ni mundos. Eliminarlo obliga a recompilar dependencias y juego en el siguiente build. No es la primera opción mientras se desarrolla P2.

## Conservar

Código/contratos/content/docs/tests/tools, .git, checkpoints/evidencias y sus backups. No borrar los mundos bajo game-*/saves ni runtime/backups ni native-builds/receipts. Mantener SDK source/stage/patched, launcher/rust, launcher/cargo y launcher/versions para continuar sin reinstalar/descargar. Conservar devtools/rustc-1.99.1-x86_64-pc-windows-gnu/rustc/bin/rustdoc.exe: es el ejecutable oficial usado por los doctests, aunque esté bajo devtools.

## Antes de ejecutar una futura limpieza autorizada

Confirmar que no hay procesos propios de juego/build activos; resolver cada destino y verificar contención estricta en runtime original, rechazar links/reparse inesperados y cambios de manifest. Guardar lista exacta/bytes en checkpoint. Usar un único shell PowerShell, sin delete genérico de runtime completo ni glob que seleccione worlds/source. Para nuevas copias equivalentes, aplicar la autorización y retención definidas en docs/execution/MAINTENANCE.md; generar siempre una lista y comprobación nuevas, sin reutilizar destinos eliminados.