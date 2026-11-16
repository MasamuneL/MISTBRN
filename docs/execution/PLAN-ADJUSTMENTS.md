# Adecuación del plan — 2026-11-16

Esta revisión autorizada modifica la secuencia y el seguimiento del plan de alomancia del 2026-11-12. Prevalece sobre sus dependencias y reglas de seguimiento cuando haya conflicto. No elimina requisitos de seguridad, revisiones ni gates nativos; no acredita pruebas nuevas. El diseño de ocho metales y la separación core/adaptador se conservan.

## Prioridad y entregables

1. Terminar y consolidar los ensayos P2 en curso antes de cambiar de frente. Resolver resultados con oráculo pendiente; una ejecución nativa exitosa sola no cierra un caso.
2. Cerrar P2 mediante los cuatro bloques siguientes, con revisión y evidencia por bloque. Los IDs son organizativos, no sustituyen las fichas P2A/P2B ni N0–N9 existentes.
3. Integrar el mínimo P3 necesario y construir V1, demostración interna hierro/acero.
4. Completar el resto de P3/P4, investigación P5 y UI P7 para C2. P6 conserva sus dependencias. P8 sigue esperando C2.

| Bloque P2 | Alcance de cierre | Evidencia requerida |
|---|---|---|
| Núcleo y formato | Reservas, límites, restos, codec, versiones y transacciones del modelo | Tests core existentes y revisión; no atribuir garantías del Host |
| Transacción nativa | Admisión, identidad, merge, aceptación/replay, rollback y reentrada | N0–N4, con resultados completos y oráculo cuando corresponda |
| Guardado y recuperación | Autosave, cortes, writer, IO y restauración | N5–N8; fronteras reales, archivos completos y alcance documentado |
| Ciclo de vida | Muerte, reconexión, respawn, dimensión y colas antiguas | N9 con estados no triviales y rutas reales |

Cada bloque puede quedar verified/reviewed por separado. P2 solo se cierra cuando todos los criterios aplicables estén verificados y revisados; cualquier excepción requiere una decisión explícita de alcance, nunca un PASS por omisión. N1 y N9 pueden compartir evidencia sin duplicar ejecuciones.

## V1 — demostración interna anterior a C2

Dependencias: P2 cerrado; subconjunto P3 integrado de vial, fuente reproducible de hierro/acero y consumo real. No depende de árboles completos, investigación ni libro final. Un desbloqueo fijo de prueba debe identificarse como tal y no sustituye progresión.

- [ ] Obtener/fabricar un vial mediante una ruta real documentada, beberlo y acreditar reserva con consumo exacto del inventario.
- [ ] Activar y apagar tirón/empujón de hierro/acero sobre un objetivo metálico controlado, con alcance, velocidad y colisión acotados.
- [ ] Mostrar metal activo y reserva en HUD mínimo; no activar con chat/menú; apagar al agotarse.
- [ ] Guardar, cerrar y reabrir: conservar saldo válido y no duplicar el vial; reconectar con poderes apagados.
- [ ] Validar cancelación por muerte, cambio de dimensión y objetivo inválido en las rutas habilitadas.
- [ ] Registrar un recorrido reproducible y evaluación humana de claridad del control, respuesta del movimiento y utilidad del HUD; capturas/video si disponibles.

V1 reutiliza implementación y pruebas de P3/P4/P7; no crea un segundo adaptador ni una UI paralela. P4 se ejecuta primero en este subconjunto y después completa monedas, frenado, desvío y demás casos del plan. P7 adelanta únicamente controles/HUD mínimos. V1 no es C2, versión pública ni aprobación de los ocho poderes. C2 conserva P2–P5 y P7 completos.

## Dos niveles de verificación

- Rápido: formato, tests y clippy del workspace puro; validación y negativos de contenido. Automatizar en CI como tarea de tooling antes de V1, con toolchain y lock fijados, sin distribuir SDK ni recursos del juego. Si la toolchain fijada requiere provisionamiento específico, documentarlo antes de activar ese job.
- Nativo: planificador/build, unicidad del core, hashes de fuentes/artefacto, ensayos del adaptador y oráculos offline. El workspace raíz excluye allomancy-horizon: un cargo test --workspace verde no acredita este nivel. Antes de distribuir, verificar también build sin native-test.

Registrar por resultado commit o hashes de WIP, build, escenario, código de salida, oráculo y alcance. Repetir únicamente lo afectado por cambios o por evidencia inválida; clasificar fallos del harness separadamente, conservando el fallo y sin relajar el resultado esperado para obtener PASS.

## Alcance del servidor dedicado

El servidor dedicado usado para comprobar FIFO/callbacks es infraestructura de ensayo P2. No adelanta P9 ni acredita cooperativo. Cada caso debe explicar qué garantía del adaptador valida y si corresponde a owner integrado o guest dedicado. No trasladar resultados entre ambas rutas sin evidencia. Mantener pendientes las pruebas cooperativas de dos clientes, autoridad, latencia y abuso de P9.

## Seguimiento y recuperación

CHECKPOINT es la única entrada vigente de reanudación. Mantenerlo breve (objetivo: hasta 80 líneas), con ruta/rama/base, operación en curso, build válida, resultados pendientes, bloqueos y próxima acción. Las intenciones/resultados detallados van a fichas o diario enlazado; conservar comandos, logs y recibos recuperables antes de acciones largas.

Mover historia completa a history/ solo al terminar la operación en curso y por su escritor principal. Releer el archivo inmediatamente antes del reemplazo y abortar si cambió; conservar copia íntegra y usar reemplazo atómico. No compactar desde otra sesión mientras se escribe. HANDOFF referencia el checkpoint y aporta contexto, sin mantener una segunda lista contradictoria de acciones vigentes.

Actualizar README y checklist del plan al cierre revisado de cada bloque. Distinguir implemented, verified, reviewed e integrated; ninguna casilla se marca por este cambio documental. Mantener una tabla por gate con estado, evidencia, limitación y próxima acción; no usar cantidad de receipts como porcentaje de producto terminado.

## Aplicación de esta revisión

Cambio documental solamente. No autoriza por sí mismo builds adicionales, merge, limpieza o alteración de mundos. La ejecución ya autorizada conserva su alcance. No reabrir gates válidos ni interrumpir ensayos en curso para reorganizar documentación.
