# P0-capabilities — brief/checkpoint previo
schema_version:1
estado: pending dispatch
modelo:gpt-6.1-terra medium
base:6e5f001da17a3bf58b12ad69badeccbc3f0a815c / codex/alomancia-p0
propiedad exclusiva: experiments/horizon-probe/bundle/api/src/lib.rs, docs/execution/tasks/P0-capabilities.md, docs/execution/evidence/p0-capabilities.md
Objetivo: ampliar probe desechable P0 con APIs públicas exactas del pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca. No mod final ni núcleo. No compilar en paralelo.
Estado previo: source actual compiló exit0 y ejecutó tick,login AttachmentCodec y key+push; controles reales confirmaron una activación sostenida,ninguna en chat/inventario. Mantener estos comportamientos.
Requisitos acotados: (1) aumentar impulso vertical único a0.30 para superar gravedad,y opcional segunda tecla X horizontal0.30 para ensayo pared; máximo3 impulsos por proceso y gate actual; no prometer velocidad total limitada. (2) HUD marcador geométrico pequeño inequívoco (sin texto hardcoded), usando firma real; (3) login servidor emite ping binario canal allomancy_probe:probe al cliente,cliente valida payload y devuelve ack,servidor valida payload recibido y registra resultado;usar versión/payload exacto y no reflejar datos arbitrarios. (4) observar hook de uso de item y muerte/loot con logs acotados sin alterar loot,daño o inventario. (5) si helper exacto disponible,encolar sonido vanilla al entrar y registrar retorno,sin inferir audición (export sin audio). (6) registrar lectura item/inventario fiable por host y ruta de evento si posible; no conceder items ni consumir todavía.
Fuente de firmas: experiments/horizon-probe/runtime/launcher/native/target/fast/build/horizon-native-game-4e31fe35dc1758a4/out/mod_api_generated{,_host}.rs; source exacta runtime/launcher/native/source/fdc87... (resolver bajo experiments/horizon-probe). APIs referenciadas en docs/execution/evidence/p0-api.md como guía,no sustituir firma real.
Checkpoints atómicos antes/después; conservar backup. Logs ALLOMANCY_PROBE sin UUID/nombre/ruta de cuenta; mensajes limitados,no loop ping continuo. Efectos solo mundo desechable. No tocar manifiestos,harness,checkpoint raíz,otros agentes ni runtime. No Cargo/launcher/commits/subagentes. Build y revisión por principal después. Indicar capacidades omitidas por firma y pruebas pendientes en informe, sin inventar API.
Siguiente acción: leer source actual y firmas generadas;anotar intención y preparar ampliación mínima.

## Intención
Ampliar el probe con firmas generadas del pin, sin compilar ni ejecutar; registrar resultados y pendientes para integración serial.

## Resultado
Implementación estática lista para compilación serial por principal. Ver evidence/p0-capabilities.md para firmas, omisión de X opcional y pruebas pendientes. No se ejecutó Cargo ni juego; sin afirmación de éxito en runtime.

## Fixround 1
Smoke v3: ping y ack válidos, inventory Some((36, 0)), sound_queued=false; corregida categoría vanilla de players a player. Pendiente compilación y smoke v4 por principal para comprobar sound_queued=true.
