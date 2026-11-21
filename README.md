# Masamune's MISTBRN

MISTBRN lleva la alomancia a [Horizon](https://playhorizon.dev). Es un mod nativo en Rust inspirado en Mistborn, pensado para que el metal que encuentras, preparas y llevas contigo determine cómo exploras y te mueves por el mundo.

## Qué puedes probar hoy

P3a incorpora **25 ítems y 25 recetas**, con texturas originales, nombres en español e inglés y la pestaña creativa **Alomancia**. Puedes empezar en un mundo existente.

1. Mina cobre para encontrar muestras de estaño y hierro para encontrar zinc. Algunos cofres de estructuras que sigan cerrados también pueden contener un vial.
2. Coloca dos bloques de vidrio en vertical para fabricar dos viales vacíos. Convierte un lingote en dos virutas y combina una viruta con un vial vacío. El hierro y el cobre usan lingotes del juego base.
3. Mantén pulsado el botón de uso durante 32 ticks en supervivencia o aventura. Si hay capacidad, beber añade **25 000 milésimas** a la reserva del metal y devuelve el envase. El servidor guarda los cambios de inventario y reserva en una misma transacción.

La fase actual permite probar el ciclo de obtención, fabricación, consumo y guardado. P4 incorporará los primeros poderes alománticos. P3a se verificó en **macOS arm64** con 24 corridas nativas aprobadas. Windows x64 y Linux x64 son objetivos de publicación con verificación pendiente. [Evidencia de P3a](docs/evidence/p3.md) · [Compatibilidad](docs/compatibility.md)

## Hacia dónde va

La primera demo gira en torno a **hierro y acero**. Atraer metal, empujar una moneda o usar un anclaje debería cambiar cómo cruzas un terreno, frenas una caída o eliges una ruta. La reserva limita cuánto puedes hacer antes de preparar otro vial. Monedas, capa de bruma y otros accesorios están previstos en P3b.

La versión completa aspira a cuatro parejas y ocho metales. Estaño y peltre mejorarán los sentidos y el cuerpo. Zinc y latón influirán en criaturas compatibles. Cobre y bronce servirán para ocultar y detectar actividad alomántica. La práctica y la investigación abrirán técnicas, con pistas opcionales, un árbol por pareja y mejoras generales de eficiencia. Los hitos de exploración y los jefes darán acceso a nuevas parejas. El cooperativo se evaluará después de validar la autoridad del servidor con dos clientes. [Diseño](docs/design.md) · [Plan de ejecución](docs/plan.md)

## Desarrollo

El mod usa el ID `mistbrn` y está fijado a Horizon `nightly-2026-11-20-0a875da`, juego 26.3 y Rust 1.99.1. Prepara esa versión en el launcher con la instancia `mistbrn-dev`. Los scripts usan Bash y las verificaciones requieren `jq`.

```sh
tools/fetch-sdk.sh
tools/check.sh
tools/build-native.sh p3-local
tools/native-test.sh c1 p3-local-c1
tools/native-test.sh c2 p3-local-c2
```

Usa nombres nuevos para cada build y corrida. C1 prueba la fabricación y C2 prueba el consumo y el guardado. Los escenarios abren Horizon y usan mundos de prueba aislados. El SDK descargado, los binarios, los mundos y los logs quedan fuera del código versionado.

- `mod/src/core/` contiene las reglas de juego en Rust puro. `mod/src/` conecta esas reglas con Horizon.
- `mod/data/` y `mod/assets/` contienen ítems, recetas, modelos, texturas e idiomas del paquete `.hmod`.
- [Estado y siguiente fase](docs/STATUS.md) · [Contrato de guardado](docs/persistence.md) · [Historial](docs/history.md)

La distribución prevista es el mod con código y recursos propios. Minecraft, el launcher y sus recursos se obtienen por separado.
