# P2 native control — helper QA local

estado: in_progress; intención registrada antes de implementación.

Propiedad exclusiva: `tools/horizon-control.cjs` y esta ficha. Sin juego/Cargo/SDK/otros fuentes/commits/subagentes. Lecturas locales del pin original `fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca` en `<ORIGINAL>/experiments/horizon-probe/runtime/launcher/native/source/`.

Protocolo comprobado: `crates/horizon-client/src/control.rs:83–115` lee port/token del endpoint JSON, conecta TCP IPv4 localhost y manda `auth TOKEN\n`; `:305–328` responde JSON `{ok:true}` al autenticar. `src/bin/horizon-ctl/main.rs:83–105` envía una línea de texto y lee una línea JSON; `:171–185` trata campo error como fallo. No se envía JSON como request.

Plan autorizado mínimo: Node estándar, menos de 100 líneas; argumentos `--socket PATH --command-file PATH`; endpoint TCP local canónico; archivo de comando UTF-8 de una sola línea, preservando espacios y quitando únicamente un salto final LF/CRLF si existe. Admitirá comandos de texto como hello, state, command give @s stone 3 y quit sin transformarlos. Timeout total 10s, una respuesta JSON en stdout, exit no cero en error, sin imprimir token ni endpoint. No hace retries de comandos mutadores. Solo `node --check` en este relevo; prueba real corresponde al principal con juego preparado.

## Resultado y congelación

estado: handed_off; helper entregado, prueba de control real pendiente del principal.

Creado `tools/horizon-control.cjs`, 82 líneas, sin dependencias externas. Sintaxis: `node --check tools/horizon-control.cjs`, exit0, sin salida. SHA256 `CC83E5A53E946DB8C3DEBA60156538EC7A47A316D08447FC7AE397ABE110AE4C`.

Uso desde el worktree: `node tools/horizon-control.cjs --socket <endpoint.json> --command-file <command.txt>`. Command file UTF-8 sin BOM, un único comando; admite LF/CRLF final opcional, conserva los demás espacios y rechaza vacío/multilínea/NUL. No traduce hello/state/command give @s stone 3/quit ni prepara mundos. Endpoint canónico exige transport tcp, puerto válido y token de 32 hex minúsculos como el writer del pin. Conecta exclusivamente 127.0.0.1 y espera auth ok antes del texto del comando.

La salida es una sola línea JSON; respuesta de juego con campo error u ok:false devuelve exit1. Auth rechazada, IO, conexión, EOF prematuro, JSON inválido y timeout devuelven errores constantes y exit1, sin errores nativos que incluyan rutas/token. Timer 10000ms para la operación asíncrona; destruye socket al terminar y no reintenta. Las respuestas se serializan a una línea y ocultan token y ruta de endpoint si aparecen en un string de respuesta. Límite conservador de buffer de respuesta 16Mi caracteres. Nunca imprime el objeto endpoint.

Fuentes de protocolo leídas, SHA256 sin modificación:

- `crates/horizon-client/src/control.rs`: `A616CBB2EF7AA73FCDA63269AB3071CD94A477B493C187707B79913BA16D5955`.
- `crates/horizon-client/src/bin/horizon-ctl/main.rs`: `0CA9561C401F33A325B040D9418DBB19B3F52D439F56935955FB885E70838027`.

No se conectó al endpoint ni se lanzó juego; no Cargo/build, mock server, prueba de texto, cambios SDK, commits o subagentes. Sintaxis aprobada no demuestra ejecución de auth/comando. Principal debe hacer hello/state/command/quit reales en la instancia de ensayo preparada y conservar JSON/exit status. Ningún gate nativo queda acreditado por este helper. Fuente congelada; propiedad de implementación cerrada.
