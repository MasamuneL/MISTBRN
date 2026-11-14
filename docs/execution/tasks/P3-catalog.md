# P3 catálogo interno
schema_version: 1
fecha: 2026-11-13 America/Mexico_City
estado: verified-local
base: 24b4b2c5759d746caab21b67f68f6766d688548e
rama: codex/alomancia-p2
propietario: subagente P3 datos auxiliares
archivos_exclusivos: content/metals.json, content/recipes.json, content/loot.json, docs/content-catalog.md, docs/execution/tasks/P3-catalog.md, docs/execution/evidence/p3-catalog.md

## Intención previa
Crear un catálogo interno explícitamente distinto de .hmod/.hnmod y del SDK Horizon, con esquema simple para validación Terra posterior. No tocar fixture horizon-probe, SDK, assets ni otros archivos. Fuentes exactas de loot/trade no verificadas: marcarlas propuestas y no afirmar integración. Antes de cerrar, validar parseo JSON y reglas básicas de IDs, referencias, cantidades enteras positivas y alcanzabilidad por fuente; no ejecutar tests/builds.

## Evidencia inicial
HEAD observado 24b4b2c5759d746caab21b67f68f6766d688548e. Cambios previos visibles ajenos: CHECKPOINT.md, p0-initial-probe.rs, bundle/api/src/lib.rs y P2.md; preservar sin modificar.

## Estado / próxima acción
Intención registrada antes de editar. Próximo: emitir los tres catálogos + documentación mediante reemplazo atómico con .tmp/.bak por archivo; comprobar el alcance del diff.

## Ajuste previo a validación
La revisión del esquema detectó que los IDs propios de ingotes/muestras/viales llenos deben declararse explícitamente y que hierro/cobre necesitan receta puente desde vanilla. Intención: añadir esas declaraciones/recetas sin cambiar forma del contrato; luego reemplazar los dos JSON con .tmp/.bak y validar cierre.


## Resultado
Tres JSON parseados y validados localmente con PowerShell (exit 0): orden de 8 metales, 4 parejas, 4 hitos explícitos de reclamo único y 26 recetas, referencias/cantidades válidas y productos alcanzables desde fuentes declaradas. Loot/trade propuestos permanecen verified:false. Schema disponible en docs/content-catalog.md antes de consumidores. No test/build/Horizon.

## Límites y siguiente acción
Sin verificar: mecanismos exactos de loot/trade y retroactividad, compatibilidad API411, materiales vanilla excepto referencia de fixture, transacción vial/inventario e integración de hitos. Siguiente: Terra implementa validador; bridge P2/P3 implementa consumo atómico y devolución de envase; P6 integra hitos únicos.


## Ajuste de hitos previo al cierre
El diseño incluye reparto de desbloqueos por hitos; para evitar IDs de milestone implícitos, registraré cuatro definiciones explícitas en metals.json y validaré sus referencias. Se mantiene reclamo único por jugador y no Warden.


## Limpieza final previa
Archivos finales comprobados en sus rutas. Voy a retirar solo los cinco .bak generados por esta tarea, todos de archivos creados en el turno (content/metals.json, content/recipes.json, docs/content-catalog.md, docs/execution/evidence/p3-catalog.md, docs/execution/tasks/P3-catalog.md); preservar otros .bak del principal/P2. Después cotejar status y finales.

## Cierre de acciones
Eliminados los cinco .bak propios listados; archivos finales presentes. Cambios ajenos visibles en status pertenecen a principal/P2 y se preservan. Cierre P3 listo para entrega al principal; integración y validación Horizon permanecen fuera de esta subtarea.
