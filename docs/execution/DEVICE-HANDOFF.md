# Traslado entre dispositivos

Rama codex/alomancia-p2-gates. Snapshot WIP; P2 parcial, sin merge main. Leer CHECKPOINT.md y PLAN-ADJUSTMENTS.md. Consultar HEAD real: el checkpoint referencia la base anterior al commit de traslado.

Para recuperar tras publicar la rama: git fetch origin; git switch --track origin/codex/alomancia-p2-gates. Preservar cualquier cambio del equipo destino antes de actualizar.

Evidencia portable: evidence/p2-portable-run-evidence.zip y su manifiesto SHA256. Contiene receipts/logs/command-replies de primer nivel, excluye endpoints de control. Sus rutas absolutas corresponden al equipo original. No contiene SDK, engines, recursos de terceros ni mundos completos. La subida del ZIP requiere aprobación explícita por revisión automática; comprobar remoto antes de publicar.

Para ensayos nativos en otro equipo se requieren instalación legítima/toolchain fijadas y transferencia separada de mundos/backups COMPLETOS del runtime original. No reconstruir mundos con DAT sueltos ni acreditar ejecución en otro equipo usando estos logs. Código y revisión pueden continuar desde el snapshot. Los artefactos originales siguen intactos localmente.

Pendientes: N5 FIFO, N7 captura writer (writer4 fallido, sin receipt de frontera), N9 restantes y revisión de cuatro bloques P2. Replay/reentrant y dedicated4 tienen oráculos/revisión documentados. Dedicated no acredita P9. Unwinder GDB y mensajes sin flush son hipótesis a investigar.

Tres diferencias ajenas excluidas del commit permanecen intactas localmente: docs/execution/evidence/p0-initial-probe.rs, docs/execution/evidence/p2-horizon-preflight.md y experiments/horizon-probe/bundle/api/src/lib.rs.