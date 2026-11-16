[CmdletBinding()]
param([Parameter(Mandatory)][string]$RuntimeRoot,[int]$Attempt=1,[ValidateSet('reconnect','dead-login','death-respawn','dimension','save-pause')][string[]]$Kinds=@('reconnect','dead-login','death-respawn','dimension','save-pause'))
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path
$repo=Split-Path -Parent $PSScriptRoot
$source=Join-Path $runtime 'game-p2-owner-econ/saves/credit'
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
foreach($kind in $Kinds) {
    $run="rich-$kind-$Attempt"
    $world=Join-Path $runtime "game-p2-gates/saves/$run"
    Checkpoint "ANTES $run N9fixturevaloresnotriviales, cloncerrado+baselinepreservado, oldbuildcuts1 core/guardproducciónintactos."
    Clone-Closed $source $world (Join-Path $evidence "$run-clone.json")
    $fixture=if($kind -eq 'dead-login'){'dead-player'}else{'player-seed'}
    & node (Join-Path $PSScriptRoot 'make-native-gate-fixture.cjs') --world $world --kind $fixture | Set-Content -LiteralPath (Join-Path $evidence "$run-fixture.json")
    if($LASTEXITCODE -ne 0){throw 'Rich fixture failed'}
    $baselineDir=Join-Path $runtime "native-gate-runs/$run/baseline"
    if($kind -in @('reconnect','dead-login')) {
        New-Item -ItemType Directory -Path $baselineDir -Force|Out-Null
        Copy-Item -LiteralPath (Join-Path $world "players/data/$owner.dat") -Destination $baselineDir
        Copy-Item -LiteralPath (Join-Path $world "players/data/$owner.dat_old") -Destination $baselineDir
        if ($kind -eq 'dead-login') {
            & (Join-Path $PSScriptRoot 'test-native-lifecycle.ps1') -BuildName p2-cuts1 -RuntimeRoot $runtime -RunId $run -WorldRoot $world -Scenario dead-login -OwnerUuid $owner
        } else {
        & (Join-Path $PSScriptRoot 'test-native-open.ps1') -BuildName p2-cuts1 -RuntimeRoot $runtime -RunId $run -WorldRoot $world -Expect accept -PlayerName AllomancyQA
        }
        $mode=if($kind -eq 'dead-login'){'death-login'}else{'reconnect'}
    } else {
        & (Join-Path $PSScriptRoot 'test-native-lifecycle.ps1') -BuildName p2-cuts1 -RuntimeRoot $runtime -RunId $run -WorldRoot $world -Scenario $kind -OwnerUuid $owner -BaselineDirectory $baselineDir
        $mode=if($kind -eq 'death-respawn'){'death-respawn'}else{'unchanged'}
    }
    Assert-Closed
    & node (Join-Path $PSScriptRoot 'inspect-native-gate-state.cjs') --baseline (Join-Path $baselineDir "$owner.dat") --world $world --mode $mode --old-expect same --owner $owner --world-id $worldId | Set-Content -LiteralPath (Join-Path $evidence "$run-state.json")
    if($LASTEXITCODE -ne 0){Checkpoint "FALLO $run oracleoffline; conservarintento/fixtures y detener.";throw 'Rich lifecycle oracle failed'}
    Checkpoint "DESPUES $run PASS native+oracle$mode exacto:fracciones/unlocks/rules/ledger/selected/inventario comprobados, nodes/noXPclaims."
}
