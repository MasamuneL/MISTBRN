# MISTBRN — Alomancia para Horizon

Mod en desarrollo para Horizon, inspirado en la alomancia. Windows x64 e individual primero. Diseño y plan aprobados; todavía no hay una versión jugable del mod completo.

## Estado comprobado

P0 ejecutó un probe nativo sobre Horizon `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` con Rust portable1.99.1. Se verificaron compilación/carga, ticks, controles con chat/inventario, impulso básico, item propio, HUD, eventos de muerte/loot, mensajes servidor/cliente y sonido encolado. Un contador versionado inline en el NBT del jugador persistió entre procesos. C0 quedó aprobado tras revisión independiente.

P1 incorpora contratos, núcleo independiente y adaptador nativo mínimo. Pasaron 13 pruebas de comportamiento, un doctest, formato y compilación/ejecución contra el SDK fijado. Tres ticks reales del servidor llegaron al núcleo en un mundo aislado. La revisión independiente sigue pendiente por límite de uso de los agentes; P1 no se ha integrado a main. Ver `docs/contracts.md` y `docs/execution/evidence/p1-integration.md`. P2–P10 siguen pendientes.

No se acreditó todavía consumo resistente a cortes, árboles, poderes completos ni cooperativo. La matriz y sus límites están en `docs/compatibility.md`; los resultados curados en `docs/execution/evidence/p0-runtime-v2.json`, `p0-runtime-v4.json` y `p0-runtime-v5.json`.

## Continuar

1. Leer `docs/execution/CHECKPOINT.md` y contrastar archivos/logs/procesos antes de repetir acciones.
2. Consultar el diseño y plan en `docs/superpowers/`.
3. Cerrar la revisión P1 antes de integrar; las pruebas de cortes y transacciones son obligatorias en P2.

El probe es desechable y está en `experiments/horizon-probe/`. No es el mod final. Sus pruebas usan carpetas y mundos dedicados respaldados.

## Distribución

No incluye Minecraft, recursos preparados, binarios del juego ni material de las novelas. La fixture original solo referencia recursos que ya posee el jugador. Runtime, fuentes upstream descargadas, screenshots, logs crudos y copias de recuperación quedan fuera de Git.
