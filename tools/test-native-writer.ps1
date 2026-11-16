[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$RuntimeRoot,
 [Parameter(Mandatory)][ValidatePattern('\A[a-zA-Z0-9-]+\z')][string]$RunId,
 [Parameter(Mandatory)][string]$SourceWorld,
 [Parameter(Mandatory)][string]$WorldRoot,
 [Parameter(Mandatory)][ValidatePattern('\A[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}\z')][string]$OwnerUuid,
 [Parameter(Mandatory)][ValidateSet('write-phase','after-primary-to-old','after-new-to-primary')][string]$Mode,
 [ValidatePattern('\A[A-Za-z0-9_]{1,16}\z')][string]$PlayerName='AllomancyQA'
)
$ErrorActionPreference='Stop'
$PSNativeCommandUseErrorActionPreference=$false
$pin='8C593D4DFBFDDF5054C8FC94DE860A27B945542584488BF42E5412E5766CFA70'
$root=Split-Path -Parent $PSScriptRoot
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path
$runDir=Join-Path $runtime "native-writer-runs/$RunId"
if (Test-Path -LiteralPath $runDir) { throw "RunId already exists; preserve it: $runDir" }
New-Item -ItemType Directory -Path $runDir | Out-Null
$resultPath=Join-Path $runDir 'result.json'
$engine=Join-Path $runtime 'p2-cuts1-engine/horizon-engine.exe'
$resources=Join-Path $runtime 'p2-cuts1-engine/.local/horizon-client/26.3'
$buildPath=Join-Path $runtime 'native-builds/p2-cuts1/receipt.json'
$bundle=Join-Path $runtime 'native-builds/p2-cuts1/bundle'
$gdb=Join-Path $runtime 'launcher/rust/1.99.1-x86_64-pc-windows-gnu/mingw64/bin/gdb.exe'
$debugHelper=Join-Path $PSScriptRoot 'native-writer-debug.gdb'
$control=Join-Path $PSScriptRoot 'horizon-control.cjs'
$oracle=Join-Path $PSScriptRoot 'inspect-native-writer.cjs'
$stdout=Join-Path $runDir 'stdout.log'; $stderr=Join-Path $runDir 'stderr.log'
$debugOut=Join-Path $runDir 'gdb.stdout.log'; $debugErr=Join-Path $runDir 'gdb.stderr.log'
$endpoint=Join-Path $runDir 'control.json'; $boundary=Join-Path $runDir 'boundary.json'
$result=[ordered]@{
 schema=1; status='preflight'; pass=$false; run_id=$RunId; mode=$Mode; error=$null; cleanup_errors=@()
 engine=$engine; engine_sha256=$null; build_receipt=$buildPath; inputs=@(); source_world=$SourceWorld; world=$WorldRoot
 owner=$OwnerUuid; pid=$null; process_start_ticks=$null; process_path=$null; exit_code=$null; termination=$null
 baseline=@(); after=@(); events=@(); commands=@(); processes=@(); reopen='pending root orchestration on another complete copy'
 source_scope='Pinned cuts1 engine and frozen staging verified. Current adapter lib/save/native_test may differ; current Rust is NOT claimed compiled.'
 limitation='Complete NtWriteFile success before fs::write close, or successful rename return. No partial kernel write, fsync, power-loss or full P2 claim.'
}
$game=$null; $debug=$null; $saveProcess=$null; $debugStreams=@(); $debugCopies=@()
$world=$null; $primary=$null; $gameOwned=$null; $debugOwned=$null; $saveOwned=$null
function Save-Result([string]$phase) {
 if ($phase) { $result.status=$phase; $result.events+=@([ordered]@{utc=[DateTime]::UtcNow.ToString('o');phase=$phase}) }
 $tmp="$resultPath.tmp"
 [IO.File]::WriteAllText($tmp,($result | ConvertTo-Json -Depth 30),[Text.UTF8Encoding]::new($false))
 if (Test-Path -LiteralPath $resultPath) { [IO.File]::Replace($tmp,$resultPath,"$resultPath.bak") }
 else { [IO.File]::Move($tmp,$resultPath) }
}
function Hash([string]$file) { (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash }
function Assert-Closed {
 if (Get-Process | Where-Object { $_.ProcessName -match 'horizon-engine|horizon-client|cargo|rustc|^gdb$' }) { throw 'Another game/build/debugger is active.' }
}
function Assert-Within([string]$path,[string]$parent) {
 if (-not [IO.Path]::GetFullPath($path).StartsWith($parent.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) { throw "Path outside allowed parent: $path" }
 $cursor=[IO.Path]::GetFullPath($path)
 while ($cursor.TrimEnd('\') -ine $parent.TrimEnd('\')) {
  if ((Test-Path -LiteralPath $cursor) -and ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "Reparse path: $cursor" }
  $cursor=Split-Path -Parent $cursor
  if (-not $cursor) { throw 'Cannot resolve path ancestry.' }
 }
 if ((Get-Item -LiteralPath $parent -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Reparse parent.' }
}
function Manifest([string]$directory) {
 $items=@(Get-Item -LiteralPath $directory -Force)+@(Get-ChildItem -LiteralPath $directory -Recurse -Force)
 if (@($items | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count) { throw 'Linked fixture rejected.' }
 @($items | Where-Object { -not $_.PSIsContainer } | Sort-Object FullName | ForEach-Object {
  [ordered]@{path=[IO.Path]::GetRelativePath($directory,$_.FullName);sha256=(Hash $_.FullName)}
 })
}
function Read-Log([string]$file) {
 if (-not (Test-Path -LiteralPath $file)) { return '' }
 $stream=$null; $reader=$null
 try {
  $stream=[IO.File]::Open($file,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
  $reader=[IO.StreamReader]::new($stream); return $reader.ReadToEnd()
 } finally { if ($reader) {$reader.Dispose()} elseif($stream) {$stream.Dispose()} }
}
function Game-Log { (Read-Log $stdout)+[Environment]::NewLine+(Read-Log $stderr) }
function Owned($process,[string]$executable) {
 $expected=[IO.Path]::GetFullPath($executable)
 $ownedPid=$process.Id; $startedTicks=$null; $lastIssue='module path not yet available'
 $deadline=[Diagnostics.Stopwatch]::StartNew()
 while ($deadline.Elapsed.TotalSeconds -lt 5) {
  $live=$null
  try {
   $process.Refresh()
   if ($process.HasExited) { throw "Owned process exited during launch: $expected (PID $ownedPid)" }
   $observedTicks=$process.StartTime.ToUniversalTime().Ticks
   if ($null -eq $startedTicks) { $startedTicks=$observedTicks }
   if ($process.Id -ne $ownedPid -or $observedTicks -ne $startedTicks) { throw 'Owned launch PID/starttime changed.' }
   $live=[Diagnostics.Process]::GetProcessById($ownedPid)
   if ($live.HasExited -or $live.StartTime.ToUniversalTime().Ticks -ne $startedTicks) { throw 'Owned launch PID/starttime changed.' }
   $modulePath=$live.MainModule.FileName
   if (-not [string]::IsNullOrWhiteSpace($modulePath)) {
    $actual=[IO.Path]::GetFullPath($modulePath)
    if ($actual -ine $expected) { throw "Owned process executable mismatch: $actual" }
    return [ordered]@{pid=$ownedPid;start_ticks=$startedTicks;path=$actual}
   }
   $lastIssue='MainModule.FileName empty'
  } catch {
   $lastIssue=$_.Exception.Message
   if ($lastIssue -match 'Owned process exited|Owned launch PID/starttime changed|Owned process executable mismatch') { throw }
  } finally { if ($live) { $live.Dispose() } }
  Start-Sleep -Milliseconds 50
 }
 throw "Cannot verify owned launch within 5 seconds: PID $ownedPid expected=$expected last=$lastIssue; refusing ownership."
}
function Stop-Owned($process,$ownership) {
 if (-not $process) { return }
 $process.Refresh()
 if ($process.HasExited) { return }
 if (-not $ownership) { throw 'Missing ownership receipt; refusing kill.' }
 $live=[Diagnostics.Process]::GetProcessById($ownership.pid)
 try {
  if ($live.StartTime.ToUniversalTime().Ticks -ne $ownership.start_ticks -or [IO.Path]::GetFullPath($live.MainModule.FileName) -ine $ownership.path -or $process.Id -ne $ownership.pid) { throw 'PID/starttime/path ownership mismatch; refusing kill.' }
  $process.Kill()
  if (-not $process.WaitForExit(15000)) { throw 'Owned process did not exit after kill.' }
  $process.Refresh()
 } finally { $live.Dispose() }
}
function Snapshots([string]$name) {
 $dest=Join-Path $runDir "$name/players/data"
 if (Test-Path -LiteralPath $dest) { throw 'Snapshot destination already exists.' }
 New-Item -ItemType Directory -Path $dest -Force | Out-Null
 foreach ($file in @($primary,"$($primary)_old","$($primary)_new")) {
  $present=Test-Path -LiteralPath $file -PathType Leaf
  $entry=[ordered]@{source=$file;present=$present;copy=$null;sha256=$null}
  if ($present) {
   $entry.copy=Join-Path $dest (Split-Path -Leaf $file)
   Copy-Item -LiteralPath $file -Destination $entry.copy
   $entry.sha256=Hash $entry.copy
   if ((Hash $file) -cne $entry.sha256) { throw 'Snapshot changed while copying.' }
  }
  $entry
 }
}
function Assert-Baseline-Unchanged {
 foreach ($entry in $result.baseline) {
  if ((Test-Path -LiteralPath $entry.source -PathType Leaf) -ne $entry.present) { throw 'Save files changed before debugger trigger.' }
  if ($entry.present -and (Hash $entry.source) -cne $entry.sha256) { throw 'Baseline already overwritten before armed save.' }
 }
}
function Send-Control([string]$name,[string]$command) {
 $file=Join-Path $runDir "$name.txt"
 [IO.File]::WriteAllText($file,$command,[Text.UTF8Encoding]::new($false))
 Save-Result "before_control_$name"
 $reply=& node $control --socket $endpoint --command-file $file
 $code=$LASTEXITCODE; $text=$reply -join [Environment]::NewLine
 [IO.File]::WriteAllText((Join-Path $runDir "$name.reply.json"),$text,[Text.UTF8Encoding]::new($false))
 $result.commands+=@([ordered]@{name=$name;command=$command;exit_code=$code;reply=$text})
 Save-Result "after_control_$name"
 $parsed=$text | ConvertFrom-Json
 if ($code -ne 0 -or -not $parsed -or $parsed.PSObject.Properties['error'] -or $parsed.ok -eq $false) { throw "Control failed: $name" }
}
function Inspect([string]$phase,[string]$location) {
 $baseline=Join-Path $runDir "baseline/players/data/$OwnerUuid.dat"
 $args=@('--baseline',$baseline,'--world',$location,'--owner',$OwnerUuid,'--world-id',$result.world_id,'--mode',$phase)
 if (Test-Path -LiteralPath "$($baseline)_old") { $args+=@('--baseline-old',"$($baseline)_old") }
 $text=& node $oracle @args 2>&1; $code=$LASTEXITCODE
 [IO.File]::WriteAllText((Join-Path $runDir "$phase-oracle.json"),($text -join [Environment]::NewLine),[Text.UTF8Encoding]::new($false))
 if ($code -ne 0) { throw "Offline oracle failed: $phase" }
 return (($text -join [Environment]::NewLine) | ConvertFrom-Json)
}
Save-Result 'preflight'
try {
 Assert-Closed
 $node=(Get-Command node -CommandType Application | Select-Object -First 1).Source
 foreach ($file in @($engine,$gdb,$debugHelper,$control,$oracle,$buildPath,(Join-Path $resources 'horizon-resources.json'))) {
  if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "Missing input: $file" }
 }
 $result.engine_sha256=Hash $engine
 if ($result.engine_sha256 -cne $pin) { throw 'N7 offsets require exact cuts1 engine.' }
 $build=Get-Content -LiteralPath $buildPath -Raw | ConvertFrom-Json
 $result['build_receipt_sha256']=Hash $buildPath
 $result['helper_hashes']=@(@($PSCommandPath,$debugHelper,$control,$oracle,$gdb) | ForEach-Object { [ordered]@{path=$_;sha256=(Hash $_)} })
 if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test -or -not $build.inputs.Count) { throw 'Invalid cuts1 build receipt.' }
 if ((Hash $build.staged_manifest.path) -cne $build.staged_manifest.sha256) { throw 'Staged manifest drift.' }
 $allowedRust=@('crates\allomancy-horizon\src\lib.rs','crates\allomancy-horizon\src\save.rs','crates\allomancy-horizon\src\native_test.rs')
 foreach ($input in $build.inputs) {
  $current=Hash $input.path; $frozen=$null; $relative=$null; $scope='unchanged'
  if ($input.path.StartsWith($root.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) {
   $relative=[IO.Path]::GetRelativePath($root,$input.path)
   $staged=if ($relative.StartsWith('crates\')) {Join-Path $bundle $relative} else {Join-Path $bundle (Split-Path -Leaf $relative)}
   $frozen=Hash $staged
   $expected=if ($relative -eq 'crates\allomancy-horizon\Cargo.toml') {$build.staged_manifest.sha256} else {$input.sha256}
   if ($frozen -cne $expected) { throw "Frozen staging drift: $relative" }
  }
  if ($current -cne $input.sha256) {
   if ($relative -and $relative -in $allowedRust) { $scope='expected current Rust WIP; NOT compiled in cuts1' }
   elseif ($input.path -ieq $build.launcher -and $current -ceq 'B48F491344AB99E98719C74DC7F539D78D4CF0F2C144D1EE6F1E4090E3B22EA1') { $scope='known build-tool drift; launcher not executed' }
   else { throw "Unexpected current input drift: $($input.path)" }
  }
  $result.inputs+=@([ordered]@{path=$input.path;recorded_sha256=$input.sha256;staged_sha256=$frozen;current_sha256=$current;scope=$scope})
 }
 $source=(Resolve-Path -LiteralPath $SourceWorld).Path
 $world=[IO.Path]::GetFullPath($WorldRoot); Assert-Within $world $runtime; Assert-Within $source $runtime
 $saves=Split-Path -Parent $world; $gameDir=Split-Path -Parent $saves
 if ((Split-Path -Leaf $saves) -ine 'saves' -or (Split-Path -Leaf $gameDir) -notmatch '^game-p2-.+$' -or (Test-Path -LiteralPath $world)) { throw 'WorldRoot must be a NEW runtime/game-p2-*/saves/name.' }
 if ($runDir.StartsWith($source.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase) -or $world.StartsWith($source.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'Source world cannot contain a destination.' }
 if (-not (Test-Path -LiteralPath (Join-Path $source "players/data/$OwnerUuid.dat") -PathType Leaf) -or -not (Test-Path -LiteralPath (Join-Path $source 'allomancy/world-identity.json') -PathType Leaf)) { throw 'Source must be a complete provisioned owner world.' }
 $result.world=$world; $result.source_world=$source
 $primary=Join-Path $world "players/data/$OwnerUuid.dat"
 $result['source_manifest']=@(Manifest $source)
 Save-Result 'before_complete_backup_and_clone'
 $backup=Join-Path $runDir 'complete-backup'
 Copy-Item -LiteralPath $source -Destination $backup -Recurse
 New-Item -ItemType Directory -Path $saves -Force | Out-Null
 Copy-Item -LiteralPath $backup -Destination $world -Recurse
 $manifestJson=$result.source_manifest | ConvertTo-Json -Depth 8 -Compress
 if ($manifestJson -cne ((Manifest $backup)|ConvertTo-Json -Depth 8 -Compress) -or $manifestJson -cne ((Manifest $world)|ConvertTo-Json -Depth 8 -Compress)) { throw 'Complete clone/backup mismatch.' }
 $identity=Get-Content -LiteralPath (Join-Path $world 'allomancy/world-identity.json') -Raw | ConvertFrom-Json
 if ($identity.world_id -notmatch '^[0-9a-f]{32}$' -or -not (Test-Path -LiteralPath $primary)) { throw 'Missing identity/owner.' }
 $result['world_id']=[string]$identity.world_id; $result['complete_backup']=$backup
 $envMap=@{
  HORIZON_HOME=(Join-Path $runtime 'homes/p2-cuts1'); HORIZON_LAUNCHER_HOME=(Join-Path $runtime 'launcher')
  XDG_DATA_HOME=(Join-Path $runtime 'data'); XDG_CONFIG_HOME=(Join-Path $runtime 'config')
  HORIZON_LAUNCHER_TRASH=(Join-Path $runtime 'trash'); HORIZON_LAUNCHER_CLIPBOARD=(Join-Path $runtime 'clipboard')
  HORIZON_LAUNCHER_DESKTOP=(Join-Path $runtime 'desktop'); HORIZON_LAUNCHER_NO_SELF_UPDATE='1'; HORIZON_NO_AUDIO='1'
  ALLOMANCY_NATIVE_TEST_RUNTIME=$runtime; ALLOMANCY_NATIVE_TEST_ROOT=$world; ALLOMANCY_NATIVE_TEST_WORLD_ID=$identity.world_id
  ALLOMANCY_NATIVE_TEST_SCENARIO='roundtrip'; ALLOMANCY_NATIVE_TEST_RUN_TOKEN=([Guid]::NewGuid().ToString('N'))
 }
 $options=@('--world',(Split-Path -Leaf $world),'--game-dir',$gameDir,'--resources',$resources,'--offscreen','--no-pause','--test-terrain','--creative','--render-distance','2','--name',$PlayerName,'--control-socket',$endpoint)
 $result['arguments']=$options
 Save-Result 'before_launch'
 $game=Start-Process -FilePath $engine -ArgumentList ($options | ForEach-Object {'"'+$_+'"'}) -Environment $envMap -WorkingDirectory $gameDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
 $gameOwned=Owned $game $engine; $result.processes+=@($gameOwned); $result.pid=$game.Id; $result.process_start_ticks=$gameOwned.start_ticks; $result.process_path=$gameOwned.path
 Save-Result 'waiting_ready'
 $timer=[Diagnostics.Stopwatch]::StartNew()
 while ($true) {
  $game.Refresh(); $log=Game-Log
  if ($game.HasExited -or $log -match 'ALLOMANCY_SAVE event=fault|result=FAIL|phase=WorldOpenFailed|scenario=roundtrip result=PASS') { throw 'Exited/fault/driver ran before baseline.' }
  if ($log -match 'ALLOMANCY_NATIVE_TEST event=enabled scenario=roundtrip' -and $log -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' -and (Test-Path -LiteralPath $endpoint)) { break }
  if ($timer.Elapsed.TotalSeconds -gt 90) { throw 'Timeout before Ready.' }
  Start-Sleep -Milliseconds 100
 }
 Send-Control 'baseline-save' 'save'
 $result.baseline=@(Snapshots 'baseline'); Save-Result 'baseline_copied'
 $a=Inspect 'baseline' (Join-Path $runDir 'baseline'); $result['transaction']=$a.transaction
 Send-Control 'give-three' 'command give @s minecraft:iron_ingot 3'
 $timer.Restart()
 while ($true) {
  $game.Refresh(); $log=Game-Log
  if ($game.HasExited -or $log -match 'ALLOMANCY_SAVE event=fault|result=FAIL|phase=WorldOpenFailed') { throw 'Failed while preparing B.' }
  if ($log -match 'ALLOMANCY_NATIVE_TEST scenario=roundtrip result=PASS') { break }
  if ($timer.Elapsed.TotalSeconds -gt 30) { throw 'Timeout waiting real roundtrip B.' }
  Start-Sleep -Milliseconds 50
 }
 Assert-Baseline-Unchanged
 Save-Result 'before_debugger_attach_B_in_memory'
 $psi=[Diagnostics.ProcessStartInfo]::new($gdb)
 $psi.UseShellExecute=$false; $psi.CreateNoWindow=$true; $psi.WorkingDirectory=$runDir
 $psi.RedirectStandardInput=$true; $psi.RedirectStandardOutput=$true; $psi.RedirectStandardError=$true
 foreach ($arg in @('-nx','-q','-iex','set non-stop off',$engine,'-p',[string]$game.Id)) { $psi.ArgumentList.Add($arg) }
 $debug=[Diagnostics.Process]::Start($psi); $debugOwned=Owned $debug $gdb; $result.processes+=@($debugOwned)
 # Buffer size 1 exposes each async output write before GDB exits.
 $debugStreams=@([IO.FileStream]::new($debugOut,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::ReadWrite,1,[IO.FileOptions]::Asynchronous),[IO.FileStream]::new($debugErr,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::ReadWrite,1,[IO.FileOptions]::Asynchronous))
 $debugCopies=@($debug.StandardOutput.BaseStream.CopyToAsync($debugStreams[0]),$debug.StandardError.BaseStream.CopyToAsync($debugStreams[1]))
 $sourceCommand='source '+$debugHelper.Replace('\','/')
 $armCommand='writer-arm '+$game.Id+' "'+$primary.Replace('\','/')+'" '+$Mode+' "'+$boundary.Replace('\','/')+'"'
 [IO.File]::WriteAllLines((Join-Path $runDir 'gdb-commands.txt'),@($sourceCommand,$armCommand,'continue'),[Text.UTF8Encoding]::new($false))
 $debug.StandardInput.WriteLine($sourceCommand); $debug.StandardInput.Flush()
 $timer.Restart()
 while ($true) {
  $debug.Refresh(); $game.Refresh()
  $startupText=(Read-Log $debugOut)+[Environment]::NewLine+(Read-Log $debugErr)
  if ($debug.HasExited -or $game.HasExited -or $startupText -match 'N7_DEBUG_ERROR|Error in sourced command file|Undefined command:|Cannot change this setting') { throw 'Debugger source failed; inspect gdb stdout/stderr.' }
  if ($startupText -match 'N7_HELPER_READY all_stop=true') { break }
  if ($timer.Elapsed.TotalSeconds -gt 15) { throw 'Timeout loading debugger helper; inspect gdb stdout/stderr.' }
  Start-Sleep -Milliseconds 100
 }
 $debug.StandardInput.WriteLine($armCommand); $debug.StandardInput.Flush()
 $timer.Restart()
 while (-not (Test-Path -LiteralPath "$boundary.armed.json")) {
  $debug.Refresh(); $game.Refresh()
  if ($debug.HasExited -or $game.HasExited -or ((Read-Log $debugOut)+[Environment]::NewLine+(Read-Log $debugErr)) -match 'N7_DEBUG_ERROR|Error in sourced command file|Undefined command:|Cannot change this setting|Traceback') { throw 'Debugger failed before arm.' }
  if ($timer.Elapsed.TotalSeconds -gt 60) { throw 'Timeout waiting debugger arm.' }
  Start-Sleep -Milliseconds 100
 }
 $armed=Get-Content -LiteralPath "$boundary.armed.json" -Raw | ConvertFrom-Json
 if ($armed.pid -ne $game.Id -or $armed.mode -cne $Mode -or [IO.Path]::GetFullPath($armed.owner) -ine $primary -or $armed.engine_sha256 -ine $pin -or [IO.Path]::GetFullPath($armed.engine) -ine $engine) { throw 'Armed receipt mismatch.' }
 Assert-Baseline-Unchanged
 $result['armed']=$armed; Save-Result 'before_continue_and_async_save'
 $debug.StandardInput.WriteLine('continue'); $debug.StandardInput.Flush()
 $saveFile=Join-Path $runDir 'boundary-save.txt'; [IO.File]::WriteAllText($saveFile,'save',[Text.UTF8Encoding]::new($false))
 $saveProcess=Start-Process -FilePath $node -ArgumentList @(('"'+$control+'"'),'--socket',('"'+$endpoint+'"'),'--command-file',('"'+$saveFile+'"')) -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $runDir 'boundary-save.stdout.log') -RedirectStandardError (Join-Path $runDir 'boundary-save.stderr.log')
 $saveOwned=Owned $saveProcess $node; $result.processes+=@($saveOwned); Save-Result 'waiting_boundary'
 $timer.Restart()
 while ($true) {
  $debug.Refresh(); $game.Refresh()
  $debugText=(Read-Log $debugOut)+[Environment]::NewLine+(Read-Log $debugErr)
  if ($game.HasExited -or $debug.HasExited -or $debugText -match 'N7_DEBUG_ERROR') { throw 'Debuggee/debugger failed before held boundary.' }
  if ((Test-Path -LiteralPath $boundary) -and $debugText -match 'N7_BOUNDARY_HELD ') { break }
  if ($timer.Elapsed.TotalSeconds -gt 30) { throw 'Timeout waiting all-stop boundary receipt and log.' }
  Start-Sleep -Milliseconds 50
 }
 $held=Get-Content -LiteralPath $boundary -Raw | ConvertFrom-Json
 $expectedBoundary=if ($Mode -eq 'write-phase') {'ntwrite-success-before-fs-write-close'} else {$Mode}
 $lines=@($debugText -split "\r?\n" | Where-Object {$_ -match '^(?:\(gdb\)\s*)*N7_BOUNDARY_HELD '})
 if ($lines.Count -ne 1) { throw 'Ambiguous boundary log.' }
 $logged=($lines[0] -replace '^(?:\(gdb\)\s*)*N7_BOUNDARY_HELD ','') | ConvertFrom-Json
 if (($logged|ConvertTo-Json -Depth 20 -Compress) -cne ($held|ConvertTo-Json -Depth 20 -Compress)) { throw 'Receipt/log mismatch.' }
 if ($held.pid -ne $game.Id -or $held.boundary -cne $expectedBoundary -or $held.engine_sha256 -ine $pin -or
  [IO.Path]::GetFullPath($held.owner_path) -ine $primary -or $held.all_stop -isnot [bool] -or -not $held.all_stop -or
  -not $held.threads.Count -or @($held.threads|Where-Object {$_.stopped -isnot [bool] -or -not $_.stopped -or $_.ptid[0] -ne $game.Id}).Count -or
  $held.ptid[0] -ne $game.Id -or -not $held.thread) { throw 'All-stop ownership/boundary validation failed.' }
 if ([IO.Path]::GetFullPath($held.write.path) -ine "$($primary)_new" -or $held.write.length -le 0 -or $held.write.sha256 -notmatch '^[0-9a-f]{64}$') { throw 'Invalid captured compressed B.' }
 if ($Mode -eq 'write-phase') {
  if ($held.ntstatus -ne 0 -or $held.io_status -ne 0 -or $held.bytes_written -ne $held.write.length) { throw 'Incomplete NtWriteFile result.' }
 } elseif ($held.std_io_result -ne 0) { throw 'Rename unsuccessful.' }
 $result['boundary']=$held; Save-Result 'before_kill_owned_all_stop'
 Stop-Owned $game $gameOwned
 $result.exit_code=$game.ExitCode; $result.termination='killed_at_verified_all_stop_writer_boundary'
 Save-Result 'after_kill_owned_all_stop'
 Stop-Owned $saveProcess $saveOwned
 $debug.Refresh()
 if (-not $debug.HasExited) {
  $debug.StandardInput.WriteLine('set confirm off'); $debug.StandardInput.WriteLine('quit'); $debug.StandardInput.Flush()
  if (-not $debug.WaitForExit(5000)) { Stop-Owned $debug $debugOwned }
 }
 foreach ($task in $debugCopies) { $task.GetAwaiter().GetResult() }
 foreach ($stream in $debugStreams) { $stream.Flush(); $stream.Dispose() }; $debugStreams=@()
 $result.after=@(Snapshots 'after'); Save-Result 'after_snapshots'
 $checked=Inspect $Mode (Join-Path $runDir 'after')
 $bFile=if ($Mode -eq 'after-new-to-primary') {$primary} else {"$($primary)_new"}
 if ((Hash $bFile) -ine $held.write.sha256 -or (Get-Item -LiteralPath $bFile).Length -ne $held.write.length) { throw 'Offline B differs from captured writer buffer.' }
 $result['offline_oracle']=$checked
 if ($manifestJson -cne ((Manifest $source)|ConvertTo-Json -Depth 8 -Compress) -or $manifestJson -cne ((Manifest $backup)|ConvertTo-Json -Depth 8 -Compress)) { throw 'Source/full backup changed.' }
 $result.pass=$true; Save-Result 'writer_boundary_and_offline_oracle_pass'
} catch {
 $result.error=$_.Exception.Message; $result.pass=$false
 Save-Result 'failed_before_cleanup'
 foreach ($pair in @(@($game,$gameOwned),@($saveProcess,$saveOwned),@($debug,$debugOwned))) {
  try { Stop-Owned $pair[0] $pair[1] } catch { $result.cleanup_errors+=@($_.Exception.Message) }
 }
 if ($game) {
  $game.Refresh()
  if ($game.HasExited) {
   $result.exit_code=$game.ExitCode
   if (-not $result.termination) { $result.termination='owned_process_exited_after_failure' }
   if ($primary -and -not (Test-Path -LiteralPath (Join-Path $runDir 'after/players/data'))) {
    try { $result.after=@(Snapshots 'after') } catch { $result.cleanup_errors+=@($_.Exception.Message) }
   }
  } else { $result.termination='left_running_ownership_guard_or_kill_failure' }
 }
 Save-Result 'failed'
 throw
} finally {
 foreach ($task in $debugCopies) { try { $null=$task.Wait(5000) } catch {} }
 foreach ($stream in $debugStreams) { try {$stream.Flush();$stream.Dispose()} catch {} }
}
Write-Output "result=$resultPath"
