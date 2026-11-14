# Catálogo interno de contenido — borrador P3

Versión `allomancy-content-internal-0.1.0`; `schema_version: 1`. Estos JSON son datos internos del proyecto. No son un paquete `.hmod`, `.hnmod`, formato de Horizon, ni promesa de que el SDK acepte sus IDs o recetas. El fixture `experiments/horizon-probe/content` solo se consultó como ejemplo de sintaxis observada y quedó intacto.

## Archivos

- `content/metals.json`: ocho metales en orden de núcleo, cuatro parejas, IDs de material/muestra, items auxiliares y valores configurables de ensayo.
- `content/recipes.json`: recetas internas puente de hierro/cobre, aleaciones, muestras, objetos auxiliares y ocho viales llenos.
- `content/loot.json`: fuentes para análisis de alcanzabilidad. Loot/comercio usa `status: "proposed"`, `verified: false`; no acredita integración ni retroactividad.

IDs propios `allomancy:<snake_case>`. Requisitos vanilla declarados: `minecraft:iron_ingot`, `minecraft:copper_ingot`, `minecraft:glass`, `minecraft:paper`, `minecraft:oak_planks`; todos siguen con `verified: false` a nivel de integración. Solo hierro aparece en la receta del fixture. Aleaciones y estaño/zinc son items internos. Estaño/zinc propuestos en bastión/guardián anciano para mundos existentes, más alternativa de comercio; sin requisito de Warden.

## Esquema mínimo para Terra

```text
Catalog = { schema_version: integer(1), catalog_version: string,
            scope: "internal-catalog-not-hmod-not-hnmod", metals: Metal[],
            pairs: Id[], items: Item[], milestones: Milestone[] }
Metal = { id: Id, order: integer(0..7), pair: Id, role: string,
          item: Id, sample: Id, assay_units_per_sample: integer > 0,
          vanilla_input?: Id }
Item = { id: Id, kind: string, properties?: object, milestone?: Id }
Recipe = { id: Id, ingredients: [{ id: Id, count: integer > 0 }...],
           result: { id: Id, count: integer > 0 } }
RecipesCatalog = { recipes: Recipe[], sample_recipes: Recipe[], filled_vial_recipes: Recipe[] }
LootCatalog = { sources: Source[], trades: Trade[] }
Source = { id: Id, kind: string, status: string, verified: boolean,
           outputs: [{ id: Id, count: integer > 0 }...] }
Trade = { id: Id, status: string, verified: boolean, output: { id: Id, count: integer > 0 } }
Milestone = { id: Id, fragment: Id, unlocks_pair?: Id, grants?: "allomancy:final_mastery", claim_once_per_player: boolean, requires_warden_kill: boolean }
Id = namespace:path; IDs propios usan namespace `allomancy`.
```

Reglas de validación:

1. Parsear los tres documentos; rechazar versiones/esquemas desconocidos e IDs duplicados dentro de cada tipo de entidad (metales, parejas, items, recetas, fuentes).
2. Comprobar orden/pareja: hierro/acero, estaño/peltre, zinc/latón, cobre/bronce.
3. Toda referencia propia en receta/fuente/hito apunta a metal, pareja, item, muestra, hito u output definido; referencias vanilla están declaradas en fuentes.
4. Cada receta tiene ingrediente(s); cada `count` y `result.count` es entero positivo dentro de `u32`.
5. Exigir tipos JSON boolean reales para los indicadores de hito y verified de fuentes/trades. El grafo ingrediente → resultado debe ser acíclico, incluso cuando un ciclo sea alcanzable desde fuentes. Cierre de alcanzabilidad desde outputs de fuentes: iterar recetas cuyos ingredientes sean alcanzables y añadir resultados hasta estabilizar. Informar productos no alcanzables; una fuente propuesta cuenta solo en el escenario provisional y retiene `verified: false`.
6. Ningún vial lleno puede ser ingrediente. Consumo de vial y devolución del envase son una operación atómica del puente P2/P3, no una receta.

`count` cuenta objetos, no XP/potencia. `assay_units_per_sample` y `empty_vial_capacity_units` son parámetros de ensayo configurables, no balance final P5/P6. No se fijan consumos, rendimientos, progresión ni poderes.

## Hitos y límites

`grants` es metadato simbólico provisional: si está presente debe ser la cadena `allomancy:final_mastery` (null, booleanos, números y cadena vacía son inválidos); no declara un item ni una API real de P6 o Horizon. Los indicadores booleanos son valores JSON `true`/`false`, no cadenas.

Fragmentos: bastión (estaño/peltre), guardián anciano (zinc/latón), ciudad antigua (cobre/bronce), dragona (maestría final). Cada hito declara reclamo único por jugador. Ninguno exige matar al Warden.

Pendiente de confirmar en runtime/API411: materiales vanilla, recetas internas, método retroactivo de loot/comercio en mundos existentes, acceso a cofres, entrega/reclamo de fragmentos y transacción vial/inventario. No se afirma validación en Horizon.
