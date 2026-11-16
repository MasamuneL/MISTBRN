[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9-]+\z')][string]$BuildName,
    [Parameter(Mandatory)][string]$RuntimeRoot,
    [Parameter(Mandatory)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
    [Parameter(Mandatory)][string]$WorldRoot,
    [ValidatePattern('\A[A-Za-z0-9_]{1,16}\z')][string]$PlayerName = 'AllomancyQA',
    [ValidateSet('dead-login','death-stay','death-respawn','death-logout','save-pause','dimension','io-create-new','io-deny-rename')][string]$Scenario = 'death-respawn',
    [ValidatePattern('\A[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\z')][string]$OwnerUuid,
    [string]$BaselineDirectory
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$runtime = (Resolve-Path -LiteralPath $RuntimeRoot).Path
$world = (Resolve-Path -LiteralPath $WorldRoot).Path
$saves = Split-Path -Parent $world
$gameDir = Split-Path -Parent $saves
if (-not (Test-Path -LiteralPath $world -PathType Container) -or
    (Split-Path -Leaf $saves) -ine 'saves' -or
    (Split-Path -Leaf $gameDir) -notmatch '^game-p2-.+$' -or
    -not $gameDir.StartsWith($runtime.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'WorldRoot debe ser runtime/.../game-p2-*/saves/NAME existente.'
}
$engine = Join-Path $runtime "$BuildName-engine/horizon-engine.exe"
$resources = Join-Path $runtime "$BuildName-engine/.local/horizon-client/26.3"
$buildReceipt = Join-Path $runtime "native-builds/$BuildName/receipt.json"
$marker = Join-Path $world 'allomancy/world-identity.json'
$helper = Join-Path $PSScriptRoot 'horizon-control.cjs'
foreach ($file in @($engine,(Join-Path $resources 'horizon-resources.json'),$buildReceipt,$marker,$helper)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Falta input: $file" }
}
$build = Get-Content -LiteralPath $buildReceipt -Raw | ConvertFrom-Json
if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test) {
    throw 'Se requiere build succeeded, exit 0 y native_test.'
}
$identity = Get-Content -LiteralPath $marker -Raw | ConvertFrom-Json
if (-not $identity.world_id) { throw 'Marker sin world_id.' }
$runDir = Join-Path $runtime "native-lifecycle-runs/$RunId"
if (Test-Path -LiteralPath $runDir) { throw "RunId existente: $runDir" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$stdout = Join-Path $runDir 'stdout.log'
$stderr = Join-Path $runDir 'stderr.log'
$endpoint = Join-Path $runDir 'control.json'
$resultPath = Join-Path $runDir 'result.json'
$env:HORIZON_HOME = Join-Path $runtime "homes/$BuildName"
$env:HORIZON_LAUNCHER_HOME = Join-Path $runtime 'launcher'
$env:XDG_DATA_HOME = Join-Path $runtime 'data'
$env:XDG_CONFIG_HOME = Join-Path $runtime 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $runtime 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $runtime 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $runtime 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
$env:ALLOMANCY_NATIVE_TEST_RUNTIME = $runtime
$env:ALLOMANCY_NATIVE_TEST_ROOT = $world
$env:ALLOMANCY_NATIVE_TEST_WORLD_ID = [string]$identity.world_id
$env:ALLOMANCY_NATIVE_TEST_SCENARIO = 'observe'
$options = @('--world',(Split-Path -Leaf $world),'--game-dir',$gameDir,'--resources',$resources,
    '--offscreen','--test-terrain','--creative','--render-distance','2',
    '--name',$PlayerName,'--control-socket',$endpoint)
if ($Scenario -ne 'save-pause') { $options += '--no-pause' }
$result = [ordered]@{
    status = 'starting'; pass = $false; phase = 'startup'; scenario = $Scenario
    pid = $null; exit_code = $null; termination = $null; error = $null; limitation = $null
    engine = $engine; engine_sha256 = (Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash
    build_receipt = $buildReceipt; build_receipt_sha256 = (Get-FileHash -LiteralPath $buildReceipt -Algorithm SHA256).Hash
    inputs = @($build.inputs | ForEach-Object {
        [ordered]@{ path = $_.path; recorded_sha256 = $_.sha256; current_sha256 = if (Test-Path -LiteralPath $_.path -PathType Leaf) { (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash } else { $null } }
    })
    helper_sha256 = (Get-FileHash -LiteralPath $helper -Algorithm SHA256).Hash
    harness_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
    arguments = $options; world = $world; log_source = @($stdout,$stderr)
    driver = 'observe'; frame_budget = $null; deadline_seconds = 120; phase_seconds = 30
    commands = [Collections.Generic.List[object]]::new(); event_counts = @{}; events = @()
    save_observed_before = $null; save_observed_after = $null; save_observed_delta = $null
    death_logout_before_apply = $null; event_order_scope = 'per source file; death order uses stderr only'
    offline_oracle = 'pending: root compares closed-world snapshot with credit B1000/ledger1 baseline'
}
function Save-Result {
    $tmp = "$resultPath.tmp"
    [IO.File]::WriteAllText($tmp,($result | ConvertTo-Json -Depth 18),[Text.UTF8Encoding]::new($false))
    if (Test-Path -LiteralPath $resultPath) { [IO.File]::Replace($tmp,$resultPath,"$resultPath.bak") }
    else { [IO.File]::Move($tmp,$resultPath) }
}
function Read-Shared([string]$file) {
    $text = ''; $stream = $null; $reader = $null
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { return $text }
    try {
        $stream = [IO.File]::Open($file,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
        $reader = [IO.StreamReader]::new($stream)
        $text = $reader.ReadToEnd()
    } catch [IO.IOException] {
        # Retry on the next poll; absent traces cannot satisfy the gate.
    } finally {
        if ($reader) { $reader.Dispose() } elseif ($stream) { $stream.Dispose() }
    }
    return $text
}
function Logs { return ([string](Read-Shared $stdout) + "`n" + [string](Read-Shared $stderr)) }
function Check-Budget {
    if ($timer.Elapsed.TotalSeconds -ge 120) { throw 'Deadline global de 120 segundos.' }
}
function Send-Control([string]$command,[bool]$allowError = $false) {
    Check-Budget
    $number = $result.commands.Count + 1
    $commandFile = Join-Path $runDir ("command-{0:D3}.txt" -f $number)
    $replyFile = Join-Path $runDir ("reply-{0:D3}.json" -f $number)
    [IO.File]::WriteAllText($commandFile,$command,[Text.UTF8Encoding]::new($false))
    $started = $timer.Elapsed.TotalSeconds
    $output = @(& node $helper --socket $endpoint --command-file $commandFile)
    $code = $LASTEXITCODE
    $text = [string]($output -join "`n")
    [IO.File]::WriteAllText($replyFile,$text,[Text.UTF8Encoding]::new($false))
    $result.commands.Add([ordered]@{ command = $command; command_file = $commandFile; reply_file = $replyFile; reply = $text; exit = $code; started_seconds = $started; ended_seconds = $timer.Elapsed.TotalSeconds })
    Save-Result
    Check-Budget
    if ($code -ne 0 -and -not $allowError) { throw "Control falló en command-$number; ver reply redacted." }
    $reply = $text | ConvertFrom-Json
    if ($null -eq $reply -or (-not $allowError -and ($reply.PSObject.Properties['error'] -or $reply.ok -eq $false))) {
        throw "Respuesta inválida/error en command-$number."
    }
    if (@($reply.chat) -match 'Unknown or incomplete command|Unknown command|<--\[HERE\]') {
        throw "Comando rechazado por el juego en command-$number; ver chat del reply."
    }
    return $reply
}
function Wait-Phase([string]$name,[scriptblock]$condition) {
    $result.phase = $name; Save-Result
    $phaseTimer = [Diagnostics.Stopwatch]::StartNew()
    while ($phaseTimer.Elapsed.TotalSeconds -lt 30) {
        Check-Budget
        $process.Refresh()
        if ($process.HasExited) { throw "Engine terminó antes de $name." }
        $log = [string](Logs)
        if ($log -match 'ALLOMANCY_SAVE event=fault|ALLOMANCY_NATIVE_TEST .*result=FAIL|phase=WorldOpenFailed') {
            throw 'Fault/FAIL nativo; inspeccionar logs.'
        }
        $matched = [bool](& $condition $log)
        Check-Budget
        if ($phaseTimer.Elapsed.TotalSeconds -ge 30) { break }
        if ($matched) { return }
        Start-Sleep -Milliseconds 250
    }
    throw "Timeout fase $name (30 segundos); no asumir soporte/éxito del comando."
}
function State-Is([string]$screen,[bool]$paused) {
    $state = Send-Control 'state'
    if ($state.screen -isnot [string] -or $state.paused -isnot [bool]) { throw 'Schema state incompatible: screen/paused.' }
    return ($state.screen -eq $screen -and $state.paused -eq $paused)
}
function Save-Events {
    $counts = @{}; $events = [Collections.Generic.List[object]]::new()
    foreach ($file in @($stdout,$stderr)) {
        $lineNumber = 0
        foreach ($line in ([string](Read-Shared $file) -split "`r?`n")) {
            $lineNumber++
            if ($line -match 'ALLOMANCY_(SAVE|NATIVE_TEST) event=([a-z_]+)(?: phase=([A-Za-z]+))?') {
                $event = $Matches[2]
                if ($Matches[3]) { $event += ':' + $Matches[3] }
                $counts[$event] = 1 + [int]$counts[$event]
                $events.Add([ordered]@{ source = $file; line = $lineNumber; event = $event })
            }
        }
    }
    $result.event_counts = $counts; $result.events = $events.ToArray()
}
Save-Result
$process = $null; $quitSent = $false; $ioLock = $null
$timer = [Diagnostics.Stopwatch]::StartNew()
try {
    $quoted = $options | ForEach-Object { '"' + $_ + '"' }
    $process = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $result.pid = $process.Id; $result.status = 'running'; Save-Result
    Wait-Phase 'ready' { param($log) $log -match 'ALLOMANCY_NATIVE_TEST event=enabled scenario=observe' -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint -PathType Leaf) }
    $initialState = Send-Control 'state'
    if ($initialState.screen -eq 'paused' -or $initialState.paused -eq $true) { $null = Send-Control 'resume' }
    if ($Scenario -eq 'dead-login') {
        Wait-Phase 'dead login screen' { State-Is 'dead' $false }
    } else { Wait-Phase 'playing' { State-Is 'playing' $false } }
    if ($BaselineDirectory -and $Scenario -ne 'dead-login') {
        if (-not $OwnerUuid -or (Test-Path -LiteralPath $BaselineDirectory)) { throw 'Baseline needs owner and fresh destination.' }
        $null = Send-Control 'save'
        New-Item -ItemType Directory -Path $BaselineDirectory | Out-Null
        $primary = Join-Path $world "players/data/$OwnerUuid.dat"
        foreach ($path in @($primary,"$primary`_old")) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw 'Baseline DAT/old missing.' }
            Copy-Item -LiteralPath $path -Destination $BaselineDirectory
        }
        $result['baseline_directory'] = $BaselineDirectory
        Save-Result
    }
    if ($Scenario -like 'io-*') {
        if (-not $OwnerUuid) { throw 'IO requiere OwnerUuid explícito del snapshot cerrado.' }
        $primary = Join-Path $world "players/data/$OwnerUuid.dat"
        $old = "$primary`_old"
        $new = "$primary`_new"
        foreach ($path in @($primary,$old)) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw 'IO requiere DAT y backup completos existentes.' }
        }
        $result.phase = 'baseline save'; Save-Result
        $null = Send-Control 'save'
        $before = @($primary,$old | ForEach-Object { [ordered]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_).Hash } })
        $result['io_before'] = $before
        if ($Scenario -eq 'io-create-new') {
            if (Test-Path -LiteralPath $new) { throw 'No sobrescribir _new preexistente.' }
            $ioLock = [IO.File]::Open($new,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
            $result['io_lock'] = [ordered]@{ path=$new; mode='CreateNew'; access='ReadWrite'; share='None' }
        } else {
            $ioLock = [IO.File]::Open($primary,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::ReadWrite)
            $result['io_lock'] = [ordered]@{ path=$primary; mode='Open'; access='Read'; share='ReadWrite without Delete' }
        }
        $result.phase = 'IO lock installed'; Save-Result
        $null = Send-Control 'command give @s minecraft:iron_ingot 1'
        $reply = Send-Control 'save' $true
        $lastCommand = $result.commands[$result.commands.Count - 1]
        if ($lastCommand.exit -ne 1 -or -not $reply.PSObject.Properties['error'] -or
            [string]$reply.error -notmatch 'os error (5|32|33)\)') {
            throw 'Save no reportó error Windows de acceso/sharing esperado; no acreditar N8.'
        }
        $result['io_error'] = [string]$reply.error
        # Keep the denied handle until the process is dead. No quit/save recovery.
        $process.Refresh()
        if ($process.HasExited) { throw 'Engine salió antes del kill controlado de IO.' }
        $process.Kill(); $process.WaitForExit(); $process.Refresh()
        $result.exit_code = $process.ExitCode
        $result.termination = 'killed_after_expected_io_error'
        $ioLock.Dispose(); $ioLock = $null
        $after = @($primary,$old | ForEach-Object { [ordered]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_).Hash } })
        $result['io_after'] = $after
        for ($i=0; $i -lt 2; $i++) {
            if ($before[$i].sha256 -ne $after[$i].sha256) { throw 'DAT/old cambió pese a bloqueo; investigar sin gate.' }
        }
        $result['io_new'] = if (Test-Path -LiteralPath $new -PathType Leaf) {
            [ordered]@{ path=$new; bytes=(Get-Item -LiteralPath $new).Length; sha256=(Get-FileHash -LiteralPath $new).Hash }
        } else { $null }
        Save-Events
        $result.pass = $true; $result.status = 'passed'; $result.phase = 'complete'
        $result.offline_oracle = 'pending: inspect primary/old and optional _new; reopen a separate closed-world copy'
        Save-Result
        Write-Output "result=$resultPath status=$($result.status) pass=$($result.pass) expected_real_io_error=true"
        return
    }
    switch ($Scenario) {
        'death-stay' {
            $null = Send-Control 'command kill @s'
            Wait-Phase 'death applied' { param($log) $log -match 'event=death_queued' -and $log -match 'event=death_applied' }
            Wait-Phase 'saved death screen' { State-Is 'dead' $false }
        }
        'death-respawn' {
            $null = Send-Control 'command kill @s'
            Wait-Phase 'death applied' { param($log) $log -match 'event=death_queued' -and $log -match 'event=death_applied' }
            Wait-Phase 'death screen' { State-Is 'dead' $false }
            $null = Send-Control 'respawn'
            Wait-Phase 'respawn' { param($log) $log -match 'ALLOMANCY_SAVE event=respawn' }
            Wait-Phase 'playing after respawn' { State-Is 'playing' $false }
        }
        'death-logout' {
            $null = Send-Control 'command kill @s'
            # No polling or death_applied wait between kill reply and quit.
            $null = Send-Control 'quit'; $quitSent = $true
        }
        'save-pause' {
            $result.save_observed_before = [regex]::Matches([string](Logs),'ALLOMANCY_SAVE event=save_observed').Count
            $null = Send-Control 'save-all'
            Wait-Phase 'save observed delta' { param($log) [regex]::Matches($log,'ALLOMANCY_SAVE event=save_observed').Count -gt $result.save_observed_before }
            $result.save_observed_after = [regex]::Matches([string](Logs),'ALLOMANCY_SAVE event=save_observed').Count
            $result.save_observed_delta = $result.save_observed_after - $result.save_observed_before
            $null = Send-Control 'pause'
            Wait-Phase 'paused' { State-Is 'paused' $true }
            $null = Send-Control 'resume'
            Wait-Phase 'resumed' { State-Is 'playing' $false }
        }
        'dimension' {
            $null = Send-Control 'command execute in minecraft:the_nether run tp @s 0 80 0'
            Wait-Phase 'dimension changed' { param($log)
                if ($log -notmatch 'ALLOMANCY_SAVE event=dimension_changed') { return $false }
                $state = Send-Control 'state'
                if ($state.dimension -isnot [string]) { throw 'Schema state incompatible: dimension.' }
                return $state.dimension -eq 'minecraft:the_nether'
            }
        }
    }
    if (-not $quitSent) { $null = Send-Control 'quit'; $quitSent = $true }
    $result.phase = 'normal close'; Save-Result
    $closeTimer = [Diagnostics.Stopwatch]::StartNew()
    do {
        Check-Budget; $process.Refresh()
        if ($process.HasExited) { break }
        if ($closeTimer.Elapsed.TotalSeconds -ge 30) { throw 'Timeout cierre normal (30 segundos).' }
        Start-Sleep -Milliseconds 250
    } while ($true)
    $process.WaitForExit(); $process.Refresh()
    $result.exit_code = $process.ExitCode; $result.termination = 'spontaneous_after_quit'
    $log = [string](Logs); Save-Events
    if (-not $quitSent -or $result.exit_code -ne 0 -or
        $log -notmatch 'ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldCloseFinal' -or
        $log -match 'ALLOMANCY_SAVE event=fault|ALLOMANCY_NATIVE_TEST .*result=FAIL|phase=WorldOpenFailed') {
        throw 'Faltan criterios de cierre normal/exit0 o hubo Fault/FAIL.'
    }
    $result.status = 'passed'; $result.pass = $true
    if ($Scenario -eq 'death-logout') {
        if (-not $result.event_counts['death_queued'] -or -not $result.event_counts['death_applied']) { throw 'Faltan trazas death_queued/death_applied.' }
        $loggingOut = @($result.events | Where-Object { $_.source -eq $stderr -and $_.event -eq 'logging_out' } | Select-Object -First 1)
        $applied = @($result.events | Where-Object { $_.source -eq $stderr -and $_.event -eq 'death_applied' } | Select-Object -First 1)
        $result.death_logout_before_apply = $loggingOut.Count -eq 1 -and $applied.Count -eq 1 -and $loggingOut[0].line -lt $applied[0].line
        if (-not $result.death_logout_before_apply) {
            $result.pass = $false; $result.status = 'limited'
            $result.limitation = 'Cierre normal observado; logging_out antes de death_applied no demostrado (timing o callback ausente). No acredita muerte→logout antes del siguiente tick.'
        }
    }
    $result.phase = 'complete'; Save-Result
} catch {
    $result.error = $_.Exception.Message
    if ($process) {
        $process.Refresh()
        if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit(); $result.termination = 'killed_after_failure' }
        elseif (-not $result.termination) { $result.termination = 'spontaneous_before_completion' }
        $process.Refresh(); $result.exit_code = $process.ExitCode
    }
    Save-Events
    $result.status = 'failed'; $result.pass = $false; Save-Result
    throw
}
finally { if ($ioLock) { $ioLock.Dispose() } }
Write-Output "result=$resultPath status=$($result.status) pass=$($result.pass)"
