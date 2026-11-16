[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('\A[a-z0-9-]+\z')]
    [string]$BuildName,
    [string]$RuntimeRoot,
    [switch]$NativeTest,
    [switch]$StageOnly
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $RuntimeRoot) { $RuntimeRoot = Join-Path $root 'experiments/horizon-probe/runtime' }
$runtime = (Resolve-Path -LiteralPath $RuntimeRoot -ErrorAction Stop).Path
$launcher = Join-Path $env:USERPROFILE 'AppData/Local/Programs/Horizon Launcher/horizon-launcher.exe'
$version = 'nightly-2026-11-08-fdc87ae'
$base = Join-Path $runtime "launcher/versions/$version"
$build = Join-Path $runtime "native-builds/$BuildName"
$bundle = Join-Path $build 'bundle'
$export = Join-Path $runtime "$BuildName-engine"
$receiptPath = Join-Path $build 'receipt.json'
$log = Join-Path $root "docs/execution/evidence/$BuildName-build.log"
$descriptor = Join-Path $root 'integration/horizon/horizon-mixin.toml'
$crates = @('allomancy-core', 'allomancy-horizon')
$helpers = @('preflight.rs', 'world_guard.rs', 'profile_capture.rs', 'owner_bootstrap.rs')
$required = @($helpers | ForEach-Object { Join-Path $root "integration/horizon/$_" })
$required += @($launcher, (Join-Path $base '.installed'), (Join-Path $base 'horizon-client.exe'), (Join-Path $base '.local/horizon-client/26.3/horizon-resources.json'), $descriptor)
foreach ($crate in $crates) {
    $required += Join-Path $root "crates/$crate/Cargo.toml"
    $required += Join-Path $root "crates/$crate/src/lib.rs"
}
foreach ($path in $required) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Falta input requerido: $path" }
}
foreach ($path in @($build, $export, $log)) {
    if (Test-Path -LiteralPath $path) { throw "Destino existente; inspeccionar: $path" }
}
$inputs = @($launcher, (Join-Path $base '.installed'), (Join-Path $base 'horizon-client.exe'), (Join-Path $base '.local/horizon-client/26.3/horizon-resources.json'), $descriptor)
foreach ($crate in $crates) {
    $inputs += Join-Path $root "crates/$crate/Cargo.toml"
    $inputs += @(Get-ChildItem -LiteralPath (Join-Path $root "crates/$crate/src") -File -Recurse | ForEach-Object { $_.FullName })
}
$inputs += @($helpers | ForEach-Object { Join-Path $root "integration/horizon/$_" })
$hashes = @($inputs | ForEach-Object { @{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash } })
$argsList = @('native-build', $bundle, '--version', $version, '--trust-native-code', '--profile', 'dev', '--server-output', $export)
$receipt = [ordered]@{ status = 'staging'; launcher = $launcher; base = $base; inputs = $hashes; arguments = $argsList; log = $log; staged_manifest = $null; exit_code = $null; error = $null; native_test = [bool]$NativeTest }
function Save-Receipt {
    $tmp = "$receiptPath.tmp"
    $receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $tmp -Encoding UTF8
    if (Test-Path -LiteralPath $receiptPath) {
        $backup = "$receiptPath.bak"
        [IO.File]::Replace($tmp, $receiptPath, $backup)
    } else { [IO.File]::Move($tmp, $receiptPath) }
}
New-Item -ItemType Directory -Path $bundle -Force | Out-Null
try {
    Save-Receipt
    Copy-Item -LiteralPath $descriptor -Destination (Join-Path $bundle 'horizon-mixin.toml')
    foreach ($helper in $helpers) {
        Copy-Item -LiteralPath (Join-Path $root "integration/horizon/$helper") -Destination (Join-Path $bundle $helper)
    }
    foreach ($crate in $crates) {
        $destination = Join-Path $bundle "crates/$crate"
        New-Item -ItemType Directory -Path $destination -Force | Out-Null
        Copy-Item -LiteralPath (Join-Path $root "crates/$crate/Cargo.toml") -Destination $destination
        Copy-Item -LiteralPath (Join-Path $root "crates/$crate/src") -Destination $destination -Recurse
    }
    # Keep bundle-relative crate paths for launcher validation and relocation.
    $api = Join-Path $bundle 'crates/allomancy-horizon'
    $manifestPath = Join-Path $api 'Cargo.toml'
    $manifest = [IO.File]::ReadAllText($manifestPath)
    $oldPath = 'path = "../allomancy-core"'
    if ([regex]::Matches($manifest, [regex]::Escape($oldPath)).Count -ne 1) {
        throw 'Expected exactly one sibling core dependency in the adapter manifest.'
    }
    # The current launcher resolves this dependency through [[crate]] and stages one core.
    if ($NativeTest) {
        if ([regex]::Matches($manifest, 'default = \[\]').Count -ne 1) { throw 'Expected empty default features in source manifest.' }
        $staged = [IO.File]::ReadAllText($manifestPath).Replace('default = []', 'default = ["native-test"]')
        [IO.File]::WriteAllText($manifestPath, $staged)
    }
    $receipt.staged_manifest = @{
        path = $manifestPath
        sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash
        transformation = 'Bundle-relative API dependency retained for launcher validation/relocation via [[crate]]; require single core post-build; opt-in native-test recorded.'
    }
    if ($StageOnly) {
        $receipt.status = 'staged'
        Save-Receipt
        return
    }
    $receipt.status = 'building'
    Save-Receipt
    $env:HORIZON_HOME = Join-Path $runtime "homes/$BuildName"
    $env:HORIZON_LAUNCHER_HOME = Join-Path $runtime 'launcher'
    $env:XDG_DATA_HOME = Join-Path $runtime 'data'
    $env:XDG_CONFIG_HOME = Join-Path $runtime 'config'
    $env:HORIZON_LAUNCHER_TRASH = Join-Path $runtime 'trash'
    $env:HORIZON_LAUNCHER_CLIPBOARD = Join-Path $runtime 'clipboard'
    $env:HORIZON_LAUNCHER_DESKTOP = Join-Path $runtime 'desktop'
    $env:HORIZON_LAUNCHER_NO_SELF_UPDATE = '1'
    $env:HORIZON_NO_AUDIO = '1'
    Push-Location -LiteralPath $root
    try {
        & $launcher @argsList 2>&1 | Tee-Object -FilePath $log
        $code = $LASTEXITCODE
    } finally { Pop-Location }
    $receipt.exit_code = $code
    if ($code -ne 0) { $receipt.status = 'failed'; Save-Receipt; exit $code }
    if (-not (Test-Path -LiteralPath (Join-Path $export 'horizon-engine.exe') -PathType Leaf)) { throw 'El launcher terminó con exit 0 pero falta horizon-engine.exe.' }
    $receipt.status = 'succeeded'
    Save-Receipt
} catch {
    $receipt.status = 'failed'
    $receipt.error = $_.Exception.Message
    Save-Receipt
    throw
}
