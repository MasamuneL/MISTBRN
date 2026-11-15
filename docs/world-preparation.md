# Preparación offline de identidad de mundo

Con Horizon y Cargo cerrados, use PowerShell 7 y una raíz dedicada existente dentro de una raíz permitida existente:

```powershell
pwsh -NoProfile -File tools/prepare-world.ps1 -WorldRoot <mundo> -AllowedRoot <raíz-permitida> -ProvisionNew
```

`-ProvisionNew` solo permite crear la identidad cuando el directorio de mundo está totalmente vacío. Nunca se asigna una identidad nueva a un mundo con contenido. Sin el switch solo se valida o repara una copia faltante a partir de la otra válida. La salida indica `Provisioned`, `Existing` o `Repaired from Primary/Backup` y el ID. Conserve ambas copias y los temporales ante cualquier error; una recuperación o importación de mundo existente requiere auditoría separada.

El archivo y su `.bak` contienen solo `schema_version` y `world_id`; se rechazan JSON inválido, duplicados, campos extra, versiones no soportadas, IDs inválidos y conflictos. Los archivos temporales se crean en el mismo directorio, se sincronizan con `Flush(true)` y se promueven sin sobrescritura. Se releen las dos copias. Esa sincronización no demuestra durabilidad ante pérdida de energía. El helper comprueba procesos por nombre y rechaza reparse points en AllowedRoot y cada descendiente hasta el archivo inspeccionado. AllowedRoot es la frontera explícita de confianza del operador; no audita ancestros externos a ella (por ejemplo, el directorio Documents gestionado por OneDrive), pero requiere exclusividad operativa: no protege contra cambios concurrentes de rutas o procesos que arranquen durante su ejecución. No lee ni escribe datos de juego y no reemplaza el guard nativo previo a apertura.

Pruebas: `pwsh -NoProfile -File tools/test-world-preparation.ps1`. Los fixtures quedan bajo `target/world-preparation-tests/<RunId>`; no se eliminan automáticamente.
