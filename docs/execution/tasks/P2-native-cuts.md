# P2 N6 — pausas nativas del adaptador

Estado: intención registrada antes de editar Rust; implementación acotada en revisión. Subagente cuts_native; integración/build/harness/checkpoint pertenecen a root. Escrituras exclusivas: save.rs, native_test.rs y esta ficha. No Cargo/build/juego/commit; no SDK/owner ni los tres diffs preexistentes.

## Contrato del controlador

Se reutilizan native-test y sus gates runtime/root/marker. Escenarios: cut-before-inventory, cut-after-inventory, cut-after-inline, cut-after-core-commit. Los cuatro requieren ALLOMANCY_NATIVE_TEST_RUN_TOKEN: 1–64 bytes ASCII alfanuméricos, guion o underscore. TransactionId fijo native-cut-<token>. Cada operación elige sólo minecraft:iron_ingot, count>=1, entre slots 0..35; conserva el stack exacto completo y consume una unidad para acreditar Iron1000.

Trace exacta: ALLOMANCY_NATIVE_CUT event=paused pid=<PID> frontier=<before-inventory|after-inventory|after-inline|after-core-commit> transaction=native-cut-<token> run_token=<token> durable=false. Comparar PID, frontier, transaction y token con el lanzamiento propio antes de terminar únicamente ese PID; no quit/save. Trace vaciada antes de bloquear; si falla stderr, controlador debe declarar timeout/intento no acreditado. Nunca se emite PASS para una pausa: si crédito retorna, el driver informa FAIL cut_not_reached.

Arming: <world_root>/allomancy/native-cut-arm-<token> debe estar ausente al configurar (incluidos directorio o symlink roto; errores IO rechazan). Después de Ready, controlador ejecuta save real, copia A y sus hashes, crea ese archivo regular y ya no emite comandos Host/control mientras espera trace/corte. Antes de is_file=true, run_native_test_driver no observa Host ni intenta crédito; los ticks normales previos siguen necesarios para login/Ready. El archivo es control de ensayo, sin economía, nunca almacén de persistencia. Usar token único por intento, retirar el arm sólo de la copia cerrada antes de reabrir para otro escenario.

Fronteras síncronas fuera de todo guard session y stderr: antes del CAS inventario, tras retorno Ok del CAS, tras true del setter inline, tras commit núcleo y salida del bloque lock. Pausa indefinida con thread::park en loop: no Host/save, ningún return a tick/driver ni reanudación por unpark espurio. Un writer ya activo puede terminar A; no se promete congelar todos los threads. N6 requiere proceso real, copias/hash y oráculos A/B de root.

## Comprobación estática focalizada (sin Cargo)

Ejecutar el bloque con PowerShell desde la raíz del worktree. Detecta posición de frontera y locks vivos; no es ejecución Rust ni prueba N6. Python era alias inaccesible: no se contó ese error como RED. Antes de implementación, el guard PowerShell de fronteras falló exit1 con RED: missing native cut frontier BeforeInventory.

```powershell
$ErrorActionPreference='Stop'
$s=Get-Content -LiteralPath crates/allomancy-horizon/src/save.rs -Raw
function Assert-Check($ok,$message) { if(-not $ok) { throw $message } }
function Assert-No-Live-Lock([string]$prefix,[string]$name) {
    $scopes=[Collections.Generic.List[bool]]::new()
    foreach($line in ($prefix -split "`n")) {
        foreach($c in $line.ToCharArray()) {
            if($c -eq '{') { $scopes.Add($false) }
            elseif($c -eq '}') { $scopes.RemoveAt($scopes.Count-1) }
        }
        if($line.Contains('.lock()')) { $scopes[$scopes.Count-1]=$true }
    }
    Assert-Check (-not $scopes.Contains($true)) "Live lock at $name"
}
$start=$s.IndexOf('    fn apply_credit(')
$end=$s.IndexOf('    // Only the current valid raw Compound',$start)
$a=$s.Substring($start,$end-$start)
foreach($f in @('BeforeInventory','AfterInventory','AfterInline','AfterCoreCommit')) {
    $call="self.pause_for_native_cut(failpoint, CreditFailpoint::Pause$f, intent);"
    Assert-Check (($a.Split($call)).Count -eq 2) "Missing/duplicate frontier $f"
    Assert-No-Live-Lock $a.Substring(0,$a.IndexOf($call)) $f
}
$needles=@('CreditFailpoint::PauseBeforeInventory','let old = match host.set_inventory_slot(','CreditFailpoint::PauseAfterInventory','host.set_player_persistent_data(uuid, merged.clone())','CreditFailpoint::PauseAfterInline','commit_credit(&mut entry.saved, prepared)','CreditFailpoint::PauseAfterCoreCommit','ALLOMANCY_SAVE event=credit_accepted')
$previous=-1
foreach($needle in $needles) {
    $position=$a.IndexOf($needle)
    Assert-Check ($position -gt $previous) "Missing/out-of-order frontier $needle"
    $previous=$position
}
$helperStart=$s.IndexOf('    fn pause_for_native_cut(')
$helper=$s.Substring($helperStart,$start-$helperStart)
Assert-Check ($helper.Contains('std::thread::park()') -and $helper.Contains('loop {')) 'Pause must block indefinitely'
Assert-Check ($helper.Contains('.flush()') -and $helper.Contains('std::process::id()')) 'Trace must flush with actual PID'
Assert-Check (-not ($helper.Contains('host.') -or $helper.Contains('self.session') -or $helper.Contains('save('))) 'Pause must not call Host/session/save'
Assert-No-Live-Lock $helper.Substring(0,$helper.IndexOf('loop {')) 'park loop'
$driver=$s.Substring($s.IndexOf('    pub(crate) fn run_native_test_driver('),$s.IndexOf('    pub fn credit_for_native_test(')-$s.IndexOf('    pub(crate) fn run_native_test_driver('))
Assert-Check ($driver.IndexOf('.is_file()') -lt $driver.IndexOf('self.test_world(host)')) 'Arm must precede Host gate'
Write-Output 'PASS: N6 source frontier/lock/no-Host/arm contract (static only)'
```

## Límites

No suite core/native anterior repetida. La comprobación anterior no compila ni ejecuta Rust, no confirma plan/build ni sincronización de procesos. Las cuatro pausas sólo existen con feature native-test; sin esa feature no hay variantes de corte, token, helper o llamadas de pausa. El cambio común al modo normal sólo acorta el scope del Mutex del commit antes de trace/return. No nuevas dependencias, APIs Host, mutaciones SDK o writers.

## Resultado estático y entrega para revisión

Guard focalizado ejecutado después de edición: exit0, `PASS: N6 source frontier/lock/no-Host/arm contract (static only)`. El mismo bloque queda arriba para repetición cuando cambien estas fronteras. No se repitieron tests viejos ni se inició Cargo/build/juego. Root solicitó conservar el alcance en cuatro fronteras más reapertura; no se añadió cut-retry.

Rust congelado para revisión con SHA256 save.rs `48F574DCB2075B32B471B4672CA0B51F030172BB03A1FC1E3E314DB30BA7FCF2`, native_test.rs `692E0EC56337BC79B62096F19EE36378F02CF6F9A6527C5C6CADE2F5897BDC93`. Pendientes de root: revisión, formato/compilación si procede, build y ensayos reales seriales. N6 no acreditado por esta entrega.

## Root: resultado posterior a entrega
Rustfmt activo launcher exit0 (sólo formato), buildp2-cuts1 exit0; cuatrocortes+cuatroreopen seriales PASS, suite0/consolidador0, evidence/p2-native-cuts.md y p2-cuts-checks.json. N6perfilownerintegrado no-save acreditado; noN7/fsync/P2global. Sin39repetidos ni coretests. Revisiónfinalindependiente en curso, commitroot posterior.
