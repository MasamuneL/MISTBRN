# P2A — revisión independiente de reservas
schema_version: 3
fecha: 2026-11-13 America/Mexico_City
estado: reviewed
base: 24b4b2c5759d746caab21b67f68f6766d688548e
corte: 5f679d06297cddef25031d239df95b4693fca5d0
rama: codex/alomancia-p2
propiedad: esta ficha y docs/execution/evidence/p2-reserves-review.md

## Checkpoint antes de revisión
Encargo: revisar únicamente P2A en el corte inmutable con git show/diff, frente a diseño aprobado, plan, contrato y evidencia. No tomar cambios posteriores de P2B/P3 como parte de este dictamen. No Cargo, launcher, runtime, commits ni subagentes. CHECKPOINT/manifiestos y fuentes pertenecen al principal/implementadores.

Leídos diseño, ficha P2/P2A, contrato docs/reserves.md, entrega evidence/p2-reserves.md y diff de lib/reserves/tests. La revisión comprobará aritmética/unidades, fracciones y agrupación, agotamiento, ocho metales, topes, rechazos sin mutación, serde y ciclo de vida, además de evidencia existente. Se inspeccionarán logs sin reejecutar pruebas. Dictamen pendiente.

## Checkpoint después de revisión
Revisión estática del corte inmutable completada. Spec compliance P2A approved y calidad/correctness P2A approved; sin hallazgos accionables P0–P3. Contrato de fracciones, ticks, agotamiento, topes, ocho metales, overflow/rechazos sin mutación, serde y muerte/reconexión cotejado con código y 16 pruebas conductuales. Dictamen y límites en docs/execution/evidence/p2-reserves-review.md.

Logs RED/GREEN/workspace/fmt leídos; sus cuatro SHA256 coinciden con entrega. RED 16 assertions, GREEN 16, workspace 13+16+1; exit 101/0/0/fmt 0 son los registrados por implementador, no ejecuciones nuevas del revisor. git diff --check 24b4b2c..5f679d0 ejecutado por revisor exit 0. Sin Cargo/launcher/runtime/commits/subagentes ni cambios fuera de informes propios. Cambios concurrentes P2B/P3 conservados y excluidos del dictamen.

Siguiente acción exacta: principal revisa/incorpora ambos informes y registra aprobación P2A; continuar codec/integración y gates pendientes. No declarar fase P2 completa. Releer los dos informes tras reemplazo atómico para verificar entrega y ausencia de temporales.

Verificación de entrega: ambos informes releídos completos tras escritura, contenido y dictamen presentes; no temporales pendientes. Solo se corrigió espaciado editorial antes de cierre.
