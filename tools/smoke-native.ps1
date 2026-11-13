[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidatePattern('\A[a-z0-9-]+\z')][string]$BuildName,
    [Parameter(Mandatory = $true)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
    [ValidateRange(180, 3600)][int]$TimeoutSeconds = 180,
    [string]$RuntimeRoot
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $RuntimeRoot) { $RuntimeRoot = Join-Path $root 'experiments/horizon-probe/runtime' }
$runtime = (Resolve-Path -LiteralPath $RuntimeRoot -ErrorAction Stop).Path
$export = Join-Path $runtime "$BuildName-engine"
$engine = Join-Path $export 'horizon-engine.exe'
$resources = Join-Path $export '.local/horizon-client/26.3'
$gameDir = Join-Path $runtime "game-p1-$RunId"
$prefix = Join-Path $gameDir 'smoke'
$stdout = "$prefix-stdout.log"
$stderr = "$prefix-stderr.log"
$report = "$prefix.json"
$result = "$prefix-result.json"
foreach ($inputPath in @($engine, (Join-Path $resources 'horizon-resources.json'))) {
    if (-not (Test-Path -LiteralPath $inputPath -PathType Leaf)) { throw "Falta input requerido: $inputPath" }
}
foreach ($path in @($gameDir, $stdout, $stderr, $report, $result)) {
    if (Test-Path -LiteralPath $path) { throw "RunId existente; inspeccionar: $path" }
}
New-Item -ItemType Directory -Path $gameDir | Out-Null
$env:HORIZON_HOME = Join-Path $runtime "homes/$BuildName"
$env:HORIZON_LAUNCHER_HOME = Join-Path $runtime 'launcher'
$env:XDG_DATA_HOME = Join-Path $runtime 'data'
$env:XDG_CONFIG_HOME = Join-Path $runtime 'config'
$env:HORIZON_LAUNCHER_TRASH = Join-Path $runtime 'trash'
$env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $runtime 'clipboard'
$env:HORIZON_LAUNCHER_DESKTOP = Join-Path $runtime 'desktop'
$env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
$env:HORIZON_NO_AUDIO = '1'
$options = @('--world', 'AllomancyP1', '--creative', '--test-terrain', '--offscreen', '--no-pause',
    '--render-distance', '2', '--width', '1280', '--height', '720', '--world-frames', '80',
    '--game-dir', $gameDir, '--resources', $resources, '--smoke-report', $report)
$quoted = $options | ForEach-Object { '"' + $_ + '"' }
$process = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
    [pscustomobject]@{ timed_out = $true; process_id = $process.Id } | ConvertTo-Json | Set-Content -LiteralPath $result
    throw "Smoke agotó el tiempo; inspeccionar proceso $($process.Id) y logs: $prefix"
}
$process.Refresh()
[pscustomobject]@{ timed_out = $false; process_id = $process.Id; exit_code = $process.ExitCode } | ConvertTo-Json | Set-Content -LiteralPath $result
Write-Output "stdout=$stdout`nstderr=$stderr`nsmoke_report=$report"
exit $process.ExitCode
