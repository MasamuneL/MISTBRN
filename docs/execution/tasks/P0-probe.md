# P0-probe — brief y checkpoint inicial
schema_version: 1
estado: candidate ready; build and integration pending
modelo: gpt-6.1-terra medium
base: sin HEAD / master / workspace compartido
propiedad exclusiva: experiments/horizon-probe/candidate/, docs/execution/tasks/P0-probe.md, docs/execution/evidence/p0-probe.md
Dependencia: API generada por build real de Horizon fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.
Requisito: probe desechable de P0 para confirmar hooks antes de P1; no núcleo ni arquitectura nueva.
Objetivo: candidato Rust compilable contra firmas exactas de API generada; evidencia estructurada prefijo ALLOMANCY_PROBE de carga, server/client tick, tecla de impulso pequeña gated por client_state.focused/screen y contador attachment del jugador guardado con codec. Registrar contador existente al reconectar, no alterar inventario, daño ni otros datos. No escribir datos personales. Señales con eprintln y contador limitado para no saturar logs.
Preservar contratos: incluir pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca del bundle actual. Solo API pública y helpers confirmados. Límite acumulado de velocidad si disponible; impulso único pequeño por pulsación, sin activar bajo chat/menús/foco perdido. Si API no permite verificar algo, documentar limitación sin simular soporte.
Pruebas: principal compilará e integrará después de finalizar build actual. No Cargo/launcher en paralelo ni tests artificiales con SDK simulado.
Siguiente acción: leer API generada y documentar intención antes de crear candidato. Guardar checkpoint atómico con backup; informe rutas/firmas y limitaciones.

Intención: preparar candidato aislado desde el bundle fijado, con hooks y helpers verificados en la API generada; sin compilación ni ejecución paralela.

Checkpoint final: candidato aislado preparado; informe docs/execution/evidence/p0-probe.md. Sin compilación o ejecución; siguiente acción: build real por principal cuando termine la actual.
