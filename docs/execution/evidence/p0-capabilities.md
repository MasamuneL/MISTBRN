# P0 capabilities — implementación estática

Pin: fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca. Probe desechable; compilación y ejecución de esta ampliación pendientes a cargo del principal.

- `Host::fill(&self, &mut HudBatch, [f32; 4], u32)` dibuja dos rectángulos; implementación interpreta x, y, ancho, alto y color ARGB.
- `net_send_to_player(u128, &str, &[u8])` y `net_send_to_server(&str, &[u8])` devuelven `Result<(), String>`; hooks reciben canal y bytes. Canal `allomancy_probe:probe`, ping exacto `[1, 1]`, ack exacto `[1, 2]`; no se reflejan bytes recibidos. Registros acotados a diez.
- `use_item` y `use_item_on` reciben stack y devuelven `Flow`; `living_death` devuelve `Flow`; `loot_drops` recibe vector mutable. Solo se observa, sin cancelar ni modificar. Sus registros comparten límite de diez. El source documenta las dos rutas de uso como servidor; el evento facilita el count del stack.
- `player_stacks(u128) -> Option<Vec<Option<ItemStack>>>` registra total y ocupados al login, sin conceder ni consumir. `play_sound_to(u128, &str, &str, Option<[f64; 3]>, f32, f32) -> bool` registra si se encoló sonido vanilla; no demuestra audición.
- Impulso vertical único de 0.30 conserva gate y límite de tres por proceso; no limita velocidad total. X horizontal era opcional y se omitió.

Pendientes: compilar contra API real y ejecutar HUD, ping/ack (incluido payload inválido), sonido encolado, inventario, uso, muerte/loot y física en mundo desechable; verificar que el gate anterior continúa funcionando. No se hicieron pruebas ni compilación aquí por instrucción. `git diff --check` pasó antes del último ajuste y debe repetirse. No se afirma persistencia, audición ni colisión.

## Fixround 1
Smoke real v3: ping y ack válidos, inventario Some((36, 0)), sonido sound_queued=false. El source exige categoría player (singular); corregido el literal. Pendiente smoke v4 para comprobar sound_queued=true; el retorno no acredita audición.
