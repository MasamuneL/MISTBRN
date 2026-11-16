[CmdletBinding()]
param([Parameter(Mandatory)][string]$RuntimeRoot,[Parameter(Mandatory)][ValidateSet('rename','marker-corrupt','marker-conflict','world-mismatch','both-owner-invalid','future-owner-primary-valid-old','restored-complete')][string]$Kind,[int]$Attempt=1)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path
$repo=Split-Path -Parent $PSScriptRoot
$source=Join-Path $runtime 'game-p2-owner-econ/saves/credit'
if ($Kind -eq 'restored-complete') { $source=Join-Path $runtime 'backups/gates-both-owner-invalid-1-complete' }
$owner='3efd2c0e-f0de-3e85-b045-dec265b3f79c'
$worldId='026e465a87434711a6951c07588dc0ee'
function Assert-Closed {
    if (Get-Process | Where-Object { $_.ProcessName -match 'horizon-engine|horizon-client|cargo|rustc' }) { throw 'Game/build still active.' }
}
function Manifest([string]$directory) {
    $items=@(Get-Item -LiteralPath $directory -Force)+@(Get-ChildItem -LiteralPath $directory -Recurse -Force)
    if (@($items | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'Linked fixture rejected.' }
    return @($items | Where-Object { -not $_.PSIsContainer } | Sort-Object FullName | ForEach-Object {
        [ordered]@{path=[IO.Path]::GetRelativePath($directory,$_.FullName);sha256=(Get-FileHash -LiteralPath $_.FullName).Hash}
    })
}
function Clone-Closed([string]$from,[string]$to,[string]$receipt) {
    Assert-Closed
    if (Test-Path -LiteralPath $to) { throw "Clone exists: $to" }
    $before=Manifest $from
    New-Item -ItemType Directory -Path (Split-Path -Parent $to) -Force | Out-Null
    Copy-Item -LiteralPath $from -Destination $to -Recurse
    $after=Manifest $to
    if (($before|ConvertTo-Json -Compress) -cne ($after|ConvertTo-Json -Compress)) { throw 'Clone mismatch.' }
    [ordered]@{source=$from;destination=$to;closed=$true;source_files=$before;copied_files=$after;equal=$true} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $receipt
}
function Checkpoint([string]$message) {
    $file=Join-Path $repo 'docs/execution/CHECKPOINT.md'
    $text=[IO.File]::ReadAllText($file)+"`r`nP2 gates: $message`r`n"
    [IO.File]::WriteAllText(($file+'.tmp'),$text)
    [IO.File]::Replace(($file+'.tmp'),$file,($file+'.gates-before.bak'))
}

$evidence=Join-Path $repo 'docs/execution/evidence'
$run="gates-$Kind-$Attempt"
$world=Join-Path $runtime "game-p2-gates/saves/$Kind-$Attempt"
Checkpoint "ANTES $run caso nuevo N1/N8 sobre clonecerrado, buildcuts1sinRustguardcambio. No repetir47; guard/sourcescore sin cambios."
Clone-Closed $source $world (Join-Path $evidence "$run-clone.json")
$backup=Join-Path $runtime "backups/$run-complete"
Clone-Closed $world $backup (Join-Path $evidence "$run-backup.json")
$marker=Join-Path $world 'allomancy/world-identity.json'
$markerBackup="$marker.bak"
$expect='reject'
switch($Kind){
 'rename' {
    $target=[IO.Path]::GetFullPath("$world-renamed")
    if (-not $target.StartsWith((Join-Path $runtime 'game-p2-gates/saves/'),[StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $target)) {throw 'Unsafe rename destination'}
    $resolved=(Resolve-Path -LiteralPath $world).Path
    if(-not $resolved.StartsWith($runtime+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'Unsafe rename source'}
    Rename-Item -LiteralPath $resolved -NewName (Split-Path -Leaf $target)
    $world=$target;$expect='accept'
 }
 'marker-corrupt' { [IO.File]::WriteAllText($marker,'{',[Text.UTF8Encoding]::new($false)) }
 'marker-conflict' { [IO.File]::WriteAllText($markerBackup,'{"schema_version":1,"world_id":"11111111111111111111111111111111"}',[Text.UTF8Encoding]::new($false)) }
 'world-mismatch' {
    $bytes=[Text.UTF8Encoding]::new($false).GetBytes('{"schema_version":1,"world_id":"11111111111111111111111111111111"}')
    [IO.File]::WriteAllBytes($marker,$bytes);[IO.File]::WriteAllBytes($markerBackup,$bytes)
 }
 'restored-complete' { $expect='accept' }
 default {
    & node (Join-Path $PSScriptRoot 'make-native-gate-fixture.cjs') --world $world --kind $Kind | Set-Content -LiteralPath (Join-Path $evidence "$run-fixture.json")
    if($LASTEXITCODE -ne 0){throw 'Fixture failed'}
 }
}
$before=Manifest $world
& (Join-Path $PSScriptRoot 'test-native-open.ps1') -BuildName p2-cuts1 -RuntimeRoot $runtime -RunId $run -WorldRoot $world -Expect $expect -PlayerName AllomancyQA
Assert-Closed
$r=Get-Content (Join-Path $runtime "native-open-runs/$run/result.json") -Raw|ConvertFrom-Json
if(-not $r.pass){throw 'Native open gate failed'}
if($expect -eq 'reject' -and ($before|ConvertTo-Json -Compress) -cne ((Manifest $world)|ConvertTo-Json -Compress)){throw 'Rejected fixture changed'}
$summary=[ordered]@{run=$run;kind=$Kind;pass=$true;expect=$expect;exit=$r.exit_code;termination=$r.termination;world=$world;complete_backup=$backup;source_manifest=(Join-Path $evidence "$run-clone.json");rejected_files_unchanged=($expect -eq 'reject')}
$summary|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $evidence "$run-check.json")
Checkpoint "DESPUES $run PASS exit$($r.exit_code) $($r.termination); rejectfullmanifestintacto/accept requiereoracleoffline. Backupfullpreservado, sinmainmerge."
