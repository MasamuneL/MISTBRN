# P1-stage — brief/checkpoint previo
schema_version:1
fecha:2026-11-13 America/Mexico_City
estado:pending dispatch
modelo:gpt-6.1-terra medium
base:54469d64852499e671faff2da710b8fea5de442b / codex/alomancia-p1
WORKSPACEEXPLÍCITO:C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN (no worktree.app readonly)
Propiedadexclusiva:tools/build-native.ps1,docs/execution/tasks/P1-stage.md,docs/execution/evidence/p1-stage.md. No fuentes,manifiestos,lock ni otros archivos. Sincommits/subagentes.
Objetivo: helpermínimo reproducibleparastage/compilación nativa real, sincopiarSDK/binarios/recursosalbundle propio ni inventarSDK. NoejecutarCargo/launcher mientrasCoreSolestácompilando;principalhará integración.
Inputsroot yaexisten:Cargomanifestscrates/allomancy-core yallomancy-horizon;descriptorintegration/horizon/horizon-mixin.toml. Adapterlib puedeestar ausente hastaprincipal. Noeditarinputs.
ScriptparamBuildNamemandatorio regexsimple[a-z0-9-],nombreexportnuevo;no otrosconfigknobs. ProyectoderivadodePSScriptRoot/..,runtimeoriginalexperiments/horizon-probe/runtime;launcherdesdeUSERPROFILE/AppData/Local/Programs/Horizon Launcher/horizon-launcher.exe(localizaciónya probada),baseversionnightly-2026-11-08-fdc87ae ytoolchain preparado.
Comprobar todosinputs(src/lib.rs ambascrates,manifiestos,descriptor,launcher,basepreparada)ANTESmodificardestino;antefaltafallarclaro. Stage enruntime/native-builds/<BuildName>/bundle,exportenruntime/<BuildName>-engine. Si directorio buildoexport yaexiste,fallarparainspección,noborrar/nosobrescribir/reinstalar.
Copiar SOLO descriptor a bundle/horizon-mixin.toml yCargo.toml+src deambascrates preservando estructura bundle/crates/...;no target,tests,SDK,Minecraft,Git ni recursos preparados. [[crate]] y [api] yaexisten en descriptor,manifestsdepsrelativascoinciden estructura.
Antesdelanzar guardar fuera de bundle unreceiptJSON conpaths/sha256inputs,comandoargs/estadoin_progress;escritura atómicatemporal/backup. Entorno HORIZON_HOME=runtime/homes/<BuildName>,HORIZON_LAUNCHER_HOME=runtime/launcher,XDG_DATA/CONFIG/TRASH/CLIPBOARD/DESKTOP todos enruntime,NO_SELF_UPDATE=1,NO_AUDIO=1. Native-build $bundle --version nightly-2026-11-08-fdc87ae --trust-native-code --profile dev --server-output $export. Logdocs/execution/evidence/<BuildName>-build.log(rawlocalignore),persistirsalida/exitcode/estado receipt,fallarconexitdelcomando. No marcaréxitosolo porexisteexe;exigirlauncherexit0yarchivoesperado. Guardarreceiptanteexcepciónsiinició. Mantenerpequeño ystdlibPowerShell,noclasesframework/nosugerenciasfuturas.
VerificarASTdelPowerShell sinlauncher;registrowritecheckpointantes/despuésyreportelimitaciones. No unitmocks/SDKfalsos ni pruebasque reflejenimplem;root probará helper en integraciónnativa real. Estado inicialinputsactorcorelibexiste;adapterlibtodavíafalta,precondicióndeberechazar. No tests/builds concurrentes conSolcore.


## Checkpoint previo
2026-11-13: implementación iniciada; sin ejecutar Cargo ni launcher. Destinos autorizados: helper y evidencia P1-stage.

## Checkpoint posterior
2026-11-13: helper implementado; AST correcto y precondición de adaptador ausente verificada sin crear destino. Sin Cargo/launcher; integración real pendiente del principal. Informe: docs/execution/evidence/p1-stage.md.

2026-11-13 revisión: preflight incluye horizon-resources.json; receipt conserva .bak y registra staging antes de copiar y building antes del launcher. AST correcto; sin ejecutar helper, Cargo ni launcher.

2026-11-13 recuperación — intención: anidar core en el bundle, transformar solo el manifiesto staged con guard de una coincidencia y registrar hash; añadir RuntimeRoot explícito a ambos helpers y corregir espera del smoke. Solo AST, sin build ni runtime.

Recuperación root: encargo nuevo Terra falló por cuota sin cambios. Root aplicó fix staging core anidado + path solo copia y RuntimeRoot build/smoke. AST0, build p1-bridge2 exit0, smoke bridge2 exit0 y verificador3markers. Evidencia central p1-integration.md/json. Sin revisión independiente completada.
