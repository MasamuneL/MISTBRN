# P2 — harness lifecycle nativo

Fecha 2026-11-15 America/Mexico_City. Agente /root/backend_quarantine, fallback Sol tras cuota Terra/Luna; no reintentos ni subagentes.
Estado: implementación autorizada, ejecución nativa pendiente exclusivamente de root.
Propiedad única: tools/test-native-lifecycle.ps1 y esta ficha. Ambos ausentes al inicio. No Rust/SDK/Cargo/juego/commits; no editar test-native-credit.ps1 ni horizon-control.cjs.

## Intención ANTES

Reutilizar patrón credit de entorno/build/input hashes, logs compartidos ReadWrite|Delete, Node control por archivos UTF-8 y receipt atómico. Startengine Hidden sin framebudget; observe sin give. WorldRoot dedicado existente runtime/.../game-p2-*/saves/NAME; baseline root clona crédito B1000/ledger1 y verifica offline después. Deadline global120s y fases30s, kill sólo Process propio ante fallo, jamás PASS por kill.
Casos: death-respawn kill→death_queued/death_applied→state dead→respawn real→eventrespawn→quit0/WorldCloseFinal; death-logout kill→quit inmediato sin esperar aplicación y evaluar orden logging_out antes death_applied sólo si realmente observado; save-pause save-all flush→save_observed delta→pause/state screenpaused+pausedtrue→resume/playing+pausedfalse sin --no-pause; dimension vanilla execute in Nether tp @s y evento+estado dimensión real, fallo/limitación explícita si no funciona. No prueba de persistencia/fsync sin oráculo offline.
SDK pin original leído: agent.rs:1026 state directo con screen/paused/dimension (state_value:481–542), :1657 command, :1714 pause; control.rs:989 resume y :1199 respawn remoto con deathscreen, :1253 quit; state screen nombres playing/paused/dead. Respuestas se conservan por helper Node redactor existente, nunca endpoint/token en stdout/receipt. pause command usa Escape real y paused bool es self.pause de frame; esperar ambos campos, no asumir éxito por ok.
## DESPUÉS — implementación congelada, ejecución pendiente

Único script creado tools/test-native-lifecycle.ps1. PowerShell Parser.ParseFile ejecutado sobre ese archivo: AST errors=0, exit0. Scoped git diff --check exit0 (archivos nuevos no tracked aún, por lo que ese comando no acredita contenido completo). Ningún engine/Cargo/build/escenario ni Node control ejecutado. No Rust, SDK, credit/control, commits o subagentes modificados. Root ejecutará después de preparar clones cerrados de crédito B1000/ledger1.
SHA256 harness congelado: 945CBAD7738034B4FD5AC8B6F98624EBEB2B3FD118875E7528D89B46D1ADE15F.
SHA256 horizon-control.cjs leído sin cambios: CC83E5A53E946DB8C3DEBA60156538EC7A47A316D08447FC7AE397ABE110AE4C.
SHA256 test-native-credit.ps1 leído sin cambios: 2B0495FFB1C7FA4182D16D63A8AB4497044B31554A182E5DA0AC507D9706ADD4.

## Contratos usados y gates por escenario

Parámetros BuildName/RuntimeRoot/RunId/WorldRoot obligatorio existente, PlayerName default AllomancyQA, Scenario death-respawn|death-logout|save-pause|dimension. Require build receipt succeeded/exit0/native_test y marker. El script no clona, no entrega ítems ni edita DAT. ALLOMANCY_NATIVE_TEST_SCENARIO=observe; exige driver enabled observe y backend ready antes de operar. Argumentos sin frames/world-frames; sólo save-pause omite --no-pause. Engine Hidden con control.json exclusivo del RunId; no se lee/publica el token en PowerShell.

| Caso | Secuencia exacta implementada | PASS solicitado / clasificación |
| --- | --- | --- |
| death-respawn | Ready→state playing/pausedfalse; command kill @s; esperar queued+applied; state dead/pausedfalse; respawn real; eventrespawn; state playing; quit | Backend acepta muerte y respawn, control quit0, engine exit0 y WorldCloseFinal sin Fault/FAIL. Persistencia/reservas/ledger pendientes de oráculo offline. |
| death-logout | Ready/playing; command kill @s; inmediatamente command quit sin poll ni wait de death_applied | Exige queued+applied, quit0/cierre0/WorldCloseFinal. Sólo PASS si orden de líneas stderr acredita logging_out antes de death_applied. Sin callback o con aplicación previa: status=limited, pass=false, cierre normal conservado y limitation explícita; NO gate antes siguiente tick. |
| save-pause | Sin --no-pause; resume inicial si hace falta; command save-all flush; esperar save_observed count>baseline; guardar delta; pause; state screenpaused+pausedtrue; resume; screenplaying+pausedfalse; quit | Respuesta ok sola no basta: exige traza de save y ambos campos estado. Cierre normal común. No se infiere fsync ni writer completo por save_observed. |
| dimension | Ready/playing; command execute in minecraft:the_nether run tp @s 0 80 0; esperar dimension_changed y state.dimension=minecraft:the_nether; quit | Si command/support/load falla o no aparece evidencia, failed con reply/logs, jamás inventar callback o éxito. No se presupone que pin soporte teleport/dimensión hasta ejecutar. |

No se omitió caso por falta de firma: todos existen en fuente pin. Su ejecución real permanece pendiente. SDK agent.rs:481–542 state_value retorna objeto directo screen, paused bool, dimension; :1026 comando state. :1657 command puede contestar ok=true con chat de error del comando vanilla: por eso los casos esperan eventos/estado, no sólo exit0 del helper. :1714 pause usa Escape y screen; control.rs:989 resume deja el menú; :1199 respawn remoto exige dead screen. En integrado, cliente remote presente usa ese respawn real. El estado paused es self.pause, actualizado por frame y propagado a Integrated::set_paused; esperar eventual cambio, no asumir reply instantánea.

## Evidencia y límites del runner

RunDir runtime/native-lifecycle-runs/RunId debe ser nuevo. Guarda stdout/stderr, result.json atómico con backup, cada command-NNN.txt UTF-8 sin BOM y reply-NNN.json tal como entrega horizon-control.cjs redactor. Receipt incluye argumentos, engine/build/input/helper/harness hashes, comandos/replies y tiempos relativos, phase, termination/exit, save delta, contadores y orden de eventos con archivo y número de línea. No publica endpoint ni token en receipt/replies. control.json privado es únicamente input del helper existente.

Logs se leen con FileShare.ReadWrite|Delete y reintento de IOException; nunca se toma falta de traza como PASS. Matches de logs usan -match sobre string explícito (no array .Contains), conteos [regex] sobre string y accesos null-safe sin StrictMode. El orden se conserva por fuente; no se inventa un orden global entre stdout y stderr. Sólo las trazas death/logging_out en stderr permiten comparar esa frontera temporal.

Deadline global120s, fase30s con poll250ms; Node helper tiene su propio timeout10s y se comprueba deadline antes/después. Una llamada bloqueada puede consumir parte del deadline pero no acreditar éxito tardío. Ready/playing/death/state/save/dimensión/cierre son fases explícitas. Fault/driverFAIL/WorldOpenFailed rechaza. Quit normal siempre requerido; salida espontánea previa no pasa. Catch sólo mata el Process concreto iniciado por el harness, registra killed_after_failure y pass=false; nunca nombre de proceso ni procesos ajenos. No reintento automático de economía/control fallido.

Un receipt limited de death-logout devuelve control tras engine normal exit0, pero pass=false; root debe leer status/pass y no interpretar exit del script solo como gate. Los otros fallos generan receipt failed y relanzan excepción. Cualquier oráculo de reservas/revisión/ledger/inventario/UUID se hace por root después, con mundo cerrado; este script no afirma valores económicos mediante Host falso ni inspecciona DAT vivo.

## Ejecución restante de root

Antes de cada caso, root prepara clon cerrado separado del baseline B de crédito, registra hashes y RunId, y asegura ningún otro juego usando ese mundo. Llamar este script con build p2-owner1 y perfil AllomancyQA. Después leer receipt/comandos/orden, comprobar exit0/WorldCloseFinal, ejecutar oráculo offline contra baseline y registrar resultado real aquí o evidencia de root. No repetir los27receipts ni14snapshots ya aprobados por esta implementación de QA. Ensayos N9 y temporización no quedan aprobados por AST.

## Actualización root: ejecución y revisión reales
Ver evidence/p2-native-lifecycle.md/json. owner-death-respawn1/save-pause2/dimension1 PASS nativo y oracleoffline; death-replay1 PASS adicional sin ítem. save-pause1 fallo preservado; comando corregido a control directo save-all (SDK integrated.call), chatunknown failfast; SHA E4C797AF9A5BD21710D2AC8A66222FB5C837A324CB7EBAF218655084BB9226CE. Revisión Sol sólolectura aprobó corrección/criterios. death-logout NO ejecutado: SDK quit omite logging_out al detener, sólo logged_out; gate necesita otra ruta nativa, no carrera de dos comandos. Estado anterior congelado es histórico. NoP2/N9completos ni fsync.
