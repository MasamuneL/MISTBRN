# Alomancia Horizon Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Construir el mod descrito en el diseño, con ejecución delegada por Sol y recuperación tras interrupciones.

**Architecture:** Núcleo Rust independiente del juego, adaptador Horizon y catálogos de contenido. El núcleo valida comandos y devuelve efectos; el adaptador obtiene estado del juego y ejecuta efectos. Servidor autoritativo si la API lo permite; nunca confiar en experiencia o daños enviados por clientes.

**Tech Stack:** Rust/Cargo; formatos nativos .hnmod/.hmod sujetos a P0; JSON para catálogos propios; herramientas de pruebas de Horizon que P0 confirme.

**Spec:** ../specs/2026-11-12-alomancia-design.md

## Restricciones globales

- Plataforma inicial Windows x64; ocho metales; español; sin dependencias Fabric/Forge.
- No inventar firmas ni compatibilidad del SDK. Fijar revisión y toolchain comprobados en P0.
- No tocar mundos personales: pruebas sobre mundos dedicados respaldados.
- No distribuir recursos de Minecraft ni ilustraciones/textos de las novelas.
- Cada cambio persistente y cada tarea deben poder retomarse conforme a CHECKPOINT.md.
- Los valores de balance del diseño son hipótesis de ensayo, configurables.

## Review Focus

1. Interrupción entre consumir objeto y guardar recompensa: P2/P3/P6, transacciones idempotentes.
2. Caída o cambio de dimensión durante un tirón: P4, cancelar anclajes inválidos y respetar colisión.
3. Evidencia obtenida antes de su pista y gasto duplicado: P5, reconocimiento retroactivo y validación atómica.
4. Teclas con chat/menús abiertos y reservas agotadas: P4/P7, no activar acciones y apagar al agotarse.
5. Actualización del juego o cliente con versión diferente: P0/P10, rechazo explícito y guardado intacto.

## Orquestación y modelos

Sol es el orquestador solicitado. ID recomendado disponible en este entorno: `gpt-6.1-sol`; no confundirlo silenciosamente con `gpt-6-sol`. El modelo principal se selecciona en la sesión; este documento no cambia el modelo del chat. Si el usuario ejecuta con otra versión de Sol, registrar la elegida.

| Trabajo | Modelo propuesto | Esfuerzo | Motivo |
|---|---|---|---|
| Orquestar, contratos, integración, autoridad y física | gpt-6.1-sol | high | Mantener decisiones entre subsistemas |
| Código acotado con contratos y pruebas, UI, QA visual | gpt-6.1-terra | medium | Trabajo delimitado y comprobable |
| Catálogos, textos de pistas, documentación y comprobaciones sencillas | gpt-6-luna | medium | Bajo acoplamiento; revisión por Sol |
| Bloqueo arquitectónico o revisión de física/persistencia compleja | gpt-6-astra | high | Escalación puntual de razonamiento |

Son asignaciones iniciales, no una garantía de óptimo ni precios. Comparar calidad/retrabajo en primeras tareas y ajustar. Fuente de orientación: https://developers.openai.com/api/docs/guides/model-selection . Disponibilidad real: herramientas de la sesión, no asumir acceso por nombre.

Máximo actual: cuatro agentes contando al principal, por tanto tres hijos. Sol reserva capacidad para revisión. Delegar solo trabajo independiente, con propiedad exclusiva de archivos. Contratos compartidos, Cargo.lock y manifiestos raíz son propiedad de Sol. Usar worktrees cuando haya commits base; en workspace compartido asignar directorios disjuntos y no permitir commits globales de hijos. Nada de git add . ni limpiar trabajo ajeno.

Para seleccionar modelos mediante `collaboration.spawn_agent`, usar `fork_turns: "none"`, `model` y `reasoning_effort` explícitos. Adjuntar contexto mediante rutas absolutas y un encargo autocontenido; el hijo no hereda el chat. Ejemplo de argumentos (solo al ejecutar):

```json
{"task_name":"p5_research","fork_turns":"none","model":"gpt-6.1-terra","reasoning_effort":"medium","message":"Lee el diseño y plan en docs/superpowers bajo el workspace absoluto entregado por Sol. Ejecuta P5, solo crates/allomancy-core/src/research.rs y sus tests. Consume los contratos fijados en P1. Antes de editar registra intención en tu checkpoint exclusivo. Entrega archivos, pruebas con exit code, limitaciones y siguiente acción. No modifiques contratos compartidos ni implementes fuera de alcance."}
```

Antes del despacho Sol completa rutas absolutas, commit base, estado de dependencias, archivos autorizados y casos de aceptación; no enviar literalmente una plantilla incompleta. Si el modelo no está disponible, registrar sustitución y preferir Sol; no entrar en reintentos indefinidos. Tras dos intentos fallidos por la misma causa, resumir evidencia y escalar una vez, no repetir desde cero.

## Estructura propuesta (P0 puede ajustar integración, no los requisitos)

```text
Cargo.toml / Cargo.lock / rust-toolchain.toml
crates/allomancy-core/src/{lib,types,commands,state,reserves,skills,research,progression,persistence}.rs
crates/allomancy-core/src/powers/{mod,iron_steel,tin_pewter,zinc_brass,copper_bronze}.rs
crates/allomancy-core/tests/{reserves,persistence,skills,research,progression,powers}.rs
crates/allomancy-horizon/src/{lib,bridge,input,world,save,network}.rs
crates/allomancy-ui/src/{lib,hud,wheel,journal,trees}.rs
content/{metals,skills,research,recipes,loot,balance}.json
assets/{textures,models,sounds,lang}/
tools/{validate-content,package,smoke}.ps1
docs/{compatibility,contracts,balance,testing,installation}.md
docs/execution/{CHECKPOINT.md,tasks/,evidence/}
```

## Contratos internos que P1 debe congelar

Estos nombres son propios, no APIs atribuidas a Horizon. Tipos en types.rs: PlayerId y WorldId (identificadores opacos serializables), Tick(u64), Metal (ocho variantes), Pair (cuatro), NodeId, ResearchId, EventId, TransactionId y Vec3. Cantidades de reserva en enteros de milésimas, sin flotantes para economía.

- `apply_command(state: &mut PlayerState, command: Command, ctx: &WorldSnapshot) -> Result<Vec<Effect>, RuleError>`.
- `advance_tick(state: &mut PlayerState, ctx: &WorldSnapshot) -> Vec<Effect>`.
- `observe(state: &mut PlayerState, event: &VerifiedEvent, catalog: &ResearchCatalog) -> Vec<Discovery>`.
- `purchase_node(state: &mut PlayerState, node: NodeId, catalog: &SkillCatalog) -> Result<(), RuleError>`.
- `claim_milestone(state: &mut PlayerState, event: &VerifiedEvent) -> Result<Reward, RuleError>`.
- `encode_save(state: &WorldState) -> Result<Vec<u8>, SaveError>` y `decode_save(bytes: &[u8]) -> Result<WorldState, SaveError>`.

P1 define campos y variantes exactos en docs/contracts.md antes de despachar consumidores. Command incluye beber, activar, avivar, apagar, comprar y reclamar; WorldSnapshot incluye tick, mundo, posición, objetivos y autoridad. VerifiedEvent solo lo construye lógica confiable. Effect distingue física, atributos, inventario, señales y persistencia. Separar validación y aplicación para no gastar dos veces ante un fallo del adaptador.

## Protocolo obligatorio de checkpoints

CHECKPOINT.md es la entrada de reanudación y lo escribe únicamente Sol. Cada hijo escribe docs/execution/tasks/<id>.md. Nunca depender de historial del chat, memoria del agente o un proceso vivo para saber qué hacer.

Estados: pending → in_progress → implemented → verified → reviewed → integrated. blocked requiere causa y trabajo independiente posible. Una tarea no se marca completada por haber generado código.

Antes de una acción que modifica archivos, instala, compila, ejecuta una migración o integra: registrar intención, rutas afectadas, estado previo, resultado esperado y cómo comprobar si ya ocurrió. Después: resultado, exit code, evidencia y siguiente acción exacta. Mantener el archivo vigente mediante reemplazo atómico con temporal en el mismo directorio; conservar copia anterior válida. Si existe un temporal al retomar, validar ambos y elegir la última versión completa, nunca sobrescribir sin leer.

Hacer checkpoint al terminar cada paso verificable, antes/después de delegar, antes de integrar y al cerrar un hito. En acciones largas guardar comando, cwd y ruta de log antes de arrancar; los IDs de sesiones y agentes son solo referencias temporales. Si se corta a mitad, inspeccionar procesos y resultados antes de relanzar.

Cada checkpoint incluye: versión del documento, fecha ISO con zona, task/step, modelo/agente, commit base, rama/worktree, archivos propios, cambios sin commit, operación en curso, pruebas y exit codes, dependencias, decisiones, bloqueos, y siguiente comando/acción. Los logs no incluyen credenciales.

Al reanudar: leer diseño, plan y checkpoint; ejecutar git status y git diff; cotejar archivos con la intención; detectar agentes/procesos vivos si aún existen; recuperar solo trabajo realmente incompleto; repetir únicamente verificaciones sin evidencia vigente. Tras un corte durante commit/merge/instalación, comprobar HEAD/estado/artefacto antes de repetir. No hacer reset, clean, checkout destructivo ni reinstalación automática para simplificar recuperación.

Un commit verificado por tarea es un punto adicional de recuperación. Trabajo sin verificar puede guardarse como WIP identificado y nunca se presenta como terminado. Los checkpoints guardan referencia al commit de código; evitar intentar incluir el hash de un commit dentro de ese mismo commit.

## Secuencia de ejecución

Para cada tarea de código: (a) registrar intención/checkpoint; (b) escribir prueba del comportamiento y observar fallo pertinente; (c) implementar; (d) ejecutar verificación; (e) revisión independiente proporcional; (f) Sol integra, registra evidencia y checkpoint. No introducir pruebas que solo repitan constantes o implementación. En tareas de exploración/documentación usar la evidencia indicada en lugar de pruebas artificiales.

### P0 — Viabilidad real de Horizon (Sol; Astra solo si hay bloqueo)

Dependencias: ninguna. Archivos: docs/compatibility.md, docs/execution/evidence/p0.md; prototipo desechable bajo experiments/horizon-probe/.

- [x] Leer instrucciones del repositorio upstream, guía de mods y ejemplos oficiales; fijar commit/release, toolchain y comandos de compilación.
- [x] Inventariar instalación local y recursos legítimos disponibles. Documentar faltantes sin confundirlos con carencias de API.
- [x] Compilar/cargar un mod mínimo y probar entrada, tick, item, impulso con colisión, UI, eventos de combate/loot, guardado, sonido y servidor.
- [x] Tabla por capacidad: firma/ruta upstream, prueba ejecutada, soportada/no soportada/no probada y alternativa.
- [x] Registrar comandos exactos y logs. Si faltan capacidades esenciales, diseñar extensión mínima o presentar fork; no avanzar afirmando compatibilidad.

Salida: checkpoint C0 con revisión fijada y decisión de integración. No estimar fechas de entrega antes de C0.

### P1 — Contratos y núcleo (Sol)

Depende P0. Archivos: manifiestos, core/{lib,types,commands,state}.rs, docs/contracts.md.

- [x] Definir los tipos anteriores, orden determinista del tick, errores y responsabilidad cliente/servidor.
- [x] Probar que comandos inválidos dejan estado intacto y entradas iguales producen efectos iguales.
- [x] Crear workspace y adaptador mínimo usando únicamente firmas verificadas.
- [x] Ejecutar cargo test --workspace y cargo fmt --all -- --check; congelar contratos para hijos.

Salida C1: harness del núcleo y esquema de datos estable.

Verificación técnica P1: 13 contratos, un doctest, formato, build nativo y smoke reales aprobados; evidencia `docs/execution/evidence/p1-integration.md`. Revisión independiente de cumplimiento y calidad **approved**, sin bloqueantes (`p1-review.md`); suite y formato premerge aprobados. C1 apto para integrar; registrar el merge verificado antes de comenzar P2.

### P2 — Reservas y guardado (Sol; revisión independiente)

Depende P1. Archivos: reserves.rs, persistence.rs, tests/reserves.rs, tests/persistence.rs, horizon/save.rs.

- [ ] Probar agotamiento exacto, dt/ticks consistentes, capacidad máxima, avivado, varias combustiones y rechazo de cantidades negativas/desbordadas.
- [ ] Implementar fórmula consumo base × modificadores acotados; reducción global máxima 20%, capacidad +50% según diseño.
- [ ] Probar round-trip, muerte, reconexión, versión antigua/migración, archivo truncado y esquema futuro rechazado sin sobrescribir.
- [ ] Implementar escritura atómica/backup y deduplicación de TransactionId con política de retención que no permita reproducir eventos antiguos.
- [ ] Simular corte antes y después de confirmar una transacción; verificar exactamente una aplicación.
- [ ] Ejecutar cargo test -p allomancy-core --test reserves y --test persistence por separado; integrar carga/guardado en mundo de prueba.

### P3 — Materiales y fabricación (Terra; datos auxiliares Luna)

Depende P1, integra con P2. Archivos: content/{metals,recipes,loot}.json, assets/, bridge.rs para Sol.

- [ ] Definir obtención de ocho metales/aleaciones, muestras, viales, monedas, libro, mesa y fragmentos sin dependencias circulares.
- [ ] Probar cantidades, tags metálicos explícitos y consumo atómico vial→reserva con inventario lleno.
- [ ] Incorporar fuentes en loot/comercio para mundos existentes; no migrar chunks automáticamente.
- [ ] Ejecutar tools/validate-content.ps1 (crear aquí), validando referencias, recetas alcanzables y IDs únicos; fabricar y beber en juego.

### P4 — Vertical hierro/acero (Sol)

Depende P2/P3. Archivos: powers/{mod,iron_steel}.rs, horizon/{world,input}.rs, tests/powers.rs.

- [ ] Probar objeto ligero/anclaje, alcance, línea de visión, metal no válido, velocidades acotadas y reserva cero.
- [ ] Implementar tirón, empujón, monedas, frenado y desvío usando el motor de colisiones; tags determinan qué es metálico.
- [ ] Probar descarga de chunk, muerte, dimensión, objetivo eliminado y menú/chat abierto durante acción.
- [ ] Ejecutar cargo test -p allomancy-core --test powers; verificar salto, caída y colisión en mundo controlado, registrar video/capturas si disponibles.

### P5 — Árboles e investigación (Terra con contratos de Sol)

Depende P1/P2; integración P4. Archivos: skills.rs, research.rs, tests/{skills,research}.rs, content/{skills,research}.json.

- [ ] Probar prerrequisitos, bifurcación excluyente, tres rangos, fondos insuficientes y compra repetida sin doble gasto.
- [ ] Implementar cuatro árboles y general como catálogos; validar DAG y referencias.
- [ ] Probar evidencias fuera de orden, anteriores a pistas, varios experimentos válidos, pistas sin penalización y recompensa única.
- [ ] Implementar diario y mesa como clientes de los mismos comandos; descubrimientos no requieren habilidades que aún bloquean.
- [ ] Probar antigrindeo: combustión pasiva=0 XP; repetir un evento ID=0 XP; rendimientos decrecientes persistidos tras recarga.
- [ ] Ejecutar cargo test -p allomancy-core --test skills y --test research por separado y validador de contenido.

Precondición de C2: P2–P5 verificados. C2 se cierra después de P7, que proporciona la interfaz del ciclo completo hierro/acero. P6 puede avanzar en paralelo; P8 espera C2.

### P6 — Hitos y desbloqueos (Terra)

Depende P3/P5. Archivos: progression.rs, tests/progression.rs, content/loot.json (propiedad transferida tras P3).

- [ ] Implementar reparto propuesto del diseño en datos, recompensa personal y participación verificable.
- [ ] Probar hito repetido, fragmento comerciado, muerte/reconexión, inventario lleno y corte en entrega: como máximo una recompensa efectiva.
- [ ] En cofres existentes sin eventos fiables, usar alternativa respaldada por P0 y documentarla; no suponer retroactividad de loot.
- [ ] Ejecutar cargo test -p allomancy-core --test progression y recorrido de hitos en mundo de pruebas.

### P7 — Libro, árboles, rueda y HUD (Terra)

Depende P4/P5. Archivos: allomancy-ui/src/*.rs, assets/lang/es_es.json; input.rs lo integra Sol.

- [ ] Construir vista de cinco árboles, rangos/costes/requisitos, diario/evidencias y tres niveles de pista.
- [ ] Implementar rueda y HUD con teclado reasignable, estados activos y reservas.
- [ ] Verificar escalas 100/150/200%, texto español largo, resolución 1280×720 y 1920×1080, foco de chat y operación sin depender solo de colores.
- [ ] Confirmar que perder/recrear libro conserva conocimiento; probar clic duplicado y mensajes tardíos.
- [ ] Registrar capturas y pasos reproducibles; Sol valida consistencia entre interfaz y estado real.

Salida C2: ciclo completo hierro/acero jugable, guardable y retomable. No ampliar parejas si falla este ciclo.

### P8 — Tres parejas restantes (Sol divide por archivos después de P5)

Depende C2; hasta tres implementadores con archivos exclusivos, sin editar mod.rs en paralelo.

- [ ] Estaño/peltre (Terra): percepción, atributos, fatiga y subnodos; probar restauración al apagar/morir y coexistencia con efectos vanilla.
- [ ] Zinc/latón (Sol hijo): comportamiento, inmunidades, duración y recuperación; probar jefes y ausencia de control de jugadores.
- [ ] Cobre/bronce (Sol hijo): emisiones, nubes, clasificación y fuentes de mundo; probar oclusión según reglas, nube propia y emisores descargados.
- [ ] Añadir a tests/powers.rs de forma serial por Sol o archivos de prueba separados por pareja.
- [ ] Validar combinadas, subnodos y consumo de las ocho variantes; registrar escenario jugable por pareja.

Salida C3: contenido completo individual, con revisión de equilibrio antes de multijugador.

### P9 — Balance y cooperativo (Sol; Astra para revisión de autoridad si hace falta)

Depende P6/P7/P8. Archivos: content/balance.json, horizon/network.rs, docs/balance.md y testing.md.

- [ ] Medir duración de reservas, tiempo hasta primer descubrimiento, progreso por hora y utilidad de cada pareja; ajustar datos conservando topes.
- [ ] Aplicar pruebas de abuso: XP automático, duplicación de fragmentos, alternar poderes para evitar coste, calma infinita y apilado de atributos.
- [ ] Si P0 acredita servidor, implementar comandos autoritativos con versión, secuencia, límites de frecuencia y deduplicación.
- [ ] Probar dos clientes, desconexión durante compra/vial, replay, objetivo falso y latencia; cliente no concede daño/XP/recompensas.
- [ ] Si servidor es inviable, entregar solo modo individual claramente identificado y registrar cooperativo como pendiente, no terminado.

### P10 — Empaquetado y aceptación (Terra + Sol integrador)

Depende C3 y decisión P9. Archivos: tools/{package,smoke}.ps1, docs/installation.md, manifiestos de paquete.

- [ ] Ejecutar cargo fmt --all -- --check; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace; validador.
- [ ] Empaquetar formato confirmado por P0, fijar versión mínima/exacta y hashes de salida.
- [ ] Instalar desde cero en perfil de prueba; verificar ciclo completo y actualización de guardado anterior, incompatibilidad y desinstalación documentadas.
- [ ] Incluir guía de instalación, controles, respaldo, limitaciones conocidas y licencias de assets originales.
- [ ] Revisión final independiente basada en diff y pruebas; Sol corrige hallazgos y registra C4 con artefacto y evidencia. Ningún test no ejecutado se marca aprobado.

## Paralelismo permitido

P0 → P1 → P2. P3 puede avanzar tras P1 con interfaces fijadas; P5 después de P2. P4 integra P2/P3. P6 y P7 pueden ir en paralelo tras P5 con archivos distintos. P8 se divide por pareja. Sol integra secuencialmente y ejecuta checks tras cada integración; no paralelizar comandos que muten el mismo workspace/lockfile.

## Reanudación para Sol

Leer docs/execution/CHECKPOINT.md primero. Continuar la primera acción no verificada del plan usando evidencia en disco. Mantener requisitos del diseño. No reabrir decisiones aceptadas ni repetir investigación vigente. Redescubrir herramientas/modelos disponibles al cambiar de sesión. Si una API/versión cambió, aislar ese cambio en P0 y documentar impacto antes de reanudar consumidores.
