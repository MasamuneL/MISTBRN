# MISTBRN — Alomancia para Horizon

Mod en desarrollo para Horizon, inspirado en la alomancia. Plataforma inicial: Windows x64 e individual. Diseño y plan aprobados; todavía no existe una versión jugable del mod.

## Estado comprobado

P0 en curso. Se compiló un probe nativo mínimo sobre Horizon `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` con la toolchain portable Rust1.99.1 del launcher. La compilación terminó con código0 y el ejecutable cargó el objeto Hooks. El arranque gráfico inicializó recursos y Vulkan, pero no produjo informe de smoke completo. Ticks, física, entrada, persistencia, UI y cooperativo siguen sin validar.

El bundle actual amplía la observabilidad de ticks, teclado y attachments; **esta ampliación todavía no está compilada ni revisada**. El probe mínimo que sí compiló se conserva en `docs/execution/evidence/p0-initial-probe.rs`.

## Continuar el desarrollo

1. Leer `docs/execution/CHECKPOINT.md` y contrastar archivos/logs/procesos antes de repetir acciones.
2. Consultar `docs/superpowers/specs/2026-11-12-alomancia-design.md` y `docs/superpowers/plans/2026-11-12-alomancia.md`.
3. Resolver las pruebas pendientes de P0 y cerrar C0 antes de empezar P1.

La matriz provisional y las limitaciones de autoridad y transacciones están en `docs/compatibility.md`. El código del probe está en `experiments/horizon-probe/bundle/`.

## Recursos y distribución

No incluye Minecraft, recursos preparados, binarios del juego, ilustraciones ni textos de las novelas. Las pruebas usan carpetas y mundos desechables. Runtime, fuentes de referencia, logs locales y copias de recuperación quedan fuera de Git. Este repositorio conserva resúmenes de evidencia; no equivale a un SDK ni a un mod instalado.