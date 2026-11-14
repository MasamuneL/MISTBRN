# Evidencia P3 — catálogo interno

Fecha: 2026-11-13 America/Mexico_City
Estado: catálogo y validación estructural local listos; no integrado al SDK.

## Alcance y diseño

Solo se preparan los seis archivos exclusivos P3. Versión `allomancy-content-internal-0.1.0`; no se publica ni transforma a `.hmod`/`.hnmod`. Fixture de Horizon consultada como muestra de formato y quedó sin cambios.

Orden: hierro, acero, estaño, peltre, zinc, latón, cobre, bronce. Hierro/cobre tienen recetas puente internas desde materiales vanilla declarados. Estaño/zinc tienen fuentes de loot/comercio propuestas para mundos existentes, pendientes de verificar. Cuatro fragmentos y definiciones explícitas de hito declaran reclamo único por jugador según el diseño, sin matar al Warden. Se definen muestra, vial vacío, ocho viales llenos, monedas, libro y mesa. Dosis/capacidad de ensayo: 1000 unidades configurables. No se fijan poderes, XP ni rendimientos P5/P6.

Las cantidades son enteros positivos. No hay recetas que usen viales llenos como ingredientes. El puente consumirá vial y entregará dosis/envase atómicamente; el catálogo no simula esa transacción.

## Verificación local

PowerShell `ConvertFrom-Json -AsHashtable` en los tres JSON más checks inline de schema_version, orden, parejas, referencias, cantidades, fuentes propuestas y cierre iterativo de recetas.

Resultado: `JSON_PARSE=PASS`, `METAL_ORDER=PASS`, `PAIR_ORDER=PASS`, `RECIPE_COUNT=26`, `MILESTONES=PASS`, `IDS_REFS_COUNTS=PASS`, `REACHABILITY=PASS`, `PROPOSED_UNVERIFIED=PASS`; exit code **0**. No se ejecutaron tests, build ni Horizon.

## Fuentes pendientes y siguiente acción

Nombres exactos/retroactividad de loot y comercio, reglas de estructuras existentes, recetas en runtime/API411, soporte de materiales vanilla y reclamación de hitos no verificados. Solo el ID de hierro se observó en fixture inalterada; no verifica compatibilidad del catálogo.

Terra puede implementar validador sobre `docs/content-catalog.md`. Luego bridge P2/P3 resuelve consumo atómico y retorno del envase; P6 integra hitos/recompensas únicas. Verificar mecanismos de API/runtime antes de afirmar integración loot/trade.
