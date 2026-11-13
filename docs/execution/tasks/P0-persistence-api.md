# P0 persistencia — revisión arquitectónica acotada
schema_version: 2
estado: verified (revisión estática; ejecución de vía inline y crash tests pendientes P2)
fecha: 2026-11-12T23:43:00-06:00
agente: /root/p0_api; modelo asignado Sol high (mismo agente)
base: workspace compartido; sin commit propio
archivos exclusivos: docs/execution/tasks/P0-persistence-api.md, docs/execution/evidence/p0-persistence-api.md
resultado: informe de snapshot, writer FIFO, backup, autosave sin fsync y gates P2; candidato individual sin fork usando NeoForgeData inline
hallazgo esencial: dueño IntegratedServer puede escribir Inventory actual + Guest.attachments desactualizado antes de refrescar y encolar snapshot; no reproducido todavía
alternativa verificada estáticamente: Host player_persistent_data/set_player_persistent_data modifica Guest.saved o acting player_extra inline; fragment y wrappers de build existentes confirmados
operación terminada: informe creado atómicamente; ficha reemplazada con backup
pruebas: lecturas rg/rangos/source/generated; validación documental rutas, fragment y contenido exit0; cero Cargo/juego/tests de corte en este encargo
incidentes: algunas búsquedas iniciales con nombres supuestos inexistentes exit1; rutas localizadas después con rg --files
limitaciones: helpers inline aún no invocados por este agente; sin garantía durable por acción/power-loss ni transacción multijugador/chunk
siguiente acción exacta: principal revisar p0-persistence-api.md; admitir C0 provisional con backend canónico NeoForgeData, añadir invocación inline y failpoints de autosave/renombres/rollback a P2 antes de aceptar viales resistentes a corte
recuperación: informe y ficha/backup son entrada; no tocar informe API previo, checkpoint global, código, source o mundos
## Fixround 1/5 — revisión final documental
estado: verified; listo para revisión/integración del principal
fecha: 2026-11-13 (America/Mexico_City)
agente: /root/p0_api; modelo asignado Sol high
archivos autorizados: docs/compatibility.md, docs/execution/evidence/p0-pin-api.md, docs/execution/evidence/p0-storage.md; estado en esta ficha
recuperación: documentos existentes recuperados; no había temporales propios pendientes; ninguna operación repetida
resultado R1: receta PowerShell completa native-build con USERPROFILE/launcher observado, entorno scratch, base preparada ya existente, bundle absoluto, pin/nightly/perfil/flags y destino v5 nuevo; matriz pública API411 separada de comparación main415
resultado R2: getter normaliza NeoForgeData inválido; root_type no protege corrupción on-disk; P2 requiere player_compound fuera de turno y conservación/rechazo de raíz/namespace inválidos antes de escribir/consumir
operación terminada: todos los documentos actualizados atómicamente con backup; no se tocó fuente, Rust, plan ni checkpoint global
verificaciones: 11 filas Hooks y 11 filas Host cotejadas contra declaraciones/wrappers generados del pin; 2 bloques PowerShell parsean por AST sin ejecutar; git diff --check exit0
restricciones cumplidas: sin build, juego, instalaciones, tests repetidos, commits ni subagentes
siguiente acción exacta: principal revisar R1/R2 y p0-pin-api.md, registrar aceptación de ronda y decisión C0 en su checkpoint; mantener gates P2 sin presentarlos como resueltos