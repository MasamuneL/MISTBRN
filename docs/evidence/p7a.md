# P7a — Evidencia de teclas y HUD (macOS arm64)

Fecha: 2026-11-21. Pin de Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`. Build `p7a-5` → `native-0a875da-4870250b4b03`, compilado desde `63cf094` con el árbol limpio (`git_dirty=no`); `libmistbrn.dylib` SHA-256 `f3b55066dad8a31ade1bc1e771ff1afd16994723af59afe3cdc7ba7dea1deef0`. Recibo en `staged/p7a-5/receipt.txt`. Corridas `p7a-5-*` en `~/Library/Application Support/MISTBRN-dev/runs/`.

## Superficie y diseño

Sin fork (detalle en `docs/plan.md`, P7a): `register_key_mapping`, `consume_click`/`is_down`/`playing`/`window_focused` en `client_tick_post`, canales `play_to_server`/`play_to_client` con el jugador de la conexión como emisor, `PlayerRef::send_payload`, `register_gui_layers`/`gui_layer_draw` y `player_container_open/close`.

Hallazgos del pin:
- `horizon-ctl screen` y `screenshot` esperan un fotograma: con la ventana tapada el juego dibuja 0 fps y no responden. `ctl state` informa de la pantalla en el tick.
- El transporte entrega los paquetes de un cliente a su ritmo (26 peticiones tardaron más de 40 ticks), así que el presupuesto por jugador se prueba inyectando en la bandeja (`/mistbrn-test flood`).
- En creativo, el inventario es la pantalla `creative`.

## Resultados

| Escenario | Observado | Resultado |
|---|---|---|
| K1 tirón por tecla | Z → `activate:iron`, tirón 512,5 → 523,7 en 59 ticks; G → `off:all`, ningún `p4` después | PASS |
| K2 alternar y avivar | Z, R, R, X, Z, Z → `activate:iron, flare:iron, activate:iron, activate:steel, activate:iron, off:iron`; gasto por envío del HUD 50 ardiendo y 150 avivado (×3,00) | PASS |
| K3 parejas | V recorre estaño/peltre, zinc/latón, cobre/bronce y vuelve; Z en estaño → `MetalLocked(Tin)`; el HUD mostró la pareja | PASS |
| K4 chat | Pantalla `chat`; Z, X, R y V no envían nada; al cerrar, Z enciende | PASS |
| K5 inventario | Pantalla `creative`; ídem | PASS |
| K6 HUD | Reserva del HUD 49 580 = servidor 49 580 tras guardar; captura local revisada (abajo) | PASS |
| K7 inválidos | 3 `length`, 1 `version`, 1 `op`, 1 `arg` rechazados sin desconectar; `flood 20`: 4 fuera por bandeja llena (una línea), 8 dentro del presupuesto y una línea `rate`; después Z enciende | PASS |
| K8 control del servidor | Cofre abierto (`container`): `activate:steel` → `InputCaptured`, `off:all` apaga el hierro (`changed=1`); cerrado, Z enciende | PASS |
| N3, N7, S1 | Regresión de flotación, agotamiento y guardado | PASS |
| N1 | 1.ª corrida: paso medio en 4 ticks 1,051 frente al tope de 1,05 (un tick con dos movimientos del cliente, paso 1,809); dos repeticiones con el mismo build: 0,985, PASS | ver límites |

11/12 a la primera y N1 en 2 de 3 corridas. P7a no toca la física de N1. El build `p7a-4` pasó 12/12 (N1 0,985), pero su captura mostró el panel bajo el chat y contra el hotbar, así que se movió arriba a la izquierda (`63cf094`). Los builds `p7a-1` … `p7a-3` encontraron los patrones de log, la pantalla `creative`, el ritmo del transporte y los hallazgos de revisión; sus corridas se descartaron.

Captura `runs/p7a-5-k6/k6-hud.png` (no versionada): panel arriba a la izquierda con «Iron / Steel», la barra de hierro con «burning» en dorado y «Steel locked»; sin solaparse con el chat ni el hotbar.

## Revisión

Codex en solo lectura, 2 rondas. La 1.ª dio NO APROBADO por un hallazgo alto (el HUD no se reintentaba tras un envío rechazado), corregido. La 2.ª dio APROBADO; sus bajos se atendieron (ancho del título, K8 con apagado efectivo) y un medio quedó documentado como límite. Gemini (agy, modo plan), 1 ronda: APROBADO con el mismo medio del HUD. Informes en la PR.

## Límites

- Solo macOS arm64; Windows x64 y Linux x64 en P10.
- El servidor no ve el chat ni el inventario propio: ese bloqueo depende del cliente (K4/K5); los menús de contenedor sí se comprueban en el servidor (K8).
- Las teclas se leen una vez por tick: dentro de 50 ms pueden contarse como una o reordenarse (V se lee la última). Una petición en vuelo al cruzar un portal actúa en el destino.
- Sin prueba en juego del reintento del HUD (cubierto por la revisión del código) ni con dos clientes (P9).
- N1 conserva el margen estrecho de P4 frente a dos movimientos del cliente en un tick.
- `/mistbrn-test flood` e `inject` existen solo con `MISTBRN_TEST_SCENARIO=p4`.
