# P2 QA nativo: preparación estática

No se ejecutaron juego, Cargo ni fixtures. Guard y bridge no están compilados; driver y feature no existen. Referencias: docs/execution/HANDOFF.md, docs/execution/evidence/p2-storage-architecture.md, tools/smoke-native.ps1, docs/world-preparation.md y SDK pin fdc87aecdbd116bf1e08adcd062fcbaea8c2e2ca.

## Apertura integrada

El engine acepta --world NAME --game-dir DIR --resources DIR --offscreen --no-pause --control-socket PATH. NAME es carpeta bajo DIR/saves. Preparar copia dedicada offline y lanzarla directamente sin wrapper preflight. --quickPlaySingleplayer NAME exige existente; --world también crea si falta. Comprobar existencia antes de iniciar. Reproducir variables de entorno aisladas de tools/smoke-native.ps1.

```powershell
$engine = Join-Path $runtime "$BuildName-engine/horizon-engine.exe"
$resources = Join-Path $runtime "$BuildName-engine/.local/horizon-client/26.3"
$socket = Join-Path $gameDir 'control.sock'
$argsList = @('--world','N0','--game-dir',$gameDir,'--resources',$resources,'--offscreen','--no-pause','--control-socket',$socket)
$quoted = $argsList | ForEach-Object { '"' + $_ + '"' }
$p = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $gameDir 'stdout.log') -RedirectStandardError (Join-Path $gameDir 'stderr.log')
# Tras observar callback y control disponible:
& $ctl --socket $socket state
& $ctl --socket $socket quit
$p.WaitForExit(180000)
$p.Refresh()
$p.ExitCode
```

Localizar $ctl de la misma versión; no se verificó que el export lo incluya. --control-socket activa control; noAi debe estar desactivado. En Windows el endpoint utiliza TCP local/token. quit solicita guardar y salir: verificar que tras rechazo no hay mundo abierto y no usarlo en rollback sin confirmar. Si control no responde, registrar timeout y conservar proceso y evidencia; no afirmar cierre limpio. --world-frames solo cuenta frames dentro de mundo y puede esperar indefinidamente tras rechazo; --frames cuenta frames generales pero puede cerrar antes de apertura asíncrona.

integrated.rs:683-689 llama native_lifecycle_world_failed(error) si integrated_open falla; native_lifecycle.rs:203-212 despacha WorldOpenFailed solo en estado Opening. El comando events no promete exponer callback nativo. El driver debe implementar/delegar Hooks::native_world_lifecycle y registrar fase, error y epoch; verificar firma en API generada. Registrar también traza del guard, ausencia de Ready/login, hash antes/después de level y playerdata, plan/receipt/hash build, logs y salida del proceso. Mensaje UI o exit code no demuestran callback. N0 requiere integrado y guard incorporado, no wrapper ni solo dedicado.

## Fixtures offline

Con Horizon cerrado, copiar mundo dedicado válido completo a gameDir/saves/N0 y conservar copia inmutable. Provisionar identidad únicamente en directorio totalmente vacío: pwsh -NoProfile -File tools/prepare-world.ps1 -WorldRoot <mundo> -AllowedRoot <raíz-existente> -ProvisionNew. Para mundo existente conservar marcador válido original.

Este ejemplo usa Python estándar para reemplazar exclusivamente una copia player dedicada por NBT gzip con clave duplicada; conservar hash y backup antes. No representa un jugador válido; solo demuestra detección si traza identifica rechazo del guard antes del decoder.

```powershell
$env:N0_PLAYER = 'C:\ruta\dedicada\saves\N0\players\data\00000000-0000-0000-0000-000000000001.dat'
@"
import gzip, os, struct
from pathlib import Path
p = Path(os.environ["N0_PLAYER"])
assert p.is_file() and p.name.endswith(".dat")
def named_int(name, value):
    b = name.encode("ascii")
    return bytes([3]) + struct.pack(">H", len(b)) + b + struct.pack(">i", value)
raw = bytes([10, 0, 0]) + named_int("duplicate", 1) + named_int("duplicate", 2) + bytes([0])
p.write_bytes(gzip.compress(raw, mtime=0))
"@ | python -
Get-FileHash -Algorithm SHA256 -LiteralPath $env:N0_PLAYER
```

En copias independientes crear gzip truncado (conservar cabecera 1f8b), backup inválido, NeoForgeData de tipo distinto de compound, transporte allomancy:state con format_version Int mayor a 1, JSON futuro dentro de ByteArray de transporte versión 1, e IO realmente denegado. El JSON futuro debe partir de payload válido del codec, cambiando solo versión y conservando UUID/identidad. Construir otros casos mediante struct/gzip de Python o mutar copia válidamente decodificada, documentando bytes y hashes. No afirmar fallo IO sin verificarlo. Guard recorre players/data/*.dat y .dat_old, con extensiones insensibles a mayúsculas; audita level y backups, y rechaza backup sin primary. No reparar ni mezclar inventario y economía entre snapshots.

## Feature y driver pendientes

Cargo.toml del bridge carece de [features] y src/native_test.rs no existe; lib.rs ya rutea hooks() a native_test::make_hooks() bajo cfg. Implementar módulo y declarar [features] native-test = [] en fuente, producción default OFF. Solo en bundle de prueba transformar manifiesto staged agregando default = ["native-test"] al mismo bloque. Cargo activa defaults de dependencias salvo que consumidor los desactive: verificar manifiesto del consumidor generado y resolución efectiva en plan/build; no se confirmó aquí que SDK mantenga defaults. Registrar transformación y hash en receipt, no editar SDK. Build actual anida core; integración pendiente requiere core único staged ../../native-crates/allomancy/core y descriptor/módulo/HEAD del handoff.

make_hooks() llama hooks_with_native_test(NativeTestConfig { world_root, world_id }, driver) con root e ID exactos del mundo dedicado. Driver firma fn(&SaveBackend, &mut dyn Host), llamado al final de tick del servidor integrado si test_world coincide. native_test_state(uuid) devuelve SavedPlayer y generation solo Ready; seed_for_native_test no concede inventario. credit_for_native_test exige CreditIntent con UUID, generation vigente, transaction, metal, cantidad, slot < 36 y expected/replacement exactos, además de failpoint. Preparar slot y comprobar estado real con Host; ejecutar pasos una vez, no repetir por tick. Registrar resultados sin alegar durabilidad. RollbackInventory devuelve error artificial: no acredita IO real. Componer/delegar observador lifecycle conservando hooks existentes. Host API411 expone player_stacks, server_players y player_compound/recheck, sin garantía contra mod ajeno reentrante.
