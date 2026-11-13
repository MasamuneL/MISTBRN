# P0-content — brief/checkpoint inicial
schema_version:1
estado:pending dispatch
modelo:gpt-6-luna medium
base:6e5f001da17a3bf58b12ad69badeccbc3f0a815c / codex/alomancia-p0
propiedad exclusiva: experiments/horizon-probe/content/, docs/execution/tasks/P0-content.md, docs/execution/evidence/p0-content.md
Objetivo: fixture declarativa mínima original para probar registro y receta de un item; no P3 ni catálogo final.
Crear horizon-mod.toml id allomancy_probe_content,version0.0.1,api0.2,side both,capability registries. Item allomancy_probe_content:probe_metal copia template minecraft:iron_ingot;max stack16 si firma declarativa real confirmada. Recipe shapeless 1minecraft:iron_ingot ->1probe_metal. Modelo/items/lang en_en(es realmente en_us) y es_es con título Muestra de prueba / Probe sample. Solo referencias a modelos/texturas vanilla existentes: no copiar png,jars ni recursos. No código ni Cargo ni binarios.
Usar esquemas exactos de revisión fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca en experiments/horizon-probe/runtime/launcher/native/source/<pin>/docs/mods.md y crates/cargo-horizon-mod/template/content/. Nota: template item.json es asset renderer,la declaración registry es data/<namespace>/horizon/item/probe_metal.json. Confirmar rutas antes de crear,no inventar campos.
No ejecutar juego/Cargo/instalación/commits/subagentes. Validar sintaxis JSON local y reportar comandos/resultados; no afirmar carga del juego. Checkpoints atómicos antes/después con backup. Principal controla --dev-mod integración y pruebas. Mantener8 archivos o menos.
Siguiente acción: leer docs/template y registrar intención antes de escribir.
