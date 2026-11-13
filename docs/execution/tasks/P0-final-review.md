# P0-final-review — brief/checkpoint previo
schema_version:1
estado:verified (fixround1: R1/R2 ADDRESSED; C0 aprobado para iniciar P1)
modelo:gpt-6-astra high (revisión arquitectura/persistencia compleja de P0)
base:6e5f001da17a3bf58b12ad69badeccbc3f0a815c / codex/alomancia-p0
propiedad exclusiva revisor: docs/execution/evidence/p0-final-review.md y docs/execution/tasks/P0-final-review.md
Principal solicita dos veredictos: cumplimiento P0 y calidad/procedencia suficiente para continuar P1. No core/modfinal todavía; no marcar C0 por generación de código ni por pruebas artificiales.
Requisitos P0 exactos:
- Leer instrucciones repositorio upstream,guía y ejemplos;fijarcommit/release,toolchain y comandos.
- Inventariar instalación local/recursos;distinguir faltantes de carenciasAPI.
- Compilar/cargar modmínimo y probar entrada,tick,item,impulso concolisión,UI,combate/loot,guardado,sonido yservidor.
- Tabla capacidad:firma/ruta upstream,pruebaejecutada,soportada/no/no probada yalternativa.
- Comandos exactos ylogs;carencias esenciales exigen extensiónmínimaoforkpropuesto,no compatibilidadinventada.
SalidaC0:revisiónfijada ydecisiónintegración. No inferir que esto entregue P2/P4/P7/P9.
Restricciones globales verbatim del plan:
- Plataforma inicial Windows x64; ocho metales; español; sin dependencias Fabric/Forge.
- No inventar firmas ni compatibilidad del SDK. Fijar revisión y toolchain comprobados en P0.
- No tocar mundos personales: pruebas sobre mundos dedicados respaldados.
- No distribuir recursos de Minecraft ni ilustraciones/textos de las novelas.
- Cada cambio persistente y cada tarea deben poder retomarse conforme a CHECKPOINT.md.
- Los valores de balance del diseño son hipótesis de ensayo, configurables.

Archivos de contexto: docs/compatibility.md; docs/execution/evidence/p0.md,p0-runtime-v2.json,p0-runtime-v4.json,p0-runtime-v5.json,p0-persistence-api.md; diseño y secciónP0 plan si falta. Paquete de diff real (tracked+untracked curados) en experiments/horizon-probe/runtime/p0-final-review.patch. SourceSDK fijada en experiments/horizon-probe/runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca/. Logs originales locales evidence/*.log si hace falta corroborar resultado,nunca reenviar datos personales.

Hechos principales a contrastar,no aceptar solo reporte: cinco builds seriales exit0; source final v5 hash enJSON; pruebas en mundoscratch respaldado; input repeat/menu/chat; motorvelocidadascenso/techo con muestrasno maxtrayectoria; itemfixturepropio adquirido/usecallback; HUDPNG local inspeccionado; death/loot de /kill; sonido encolado no audible; net delservidorintegrado no2clientes; inlineNeoForgeData0→1 entreprocesos. La vía Attachment presenta autosavestale; decisión propuesta usa inlineenGuest.saved sinfork y exige gatesP2crash/rollback/fsyncnoacreditado. Valora si justifica C0 y qué queda como riesgo explícito;si gapcargaesencial,identificar concreto sin darporhecho.
No ejecutar Cargo/juego/instalación/testsyaacreditados ni editar fuente/otrosinformes/Checkpointroot ni subagentes ni commits. Leer diff/evidencias y APIs relevantes. Ficha atómica conbackup antes/después. Informe archivo con hallazgospriorizados,veredictos separados,recomendaciónC0y límitesnoverificables. No necesitamos reproducir cronología ni copiarlogslargos.
Siguienteacción:leerdiff/brief/evidencias y emitir revisión independiente.

Revisión iniciada: contraste independiente de diff, procedencia runtime y snapshot inline; sin ejecutar builds ni juego.

Resultado: informe p0-final-review.md creado. Calidad y viabilidad aprobadas para P1; cierre P0 requiere receta build reproducible y referencias API411 (R1). R2 documenta raíz NeoForgeData inválida normalizada por SDK, requisito P2. No builds/juego/tests/commits. Verificados hash v5, resultados storage1/2, logs controles/capacidades, HUD y fuentes snapshot/writer. Lecturas y escritura terminaron exit0. Siguiente acción: principal corregir R1, registrar R2 y cerrar C0 sin repetir pruebas vigentes.

QA final: ruta de guard corregida a lib.rs:41. Replace con backup nulo falló antes de reemplazar; temporal inspeccionado y reemplazo con backup explícito completado. Informe vigente y backups confirmados. Sin mutaciones fuera de propiedad.

Re-review fixround1 iniciada: revisar únicamente R1/R2 documentales en compatibility.md, p0-pin-api.md y p0-storage.md; actualizar veredictos e informe propios. Sin nuevas pruebas, builds ni juego. Recuperación: leer sección de ronda final antes de repetir.

Re-review fixround1 terminada: R1 ADDRESSED (receta completa build y matriz pública API411); R2 ADDRESSED documentalmente (normalización getter y validación raw fuera de turno como gate P2). Informe vigente aprueba P0/C0 para iniciar P1 sin fork. Lecturas documentales y referencias existentes exit0; cero build/juego/tests repetidos/commits/subagentes. Informe y ficha reemplazados atómicamente con backups. Siguiente acción: principal registrar C0 integrado e iniciar P1 manteniendo gates P2.
