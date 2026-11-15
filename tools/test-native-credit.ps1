[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9-]+\z')][string]$BuildName,
    [Parameter(Mandatory)][string]$RuntimeRoot,
    [Parameter(Mandatory)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
    [Parameter(Mandatory)][string]$WorldRoot,
    [ValidatePattern('\A[A-Za-z0-9_]{1,16}\z')][string]$PlayerName = 'Player',
    [ValidateSet('roundtrip','replay','rollback-inventory','rollback-inline','quarantine')][string]$Scenario = 'roundtrip'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
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
foreach ($file in @($engine, (Join-Path $resources 'horizon-resources.json'), $buildReceipt, $marker, $helper)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Falta input: $file" }
}
$build = Get-Content -LiteralPath $buildReceipt -Raw | ConvertFrom-Json
if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test) {
    throw 'Se requiere build succeeded, exit 0 y native_test.'
}
$identity = Get-Content -LiteralPath $marker -Raw | ConvertFrom-Json
if (-not $identity.world_id) { throw 'Marker sin world_id.' }
$runDir = Join-Path $runtime "native-credit-runs/$RunId"
if (Test-Path -LiteralPath $runDir) { throw "RunId existente: $runDir" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$stdout = Join-Path $runDir 'stdout.log'
$stderr = Join-Path $runDir 'stderr.log'
$endpoint = Join-Path $runDir 'control.json'
$giveFile = Join-Path $runDir 'give.txt'
$quitFile = Join-Path $runDir 'quit.txt'
$resultPath = Join-Path $runDir 'result.json'
[IO.File]::WriteAllText($giveFile, 'command give @s minecraft:iron_ingot 3', [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText($quitFile, 'quit', [Text.UTF8Encoding]::new($false))
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
$env:ALLOMANCY_NATIVE_TEST_SCENARIO = $Scenario
$options = @('--world',(Split-Path -Leaf $world),'--game-dir',$gameDir,'--resources',$resources,
    '--offscreen','--no-pause','--test-terrain','--creative','--render-distance','2',
    '--name',$PlayerName,'--control-socket',$endpoint)
$inputs = @($build.inputs | ForEach-Object {
    [ordered]@{ path = $_.path; recorded_sha256 = $_.sha256; current_sha256 = if (Test-Path -LiteralPath $_.path -PathType Leaf) { (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash } else { $null } }
})
$result = [ordered]@{
    status = 'starting'; pass = $false; pid = $null; exit_code = $null; termination = $null; error = $null; scenario = $Scenario
    engine = $engine; engine_sha256 = (Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash
    build_receipt = $buildReceipt; build_receipt_sha256 = (Get-FileHash -LiteralPath $buildReceipt -Algorithm SHA256).Hash
    inputs = $inputs; helper = $helper; helper_sha256 = (Get-FileHash -LiteralPath $helper -Algorithm SHA256).Hash
    arguments = $options; world = $world; stdout = $stdout; stderr = $stderr
    give_command = 'command give @s minecraft:iron_ingot 3'; give_reply = $null; give_exit = $null
    quit_command = 'quit'; quit_reply = $null; quit_exit = $null; log_source = @($stdout,$stderr)
}
function Save-Result {
    $tmp = "$resultPath.tmp"
    $result | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $tmp -Encoding utf8
    if (Test-Path -LiteralPath $resultPath) { [IO.File]::Replace($tmp,$resultPath,"$resultPath.bak") }
    else { [IO.File]::Move($tmp,$resultPath) }
}
function Logs {
    $text = ''
    foreach ($file in @($stdout,$stderr)) {
        if (-not (Test-Path -LiteralPath $file)) { continue }
        $stream = $null
        $reader = $null
        try {
            $stream = [IO.File]::Open($file,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
            $reader = [IO.StreamReader]::new($stream)
            $text += $reader.ReadToEnd() + "`n"
        } catch [IO.IOException] {
            # The next poll retries; timeout or missing required trace cannot pass.
        } finally {
            if ($reader) { $reader.Dispose() } elseif ($stream) { $stream.Dispose() }
        }
    }
    return $text
}
function Control([string]$commandFile) {
    $reply = & node $helper --socket $endpoint --command-file $commandFile
    $code = $LASTEXITCODE
    return @{ reply = ($reply -join "`n"); exit = $code }
}
Save-Result
$process = $null
try {
    $quoted = $options | ForEach-Object { '"' + $_ + '"' }
    $process = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $result.pid = $process.Id
    $result.status = 'running'
    Save-Result
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $gave = $Scenario -eq 'replay'
    $quit = $false
    while ($timer.Elapsed.TotalSeconds -lt 120) {
        $process.Refresh()
        $log = Logs
        if ($log -match "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=FAIL") { throw 'Driver reportó FAIL; ver logs.' }
        if (-not $gave -and $log -match "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=PASS") { throw 'Driver actuó antes del comando fixture.' }
        if (-not $gave -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint -PathType Leaf)) {
            $response = Control $giveFile
            $result.give_reply = $response.reply
            $result.give_exit = $response.exit
            Save-Result
            if ($response.exit -ne 0) { throw 'Control give falló; ver receipt.' }
            $gave = $true
        }
        if ($gave -and -not $quit -and $log -match "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=PASS" -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint -PathType Leaf)) {
            $response = Control $quitFile
            $result.quit_reply = $response.reply
            $result.quit_exit = $response.exit
            Save-Result
            if ($response.exit -ne 0) { throw 'Control quit falló; ver receipt.' }
            $quit = $true
        }
        if ($process.HasExited) { break }
        Start-Sleep -Milliseconds 250
    }
    $process.Refresh()
    if (-not $process.HasExited) { throw 'Timeout esperando driver o salida.' }
    $process.WaitForExit()
    $process.Refresh()
    $result.exit_code = $process.ExitCode
    $result.termination = 'spontaneous_after_quit'
    $log = Logs
    if (-not $gave -or -not $quit -or $result.exit_code -ne 0 -or
        $log -notmatch "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=PASS" -or
        $log -notmatch 'ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldCloseFinal' -or
        $log -match "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=FAIL") {
        throw 'Faltan criterios de PASS; ver logs.'
    }
    if ($Scenario -in @('rollback-inventory','rollback-inline') -and
        ($log -notmatch 'ALLOMANCY_SAVE event=rollback_verified' -or
         $log -notmatch 'ALLOMANCY_SAVE event=credit_accepted')) {
        throw 'Falta evidencia de rollback inducido y crédito posterior.'
    }
    $result.pass = $true
    $result.status = 'passed'
    Save-Result
} catch {
    $result.error = $_.Exception.Message
    if ($process) {
        $process.Refresh()
        if (-not $process.HasExited) {
            $process.Kill()
            $process.WaitForExit()
            $result.termination = 'killed_after_failure'
        } elseif (-not $result.termination) { $result.termination = 'spontaneous_before_completion' }
        $process.Refresh()
        $result.exit_code = $process.ExitCode
    }
    $result.status = 'failed'
    $result.pass = $false
    Save-Result
    throw
}
Write-Output "result=$resultPath"
