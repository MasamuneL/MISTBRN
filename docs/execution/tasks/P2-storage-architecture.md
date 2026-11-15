# P2 — revisión acotada de arquitectura de persistencia
schema_version: 1
fecha: 2026-11-13
estado: verified (revisión documental; P2 nativo pendiente)
agente: /root/p2_storage_architecture
rama: codex/alomancia-p2
base_P2A: 5f679d06297cddef25031d239df95b4693fca5d0
propiedad: docs/execution/tasks/P2-storage-architecture.md; docs/execution/evidence/p2-storage-architecture.md

## Intención antes de editar
Revisar contrato congelado SavedPlayer y backend NeoForgeData inline del pin API411; concretar identidad estable, validación NBT, secuencia inventario/namespace/core, rollback/reentrada y ensayos nativos P2. Separar dictamen arquitectónico de aprobación nativa. No reabrir C0 sin evidencia nueva. Solo lectura del SDK original y documentos ajenos. Sin código, Cargo, runtime, commits ni subagentes.

Ambos documentos propios no existían al comenzar. Escrituras mediante temporal en el mismo directorio y promoción; toda sustitución conservará .bak. Otros cambios del workspace pertenecen a principal/agentes y se preservan. No afirmar crash/durabilidad con pruebas del modelo.

Siguiente acción: leer fuentes locales con líneas exactas, comunicar defectos concretos al principal, redactar evidencia y verificar diff documental.
## Reanudación y última intención
Fecha: 2026-11-13, America/Mexico_City. HEAD observado al retomar: 1da2b11. No se revisó el código de ese commit.
El informe preliminar ya existe. Conservarlo como .bak y corregir la distinción producto/ensayo: el wrapper externo no satisface la protección nativa. Incorporar fuentes ya leídas sobre world_root vacío, preparación integrada anticipada y HEAD cancellable en WorldRequest::prepare_world. No repetir investigación, ejecutar planificador, compilar ni tocar otras rutas. Después verificar solo ambos documentos y entregar al principal.
## Resultado y cierre
fecha_cierre: 2026-11-13T21:20:33.8612466-06:00
Informe entregable: docs/execution/evidence/p2-storage-architecture.md. Contrato P2B apto como frontera, implementación no evaluada; C0 inline se conserva, P2 nativo pendiente. Añadidos protocolo Ready/Busy/Fault, marker estable, ByteArray NBT, rollback/reentrada, ciclo de vida, riesgos priorizados y gates N0–N9.

Requisito de producto aclarado: preflight externo no sustituye guard nativo. server_about_to_start/config_loading/WorldOpenReady no dan barrera previa suficiente; fuentes prueban world_root vacío y WorldLoader iniciado antes. Candidato concreto: HEAD cancellable del bundle sobre WorldRequest::prepare_world, ruta desde request.save_root, ControlFlow<Result<PreparedWorld>>, cancelación con Err anterior a migración/adopción. Mecanismo documentado en mixins instalado5a64af9; target aplicado/compilado y N0 todavía pendientes. No se modificó SDK ni se propone fork.

Verificación realizada: lecturas selectivas del pin y generados existentes; branch/HEAD al retomar1da2b11; validación UTF8 estricta, secciones/gates requeridos, ausencia de temporales, whitespace y marcadores. Script documental exit0/PASS. git diff --no-index --check entre cada .bak y vigente emitió cero diagnósticos; exit1 esperado porque los documentos difieren. El primer script trató erróneamente ese exit1 como fallo; se corrigió solo la interpretación, sin ocultar errores de contenido. Avisos CRLF se evitaron en la comprobación con core.autocrlf=false sin cambiar configuración.

Alcance: solo ficha/informe propios y sus .bak mediante temporales/promoción atómica. Sin Cargo, launcher, planificador mixins, juego, corte real, Rust, commits ni subagentes. Los gates son propuestas verificables, no resultados ejecutados.

Siguiente acción exacta del principal: revisar este dictamen; asignar implementación nativa del guard de bundle y horizon/save.rs con plan/fingerprint y build del pin; luego probar N0 sin wrapper y N1–N9 en mundos dedicados. No habilitar gameplay por el solo visto bueno del contrato ni por pruebas core.