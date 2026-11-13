# P0 — firmas públicas del pin ejecutado

Revisión única: `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` (`nightly-2026-11-08-fdc87ae`), `NATIVE_HOOKS_API = 411`. Fecha: 2026-11-13. Autor: `/root/p0_api`, Sol high. Cotejo documental de source y generados existentes; ningún build, juego o test ejecutado en esta revisión.

`S = experiments/horizon-probe/runtime/launcher/native/source/fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca/`; `C = S/crates/horizon-client/src/`; `W = S/crates/horizon-world/src/`.

`G = experiments/horizon-probe/runtime/launcher/native/target/fast/build/horizon-native-game-4e31fe35dc1758a4/out/`.

`H = G/mod_api_generated.rs` (trait público Hooks), `T = G/mod_api_generated_host.rs` (trait público Host), `I = G/mod_api_host_generated.rs` (implementación concreta de Host). Tipos reexportados por `horizon_client::mod_api`. `C/game/native_hooks.rs:1170` fija API411; `S/.horizon-revision` contiene el SHA completo. El bundle exporta `pub fn hooks() -> Box<dyn Hooks>` y consume los contratos siguientes. Los métodos privados `Game::native_*` solo señalan el cuerpo que ejecuta Host/dispatch; **el adaptador no los invoca como API pública**.

## Hooks implementados por el probe v5

Las firmas de esta tabla son métodos de `Hooks`. `Flow::Continue` conserva la acción normal; no representa consumo o daño confirmado. La presencia/ejecución del hook no acredita todas sus rutas.

| Contrato público exacto | Declaración | Cuerpo/dispatch de la revisión fijada | Evidencia runtime acotada |
|---|---|---|---|
| `fn tick(&self, host: &mut dyn Host, event: &mut TickEvent)` | H:10 | C/game/native_hooks.rs:2361 | Marcador cliente, filtrado por window. |
| `fn server_tick(&self, host: &mut dyn Host, event: &mut ServerTickEvent<'_>)` | H:47 | C/game/native_hooks.rs:2571 | Marcador servidor integrado. |
| `fn hud(&self, host: &mut dyn Host, event: &mut HudEvent<'_>)` | H:99 | C/game/native_hooks.rs:2623 | Dibujo cyan/negro capturado. |
| `fn key(&self, host: &mut dyn Host, event: &mut KeyEvent<'_>)` | H:103 | C/game/native_hooks.rs:2813 | Z down/repeat/up y gate menús/chat; no Alt-tab físico. |
| `fn use_item_on(&self, host: &mut dyn Host, event: &mut UseItemOnEvent<'_>) -> Flow` | H:279 | C/game/native_hooks.rs:7963 | Compilado; callback específico on-block no acreditado por el resultado use_item. |
| `fn use_item(&self, host: &mut dyn Host, event: &mut UseItemEvent<'_>) -> Flow` | H:287 | C/game/native_hooks.rs:8004 | Callback del item propio con stack count1; no consumo. |
| `fn net_from_client(&self, host: &mut dyn Host, event: &mut NetFromClientEvent<'_>)` | H:359 | C/game/native_hooks.rs:8361 | ACK de payload fijo validado. |
| `fn net_from_server(&self, host: &mut dyn Host, event: &mut NetFromServerEvent<'_>)` | H:362 | C/game/native_hooks.rs:8366 | Ping fijo y envío ACK. |
| `fn player_logged_in(&self, host: &mut dyn Host, event: &mut PlayerLoggedInEvent<'_>)` | H:579 | C/game/native_hooks.rs:8442 | Login, inventario leído, contador inline y mensajes. |
| `fn loot_drops(&self, host: &mut dyn Host, event: &mut LootDropsEvent<'_>)` | H:772 | C/game/native_hooks.rs:8951 | `/kill` de pig generó callback; no participación/jefes. |
| `fn living_death(&self, host: &mut dyn Host, event: &mut LivingDeathEvent<'_>) -> Flow` | H:3019 | C/game/native_hooks.rs:6567 | `/kill` generó callback; no atribución de daño. |

## Host consumido por el probe

Las firmas son métodos de `Host`; la columna I confirma que el Game real implementa el método, además de la declaración/default del trait T.

| Contrato público exacto | Declaración / implementación Host | Cuerpo de la revisión fijada | Límite relevante |
|---|---|---|---|
| `fn window(&self) -> bool` | T:831 / I:178 | C/game/native_hooks.rs:10726 | Distingue Game de ventana; no autorización del comando. |
| `fn mod_key(&self, mod_id: &str, name: &str, default: &str) -> String` | T:418 / I:86 | C/game/native_hooks.rs:11189 | Lee binding elegido/default; UI de reasignación no ensayada. |
| `fn client_state(&self) -> Option<NativeClientState>` | T:803 / I:165 | C/game/native_client_services.rs:239 | Probe filtra focused, logged_in y screen==Playing. |
| `fn push_player(&mut self, kick: [f32; 3])` | T:491 / I:99 | C/game/native_motion.rs:30 | Impulso cliente; clamp del impulso por componente, no de velocidad total. |
| `fn fill(&self, batch: &mut HudBatch, rect: [f32; 4], argb: u32)` | T:375 / I:74 | C/game/native_hooks.rs:10779 | Dibujo HUD observado a1280×720. |
| `fn net_send_to_player(&mut self, player: u128, channel: &str, data: &[u8]) -> Result<(), String>` | T:502 / I:101 | C/game/native_net.rs:174 | Aceptado/encolado no garantiza entrega. |
| `fn net_send_to_server(&mut self, channel: &str, data: &[u8]) -> Result<(), String>` | T:507 / I:102 | C/game/native_net.rs:230 | Payload cliente debe validarse en servidor. |
| `fn play_sound_to(&mut self, uuid: u128, sound: &str, category: &str, at: Option<[f64; 3]>, volume: f32, pitch: f32) -> bool` | T:1569 / I:293 | C/game/native_hooks.rs:10054 | Retorno encolado; categoría usada `player`; sin audición real. |
| `fn player_stacks(&self, uuid: u128) -> Option<Vec<Option<ItemStack>>>` | T:524 / I:107 | C/game/native_menus.rs:232 | Lee 36 slots; no modifica ni consume. |
| `fn player_persistent_data(&self, uuid: u128) -> Option<Tag>` | T:1680 / I:309 | C/game/native_player_state.rs:386 | Getter normaliza NeoForgeData inválido a vacío: no valida raw on-disk. |
| `fn set_player_persistent_data(&mut self, uuid: u128, data: Tag) -> bool` | T:1685 / I:310 | C/game/native_player_state.rs:398 | Sustituye compound completo de mods; preservar namespaces. No commit durable. |

El contador AttachmentType usa `AttachmentType<T>::get<H: AttachmentHost + ?Sized>(&self, host: &mut H, holder: Holder) -> Result<T, AttachmentError>` y `set<H: AttachmentHost + ?Sized>(&self, host: &mut H, holder: Holder, value: T) -> Result<(), AttachmentError>`, con `T: Clone + Send + Sync + 'static` (**W/attachments.rs:95–102,122–128**). Ambas delegan en `Host::attachments(&mut self, holder: Holder) -> AttachmentResult<&mut Attachments>` (**T:46 / I:10**, cuerpo **C/game/native_hooks.rs:10355**). Ese round-trip no convierte Attachments en autoridad económica aceptada.

## Contratos disponibles para P2, sin ensayo específico

- `Host::player_compound(&self, uuid: u128) -> Option<Tag>`: **T:1637 / I:301**; cuerpo **C/game/native_player_state.rs:131–135**. Snapshot completo de un jugador alcanzable **fuera de su turno**; devuelve None dentro del turno. P2 debe usarlo antes de escribir para distinguir NeoForgeData ausente de raíz presente inválida y preservar/rechazar esta última sin consumir ítems. El guard `root_type` del probe solo ve el valor ya normalizado por el getter (**mismo archivo:70–75**); no protege corrupción de raíz on-disk.
- `Host::set_mod_key(&mut self, mod_id: &str, name: &str, key: Option<&str>) -> bool`: **T:823 / I:174**; cuerpo **C/game/native_client_services.rs:476**. Disponible y generado; guardado/reasignación por UI todavía no probado.

Los helpers de persistencia proceden de **S/tools/native-hook-fragments/player_persistent_data.json** y player_compound de su fragment homónimo. El generador combina catálogo y fragments (**S/crates/horizon-mod-api-gen/src/lib.rs:1010–1033**); los placeholders `@API:...@` no invalidan la presencia de esas firmas en los generados exactos. No atribuirles una numeración individual inventada.

Las pruebas runtime que sustentan esta matriz son las curadas en `p0-runtime-v2/v4/v5.json` y los informes P0 asociados. Este índice solo fija los contratos del target; no ejecuta ni amplía aquellos ensayos. P2 mantiene pendientes corrupción/migración, transacciones de inventario, replay, rollback, autosave y cortes.
## Cierre documental — fixround 1/5

R1: receta completa de `native-build` recuperada en `docs/compatibility.md`, con launcher resuelto por USERPROFILE, entorno scratch completo, base preparada como prerrequisito y destino v5 nuevo; matriz pública del pin en este archivo. R2: alcance del guard de raíz corregido en compatibilidad y `p0-storage.md`; validación raw fuera de turno es gate P2. Verificaciones de esta ronda: firmas y referencias generadas cotejadas, ambos bloques PowerShell parseados sin ejecutar, `git diff --check` exit0. No se repitieron build, juego ni tests. La decisión de integración C0 permanece a cargo del principal.