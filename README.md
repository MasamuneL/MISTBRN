# MISTBRN — Alomancia para Horizon

Mod en desarrollo para Horizon, inspirado en la alomancia. El objetivo inicial es Windows x64 y juego individual. El diseño y el plan están aprobados; **todavía no existe una versión jugable del mod completo**.

## Estado

- **P0 y P1 integrados.** P0 verificó un probe nativo sobre el SDK fijado, incluyendo carga, ticks, controles, persistencia inline y otras capacidades básicas. P1 incorporó contratos, núcleo independiente y un adaptador nativo mínimo; sus pruebas y un smoke nativo quedaron documentados y revisados.
- **P2 parcial.** Hay trabajo de reservas, persistencia y adaptación nativa, con pruebas de apertura, economía y cortes en condiciones delimitadas. Quedan gates por completar y revisar; los resultados de casos concretos no acreditan P2 completa ni durabilidad general.
- **P3 parcial.** Existe trabajo de catálogo y validación interna, sin poderes jugables integrados. Las fases posteriores siguen pendientes.

Los resultados, condiciones y límites constan en [compatibilidad](docs/compatibility.md), [contratos](docs/contracts.md) y las evidencias de [P1](docs/execution/evidence/p1-integration.md) y [P2](docs/execution/evidence/p2-native-cuts.md). Para el estado operativo más reciente y sus excepciones, consultar el [checkpoint](docs/execution/CHECKPOINT.md); sus entradas posteriores pueden actualizar un resumen anterior.

## Comprobaciones de desarrollo

La ruta rápida comprueba el núcleo independiente desde la raíz del repositorio, con Rust y las dependencias fijadas disponibles:

```powershell
cargo test --locked --offline --workspace
cargo fmt --all -- --check
```

El workspace raíz incluye `crates/allomancy-core` y **excluye `crates/allomancy-horizon`**. Estas órdenes no compilan ni prueban el adaptador en Horizon. Antes de repetir pruebas, contrastar el checkpoint y la evidencia existente; coordinar Cargo con cualquier ejecución activa.

La ruta nativa requiere el SDK y runtime fijados, los recursos legítimos del jugador y el staging del adaptador con el núcleo. Consultar [compatibilidad](docs/compatibility.md), [evidencia de staging](docs/execution/evidence/p1-stage.md) y el checkpoint antes de usar `tools/build-native.ps1` o ejecutar el juego. Una compilación nativa por sí sola no acredita los gates de comportamiento, persistencia o recuperación.

## Continuar el trabajo

1. Leer el [checkpoint](docs/execution/CHECKPOINT.md) completo, en especial las entradas más recientes, y contrastar el worktree, cambios, procesos y artefactos reales antes de actuar.
2. Consultar las [adecuaciones vigentes del plan](docs/execution/PLAN-ADJUSTMENTS.md), el diseño y plan en `docs/superpowers/` y las evidencias y tareas enlazadas desde el checkpoint.
3. Preservar cambios y resultados existentes; continuar los gates pendientes según el estado real, sin repetir ensayos válidos sin motivo ni declarar completa una fase por un resultado parcial.

El probe desechable está en `experiments/horizon-probe/`; no es el mod final. Sus pruebas usan carpetas y mundos dedicados respaldados.

## Distribución

El repositorio no incluye Minecraft, recursos preparados, binarios del juego ni material de las novelas. La fixture original solo referencia recursos que ya posee el jugador. El runtime, fuentes upstream descargadas, capturas, logs crudos y copias de recuperación quedan fuera de Git.
