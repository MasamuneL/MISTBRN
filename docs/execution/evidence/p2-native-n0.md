# P2 — compilación y N0 nativo parcial

Fecha: 2026-11-15 America/Mexico_City. Rama `codex/alomancia-p2-native`, base `fed744b`. **P2 y N0 de producto NO cerrados.**

`p2-native2` compiló core, guard y backend contra SDK fijado/API411, launcher exit0 y receipt succeeded. Engine SHA256 `0A51CCBF7D1D695B0467E41842214B9F4E3ABCFF7CD418CE7AE97F391DB9E0BB`. Todos inputs del receipt coincidieron con las fuentes; Cargo metadata offline/locked comprobó un único paquete core compartido y feature native-test sólo staged. Ver `p2-native2-build-check.json`, `p2-native2-package-check.json` y logs locales ignorados. Producción mantiene default features vacío; no hay poderes jugables.

## Ejecución real y alcance

17 ensayos pasaron con el mismo engine: primera apertura y reapertura de la baseline, fixture sana con namespace ajeno y 14 rechazos. Perfil offline explícito **Player**; mundo dedicado UUID `e0f1ae8032614d8795371acf11dac6d2`. Se lanzó engine directamente; ningún wrapper ejecutó la validación antes de Horizon. El HEAD real emitió accept/reject y Hooks::world_lifecycle registró WorldOpenReady/WorldOpenFailed. Aperturas válidas terminaron espontáneamente con exit0 y backend Ready. Tras rechazos se terminó sólo el PID propio, con exit-1 clasificado controlled_after_rejection, después del fallo y sin login/Ready. No se presentan esos exits como cierre limpio.

Casos: namespace ajeno; marca canónica sin state, scalar y futura; transporte futuro; JSON futuro (`UnsupportedVersion(999)`) y truncado; NeoForgeData inválido; duplicado NBT; gzip principal y backup truncados; extensión `.DAT`; backup marcado; IO real con FileShare.None; namespace sin ambos markers; marker futuro. En cada rechazo DAT/markers quedaron exactamente iguales (incluidos archivos ausentes/nuevos). En el caso sano, el archivo extra ajeno quedó intacto. IO produjo `snapshot open IO error`, se liberó el handle sólo tras terminar el proceso y los hashes siguieron iguales. No es un failpoint artificial.

Resultados por caso, razones y hash de receipts: `p2-native-n0.json`. Receipts/logs/mundos/backup originales: runtime compartido `native-open-runs/<run>/`, `game-p2-aligned/`, `game-p2-n0aligned/`, `backups/p2-aligned-after-reopen`. No se distribuyen recursos del juego. No repetir estos casos mientras fuentes/artefacto/oráculos no cambien.

## Fallo real que impide cerrar compatibilidad

Con el perfil automático original, la primera apertura llegó a Ready, pero reabrir rechazó `PlayerMismatch`. El SDK conserva en Level el UUID de offline Player y copia el namespace del Guest actual; escribe dos snapshots con UUID diferentes y payload del mismo Guest. El rechazo es correcto; no se relajó identidad ni se repararon DAT. Diagnóstico causal y hashes: `../tasks/P2-native-identity-debug.md`. Las fixtures antiguas derivadas de ese mundo no se ensayaron ni acreditan estos resultados. Se conservan como evidencia.

Los ensayos de perfil Player aíslan otros comportamientos N0; **no corrigen ese defecto del producto**. Ruling arquitectónico pendiente de mecanismo mínimo sin fork. N1–N9, rollback/cortes/autosave/lifecycle y compatibilidad entre perfiles siguen pendientes. La aprobación estática de cuarentena no sustituye su ejecución Host real. Setter/commit/SaveToFile no son fsync.

## Incidencias del harness preservadas

`n0-base1` falló en el harness por lectura compartida del log; no tenía exit code capturado. `n0-base2` falló legítimamente por identidad. Se corrigieron logging y manejo de rechazo inesperado; no se recicló un resultado failed a pass. El preparador acota reparse checks a AllowedRoot inclusive; suite tras fix FileInfo.Directory pasó, junction real passed, symlink no disponible por privilegio. Todo está en CHECKPOINT y logs correspondientes.
