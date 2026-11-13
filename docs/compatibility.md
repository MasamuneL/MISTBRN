# Compatibilidad de Alomancia con Horizon — P0 en curso

No hay todavía un mod jugable ni una versión de Horizon certificada. C0 requiere compilación, carga y ensayos reales; leer código no los sustituye.

## Objetivo fijado

Windows x64. Primer candidato: `nightly-2026-11-08-fdc87ae`, commit `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca`, resuelto desde el identificador de la instalación local mediante GitHub. El nombre de carpeta y hash PE no demuestran por sí solos la procedencia criptográfica del ejecutable. Copia cliente SHA256: `43B6335E03C6EB77CEBD76528A7D1489E5714E21F60DC06635D93D8F0C83F71A`.

Comparación upstream: `36eb98b6f6ec4f22649cbe65bbfd32bc0dc2083f`, catálogo global415. Varios hooks nuevos usan placeholders y no están en el catálogo raíz: no adoptarlos sin compilación. Ambos manifiestos leídos exigen Rust>=1.95/edition2024. El README resumía1.90+, pero el manifiesto fijado gobierna.

El launcher instalado ofrece Rust portable1.99.1 para x86_64-pc-windows-gnu y linker gestionado. Instalación realizada en `experiments/horizon-probe/runtime/launcher/rust/`, exit0. Sin modificación del Rust del sistema. JDK25/Python no acreditados: importan si se regeneran recursos, no deducir que bloquean reutilizar los ya preparados.

## Matriz de capacidad

Todas las capacidades de juego siguen **no probadas**. La tabla detallada de firmas, rutas y líneas, con separación código/documentación/catálogo, está en [p0-api.md](execution/evidence/p0-api.md). Inventario local en [p0-inventory.md](execution/evidence/p0-inventory.md).

| Capacidad | Evidencia estática | Ensayo pendiente / límite |
|---|---|---|
| Entrada | key, mod_key, set_mod_key, client_state | Reasignar; chat/menú/foco; no activar al escribir. |
| Tick | tick, server_tick, player_tick en ambos catálogos | Ejecutar en cliente/servidor; comprobar pausa y freeze. |
| Item/inventario | take/give_items, give_stack, exchange_items | Stack exacto, inventario lleno y corte entre objeto/progreso. |
| Impulso jugador | native_push_player(kick:[f32;3]), cliente, clamp impulso±4 | Colisión normal, velocidad acumulada y anticheat no acreditados. |
| Entidad/proyectil | EntityRef set_velocity; excluye jugadores | Colisión por especie, descarga, cambio dimensión y tags. |
| UI | hud, mod_screen, panel_event | Carga, escalas, foco, accesibilidad y estado del servidor. |
| Combate/loot | attack_entity, mob_killed, loot_drops | Atribución, jefes y cofres ya generados requieren escenarios. |
| Guardado | Attachments + codec/copy_on_death/sync | Round-trip, muerte, migraciones y commit conjunto con inventario. |
| Sonido | play_sound servidor/local | Encolado no prueba audición; custom necesita recursos propios. |
| Servidor | Mensajes limitados por canal/tamaño/frecuencia | Validar protocolo, secuencia, replay y dos clientes; no vanilla. |

## Decisión provisional

Individual sin fork es una hipótesis razonable según el código, pendiente del probe. Conservar progresión/economía en servidor integrado y enviar presentación/impulso a cliente. No congelar todavía el adaptador ni comenzar P1. No hay justificación comprobada para adoptar un fork; si los ensayos revelan falta esencial, presentar extensión mínima y cambio de alcance al usuario.

`EntityRef::set_velocity` excluye jugadores. Un mensaje del servidor seguido de impulso cliente no demuestra física autoritativa: cooperativo queda condicionado a ensayo y posible extensión del motor. Un intercambio atómico en memoria tampoco acredita resistencia a un corte entre inventario y datos del mod.

## Compilación reproducible intentada

El bundle desechable vive en `experiments/horizon-probe/bundle/`: manifest fijo y objeto Hooks con marcador de carga; no implementa alomancia. El launcher compila el juego completo, no una DLL independiente. La primera prueba usa exportación de servidor para evitar modificar instancias.

```powershell
# Ejecutar con HORIZON_* y XDG_* apuntando al runtime aislado descrito en P0.md.
horizon-launcher.exe rust install
horizon-launcher.exe native-build experiments/horizon-probe/bundle --server-output <runtime>/server-engine --version nightly-2026-11-08-fdc87ae --trust-native-code --profile dev
```

Primer build exit1: recursos no preparados en scratch. Se copiaron los recursos existentes de la misma instalación con robocopy /E (sin borrados),30051 archivos,0 fallidos. Segundo build en curso: consultar log/checkpoint antes de repetir. No usar /ignore-pins. No compartir runtime, jars, recursos preparados ni snapshot de cuenta. La copia scratch no incluye cuentas ni mundos.
## Resultado actualizado

El build mínimo terminó con exit0 y el ejecutable emitió el marcador de carga de Hooks al ejecutar --help (exit0). El smoke solo acreditó inicialización de recursos/Vulkan, sin informe final. La ampliación actual del bundle no ha compilado: su invocación fue bloqueada antes de ejecutarse por un límite externo de revisión automática. La revisión independiente también está pendiente por cuota. Todas las filas de juego continúan no probadas; C0 sigue abierto.