[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9-]+\z')][string]$BuildName,
    [Parameter(Mandatory)][string]$RuntimeRoot,
    [Parameter(Mandatory)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
    [Parameter(Mandatory)][string]$WorldRoot,
    [ValidatePattern('\A[A-Za-z0-9_]{1,16}\z')][string]$PlayerName = 'AllomancyQA',
    [Parameter(Mandatory)][ValidateSet('before-inventory','after-inventory','after-inline','after-core-commit')][string]$Frontier,
    [Parameter(Mandatory)][ValidatePattern('\A[0-9a-f-]{36}\z')][string]$OwnerUuid,
    [ValidatePattern('\A([0-9A-F]{64})?\z')][string]$ExpectedLauncherSha256 = '',
    [ValidatePattern('\A[0-9A-F]{64}\z')][string]$ExpectedEngineSha256
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$Scenario = "cut-$Frontier"
$token = [Guid]::NewGuid().ToString('N')
$transaction = "native-cut-$token"
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
$runDir = Join-Path $runtime "native-cut-runs/$RunId"
if (Test-Path -LiteralPath $runDir) { throw "RunId existente: $runDir" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$stdout = Join-Path $runDir 'stdout.log'
$stderr = Join-Path $runDir 'stderr.log'
$endpoint = Join-Path $runDir 'control.json'
$resultPath = Join-Path $runDir 'result.json'
$arm = Join-Path $world "allomancy/native-cut-arm-$token"
$primary = Join-Path $world "players/data/$OwnerUuid.dat"
if (Test-Path -LiteralPath $arm) { throw 'Stale arm file.' }
if (-not (Test-Path -LiteralPath $primary -PathType Leaf)) { throw 'Owner DAT missing.' }
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
$env:ALLOMANCY_NATIVE_TEST_RUN_TOKEN = $token
$options = @('--world',(Split-Path -Leaf $world),'--game-dir',$gameDir,'--resources',$resources,
    '--offscreen','--no-pause','--test-terrain','--creative','--render-distance','2',
    '--name',$PlayerName,'--control-socket',$endpoint)
$inputs = @($build.inputs | ForEach-Object {
    [ordered]@{ path = $_.path; recorded_sha256 = $_.sha256; current_sha256 = if (Test-Path -LiteralPath $_.path -PathType Leaf) { (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash } else { $null } }
})
if (-not $inputs.Count -or @($inputs | Where-Object { -not $_.current_sha256 -or ($_.current_sha256 -ne $_.recorded_sha256 -and ($_.path -ine $build.launcher -or $_.current_sha256 -cne $ExpectedLauncherSha256)) }).Count) { throw 'Missing/drifted runtime/source inputs.' }
if ($ExpectedEngineSha256 -and (Get-FileHash -LiteralPath $engine).Hash -cne $ExpectedEngineSha256) { throw 'Unexpected engine hash.' }
if (Get-Process | Where-Object { $_.ProcessName -match 'horizon-engine|horizon-client|cargo|rustc' }) { throw 'Another game/build is active.' }
$current = $world
while ($current.TrimEnd('\') -ine $runtime.TrimEnd('\')) {
    if ((Get-Item -LiteralPath $current -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse world path.' }
    $parent = Split-Path -Parent $current
    if (-not $parent -or $parent -eq $current) { throw 'World outside runtime.' }
    $current = $parent
}
$result = [ordered]@{
    status = 'starting'; pass = $false; pid = $null; exit_code = $null; termination = $null; error = $null; scenario = $Scenario
    engine = $engine; engine_sha256 = (Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash
    build_receipt = $buildReceipt; build_receipt_sha256 = (Get-FileHash -LiteralPath $buildReceipt -Algorithm SHA256).Hash
    launcher_drift_scope = 'Build tool recorded before build; direct engine launch does not execute launcher. Source/runtime drift remains rejected.'
    inputs = $inputs; helper = $helper; helper_sha256 = (Get-FileHash -LiteralPath $helper -Algorithm SHA256).Hash
    arguments = $options; world = $world; stdout = $stdout; stderr = $stderr
    frontier = $Frontier; run_token = $token; transaction = $transaction; owner = $OwnerUuid; world_id = $identity.world_id; baseline = @(); after = @(); trace = $null; trace_line = $null
    harness_sha256 = (Get-FileHash -LiteralPath $PSCommandPath).Hash; log_source = @($stdout,$stderr)
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
    $ready = $false
    while ($timer.Elapsed.TotalSeconds -lt 90) {
        $process.Refresh()
        if ($process.HasExited) { throw 'Exited before Ready.' }
        $log = Logs
        if ($log -match 'ALLOMANCY_SAVE event=fault|result=FAIL|phase=WorldOpenFailed') { throw 'Native failure before arm.' }
        if ($log -match "ALLOMANCY_NATIVE_TEST event=enabled scenario=$Scenario" -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint)) { $ready=$true; break }
        Start-Sleep -Milliseconds 100
    }
    if (-not $ready) { throw 'Timeout before Ready.' }
    $commandFile = Join-Path $runDir 'save.txt'
    [IO.File]::WriteAllText($commandFile,'save',[Text.UTF8Encoding]::new($false))
    $response = Control $commandFile
    $result['baseline_save'] = $response
    Save-Result
    $reply = $response.reply | ConvertFrom-Json
    if ($response.exit -ne 0 -or -not $reply -or $reply.PSObject.Properties['error'] -or $reply.ok -eq $false) { throw 'Baseline save failed.' }
    function Copy-Snapshots([string]$name) {
        $destination = Join-Path $runDir "$name/players/data"
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
        return @(@($primary,"$primary`_old","$primary`_new") | ForEach-Object {
            if (Test-Path -LiteralPath $_ -PathType Leaf) {
                $copy = Join-Path $destination (Split-Path -Leaf $_)
                Copy-Item -LiteralPath $_ -Destination $copy
                [ordered]@{ source=$_; copy=$copy; sha256=(Get-FileHash -LiteralPath $copy).Hash }
            }
        })
    }
    $result.baseline = Copy-Snapshots 'baseline'
    $baselinePath = Join-Path $runDir "baseline/players/data/$OwnerUuid.dat"
    if (-not (Test-Path -LiteralPath $baselinePath -PathType Leaf)) { throw 'Baseline primary missing; do not arm.' }
    $oracle = Join-Path $PSScriptRoot 'inspect-native-cuts.cjs'
    $oracleArgs = @('--baseline',$baselinePath,'--world',(Join-Path $runDir 'baseline'),'--owner',$OwnerUuid,'--world-id',$identity.world_id,'--mode','cut','--expect','A','--transaction',$transaction,'--slot','0')
    $oldPath = "$baselinePath`_old"
    if (Test-Path -LiteralPath $oldPath) { $oracleArgs += @('--baseline-old',$oldPath) }
    & node $oracle @oracleArgs | Set-Content -LiteralPath (Join-Path $runDir 'baseline-oracle.json')
    if ($LASTEXITCODE -ne 0) { throw 'Invalid baseline/candidate; do not arm.' }
    $result['arm'] = $arm
    $result['phase'] = 'armed; no further control or Host requests'
    Save-Result
    [IO.File]::WriteAllText($arm,$token,[Text.UTF8Encoding]::new($false))
    $expected = "ALLOMANCY_NATIVE_CUT event=paused pid=$($process.Id) frontier=$Frontier transaction=$transaction run_token=$token durable=false"
    $timer.Restart(); $captured=$false
    while ($timer.Elapsed.TotalSeconds -lt 30) {
        $process.Refresh()
        if ($process.HasExited) { throw 'Exited before exact frontier.' }
        $stream=$null; $reader=$null
        try {
            $stream=[IO.File]::Open($stderr,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
            $reader=[IO.StreamReader]::new($stream); $lines=$reader.ReadToEnd() -split "`r?`n"
        } finally { if ($reader) { $reader.Dispose() } elseif ($stream) { $stream.Dispose() } }
        $hits = @($lines | Where-Object { $_ -ceq $expected })
        if ($hits.Count -gt 1) { throw 'Repeated ambiguous trace.' }
        if (($lines -join "`n") -match 'result=FAIL|ALLOMANCY_SAVE event=fault' -or @($lines | Where-Object { $_ -match 'ALLOMANCY_NATIVE_CUT event=' -and $_ -cne $expected }).Count) { throw 'Failure or mismatched frontier trace.' }
        if ($hits.Count -eq 1) {
            $result.trace=$expected; $result.trace_line=1+[Array]::IndexOf($lines,$expected)
            Save-Result
            $process.Kill(); $process.WaitForExit(); $process.Refresh()
            $result.exit_code=$process.ExitCode; $result.termination='killed_at_exact_adapter_frontier'
            $captured=$true; break
        }
        if (($lines -join "`n") -match 'ALLOMANCY_NATIVE_CUT event=|result=FAIL|ALLOMANCY_SAVE event=fault') { throw 'Unexpected frontier/failure.' }
        Start-Sleep -Milliseconds 50
    }
    if (-not $captured) { throw 'Timeout waiting exact pause.' }
    $result.after = Copy-Snapshots 'after'
    $result['offline_oracle'] = 'pending complete A/B and separate-copy reopen'
    $result.pass = $true
    $result.status = 'frontier_captured'
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
