[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$RuntimeRoot,
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$BuildName,
 [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9-]+$')][string]$RunId,
 [Parameter(Mandatory)][ValidateSet('conflict','replay-new','capacity','full-ledger','stale-slot','stale-generation','reentrant')][string]$Scenario,
 [ValidateSet('baseline','reconnect','same')][string]$OldExpect,
 [ValidatePattern('^[A-Za-z0-9_]{1,16}$')][string]$PlayerName='AllomancyQA',
 [switch]$AllowLauncherDrift,
 [ValidateRange(30,300)][int]$TimeoutSeconds=120
)
$ErrorActionPreference='Stop'; $PSNativeCommandUseErrorActionPreference=$false
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path.TrimEnd('\')
$source=Join-Path $runtime 'game-p2-owner-econ/saves/credit'
$game=Join-Path $runtime 'game-p2-gates'; $world=Join-Path $game "saves/$RunId"
$run=Join-Path $runtime "native-gate-credit-runs/$RunId"
$engine=Join-Path $runtime "$BuildName-engine/horizon-engine.exe"; $resources=Join-Path $runtime "$BuildName-engine/.local/horizon-client/26.3"
$buildPath=Join-Path $runtime "native-builds/$BuildName/receipt.json"
$control=Join-Path $PSScriptRoot 'horizon-control.cjs'; $fixture=Join-Path $PSScriptRoot 'make-native-gate-fixture.cjs'; $oracle=Join-Path $PSScriptRoot 'inspect-native-gate-state.cjs'
$owner='3efd2c0e-f0de-3e85-b045-dec265b3f79c'; $worldId='026e465a87434711a6951c07588dc0ee'
function Assert-Closed { if (Get-Process horizon-engine,horizon-client,cargo,rustc,gdb -ErrorAction SilentlyContinue) { throw 'Game/build/debug process active; run serially.' } }
function Check-Path([string]$p) { $item=Get-Item -LiteralPath $p -Force; if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point: $p" } }
function Manifest([string]$dir) { $items=@(Get-Item -LiteralPath $dir -Force)+@(Get-ChildItem -LiteralPath $dir -Recurse -Force); if (@($items | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'Reparse point in tree.' }; return @($items | Where-Object { -not $_.PSIsContainer } | Sort-Object FullName | ForEach-Object { [ordered]@{path=[IO.Path]::GetRelativePath($dir,$_.FullName);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash} }) }
function Same($a,$b) { return (($a | ConvertTo-Json -Depth 12 -Compress) -ceq ($b | ConvertTo-Json -Depth 12 -Compress)) }
foreach ($p in @($runtime,(Join-Path $runtime 'game-p2-owner-econ'),(Join-Path $runtime 'game-p2-owner-econ/saves'),$source,$game,(Join-Path $game 'saves'))) { Check-Path $p }
foreach ($f in @($engine,(Join-Path $resources 'horizon-resources.json'),$buildPath,$control,$fixture,$oracle,(Join-Path $source 'allomancy/world-identity.json'),(Join-Path $source "players/data/$owner.dat"))) { if (-not (Test-Path -LiteralPath $f -PathType Leaf)) { throw "Missing input: $f" } }
if ((Test-Path -LiteralPath $world) -or (Test-Path -LiteralPath $run)) { throw 'RunId or world exists.' }
$build=Get-Content -LiteralPath $buildPath -Raw | ConvertFrom-Json
if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or $build.native_test -ne $true -or @($build.inputs).Count -eq 0) { throw 'Successful native-test build receipt required.' }
$inputs=@(foreach ($i in $build.inputs) { if (-not $i.path -or -not $i.sha256 -or -not (Test-Path -LiteralPath $i.path -PathType Leaf)) { throw 'Missing build input.' }; $current=(Get-FileHash -LiteralPath $i.path -Algorithm SHA256).Hash; $exception=$AllowLauncherDrift -and $i.path -ieq $build.launcher; if ($current -ne $i.sha256 -and -not $exception) { throw "Build input drift: $($i.path)" }; [ordered]@{path=$i.path;recorded_sha256=$i.sha256;current_sha256=$current;launcher_exception_used=($current -ne $i.sha256 -and $exception)} })
Assert-Closed
New-Item -ItemType Directory -Path $run | Out-Null
$resultPath=Join-Path $run 'result.json'; $stdout=Join-Path $run 'stdout.log'; $stderr=Join-Path $run 'stderr.log'; $endpoint=Join-Path $run 'control.json'; $baseline=Join-Path $run 'baseline'; $backup=Join-Path $run 'full-backup'
$result=[ordered]@{status='preparing';pass=$false;native_pass=$false;oracle='pending';scenario=$Scenario;source=$source;world=$world;backup=$backup;baseline=$baseline;engine=$engine;engine_sha256=(Get-FileHash -LiteralPath $engine).Hash;build_receipt=$buildPath;build_receipt_sha256=(Get-FileHash -LiteralPath $buildPath).Hash;inputs=$inputs;harness_sha256=(Get-FileHash -LiteralPath $PSCommandPath).Hash;control_sha256=(Get-FileHash -LiteralPath $control).Hash;fixture_sha256=(Get-FileHash -LiteralPath $fixture).Hash;oracle_sha256=(Get-FileHash -LiteralPath $oracle).Hash;pid=$null;exit_code=$null;termination=$null;error=$null;commands=@()}
function Save-Result { $tmp="$resultPath.tmp"; [IO.File]::WriteAllText($tmp,($result|ConvertTo-Json -Depth 20),[Text.UTF8Encoding]::new($false)); if (Test-Path -LiteralPath $resultPath) { [IO.File]::Replace($tmp,$resultPath,"$resultPath.bak") } else { [IO.File]::Move($tmp,$resultPath) } }
function Logs { $text=''; foreach ($p in @($stdout,$stderr)) { if (Test-Path -LiteralPath $p) { try { $f=[IO.File]::Open($p,'Open','Read',([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)); try { $r=[IO.StreamReader]::new($f); try { $text+=$r.ReadToEnd()+"`n" } finally { $r.Dispose() } } finally { $f.Dispose() } } catch [IO.IOException] {} } }; return $text }
function Send-Quit { $cmd=Join-Path $run 'quit.txt'; $out=Join-Path $run 'quit-reply.json'; [IO.File]::WriteAllText($cmd,'quit',[Text.UTF8Encoding]::new($false)); $lines=@(& node $control --socket $endpoint --command-file $cmd); $code=$LASTEXITCODE; [IO.File]::WriteAllText($out,($lines -join "`n"),[Text.UTF8Encoding]::new($false)); $result.commands+=@{command='quit';reply=$out;exit=$code}; Save-Result; if ($code -ne 0) { throw 'quit control error' }; $reply=($lines -join "`n") | ConvertFrom-Json; if ($null -eq $reply -or $reply -is [array] -or $reply.ok -ne $true -or $reply.PSObject.Properties['error']) { throw 'quit response schema invalid' } }
$process=$null; Save-Result
try {
 $result.source_before=@(Manifest $source); Save-Result
 Copy-Item -LiteralPath $source -Destination $world -Recurse -Force
 Copy-Item -LiteralPath $source -Destination $backup -Recurse -Force
 $result.source_after=@(Manifest $source); $result.clone_before=@(Manifest $world); $result.backup_files=@(Manifest $backup)
 if (-not (Same $result.source_before $result.source_after) -or -not (Same $result.source_before $result.clone_before) -or -not (Same $result.source_before $result.backup_files)) { throw 'Source/backup/clone mismatch.' }; Save-Result
 $kind=switch ($Scenario) { 'capacity' {'capacity-player'} 'full-ledger' {'ledger-full-player'} default {$null} }
 if ($kind) { $fixturePath=Join-Path $run 'fixture.json'; $fixtureOut=@(& node $fixture --world $world --kind $kind 2>&1); $code=$LASTEXITCODE; [IO.File]::WriteAllText($fixturePath,($fixtureOut -join "`n"),[Text.UTF8Encoding]::new($false)); $result.fixture=@{kind=$kind;output=$fixturePath;exit=$code}; Save-Result; if ($code -ne 0) { throw 'Fixture failed.' } }
 $identity=Get-Content -LiteralPath (Join-Path $world 'allomancy/world-identity.json') -Raw | ConvertFrom-Json
 if ($identity.world_id -cne $worldId) { throw 'Unexpected world id.' }
 Copy-Item -LiteralPath $world -Destination $baseline -Recurse -Force
 $result.baseline_files=@(Manifest $baseline); $result.prelaunch_files=@(Manifest $world)
 if (-not (Same $result.baseline_files $result.prelaunch_files)) { throw 'Fixture baseline mismatch.' }
 $env:HORIZON_HOME=Join-Path $runtime "homes/$BuildName"; $env:HORIZON_LAUNCHER_HOME=Join-Path $runtime 'launcher'; $env:XDG_DATA_HOME=Join-Path $runtime 'data'; $env:XDG_CONFIG_HOME=Join-Path $runtime 'config'; $env:HORIZON_LAUNCHER_TRASH=Join-Path $runtime 'trash'; $env:HORIZON_LAUNCHER_CLIPBOARD=Join-Path $runtime 'clipboard'; $env:HORIZON_LAUNCHER_DESKTOP=Join-Path $runtime 'desktop'; $env:HORIZON_LAUNCHER_NO_SELF_UPDATE='1'; $env:HORIZON_NO_AUDIO='1'
 $env:ALLOMANCY_NATIVE_TEST_RUNTIME=$runtime; $env:ALLOMANCY_NATIVE_TEST_ROOT=$world; $env:ALLOMANCY_NATIVE_TEST_WORLD_ID=$worldId; $env:ALLOMANCY_NATIVE_TEST_SCENARIO=$Scenario
 $argsList=@('--world',$RunId,'--game-dir',$game,'--resources',$resources,'--offscreen','--no-pause','--test-terrain','--creative','--render-distance','2','--name',$PlayerName,'--control-socket',$endpoint)
 $result.arguments=$argsList; $result.status='starting'; Save-Result; Assert-Closed
 $process=Start-Process -FilePath $engine -ArgumentList @($argsList|ForEach-Object { '"'+$_+'"' }) -WorkingDirectory $game -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
 $result.pid=$process.Id; $result.process_start=$process.StartTime.ToUniversalTime().ToString('o'); $result.status='running'; Save-Result
 $timer=[Diagnostics.Stopwatch]::StartNew(); $quit=$false
 while ($timer.Elapsed.TotalSeconds -lt $TimeoutSeconds) {
  $process.Refresh(); $log=Logs
  if ($log -match 'ALLOMANCY_NATIVE_TEST .*result=FAIL|ALLOMANCY_SAVE event=(fault|world_fault|quarantine_confirmed|quarantine_unconfirmed|rollback_unverified)') { throw 'Native failure/fault/quarantine observed.' }
  if (-not $quit -and $log -match "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=PASS durable=false" -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint -PathType Leaf)) { Send-Quit; $quit=$true }
  if ($process.HasExited) { break }; Start-Sleep -Milliseconds 250
 }
 $process.Refresh(); if (-not $process.HasExited) { throw 'Timeout waiting for native result/exit (fixture may lack live item or ledger).' }; $process.WaitForExit(); $result.exit_code=$process.ExitCode; $result.termination='spontaneous_after_quit'
 $log=Logs
 if (-not $quit -or $result.exit_code -ne 0 -or $log -notmatch "ALLOMANCY_NATIVE_TEST scenario=$Scenario result=PASS durable=false" -or $log -notmatch 'ALLOMANCY_NATIVE_TEST event=world_lifecycle phase=WorldCloseFinal' -or $log -match 'ALLOMANCY_NATIVE_TEST .*result=FAIL|ALLOMANCY_SAVE event=(fault|world_fault|quarantine_confirmed|quarantine_unconfirmed|rollback_unverified)') { throw 'Native completion criteria not met.' }
 if ($Scenario -in @('conflict','capacity','full-ledger','stale-slot','stale-generation') -and $log -notmatch "ALLOMANCY_NATIVE_TEST event=rejected scenario=$Scenario reason=.* unchanged=true") { throw 'Missing rejection trace.' }
 if ($Scenario -eq 'replay-new' -and $log -notmatch 'event=replay_unchanged scenario=replay-new unchanged=true') { throw 'Missing replay trace.' }
 if ($Scenario -eq 'reentrant' -and $log -notmatch 'event=reentrant_rejected inner=Busy outer=Busy induced_adapter_recursion=true unchanged=true') { throw 'Missing reentry trace.' }
 $result.native_pass=$true; $result.after_files=@(Manifest $world); $result.baseline_after=@(Manifest $baseline)
 if (-not (Same $result.baseline_files $result.baseline_after)) { throw 'Baseline changed.' }
 if ($Scenario -in @('conflict','capacity','full-ledger','stale-slot','stale-generation') -and $OldExpect) {
  $out=Join-Path $run 'oracle.json'; $err=Join-Path $run 'oracle.stderr'; $baseDat=Join-Path $baseline "players/data/$owner.dat"
  $oracleOut=@(& node $oracle --baseline $baseDat --world $world --mode reconnect --owner $owner --world-id $worldId --old-expect $OldExpect 2>&1); $code=$LASTEXITCODE; [IO.File]::WriteAllText($out,($oracleOut -join "`n"),[Text.UTF8Encoding]::new($false)); $result.oracle=@{output=$out;exit=$code;mode='reconnect';old_expect=$OldExpect}; Save-Result
  if ($code -ne 0) { throw 'Offline rejection oracle failed.' }; $o=($oracleOut -join "`n") | ConvertFrom-Json; if ($o.oracle -ne 'PASS' -or $o.mode -ne 'reconnect') { throw 'Oracle schema invalid.' }
  $result.pass=$true; $result.status='passed'
 } else { $result.status='native_pass_oracle_pending' }
} catch { $result.error=$_.Exception.Message; $result.status='failed'; $result.pass=$false } finally {
 if ($process) { $process.Refresh(); if (-not $process.HasExited) { $process.Kill(); $process.WaitForExit(); $result.termination='killed_after_failure' }; $process.Refresh(); $result.exit_code=$process.ExitCode }
 if (Test-Path -LiteralPath $world) { $result.after_files=@(Manifest $world) }; Save-Result
}
Write-Output "result=$resultPath status=$($result.status) pass=$($result.pass)"
if ($result.status -eq 'failed') { throw $result.error }
