param(
    [Parameter(Mandatory)][ValidatePattern('^[a-zA-Z0-9-]+$')][string]$RunId,
    [switch]$Controls,
    [ValidateSet('probe-engine-v2', 'probe-engine-v3', 'probe-engine-v4', 'probe-engine-v5')][string]$EngineVersion = 'probe-engine-v2',
    [switch]$Content,
    [switch]$Capabilities,
    [ValidateRange(30, 300)][int]$TimeoutSeconds = 45
)
$ErrorActionPreference = 'Stop'
if ($Capabilities) { $Controls = $true }
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$runtime = Join-Path $PSScriptRoot 'runtime'
$engine = Join-Path $runtime "$EngineVersion/horizon-engine.exe"
$gameDir = Join-Path $runtime 'game-probe-v2'
$evidence = Join-Path $projectRoot 'docs/execution/evidence'
$prefix = Join-Path $evidence "p0-smoke-$RunId"
$socket = Join-Path $runtime "probe-$RunId-control.sock"
if (!(Test-Path -LiteralPath $engine)) { throw 'Build the pinned v2 probe first.' }
if (Test-Path -LiteralPath ($prefix + '.json')) { throw 'RunId already has evidence; inspect it or choose a new RunId.' }
New-Item -ItemType Directory -Force $gameDir | Out-Null
$env:HORIZON_HOME = Join-Path $runtime 'home-v2'
$env:HORIZON_LAUNCHER_HOME = Join-Path $runtime 'launcher'
$env:XDG_DATA_HOME = Join-Path $runtime 'data'
$env:XDG_CONFIG_HOME = Join-Path $runtime 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $runtime 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $runtime 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $runtime 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
$options = @('--world', 'AllomancyP0', '--creative', '--test-terrain', '--offscreen', '--no-pause',
    '--render-distance', '2', '--width', '1280', '--height', '720',
    '--game-dir', $gameDir, '--resources', (Join-Path $runtime "$EngineVersion/.local/horizon-client/26.3"),
    '--smoke-report', ($prefix + '.json'), '--control-socket', $socket)
if (!$Controls) { $options += @('--world-frames', '80') }
if ($Content) { $options += @('--dev-mod', (Join-Path $PSScriptRoot 'content')) }
# Start-Process joins argument arrays; quote each argument to preserve workspace spaces.
$quoted = $options | ForEach-Object { '"' + $_ + '"' }
$process = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput ($prefix + '-stdout.log') -RedirectStandardError ($prefix + '-stderr.log')
$process.Id | Set-Content -LiteralPath (Join-Path $runtime "smoke-$RunId.pid")
if ($Controls) {
    $ctl = Join-Path $env:APPDATA 'horizon-launcher/versions/nightly-2026-11-08-fdc87ae/horizon-ctl.exe'
    $controlResults = [Collections.Generic.List[object]]::new()
    function Invoke-ProbeControl([string[]]$Words) {
        $reply = & $ctl --socket $socket --timeout 30 @Words 2>&1
        $code = $LASTEXITCODE
        $controlResults.Add([pscustomobject]@{ command = $Words; exit_code = $code; reply = ($reply -join "`n") })
        if ($code -ne 0) { throw "Control failed: $($Words -join ' ')" }
    }
    try {
        # Wait for this explicit endpoint to appear, not the user's default control session.
        $ready = [Diagnostics.Stopwatch]::StartNew()
        while (!(Test-Path -LiteralPath $socket) -and !$process.HasExited -and $ready.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
            Start-Sleep -Milliseconds 200
        }
        Invoke-ProbeControl @('wait-ready', '30')
        if ($Capabilities) {
            Invoke-ProbeControl @('tp', '6.5', '79', '4.5')
            Invoke-ProbeControl @('setblock', '6', '81', '4', 'minecraft:air')
            Invoke-ProbeControl @('wait', '10')
        }
        Invoke-ProbeControl @('state')
        Invoke-ProbeControl @('key', 'z', 'down')
        if ($Capabilities) {
            Invoke-ProbeControl @('wait', '1')
            Invoke-ProbeControl @('state')
        }
        Invoke-ProbeControl @('key', 'z', 'repeat')
        Invoke-ProbeControl @('key', 'z', 'up')
        Invoke-ProbeControl @('open-inventory')
        Invoke-ProbeControl @('key', 'z', 'tap')
        Invoke-ProbeControl @('close')
        Invoke-ProbeControl @('key', 't', 'tap')
        Invoke-ProbeControl @('screen')
        Invoke-ProbeControl @('key', 'z', 'tap')
        Invoke-ProbeControl @('key', 'esc', 'tap')
        Invoke-ProbeControl @('state')
        if ($Capabilities) {
            Invoke-ProbeControl @('wait', '10')
            Invoke-ProbeControl @('setblock', '6', '81', '4', 'minecraft:stone')
            Invoke-ProbeControl @('key', 'z', 'tap')
            Invoke-ProbeControl @('wait', '1')
            Invoke-ProbeControl @('state')
            Invoke-ProbeControl @('wait', '3')
            Invoke-ProbeControl @('state')
            Invoke-ProbeControl @('setblock', '6', '81', '4', 'minecraft:air')
            Invoke-ProbeControl @('wait', '10')
            $sample = if ($Content) { 'allomancy_probe_content:probe_metal' } else { 'minecraft:iron_ingot' }
            Invoke-ProbeControl @('give', $sample, '1')
            Invoke-ProbeControl @('select', $sample)
            Invoke-ProbeControl @('use-item')
            Invoke-ProbeControl @('inventory')
            Invoke-ProbeControl @('command', 'summon minecraft:pig 6.5 79 6.0 {NoAI:1b}')
            Invoke-ProbeControl @('command', 'kill @e[type=minecraft:pig,distance=..6]')
            Invoke-ProbeControl @('wait', '2')
            Invoke-ProbeControl @('screenshot', ($prefix + '-hud.png'))
        }
    } finally {
        try { Invoke-ProbeControl @('quit') } catch {
            Write-Warning "Probe cleanup failed: $($_.Exception.Message)"
            if (!$process.HasExited) { [void]$process.CloseMainWindow() }
        } finally {
            $controlResults | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath ($prefix + '-controls.json')
        }
    }
}
$elapsed = [Diagnostics.Stopwatch]::StartNew()
$exited = $false
while (!$exited -and $elapsed.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
    $remaining = [Math]::Max(1, ($TimeoutSeconds - $elapsed.Elapsed.TotalSeconds) * 1000)
    $exited = $process.WaitForExit([int][Math]::Min(45000, $remaining))
    if (!$exited) { Write-Output "smoke_running elapsed_seconds=$([int]$elapsed.Elapsed.TotalSeconds)" }
}
$timedOut = !$exited
if ($timedOut) {
    $closeRequested = $process.CloseMainWindow()
    $exitedAfterClose = $process.WaitForExit(5000)
    [pscustomobject]@{ timed_out = $true; close_requested = $closeRequested; exited_after_close = $exitedAfterClose } | ConvertTo-Json | Set-Content -LiteralPath ($prefix + '-result.json')
    throw 'Smoke timed out: inspect its process and logs before another run.'
}
$process.Refresh()
$exitCode = $process.ExitCode
[pscustomobject]@{ timed_out = $false; exit_code = $exitCode } | ConvertTo-Json | Set-Content -LiteralPath ($prefix + '-result.json')
Get-Content -LiteralPath ($prefix + '-stderr.log') -Tail 18
if (Test-Path -LiteralPath ($prefix + '.json')) { Get-Content -LiteralPath ($prefix + '.json') }
exit $exitCode
