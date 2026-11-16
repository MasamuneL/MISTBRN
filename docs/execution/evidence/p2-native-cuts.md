# N6 — cortes reales del adaptador, perfil owner integrado

Build `p2-cuts1` succeeded exit 0 con API411/fdc87ae y Rust1.99.1 compartidos; engine SHA256 `8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70`. Feature native-test sólo opt-in en staging; default de fuente vacío. No SDK/fork ni poderes. Núcleo intacto, un único paquete allomancy-core en lock staged.

| Frontera | Corte | Offline | Reapertura de otra copia |
|---|---|---|---|
| Antes de inventario | traza exacta + kill PID propio | A completo | reconnect(A) completo, exit0 |
| Después de inventario | traza exacta + kill PID propio | A completo | reconnect(A) completo, exit0 |
| Después de namespace inline | traza exacta + kill PID propio | A completo | reconnect(A) completo, exit0 |
| Después de core commit, antes de save | traza exacta + kill PID propio | A completo | reconnect(A) completo, exit0 |

Los cuatro procesos terminados tienen exit -1 intencional; no son cierres limpios. Cuatro cortes y cuatro reaperturas: ocho receipts nuevos, 47 observados sumados a los 39 previos. `p2-cuts-checks.json` conserva PID/token/T/traza y hashes; `tools/check-native-cuts.cjs RUNTIME` verifica los receipts y trazas reales sin repetir juegos. Suite serial exit0 y consolidación exit0. Los 39 ensayos anteriores y la suite core no se repitieron.

Cada mundo parte de copia completa cerrada de `game-p2-owner-econ/saves/credit`, con manifiestos source/copied iguales y rechazo de reparse. Ready → save real integrado → copia diagnóstica DAT/old → oráculo A → archivo de armado con token único. A partir de armado el controlador sólo lee stderr; mata el objeto Process propio después de una única línea exacta con PID/frontera/T/token. Fallos o trazas ajenas se rechazan antes de aceptar. La pausa bloquea la pila del callback indefinidamente; no Host/save/dispatch de ticks y ningún mutex Session/stderr retenido. Un writer independiente puede terminar A.

El oráculo compara inventario completo con tipos NBT, SavedPlayer completo (estado, reglas, fracciones, revisión y ledger), namespaces ajenos y metadatos del wrapper/envelope. Rechaza duplicados/trailing/mixed state. Estos ensayos no producen un snapshot B posterior al armado: se exige A, nunca aceptar B genéricamente. El candidato B sólo fija el contraste esperado (una unidad de hierro consumida, +1000 Iron, T exacta y revisión+1). Reapertura exige la transformación explícita reconnect(A), no elegir A/B otra vez. DAT/old/new presentes se inspeccionan; new está ausente en los cuatro cortes. Las copias originales del corte conservan sus manifiestos íntegros después de abrir otras copias.

Alcance: N6 acreditado para estas cuatro fronteras en owner integrado con pausas sin save posterior. No N7 entre escrituras/renombres, fsync/pérdida de energía, writer FIFO, dedicated, ni P2 completo. Un ACK o core commit no se presenta como durabilidad. Persisten los gates/variantes del CHECKPOINT.

Proveniencia: 21/22 inputs de build siguen idénticos; único cambio observado es launcher.exe herramienta de build 551B2681785D7AC0D12CABA11A266C3A9C602CBBEBC2A76072FAEB5E5F658B30 → B48F491344AB99E98719C74DC7F539D78D4CF0F2C144D1EE6F1E4090E3B22EA1. Los ensayos lanzan engine directamente y fijan hash engine/launcher observado; todo otro drift se rechaza. No se repitió build ni se atribuye identidad del launcher a todo el intervalo de compilación.

Revisiones previas: Sol fuente/harness/suite; Astra fuente y cierre del falso-pass A↔B; Terra oracle; Luna alcance documental. Revisión final independiente Sol high comprobó receipts/logs, ocho oráculos reproducidos offline, manifiestos de fuente/cortes intactos y hashes: sin bloqueantes. Fuentes/modelos disponibles en esta sesión; un límite de uso interrumpió sólo comprobación, no la build, y se retomó tras petición del usuario.
