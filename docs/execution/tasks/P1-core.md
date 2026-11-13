# P1-core — implementación de contratos/núcleo
schema_version:1
fecha:2026-11-13 America/Mexico_City
estado:pending dispatch
modelo:gpt-6.1-sol high
base:54469d64852499e671faff2da710b8fea5de442b / codex/alomancia-p1
WORKSPACE EXPLÍCITO: C:/Users/alanj.VIKINGO/OneDrive/Documents/ChatGPT/MISTBRN. El worktreegestionado.app no permiteescritura; todos comandos/escrituras coneste workdir/rutasabsolutas.
Propiedadexclusiva: crates/allomancy-core/src/{lib.rs,types.rs,commands.rs,state.rs}, crates/allomancy-core/tests/contracts.rs, docs/contracts.md, docs/execution/tasks/P1-core.md, docs/execution/evidence/p1-core.md. PrincipalretieneCargo.toml,Cargo.lock,rust-toolchain,adaptador,harness ycheckpointglobal. Noeditarfuera,no commits,no subagentes.

## Requisitos P1
NúcleoRustindependientedeljuego,ochometales/cuatro parejas,todoslosIDs opacosserializables. Cantidadesenterasdemilésimas,no floatspara economía. Tipos: PlayerId,WorldId,Tick(u64),Metal,Pair,NodeId,ResearchId,EventId,TransactionId,Vec3. PlayerState yWorldState.
Fijar docs/contracts.md con campos/variantes/tipos públicos exactos ANTESdeconsumidores (principal esperará aviso contrato listo).
Contratos obligatorios propios(no atribuirlosSDK):
apply_command(&mut PlayerState, Command, &WorldSnapshot)->Result<Vec<Effect>,RuleError>
advance_tick(&mut PlayerState,&WorldSnapshot)->Vec<Effect>
WorldSnapshot incluye tick,mundo,actor,posición,objetivos,autoridad ycontexto de input si necesario. Command incluye beber,activar,avivar,apagar,comprar,reclamar;selecciónpareja si requeridaUI. Effect distingue física,atributos,inventario,señales,persistencia;campospara separar propuesta/aceptación en memoria de aplicación alHost ydurabilidad.
VerifiedEvent solo lo construye lógica confiable;NO Deserialize de mensajesclientes eneventosverificados. Define tipos/EventKind mínimos para observar/progresiónfutura;noimplementarP5/P6. Firmasfuturas observe/purchase/claim/encode_save se documentan reservadas,NO stubstodo!/panic ni funcionesvacías fingiendoimplementación.
Probarcomandosinválidosdejanestadoexactamenteintacto yentradasigualesproducenefectos/estadoiguales. Casos incluyen autoridadcliente,identidadmundo/jugadormismatch,posición/objetivono finito cuando pertinente,metalbloqueado/reservacero,ycomando válido real de apagar para evitar determinismotautológico de coleccionesvacías. Defineordenfijo/determinista de ticks yduplicado/tickatrasado. No implementarconsumo/economía/guardar(P2)nipoderes(P4) anticipadamente; ramas futuras deben rechazar explícitamente operación no implementada conestadointacto,documentarqué estáhabilitadoenP1.
Noefectosrecompensa/daño/XPcomocliente;núcleovalidaauthorityserver. Validarantesdeasignarestado;falloadaptadorrequierepending/rollback explícito delcontrato,no asumir retornar Effect::Persist esfsync. Noarbitrariedadeconomía floats. ValidarVec3 finito en fronteradeconfianza.
PersistenciafuturaC0: estado canónico enNeoForgeData namespaced,completemerge preservaotrosmods;SDKgetterocultaraízcorrupta,adaptadorP2leeráplayer_compoundFUERA de turno antes deescribir/consumir. No declarartransaccióncrashsafeenP1. InventarioSDKslotexact fuera deacting,aplicaciónencallbackservidorsinawait,rollbackP2.
Nativeadapter mínimo lo haráprincipal usando tucontrato. PuedeusarWorldId transitorio porNativeWorldEpoch para harnesssinpersistir;identidadmundopersistente ycodecP2,no convertir epochdecarga enidentidadguardadadefinitiva. IDs opacos debenpermitirclavemundoestabledeadaptadorposterior.

## Proceso/pruebas
Lee skillsPonytail/TDD/verification relevantes yaanunciadas porprincipal,aplicalosmétodosmínimos. Manifestylibvacío fueroncreadosporprincipal;Cargolock será generadoantesdespacho. Noañadirdependencias: serde=1.0.229,devserde_json=1.0.151 delSDKpin. Sólostdlib+éstas.
RED real pertinente antesdeimplementación;después implementar yverificar. No testsqueleanfuente/mirrenconstantes ni pruebasdeSDKfalso. Puedecrear tipos/firmaspara que las pruebascompilen y fallen porcomportamiento ausente;noaceptartyposcomorojo.
Cargo usar --locked --offline para noeditarCargo.lock. PATHbinportable ymingw64bin delruntime original;CARGO_HOME=experiments/horizon-probe/runtime/launcher/cargo;RUSTC/RUSTDOCexplícitos. Principal no correráCargoenparalelo. Targetartifacts root/target ignorados. Comandos: cargo test --locked --offline --workspace;cargo fmt --all -- --check. Si lock/dependenciafaltan avisa,sinmodificarmanifiestos/lock. Fmtmutante permitidosolo tus*.rs,no otrosficheros.
Antes/despuésaccionesrelevantesactualiza ficha atómica temporal+backup;guardarRED/GREEN exitcodes/logs propios docs/execution/evidence/p1-core-{red,green}.log(localesignorados). Informe conarchivos,tests,códigos ylimitaciones,siguientepaso. Avisacontratoscongelados yterminadoconciso.


## Registro activo
estado:in_progress; acción: fijar contratos P1 y escribir pruebas antes de comportamiento; RED pendiente.

Contratos congelados y principal avisado. Antes de RED: 13 pruebas de contrato escritas; apply_command/advance_tick/event constructor son scaffold temporal explícito. Acción: cargo test --locked --offline --workspace; log p1-core-red.log.

RED observado: cargo test --locked --offline --workspace exit101; 13 casos compilan, 11 fallan por comportamiento ausente (rechazos, apagado, tick y evento), 2 pasan (snapshot inválido en no-op y roundtrip serde). Log docs/execution/evidence/p1-core-red.log. Antes de GREEN: implementar validación y únicas operaciones habilitadas; sin cambios de contratos.

Después de GREEN: cargo test --locked --offline --workspace exit0; 13/13 contratos ejecutados y aprobados. Avisos del filesystem OneDrive: hardlinks de cache incremental fallaron y se copiaron archivos. cargo fmt --all -- --check exit0. Comprobación adicional cargo test --locked --offline --doc --verbose exit101 porque bin/rustdoc.exe desapareció/no existe (os error2; nunca ejecutado). No atribuir éxito al ejemplo compile_fail ni cerrar P1 verified. Principal investigará/restaurará componente oficial; no Cargo en paralelo ni nuevos builds del core hasta aviso. Estado:implementation_ready; verification_pending_doc_toolchain. Antes de cierre: preparar informe/hashes, sin cambiar contratos ni fuente.

Informe docs/execution/evidence/p1-core.md escrito con corte SHA-256, 13 casos, RED101/GREEN0/fmt0/doc101, límites y siguiente paso. Fuente/contratos estables para revisión. Cargo liberado al principal; pendiente restauración rustdoc y gate documental, no marcar verificado aún.
