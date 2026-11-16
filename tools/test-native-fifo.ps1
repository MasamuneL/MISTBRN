[CmdletBinding()]
param(
 [Parameter(Mandatory)][string]$RuntimeRoot,
 [Parameter(Mandatory)][ValidatePattern('^[a-z0-9-]+$')][string]$BuildName,
 [Parameter(Mandatory)][int]$ServerPid,
 [Parameter(Mandatory)][datetime]$ServerStartUtc,
 [Parameter(Mandatory)][string]$ServerControl,
 [Parameter(Mandatory)][string]$ServerStderr,
 [Parameter(Mandatory)][string]$WorldRoot,
 [Parameter(Mandatory)][ValidatePattern('^[a-fA-F0-9]{32}$')][string]$GuestUuid,
 [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9_-]{1,64}$')][string]$RunToken,
 [Parameter(Mandatory)][string]$RunDirectory,
 [ValidateRange(10,300)][int]$TimeoutSeconds=90
)
$ErrorActionPreference='Stop'
$runtime=(Resolve-Path -LiteralPath $RuntimeRoot).Path.TrimEnd('\')
$world=(Resolve-Path -LiteralPath $WorldRoot).Path.TrimEnd('\')
$run=[IO.Path]::GetFullPath($RunDirectory)
foreach($path in @($world,$run,$ServerControl,$ServerStderr)) { if(-not [IO.Path]::GetFullPath($path).StartsWith($runtime+'\',[StringComparison]::OrdinalIgnoreCase)){throw 'All target paths must remain inside runtime'} }
if(Test-Path -LiteralPath $run){throw 'FIFO run directory already exists'}
$engine=(Resolve-Path -LiteralPath (Join-Path $runtime "$BuildName-engine/horizon-engine.exe")).Path
$pin=(Get-FileHash -LiteralPath $engine).Hash
$build=Get-Content -Raw -LiteralPath (Join-Path $runtime "native-builds/$BuildName/receipt.json") | ConvertFrom-Json
if($build.status -ne 'succeeded' -or $build.exit_code -ne 0 -or -not $build.native_test){throw 'Succeeded native test build required'}
$server=Get-Process -Id $ServerPid
function Assert-Server {
 $server.Refresh()
 if($server.HasExited -or $server.Path -ine $engine -or $server.StartTime.ToUniversalTime().Ticks -ne $ServerStartUtc.ToUniversalTime().Ticks){throw 'Exact owned server PID/start/executable mismatch'}
}
Assert-Server
$ownedCommand=(Get-CimInstance Win32_Process -Filter "ProcessId=$ServerPid").CommandLine
if($ownedCommand -notmatch '(?:^|[\s"\x27])--dedicated(?:[\s"\x27]|$)' -or $ownedCommand -notmatch '(?:^|[\s"\x27])--nogui(?:[\s"\x27]|$)'){throw 'Exact process is not a dedicated nogui server'}
$listeners=@(Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue | Where-Object {$_.OwningProcess -eq $ServerPid})
if(-not $listeners.Count -or @($listeners|Where-Object {$_.LocalAddress -ne '127.0.0.1'}).Count){throw 'Owned server listeners must all be IPv4 loopback'}
$identity=Get-Content -Raw -LiteralPath (Join-Path $world 'allomancy/world-identity.json') | ConvertFrom-Json
if(-not $identity.world_id){throw 'Missing world identity'}
$uuid=[Guid]::ParseExact($GuestUuid,'N').ToString('D')
$dat=Join-Path $world "players/data/$uuid.dat"
$old=$dat+'_old'
foreach($file in @($dat,$old,$ServerControl,$ServerStderr)){if(-not(Test-Path -LiteralPath $file -PathType Leaf)){throw "Missing prepared input $file"}}
$arm=Join-Path $world "allomancy/native-autosave-arm-$RunToken"
if(Test-Path -LiteralPath $arm){throw 'Stale arm file'}
$tools=Join-Path $runtime 'launcher/rust/1.99.1-x86_64-pc-windows-gnu/mingw64/bin'
$gdbExe=Join-Path $tools 'gdb.exe'; $objdump=Join-Path $tools 'objdump.exe'
$gdbScript=Join-Path $PSScriptRoot 'native-fifo-debug.gdb'; $controlHelper=Join-Path $PSScriptRoot 'horizon-control.cjs'
$symbols=@(& $objdump -t $engine | Where-Object {$_ -match '\s(_[A-Za-z0-9_.$]*12write_dat_as(?:\.llvm\.[0-9]+)?)$'} | ForEach-Object {$Matches[1]} | Select-Object -Unique)
if($LASTEXITCODE -ne 0 -or $symbols.Count -ne 1){throw 'Require exactly one write_dat_as function symbol from this build'}
New-Item -ItemType Directory -Path $run | Out-Null
$receipt=Join-Path $run 'writer-held.json'; $resultPath=Join-Path $run 'result.json'
$result=[ordered]@{schema=1;status='preparing';pass=$false;scope='dedicated guest FIFO runtime only';offline_oracle='pending';pid=$ServerPid;start_utc=$ServerStartUtc.ToUniversalTime().ToString('o');engine=$engine;engine_sha256=$pin;world=$world;guest_uuid=$GuestUuid.ToLowerInvariant();run_token=$RunToken;symbol=$symbols[0];baseline=@();commands=@();error=$null;cleanup=@();harness_sha256=(Get-FileHash $PSCommandPath).Hash;debugger_helper_sha256=(Get-FileHash $gdbScript).Hash}
function Save-Result { $temp=$resultPath+'.tmp'; [IO.File]::WriteAllText($temp,($result|ConvertTo-Json -Depth 16),[Text.UTF8Encoding]::new($false)); [IO.File]::Move($temp,$resultPath,$true) }
function Read-Shared([string]$path) { $f=[IO.File]::Open($path,'Open','Read',([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)); try{$r=[IO.StreamReader]::new($f); try{$r.ReadToEnd()}finally{$r.Dispose()}}finally{$f.Dispose()} }
$initialLog=Read-Shared $ServerStderr
if($initialLog -notmatch 'ALLOMANCY_NATIVE_TEST event=enabled scenario=autosave-pending'){throw 'Root must launch this exact server with autosave-pending scenario and RunToken'}
$logOffset=$initialLog.Length
function New-Log { $all=Read-Shared $ServerStderr; if($all.Length -lt $logOffset){throw 'Server log truncated'}; $all.Substring($logOffset) }
function Control([string]$command) {
 $n=$result.commands.Count+1; $file=Join-Path $run "command-$n.txt"; $reply=Join-Path $run "reply-$n.json"
 [IO.File]::WriteAllText($file,$command,[Text.UTF8Encoding]::new($false))
 $out=@(& node $controlHelper --socket $ServerControl --command-file $file); $code=$LASTEXITCODE
 [IO.File]::WriteAllText($reply,($out -join "`n"),[Text.UTF8Encoding]::new($false)); $result.commands+=@{command=$command;exit=$code;reply=$reply}; Save-Result
 if($code -ne 0){throw "Control failed $command"}; $value=($out -join "`n")|ConvertFrom-Json
 if($value.ok -eq $false -or $value.error -or ($out -join ' ') -match 'Unknown or incomplete command'){throw "Control rejected $command"}
 return $value
}
# Native handles only belong to the exact root-owned process and its verified writer.
if(-not ('N5FifoNative' -as [type])) { Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices; using System.Text;
public static class N5FifoNative {
 [DllImport("kernel32.dll",SetLastError=true)] public static extern IntPtr OpenThread(uint access,bool inherit,uint tid);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern uint GetProcessIdOfThread(IntPtr thread);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern uint SuspendThread(IntPtr thread);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern uint ResumeThread(IntPtr thread);
 [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr handle);
 [DllImport("kernel32.dll")] public static extern int GetThreadDescription(IntPtr thread,out IntPtr description);
 [DllImport("kernel32.dll")] public static extern IntPtr LocalFree(IntPtr memory);
 [DllImport("kernel32.dll",SetLastError=true)] public static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll")] public static extern IntPtr GetCurrentProcess();
 [DllImport("kernel32.dll",SetLastError=true)] public static extern bool DuplicateHandle(IntPtr source,IntPtr sourceHandle,IntPtr target,out IntPtr duplicate,uint access,bool inherit,uint options);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] public static extern uint GetFinalPathNameByHandle(IntPtr handle,StringBuilder path,uint length,uint flags);
}
"@ }
$debugger=$null; $readerTask=$null; $stderrTask=$null; $debugLines=[Collections.Generic.List[string]]::new(); $miToken=0; $attached=$false; $stopped=$false; $debugError=$false; $cleaning=$false; $threadHandle=[IntPtr]::Zero; $suspended=$false
function Pump-Gdb {
 while($readerTask -and $readerTask.IsCompleted){$line=$readerTask.GetAwaiter().GetResult(); if($null -eq $line){$script:readerTask=$null;break}; $debugLines.Add($line); if($line -match '^([0-9]+\^running|\*running,)'){$script:stopped=$false}; if($line -like '*N5_DEBUG_ERROR*'){$script:debugError=$true;$script:stopped=$true}; if($line -like '*N5_WRITER_HELD*' -or $line -like '*stopped,*'){$script:stopped=$true}; $script:readerTask=$debugger.StandardOutput.ReadLineAsync()}
}
function Wait-For([scriptblock]$condition,[string]$phase,[int]$seconds=$TimeoutSeconds) { $watch=[Diagnostics.Stopwatch]::StartNew(); while($watch.Elapsed.TotalSeconds -lt $seconds){Assert-Server; Pump-Gdb; if($debugError -and -not $cleaning){throw 'GDB decoder stopped with N5_DEBUG_ERROR'}; if(& $condition){return}; if($debugger -and $debugger.HasExited -and $attached){throw 'Debugger exited while attached'}; Start-Sleep -Milliseconds 20}; throw "Timeout $phase" }
function Send-Mi([string]$command,[switch]$NoWait){ $script:miToken++; $token=$miToken; $debugger.StandardInput.WriteLine("$token$command"); $debugger.StandardInput.Flush(); if(-not $NoWait){Wait-For { @($debugLines | Where-Object {$_ -match "^$token\^(done|running|connected|error)"}).Count -gt 0 } "GDB command $token"; if($debugLines | Where-Object {$_ -match "^$token\^error"}){throw "GDB command rejected $command"}} }
function Console-Mi([string]$command){Send-Mi ('-interpreter-exec console '+($command|ConvertTo-Json -Compress))}
function Gdb-Quote([string]$path){'"'+$path.Replace('\','/').Replace('"','\"')+'"'}
Save-Result
try {
 $result.baseline=@($dat,$old | ForEach-Object {@{path=$_;sha256=(Get-FileHash -LiteralPath $_).Hash;length=(Get-Item -LiteralPath $_).Length}}); Save-Result
 $null=Control 'console tick rate 1'
 $pi=[Diagnostics.ProcessStartInfo]::new(); $pi.FileName=$gdbExe; $pi.UseShellExecute=$false; $pi.CreateNoWindow=$true; $pi.RedirectStandardInput=$true; $pi.RedirectStandardOutput=$true; $pi.RedirectStandardError=$true
 foreach($arg in @('-nx','-q','--interpreter=mi2',$engine,'-p',"$ServerPid")){$pi.ArgumentList.Add($arg)}
 $debugger=[Diagnostics.Process]::new(); $debugger.StartInfo=$pi; if(-not $debugger.Start()){throw 'GDB start failed'}; $attached=$true; $readerTask=$debugger.StandardOutput.ReadLineAsync(); $stderrTask=$debugger.StandardError.ReadToEndAsync()
 Console-Mi ('source '+$gdbScript.Replace('\','/'))
 Console-Mi ('fifo-arm '+$ServerPid+' '+(Gdb-Quote $dat)+' '+$pin+' '+$symbols[0]+' '+$RunToken+' '+(Gdb-Quote $receipt))
 Wait-For {Test-Path -LiteralPath ($receipt+'.armed.json')} 'debugger armed'
 $armed=Get-Content -Raw ($receipt+'.armed.json')|ConvertFrom-Json
 if($armed.pid -ne $ServerPid -or $armed.engine_sha256 -ine $pin -or $armed.run_token -cne $RunToken -or $armed.owner -ine $dat){throw 'Debugger armed receipt mismatch'}
 $armStream=[IO.File]::Open($arm,[IO.FileMode]::CreateNew,[IO.FileAccess]::Write,[IO.FileShare]::Read); $armStream.Dispose()
 $stopped=$false; Send-Mi '-exec-continue'
 $null=Control 'console tick rate 1000'
 Wait-For {Test-Path -LiteralPath $receipt} 'actual guest writer NtWrite call entry'
 $held=Get-Content -Raw $receipt|ConvertFrom-Json
 if($held.pid -ne $ServerPid -or $held.engine_sha256 -ine $pin -or $held.run_token -cne $RunToken -or $held.owner -ine $dat -or -not $held.all_stop -or @($held.threads|Where-Object {-not $_.stopped}).Count){throw 'Held writer receipt mismatch'}
 $candidateTids=@($held.ptid | Where-Object {$_ -ne 0 -and $_ -ne $ServerPid} | Select-Object -Unique)
 if($candidateTids.Count -ne 1){throw 'Ambiguous Windows native writer TID'}
 $writerTid=[uint32]$candidateTids[0]; $threadHandle=[N5FifoNative]::OpenThread(0x802,$false,$writerTid)
 if($threadHandle -eq [IntPtr]::Zero -or [N5FifoNative]::GetProcessIdOfThread($threadHandle) -ne $ServerPid){throw 'Writer TID not owned by exact server'}
 $description=[IntPtr]::Zero; $hr=[N5FifoNative]::GetThreadDescription($threadHandle,[ref]$description)
 try{$threadName=if($hr -ge 0 -and $description -ne [IntPtr]::Zero){[Runtime.InteropServices.Marshal]::PtrToStringUni($description)}else{''}}finally{if($description -ne [IntPtr]::Zero){$null=[N5FifoNative]::LocalFree($description)}}
 if($threadName -cne 'horizon-player-writer'){throw "Wrong native thread description $threadName"}
 $processHandle=[N5FifoNative]::OpenProcess(0x40,$false,[uint32]$ServerPid); $duplicate=[IntPtr]::Zero
 try {
  if($processHandle -eq [IntPtr]::Zero -or -not [N5FifoNative]::DuplicateHandle($processHandle,[IntPtr][long]$held.handle,[N5FifoNative]::GetCurrentProcess(),[ref]$duplicate,0,$false,2)){throw 'Cannot duplicate captured IO handle for path query'}
  $buffer=[Text.StringBuilder]::new(32768); $len=[N5FifoNative]::GetFinalPathNameByHandle($duplicate,$buffer,32768,0)
  if($len -eq 0 -or $len -ge 32768){throw 'Cannot resolve captured IO handle path'}
  $ioPath=$buffer.ToString(); if($ioPath.StartsWith('\\?\')){$ioPath=$ioPath.Substring(4)}
  if([IO.Path]::GetFullPath($ioPath) -ine [IO.Path]::GetFullPath($dat+'_new')){throw "Captured IO target mismatch $ioPath"}
 }finally{if($duplicate -ne [IntPtr]::Zero){$null=[N5FifoNative]::CloseHandle($duplicate)};if($processHandle -ne [IntPtr]::Zero){$null=[N5FifoNative]::CloseHandle($processHandle)}}
 Assert-Server; $count=[N5FifoNative]::SuspendThread($threadHandle)
 if($count -eq [uint32]::MaxValue){throw 'SuspendThread failed'}
 $suspended=$true
 if($count -ne 0){throw 'Writer had an existing suspension; refuse to use it'}
 $result.writer=@{tid=$writerTid;name=$threadName;io_path=$ioPath;suspend_previous=$count;suspended_utc=[DateTime]::UtcNow.ToString('o');held_receipt=$receipt}; $result.status='writer_suspended'; Save-Result
 Send-Mi '-target-detach'; $attached=$false; $stopped=$false
 $uuidPattern=$GuestUuid.ToLowerInvariant(); $tokenPattern=[Regex]::Escape($RunToken)
 $beginPattern="ALLOMANCY_NATIVE_TEST event=inventory_helper_begin pid=$ServerPid player=$uuidPattern transaction=([^ ]+) run_token=$tokenPattern outer=Busy durable=false"
 Wait-For {(New-Log) -match $beginPattern} 'real inventory helper entry'
 $pendingLog=New-Log; $begin=[Regex]::Match($pendingLog,$beginPattern)
 if($pendingLog -match 'event=inventory_helper_(end|save_callback)' -or $pendingLog -match 'scenario=autosave-pending result=(PASS|FAIL)'){throw 'Inventory helper completed/callback fired before our writer release'}
 $result.helper_begin_offset=$logOffset+$begin.Index; $result.transaction=$begin.Groups[1].Value
 $resumed=[N5FifoNative]::ResumeThread($threadHandle); $suspended=$false
 $result.writer.resume_previous=$resumed; $result.writer.resumed_utc=[DateTime]::UtcNow.ToString('o'); Save-Result
 if($resumed -ne 1){throw 'Unexpected ResumeThread suspension count'}
 $endPattern="ALLOMANCY_NATIVE_TEST event=inventory_helper_end pid=$ServerPid player=$uuidPattern transaction=$([Regex]::Escape($result.transaction)) run_token=$tokenPattern callback_count=([0-9]+) durable=false"
 Wait-For {((New-Log) -match $endPattern) -and ((New-Log) -match 'scenario=autosave-pending result=(PASS|FAIL)')} 'real callback, helper return and scenario result'
 $finished=New-Log; $end=[Regex]::Match($finished,$endPattern); $callback=[Regex]::Match($finished,"ALLOMANCY_NATIVE_TEST event=inventory_helper_save_callback player=$uuidPattern outer=Busy durable=false")
 if(-not $callback.Success -or [long]$end.Groups[1].Value -lt 1 -or $callback.Index -le $begin.Index -or $callback.Index -ge $end.Index -or $finished -notmatch 'scenario=autosave-pending result=PASS durable=false' -or $finished -match 'scenario=autosave-pending result=FAIL'){throw 'Actual FIFO callback/result order failed'}
 $result.callback_count=[long]$end.Groups[1].Value; $result.callback_offset=$logOffset+$callback.Index; $result.helper_end_offset=$logOffset+$end.Index
 $null=Control 'console tick rate 20'
 $result.status='runtime_pass'; $result.pass=$true; $result.limitation='Pending target writer before NtWriteFile call completed; real callback drained inside inventory helper. No integrated-owner pending FIFO, fsync, in-kernel cut or offline/reopen result claimed.'
}catch{$result.status='failed';$result.error=$_.Exception.Message}finally{
 $cleaning=$true
 if($suspended -and $threadHandle -ne [IntPtr]::Zero){$release=[N5FifoNative]::ResumeThread($threadHandle);$result.cleanup+=@{resume_previous=$release};$suspended=$false}
 if($threadHandle -ne [IntPtr]::Zero){$null=[N5FifoNative]::CloseHandle($threadHandle)}
 if($debugger){
  try{
   if($attached -and -not $debugger.HasExited){if(-not $stopped){Send-Mi '-exec-interrupt --all' -NoWait;Wait-For {$stopped} 'cleanup debugger stop' 5};Send-Mi '-target-detach';$attached=$false}
   if(-not $debugger.HasExited -and -not $attached){Send-Mi '-gdb-exit' -NoWait;$null=$debugger.WaitForExit(5000)}
  }catch{$result.cleanup+=@{error=$_.Exception.Message;debugger_pid=$debugger.Id;attached=$attached};if($attached){$result.status='attention_required_debugger_attached';$result.pass=$false}}
  try{Pump-Gdb}catch{$result.cleanup+=@{error=$_.Exception.Message}}
  [IO.File]::WriteAllLines((Join-Path $run 'gdb.stdout'),$debugLines)
  if($stderrTask -and $stderrTask.IsCompleted){[IO.File]::WriteAllText((Join-Path $run 'gdb.stderr'),$stderrTask.GetAwaiter().GetResult())}
 }
 Save-Result
}
Write-Output "result=$resultPath status=$($result.status) pass=$($result.pass)"
if(-not $result.pass){throw $result.error}