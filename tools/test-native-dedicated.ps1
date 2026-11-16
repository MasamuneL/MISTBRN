[CmdletBinding()]
param(
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$BuildName,
 [Parameter(Mandatory)][string]$RuntimeRoot,
 [Parameter(Mandatory)][string]$ClosedWorld,
 [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9-]+$')][string]$RunId,
 [ValidatePattern('^[A-Za-z0-9_]{1,16}$')][string]$PlayerName='AllomancyQA',
 [ValidateRange(1024,65535)][int]$Port=25581,
 [ValidateSet('observe','disconnected-intent','autosave-pending','death-logout-frozen')][string]$Scenario='observe',
 [ValidateRange(30,300)][int]$TimeoutSeconds=180
)
$ErrorActionPreference='Stop'
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path.TrimEnd('\')
$source=(Resolve-Path -LiteralPath $ClosedWorld).Path.TrimEnd('\')
if (-not $source.StartsWith($runtime+'\',[StringComparison]::OrdinalIgnoreCase)) { throw 'ClosedWorld must be inside runtime.' }
# Reject junctions anywhere in the source tree or its path; never copy through them.
$p=$source
while ($p.Length -ge $runtime.Length) {
 if ((Get-Item -LiteralPath $p -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Reparse point: $p" }
 if ($p -ieq $runtime) { break }; $p=Split-Path -Parent $p
}
if (Get-ChildItem -LiteralPath $source -Force -Recurse | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint } | Select-Object -First 1) { throw 'World contains a reparse point.' }
$engine=Join-Path $runtime "$BuildName-engine/horizon-engine.exe"
$resources=Join-Path $runtime "$BuildName-engine/.local/horizon-client/26.3"
$buildPath=Join-Path $runtime "native-builds/$BuildName/receipt.json"
$helper=Join-Path $PSScriptRoot 'horizon-control.cjs'
foreach ($f in @($engine,(Join-Path $resources 'horizon-resources.json'),$buildPath,$helper,(Join-Path $source 'allomancy/world-identity.json'))) { if (-not (Test-Path -LiteralPath $f -PathType Leaf)) { throw "Missing input: $f" } }
$build=Get-Content -LiteralPath $buildPath -Raw | ConvertFrom-Json
if ($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test) { throw 'Native test build receipt required.' }
foreach ($i in $build.inputs) { if (-not (Test-Path -LiteralPath $i.path -PathType Leaf)) { throw "Missing build input: $($i.path)" }; if ((Get-FileHash -LiteralPath $i.path).Hash -ne $i.sha256) { throw "Build input drift: $($i.path)" } }
if (Get-Process horizon-engine,cargo,rustc,gdb -ErrorAction SilentlyContinue) { throw 'Existing engine/build/debug process; run serially.' }
if (Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction SilentlyContinue) { throw 'Port already listening.' }
$run=Join-Path $runtime "native-dedicated-runs/$RunId"
if (Test-Path -LiteralPath $run) { throw 'RunId already exists.' }
New-Item -ItemType Directory -Path $run | Out-Null
$lock=$null; $server=$null; $guest=$null; $resultPath=Join-Path $run 'result.json'
$result=[ordered]@{ status='preparing'; pass=$false; limitation='Preparation and runtime observations only; N0/N5 and offline oracle require review.'; scenario=$Scenario; port=$Port; source=$source; engine=$engine; engine_sha256=(Get-FileHash $engine).Hash; build_receipt_sha256=(Get-FileHash $buildPath).Hash; harness_sha256=(Get-FileHash $PSCommandPath).Hash; helper_sha256=(Get-FileHash $helper).Hash; pids=@{}; commands=@(); error=$null; n5='pending'; offline_oracle='pending' }
function Save-Result { $tmp="$resultPath.tmp"; [IO.File]::WriteAllText($tmp,($result | ConvertTo-Json -Depth 16),[Text.UTF8Encoding]::new($false)); if (Test-Path $resultPath) { [IO.File]::Replace($tmp,$resultPath,"$resultPath.bak") } else { [IO.File]::Move($tmp,$resultPath) } }
function Manifest([string]$root) { @(Get-ChildItem -LiteralPath $root -File -Force -Recurse | Sort-Object FullName | ForEach-Object { [ordered]@{path=[IO.Path]::GetRelativePath($root,$_.FullName); length=$_.Length; sha256=(Get-FileHash -LiteralPath $_.FullName).Hash} }) }
function LogText([string]$path) { if (-not (Test-Path $path)) { return '' }; $f=[IO.File]::Open($path,'Open','Read',([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)); try { $r=[IO.StreamReader]::new($f); try { $r.ReadToEnd() } finally { $r.Dispose() } } finally { $f.Dispose() } }
function Control([string]$endpoint,[string]$command) {
 $n=$result.commands.Count+1; $file=Join-Path $run ("command-{0:D3}.txt" -f $n); $reply=Join-Path $run ("reply-{0:D3}.json" -f $n)
 [IO.File]::WriteAllText($file,$command,[Text.UTF8Encoding]::new($false)); $output=@(& node $helper --socket $endpoint --command-file $file); $code=$LASTEXITCODE
 [IO.File]::WriteAllText($reply,($output -join "`n"),[Text.UTF8Encoding]::new($false)); $result.commands+=@{command=$command; reply=$reply; exit=$code}; Save-Result
 if ($code -ne 0) { throw "Control failed: $command" }; $value=($output -join "`n") | ConvertFrom-Json; if ($value.error -or $value.ok -eq $false) { throw "Control error: $command" }; return $value
}
function WaitFor([scriptblock]$condition,[string]$phase,[bool]$allowGuestExit=$false) { $t=[Diagnostics.Stopwatch]::StartNew(); while ($t.Elapsed.TotalSeconds -lt $TimeoutSeconds) { if ($server -and $server.HasExited) { throw "Server exited during $phase" }; if ($guest -and $guest.HasExited -and -not $allowGuestExit) { throw "Guest exited during $phase" }; if (& $condition) { return }; Start-Sleep -Milliseconds 250 }; throw "Timeout: $phase" }
Save-Result
try {
 $lock=[IO.File]::Open((Join-Path $runtime 'native-dedicated.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
 $backup=Join-Path $run 'backup'; $game=Join-Path $run 'game-p2-dedicated'; $world=Join-Path $game 'saves/world'; $serverDir=Join-Path $run 'server'; $guestDir=Join-Path $run 'guest'
 foreach ($d in @($backup,$world,$serverDir,$guestDir)) { New-Item -ItemType Directory -Path $d -Force | Out-Null }
 $result.before=@(Manifest $source); Save-Result
 Copy-Item -Path (Join-Path $source '*') -Destination $backup -Recurse -Force
 Copy-Item -Path (Join-Path $source '*') -Destination $world -Recurse -Force
 $result.backup=@(Manifest $backup); $result.clone=@(Manifest $world)
 if (($result.before | ConvertTo-Json -Depth 8 -Compress) -cne ($result.backup | ConvertTo-Json -Depth 8 -Compress) -or ($result.before | ConvertTo-Json -Depth 8 -Compress) -cne ($result.clone | ConvertTo-Json -Depth 8 -Compress) -or ($result.before | ConvertTo-Json -Depth 8 -Compress) -cne (@(Manifest $source) | ConvertTo-Json -Depth 8 -Compress)) { throw 'Full source/backup/clone manifest mismatch.' }
 $identity=Get-Content (Join-Path $world 'allomancy/world-identity.json') -Raw | ConvertFrom-Json
 if (-not $identity.world_id) { throw 'Missing world_id.' }
 [IO.File]::WriteAllText((Join-Path $serverDir 'eula.txt'),"eula=true`n",[Text.UTF8Encoding]::new($false))
 [IO.File]::WriteAllText((Join-Path $serverDir 'server.properties'),"server-ip=127.0.0.1`nserver-port=$Port`nonline-mode=false`nwhite-list=false`nenforce-whitelist=false`n",[Text.UTF8Encoding]::new($false))
 $env:HORIZON_HOME=Join-Path $runtime "homes/$BuildName"; $env:HORIZON_LAUNCHER_HOME=Join-Path $runtime 'launcher'; $env:XDG_DATA_HOME=Join-Path $runtime 'data'; $env:XDG_CONFIG_HOME=Join-Path $runtime 'config'; $env:HORIZON_NO_AUDIO='1'; $env:HORIZON_LAUNCHER_NO_SELF_UPDATE='1'
 $env:ALLOMANCY_NATIVE_TEST_RUNTIME=$runtime; $env:ALLOMANCY_NATIVE_TEST_ROOT=$world; $env:ALLOMANCY_NATIVE_TEST_WORLD_ID=[string]$identity.world_id; $env:ALLOMANCY_NATIVE_TEST_SCENARIO=if($Scenario -eq 'death-logout-frozen'){'observe'}else{$Scenario}; $env:ALLOMANCY_NATIVE_TEST_RUN_TOKEN=[Guid]::NewGuid().ToString('N'); $result.run_token=$env:ALLOMANCY_NATIVE_TEST_RUN_TOKEN
 $se=Join-Path $run 'server-control.json'; $ge=Join-Path $run 'guest-control.json'; $so=Join-Path $run 'server.stdout'; $serr=Join-Path $run 'server.stderr'; $go=Join-Path $run 'guest.stdout'; $gerr=Join-Path $run 'guest.stderr'
 $sa=@('--resources',$resources,'--game-dir',$serverDir,'--control-socket',$se,'--dedicated','--nogui','--port',"$Port",'--universe',(Join-Path $game 'saves'),'--world','world')
 $ga=@('--resources',$resources,'--game-dir',$guestDir,'--headless','--render-distance','2','--name',$PlayerName,'--server',"127.0.0.1:$Port",'--control-socket',$ge)
 $result.server_arguments=$sa; $result.guest_arguments=$ga; $result.world=$world; $result.status='starting_server'; Save-Result
 $server=Start-Process -FilePath $engine -ArgumentList @($sa | ForEach-Object { '"'+$_+'"' }) -WorkingDirectory $serverDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $so -RedirectStandardError $serr
 $result.pids.server=@{id=$server.Id; start=$server.StartTime.ToUniversalTime().ToString('o'); path=$engine}; Save-Result
 WaitFor { (Test-Path $se) -and ((LogText $so)+(LogText $serr)) -match 'Done \(' } 'server ready'
 $listeners=@(Get-NetTCPConnection -State Listen -LocalPort $Port -ErrorAction SilentlyContinue)
 if ($listeners.Count -ne 1 -or $listeners[0].LocalAddress -ne '127.0.0.1' -or $listeners[0].OwningProcess -ne $server.Id) { throw 'Listener is not exactly owned IPv4 loopback.' }
 $result.listener=@{address=$listeners[0].LocalAddress; pid=$listeners[0].OwningProcess}; $result.status='starting_guest'; Save-Result
 $guest=Start-Process -FilePath $engine -ArgumentList @($ga | ForEach-Object { '"'+$_+'"' }) -WorkingDirectory $guestDir -WindowStyle Hidden -PassThru -RedirectStandardOutput $go -RedirectStandardError $gerr
 $result.pids.guest=@{id=$guest.Id; start=$guest.StartTime.ToUniversalTime().ToString('o'); path=$engine}; Save-Result
 WaitFor { (Test-Path $ge) -and ((LogText $serr)+(LogText $so)) -match 'ALLOMANCY_SAVE event=ready accepted_in_memory=true' } 'guest native ready'
 if ($Scenario -eq 'autosave-pending') {
  $null=Control $se 'console save-all flush'
  $baseline=Join-Path $run 'live-baseline'; New-Item -ItemType Directory -Path $baseline|Out-Null
  $uuid='3efd2c0e-f0de-3e85-b045-dec265b3f79c'
  foreach($file in @("$uuid.dat","$uuid.dat_old")){Copy-Item -LiteralPath (Join-Path $world "players/data/$file") -Destination $baseline}
  $result.live_baseline=@(Manifest $baseline); Save-Result
  & (Join-Path $PSScriptRoot 'test-native-fifo.ps1') -RuntimeRoot $runtime -BuildName $BuildName -ServerPid $server.Id -ServerStartUtc $server.StartTime.ToUniversalTime() -ServerControl $se -ServerStderr $serr -WorldRoot $world -GuestUuid ($uuid.Replace('-','')) -RunToken $result.run_token -RunDirectory (Join-Path $run 'fifo')
  $result.fifo_receipt=Join-Path $run 'fifo/result.json'; Save-Result
  $null=Control $se 'console save-all flush'
 }
 if ($Scenario -eq 'death-logout-frozen') {
  $null=Control $se 'console tick freeze'
  $result.frozen_state=Control $se 'console tick query'; Save-Result
  $null=Control $se "console kill $PlayerName"
  WaitFor { (LogText $serr) -match 'ALLOMANCY_SAVE event=death_queued' } 'death queued while frozen'
  if ((LogText $serr) -match 'ALLOMANCY_SAVE event=death_applied') { throw 'Death already applied before actual departure.' }
 } if ($Scenario -eq 'disconnected-intent') { WaitFor { ((LogText $serr)+(LogText $so)) -match 'ALLOMANCY_NATIVE_TEST event=disconnect_armed' } 'disconnect armed' }
 $null=Control $ge 'quit'
 $t=[Diagnostics.Stopwatch]::StartNew(); while (-not $guest.HasExited -and $t.Elapsed.TotalSeconds -lt 30) { Start-Sleep -Milliseconds 250; $guest.Refresh() }; if (-not $guest.HasExited) { throw 'Guest quit timeout.' }; $result.guest_exit=$guest.ExitCode; Save-Result; if ($result.guest_exit -ne 0) { throw "Guest nonzero exit: $($result.guest_exit)" }
 WaitFor { ((LogText $serr)+(LogText $so)) -match 'ALLOMANCY_SAVE event=logged_out' } 'server logged_out' $true
 $serverLog=LogText $serr
 if ($Scenario -eq 'death-logout-frozen') {
  $queued=$serverLog.IndexOf('ALLOMANCY_SAVE event=death_queued'); $leaving=$serverLog.IndexOf('ALLOMANCY_SAVE event=logging_out'); $applied=$serverLog.IndexOf('ALLOMANCY_SAVE event=death_applied')
  if($queued -lt 0 -or $leaving -le $queued -or $applied -le $leaving){throw 'Death/logging_out/applied order not proven.'}
  $result.death_order=@{queued=$queued;logging_out=$leaving;applied=$applied}; Save-Result
 }
 if ($Scenario -eq 'disconnected-intent' -and $serverLog -notmatch 'scenario=disconnected-intent result=PASS departed=true reason=Unavailable unchanged=true'){throw 'Real disconnected intent PASS missing.'}
 $null=Control $se 'console stop'
 $t=[Diagnostics.Stopwatch]::StartNew(); while (-not $server.HasExited -and $t.Elapsed.TotalSeconds -lt 30) { Start-Sleep -Milliseconds 250; $server.Refresh() }; if (-not $server.HasExited) { throw 'Server stop timeout.' }; $result.server_exit=$server.ExitCode; Save-Result; if ($result.server_exit -ne 0) { throw "Server nonzero exit: $($result.server_exit)" }
 $result.status='observed_review_required'; $result.limitation='Guest quit and server logged_out observed; review per-file event order, native role, logs and offline oracle. No gate automatically passed.'
} catch { $result.status='failed'; $result.error=$_.Exception.Message } finally {
 foreach ($owned in @($guest,$server)) { if ($owned) { $owned.Refresh(); if (-not $owned.HasExited) { $owned.Kill(); $owned.WaitForExit(); $result["killed_$($owned.Id)"]=$true } } }
 if ($world -and (Test-Path $world)) { $result.after=@(Manifest $world) }
 if ($lock) { $lock.Dispose() }; Save-Result
}
Write-Output "result=$resultPath status=$($result.status) pass=$($result.pass)"
if ($result.status -eq 'failed') { throw $result.error }
