schema_version: 1
estado: in_progress
fecha: 2026-11-13
agente: /root/p2_horizon_save
propiedad: crates/allomancy-horizon/src/save.rs, src/lib.rs, docs/horizon-storage.md y esta ficha/evidencia
checkpoint: contratos leídos; firmas411 cotejadas; implementación pendiente; sin Cargo/build/juego
siguiente: implementar helpers Tag y backend Host real con Ready/Busy/Fault; conservar gameplay cerrado
## Intención 2026-11-14 — /root/backend_quarantine
Estado: implementación acotada autorizada por /root; sin Cargo/build/juego/commits.
Rama observada codex/alomancia-p2-native; HEAD fed744b9252e14ae6fcd12b314fba61040250fe0. git status sin --work-tree explícito falló con «must be run in a work tree»; no se certifica limpieza, WIP se conserva.
Contrato leído primero: P2-storage-review.md y P2-native.md. Archivos autorizados exclusivos: crates/allomancy-horizon/src/save.rs, docs/horizon-storage.md, esta ficha y docs/execution/evidence/P2-horizon-save.md. No se modifica lib.rs.
Intención previa: tombstone sibling storage_fault por presencia, lectura estructural separada para confirmación, Fault ante inspección incierta y rollback no confirmado con preimagen; marcado desde Compound actual preservando claves; cierre central world_fault; canonicalize en test_world para equivalencia real de rutas Windows. APIs NativeTestConfig/CreditIntent congeladas.
SHA256 previo save.rs: 76DB5E05AA76126DBCB0D998A34119CCC537F25D6DD1A752E792FE8F9C994687.
SHA256 previo ficha (antes de esta intención): B046B2CCC563F6760CF8B539E429CDCB3D45CD376A001F1D881FB99F065FD464.
Pruebas SDK/Host real y gates nativos quedan a /root, dueño exclusivo de Cargo. No Host falso ni tests de texto; no se declara TDD o compilación validada. Edición atómica con temporal y backup únicos; conservar parciales anteriores.

## Entrega para revisión 2026-11-14 — /root/backend_quarantine
Estado: implemented/no native verification. Código y documentación editados; compilación SDK y gates nativos PENDIENTES de /root. No declarar P2 aprobada.
Cambios: sibling por presencia; lectura estructural privada; checks login/acting/normal/replay; Fault ante inspección previa incierta; preimagen + Fault antes de marcar desde Compound actual; preservación state/ajenas/marca existente; confirmada/NO confirmada; cierre world_fault; canonicalize test_world.
Validación: lectura del SDK NBT real, revisión estática y git diff --check exit 0 (avisos EOL de WIP ajeno); ningún Cargo/build/juego/test. Delta de entrega contra preimagen exacta p2-stage1 en evidencia, incluyendo formato de /root. Sin cambios fuera de propiedad. Fuente congelada para /root.
SHA256 save.rs entrega: 00043AF15B0516BBA1CD7EB7E30568D95C031C932FF695F7304C3019866E3BD0.
SHA256 horizon-storage.md: CDD7FCC8D231B62EF610845B0E80AD50672547CB03E7D4C8E3B0FA1C91F38487.
SHA256 delta UTF-8 LF: 4DB750A5B7226B22D8479A241F72C9A948EAFDE60A62DF143CC364226F2FD2D7.
SHA256 evidence/P2-horizon-save.md: 5B1DC8CB16D6A5888B8E3E19D351468FC6CC9546A219080A5E84584ABF49536D.
Límites: aceptación en memoria no fsync; marca NO confirmada conserva Fault RAM y no garantiza próxima apertura. RollbackInventory es fallo artificial del adaptador. Pendientes pruebas SDK/Host real y N0–N9 según evidencia/contrato. No se elimina parcial o backup anterior.
