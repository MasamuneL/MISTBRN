[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('\A[a-z0-9-]+\z')][string]$BuildName,
    [Parameter(Mandatory)][string]$RuntimeRoot,
    [Parameter(Mandatory)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
    [Parameter(Mandatory)][string]$WorldRoot,
    [Parameter(Mandatory)][ValidateSet('accept','reject')][string]$Expect,
    [ValidateRange(1,180)][int]$TimeoutSeconds = 180,
    [string]$DenyReadRelativePath,
    [ValidatePattern('\A[A-Za-z0-9_]{1,16}\z')][string]$PlayerName,
    [switch]$BackendDisabled
)
$ErrorActionPreference = 'Stop'
$runtime = (Resolve-Path -LiteralPath $RuntimeRoot).Path
$world = (Resolve-Path -LiteralPath $WorldRoot).Path
$saves = Split-Path -Parent $world
$gameDir = Split-Path -Parent $saves
$worldName = Split-Path -Leaf $world
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
foreach ($file in @($engine, (Join-Path $resources 'horizon-resources.json'), $buildReceipt)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Falta input: $file" }
}
$build = Get-Content -LiteralPath $buildReceipt -Raw | ConvertFrom-Json
if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test) {
    throw 'Se requiere receipt succeeded, exit 0 y native_test activado.'
}
$worldId = ''
try {
    if (Test-Path -LiteralPath $marker -PathType Leaf) {
        $identity = Get-Content -LiteralPath $marker -Raw | ConvertFrom-Json -ErrorAction Stop
        if ($identity.world_id -is [string]) { $worldId = $identity.world_id }
    }
} catch { $worldId = '' }
$denyPath = $null
if ($DenyReadRelativePath) {
    if ([IO.Path]::IsPathRooted($DenyReadRelativePath) -or
        $DenyReadRelativePath -notmatch '\.dat(_old)?$' -or
        ($DenyReadRelativePath -split '[\\/]' | Where-Object { $_ -eq '..' -or $_ -eq '.' -or $_ -eq '' })) {
        throw 'DenyReadRelativePath debe ser una ruta relativa DAT sin componentes especiales.'
    }
    $denyPath = [IO.Path]::GetFullPath((Join-Path $world $DenyReadRelativePath))
    if (-not $denyPath.StartsWith($world.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
        throw 'DenyReadRelativePath sale del mundo.'
    }
    $current = $world
    while ($true) {
        $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'No se permiten enlaces en la ruta del mundo.' }
        if ($current.TrimEnd('\') -ieq $runtime.TrimEnd('\')) { break }
        $parent = Split-Path -Parent $current
        if (-not $parent -or $parent -eq $current) { throw 'Ruta del mundo fuera de runtime.' }
        $current = $parent
    }
    $current = $world
    foreach ($part in ($DenyReadRelativePath -split '[\\/]')) {
        $current = Join-Path $current $part
        $item = Get-Item -LiteralPath $current -Force -ErrorAction Stop
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'No se permiten enlaces en la ruta bloqueada.' }
    }
    if (-not (Test-Path -LiteralPath $denyPath -PathType Leaf)) { throw 'DAT a bloquear no existe.' }
}
$runDir = Join-Path $runtime "native-open-runs/$RunId"
if (Test-Path -LiteralPath $runDir) { throw "RunId existente: $runDir" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$stdout = Join-Path $runDir 'stdout.log'
$stderr = Join-Path $runDir 'stderr.log'
$resultPath = Join-Path $runDir 'result.json'
$report = Join-Path $runDir 'smoke.json'
function Snapshot {
    $files = @()
    foreach ($relative in @('level.dat','level.dat_old','allomancy/world-identity.json','allomancy/world-identity.json.bak')) {
        $path = Join-Path $world $relative
        if (Test-Path -LiteralPath $path -PathType Leaf) { $files += Get-Item -LiteralPath $path }
    }
    $players = Join-Path $world 'players/data'
    if (Test-Path -LiteralPath $players -PathType Container) {
        $files += @(Get-ChildItem -LiteralPath $players -File -Recurse | Where-Object { $_.Name -imatch '\.dat(_old)?$' })
    }
    return @($files | Sort-Object FullName | ForEach-Object {
        [ordered]@{ path = [IO.Path]::GetRelativePath($world, $_.FullName); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
    })
}
function Save-Result {
    $tmp = "$resultPath.tmp"
    $result | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $tmp -Encoding utf8
    if (Test-Path -LiteralPath $resultPath) {
        [IO.File]::Replace($tmp, $resultPath, "$resultPath.bak")
    } else { [IO.File]::Move($tmp, $resultPath) }
}
function Logs {
    if (-not $script:logReadErrors) { $script:logReadErrors = @{} }
    $text = ''
    foreach ($file in @($stdout, $stderr)) {
        if (-not (Test-Path -LiteralPath $file)) { continue }
        $stream = $null
        $reader = $null
        try {
            $share = [IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
            $stream = [IO.File]::Open($file, [IO.FileMode]::Open, [IO.FileAccess]::Read, $share)
            $reader = [IO.StreamReader]::new($stream)
            $text += $reader.ReadToEnd() + "`n"
            $script:logReadErrors[$file] = 0
        } catch [IO.IOException] {
            $script:logReadErrors[$file] = 1 + [int]$script:logReadErrors[$file]
            if ($script:logReadErrors[$file] -ge 10) { throw }
        } finally {
            if ($reader) { $reader.Dispose() } elseif ($stream) { $stream.Dispose() }
        }
    }
    return $text
}
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
$env:ALLOMANCY_NATIVE_TEST_WORLD_ID = $worldId
$env:ALLOMANCY_NATIVE_TEST_SCENARIO = 'observe'
$options = @('--world',$worldName,'--game-dir',$gameDir,'--resources',$resources,'--offscreen','--no-pause',
    '--test-terrain','--creative','--render-distance','2','--world-frames','80','--frames','1000000','--smoke-report',$report)
if ($PlayerName) { $options += @('--name', $PlayerName) }
$inputHashes = @($build.inputs | ForEach-Object {
    [ordered]@{ path = $_.path; recorded_sha256 = $_.sha256; current_sha256 = if (Test-Path -LiteralPath $_.path -PathType Leaf) { (Get-FileHash -LiteralPath $_.path -Algorithm SHA256).Hash } else { $null } }
})
$result = [ordered]@{
    status = 'starting'; pass = $false; expect = $Expect; backend_disabled = [bool]$BackendDisabled; pid = $null; exit_code = $null
    termination = $null; timed_out = $false; engine = $engine
    engine_sha256 = (Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash
    build_receipt = $buildReceipt; build_receipt_sha256 = (Get-FileHash -LiteralPath $buildReceipt -Algorithm SHA256).Hash
    inputs = $inputHashes; arguments = $options; world = $world
    stdout = $stdout; stderr = $stderr; before = @(Snapshot); after = $null; error = $null
    deny_read_path = $denyPath; deny_read_lock = 'not_requested'
}
Save-Result
$process = $null
$readLock = $null
try {
    if ($denyPath) {
        $readLock = [IO.File]::Open($denyPath, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::None)
        $result.deny_read_lock = 'held'
        Save-Result
    }
    $quoted = $options | ForEach-Object { '"' + $_ + '"' }
    $process = Start-Process -FilePath $engine -ArgumentList $quoted -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    $result.pid = $process.Id
    $result.status = 'running'
    Save-Result
    $timer = [Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
        $process.Refresh()
        $log = Logs
        $failed = $log.Contains('ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldOpenFailed')
        $rejected = $log.Contains('Allomancy preflight: reject')
        $ready = $log.Contains('ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldOpenReady')
        $login = $log.Contains('ALLOMANCY_SAVE event=login')
        if ($Expect -eq 'reject' -and $failed -and $rejected -and -not $ready -and -not $login -and -not $process.HasExited) {
            $process.Kill()
            $process.WaitForExit()
            $result.termination = 'controlled_after_rejection'
            break
        }
        if ($Expect -eq 'accept' -and $failed -and -not $process.HasExited) {
            $process.Kill()
            $process.WaitForExit()
            $result.termination = 'controlled_unexpected_rejection'
            break
        }
        if ($process.HasExited) { $result.termination = 'spontaneous'; break }
        Start-Sleep -Milliseconds 250
    }
    $process.Refresh()
    if (-not $process.HasExited) {
        $result.timed_out = $true
        if ($readLock) {
            $process.Kill()
            $process.WaitForExit()
            $result.termination = 'timeout_failed'
            $result.exit_code = $process.ExitCode
            throw "Timeout; proceso propio detenido antes de liberar bloqueo."
        }
        $result.termination = 'left_running'
        throw "Timeout; proceso $($process.Id) permanece activo."
    }
    $process.WaitForExit()
    $process.Refresh()
    $result.exit_code = $process.ExitCode
    if ($readLock) { $readLock.Dispose(); $readLock = $null; $result.deny_read_lock = 'released_after_exit' }
    $result.after = @(Snapshot)
    $log = Logs
    if ($Expect -eq 'reject') {
        $result.pass = $result.termination -eq 'controlled_after_rejection' -and
            $log.Contains('ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldOpenFailed') -and
            $log.Contains('Allomancy preflight: reject') -and
            -not $log.Contains('ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldOpenReady') -and
            -not $log.Contains('ALLOMANCY_SAVE event=login') -and
            (ConvertTo-Json -InputObject @($result.before) -Depth 6 -Compress) -ceq (ConvertTo-Json -InputObject @($result.after) -Depth 6 -Compress)
    } else {
        $result.pass = $result.termination -eq 'spontaneous' -and $result.exit_code -eq 0 -and
            $log.Contains('Allomancy preflight: accept') -and
            $log.Contains('ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldOpenReady') -and
            $(if ($BackendDisabled) {
                -not $log.Contains('ALLOMANCY_SAVE event=ready accepted_in_memory=true')
            } else {
                $log.Contains('ALLOMANCY_SAVE event=ready accepted_in_memory=true')
            })
    }
    $result.status = if ($result.pass) { 'passed' } else { 'failed' }
    Save-Result
    if (-not $result.pass) { throw 'No se cumplen los criterios; inspeccionar evidencia.' }
} catch {
    $message = $_.Exception.Message
    if ($readLock) {
        if ($process) {
            $process.Refresh()
            if (-not $process.HasExited) {
                $process.Kill()
                $process.WaitForExit()
                $result.termination = 'error_failed'
            }
            $process.Refresh()
            $result.exit_code = $process.ExitCode
        }
        $readLock.Dispose()
        $readLock = $null
        $result.deny_read_lock = 'released_after_exit'
        if ($process -and $process.HasExited) { $result.after = @(Snapshot) }
    }
    $result.status = 'failed'
    $result.error = $message
    Save-Result
    throw
}
Write-Output "result=$resultPath"
