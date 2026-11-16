[CmdletBinding()]
param([Parameter(Mandatory)][string]$RuntimeRoot,[string]$BuildName='p2-cuts1',[int]$Attempt=1,[string[]]$Frontiers=@('before-inventory','after-inventory','after-inline','after-core-commit'))
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
    $text=[IO.File]::ReadAllText($file)+"`r`nN6 suite: $message`r`n"
    [IO.File]::WriteAllText(($file+'.tmp'),$text)
    [IO.File]::Replace(($file+'.tmp'),$file,($file+'.n6-before.bak'))
}
$evidence=Join-Path $repo 'docs/execution/evidence'
foreach ($frontier in $Frontiers) {
    $run="cuts-$frontier-$Attempt"
    $world=Join-Path $runtime "game-p2-cuts/saves/$frontier-$Attempt"
    Checkpoint "ANTES $run clonecerrado nuevo desde ownercredit; baseline save antesarm, traceexacta y killPIDpropio, oracleA/reopen pendientes."
    Clone-Closed $source $world (Join-Path $evidence "$run-clone.json")
    & (Join-Path $PSScriptRoot 'test-native-cuts.ps1') -BuildName $BuildName -RuntimeRoot $runtime -RunId $run -WorldRoot $world -OwnerUuid $owner -Frontier $frontier -ExpectedLauncherSha256 B48F491344AB99E98719C74DC7F539D78D4CF0F2C144D1EE6F1E4090E3B22EA1 -ExpectedEngineSha256 8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70
    $runDir=Join-Path $runtime "native-cut-runs/$run"
    $receipt=Get-Content (Join-Path $runDir 'result.json') -Raw | ConvertFrom-Json
    if (-not $receipt.pass -or $receipt.termination -ne 'killed_at_exact_adapter_frontier') { throw 'Cut capture failed.' }
    Assert-Closed
    $baseline=Join-Path $runDir "baseline/players/data/$owner.dat"
    $oracleArgs=@('--baseline',$baseline,'--baseline-old',"$baseline`_old",'--owner',$owner,'--world-id',$worldId,'--transaction',$receipt.transaction,'--slot','0','--expect','A')
    & node (Join-Path $PSScriptRoot 'inspect-native-cuts.cjs') @oracleArgs --world $world --mode cut | Set-Content -LiteralPath (Join-Path $evidence "$run-oracle.json")
    if ($LASTEXITCODE -ne 0) { Checkpoint "FALLO $run offlineoracle; conservarintento y detener."; throw 'Cut oracle failed.' }
    Checkpoint "DESPUES $run exacttracePID=$($receipt.pid) killconfirmado; offlineA PASS. Antesreopen OTRA copia completa, originalcut intacto."
    $reopenWorld=Join-Path $runtime "game-p2-cuts/saves/$frontier-$Attempt-reopen"
    Clone-Closed $world $reopenWorld (Join-Path $evidence "$run-reopen-clone.json")
    $cutManifest=Manifest $world
    & (Join-Path $PSScriptRoot 'test-native-open.ps1') -BuildName $BuildName -RuntimeRoot $runtime -RunId "$run-reopen" -WorldRoot $reopenWorld -Expect accept -PlayerName AllomancyQA
    Assert-Closed
    $reopen=Get-Content (Join-Path $runtime "native-open-runs/$run-reopen/result.json") -Raw | ConvertFrom-Json
    if (-not $reopen.pass -or $reopen.exit_code -ne 0) { throw 'Reopen failed.' }
    & node (Join-Path $PSScriptRoot 'inspect-native-cuts.cjs') @oracleArgs --world $reopenWorld --mode reopen | Set-Content -LiteralPath (Join-Path $evidence "$run-reopen-oracle.json")
    if ($LASTEXITCODE -ne 0) { Checkpoint "FALLO $run reopenoracle; conservar copias."; throw 'Reopen oracle failed.' }
    if (($cutManifest|ConvertTo-Json -Compress) -cne ((Manifest $world)|ConvertTo-Json -Compress)) { throw 'Original cut evidence changed.' }
    Checkpoint "DESPUES $run reopen PASS exit0 oracle reconnectA completo; originalcut hashesinalterados. N6 caso acreditado, no N7/fsync/P2global."
}
