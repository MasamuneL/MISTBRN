# P2 harness de crédito nativo

tools/test-native-credit.ps1 preparado; AST PowerShell correcto. Sin juego ni Cargo ejecutados durante su implementación. El intento inline roundtrip1 previo falló al leer un log inicialmente null; PID 12176 terminó y su receipt running quedó obsoleto. Ese intento no ejecutó crédito.

Requiere build native-test succeeded, mundo dedicado ya clonado/preparado, identidad primaria legible, control habilitado y Node disponible. Usar inventario de prueba sin stacks previos: el driver toma el primer slot ocupado y un PASS antes del give falla la corrida. No copia el mundo ni edita DAT. Ejemplo:

```powershell
pwsh -NoProfile -File tools/test-native-credit.ps1 -BuildName p2-native2 -RuntimeRoot '<runtime>' -RunId n1-credit2 -WorldRoot '<runtime>/game-p2-n1-credit2/saves/N0' -PlayerName Player
```

RunId nuevo crea runtime/native-credit-runs/<RunId>. Receipt atómico registra argumentos, PID, hash engine, receipt build, hashes declarados/actuales de inputs, hash del cliente de control, rutas de logs y comandos/respuestas. Engine se lanza directamente con roundtrip y --control-socket, sin límites de frames; deadline de espera 120 s. Al observar ready y endpoint invoca mediante tools/horizon-control.cjs el archivo con línea exacta command give @s minecraft:iron_ingot 3; exige exit 0. Luego espera PASS/FAIL del driver y en PASS solicita quit, exige exit 0 del control, salida 0 del engine y WorldCloseFinal. Lecturas compartidas de logs toleran IOException transitoria; ausencia de trazas no puede dar PASS. El helper Node no ha sido conectado realmente en esta revisión; su fallo es fallo explícito de control. Cada invocación Node tiene su propio timeout de diez segundos, por lo que la duración total puede rebasar ligeramente el deadline de polling.

Ante FAIL, timeout o error de control detiene solamente el proceso iniciado y registra fallo; matar nunca es PASS. El comando give es la única concesión de fixture, y el driver debe acreditar con su propia traza crédito, ledger, stack y replay en memoria. Este ensayo es N1 parcial: no verifica replay después de reinicio, escritura durable ni pérdida de energía. Conservar logs, receipt y mundo tras el ensayo para análisis.
