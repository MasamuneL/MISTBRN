$ErrorActionPreference = 'Stop'
$run = Join-Path $PSScriptRoot "../target/world-preparation-tests/$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force $run | Out-Null
$helper = Join-Path $PSScriptRoot 'prepare-world.ps1'
function World([string]$name) { $w = Join-Path $run $name; New-Item -ItemType Directory $w | Out-Null; return $w }
function Invoke-Helper([string]$w, [bool]$new = $false) { & $helper -WorldRoot $w -AllowedRoot $run -ProvisionNew:$new | Out-Null }
function Must-Fail([scriptblock]$action) { $failed = $false; try { & $action } catch { $failed = $true }; if (-not $failed) { throw 'Expected rejection' } }
function Marker([string]$w) { Join-Path $w 'allomancy/world-identity.json' }
$w = World 'fresh'; Invoke-Helper $w $true
$p = Marker $w; $b = "$p.bak"
$original = [IO.File]::ReadAllBytes($p); $hash = (Get-FileHash $p).Hash
if ((Get-FileHash $b).Hash -ne $hash) { throw 'Backup differs' }
Invoke-Helper $w
if ((Get-FileHash $p).Hash -ne $hash) { throw 'Idempotence failed' }
Remove-Item -LiteralPath $b; Invoke-Helper $w
if ((Get-FileHash $b).Hash -ne $hash) { throw 'Backup recovery failed' }
Remove-Item -LiteralPath $p; Invoke-Helper $w
if ((Get-FileHash $p).Hash -ne $hash) { throw 'Primary recovery failed' }
foreach ($bad in @('{', '{"schema_version":2,"world_id":"0123456789abcdef0123456789abcdef"}', '{"schema_version":1,"schema_version":1,"world_id":"0123456789abcdef0123456789abcdef"}', '{"schema_version":1,"world_id":"0123456789abcdef0123456789abcdef","extra":0}', '[]', '{"schema_version":"1","world_id":"0123456789abcdef0123456789abcdef"}', '{"schema_version":1}', '{"schema_version":1,"world_id":"ABCDEF0123456789abcdef0123456789"}')) {
    [IO.File]::WriteAllText($p, $bad); $before = (Get-FileHash $p).Hash
    Must-Fail { Invoke-Helper $w }; if ((Get-FileHash $p).Hash -ne $before) { throw 'Invalid primary overwritten' }
}
[IO.File]::WriteAllBytes($p, $original)
[IO.File]::WriteAllText($b, '{"schema_version":1,"world_id":"00000000000000000000000000000000"}')
Must-Fail { Invoke-Helper $w }
$partial = World 'partial'; New-Item -ItemType Directory (Join-Path $partial 'allomancy') | Out-Null
[IO.File]::WriteAllText("$(Marker $partial).tmp", 'partial')
Must-Fail { Invoke-Helper $partial $true }
if (-not (Test-Path "$(Marker $partial).tmp")) { throw 'Partial removed' }
$existing = World 'existing'; [IO.File]::WriteAllText((Join-Path $existing 'level.dat'), 'fixture')
Must-Fail { Invoke-Helper $existing $true }
$empty = World 'empty'; Must-Fail { Invoke-Helper $empty }
Must-Fail { & $helper -WorldRoot $run -AllowedRoot $run -ProvisionNew }
$link = Join-Path $run 'link'
try { New-Item -ItemType SymbolicLink -Path $link -Target $empty -ErrorAction Stop | Out-Null; Must-Fail { Invoke-Helper $link $true }; $linkResult = 'passed' } catch { if (Test-Path $link) { throw }; $linkResult = "unavailable: $($_.Exception.Message)" }
$junction = Join-Path $run 'junction'
try { New-Item -ItemType Junction -Path $junction -Target $empty -ErrorAction Stop | Out-Null; Must-Fail { Invoke-Helper $junction $true }; $junctionResult = 'passed' } catch { if (Test-Path $junction) { throw }; $junctionResult = 'unavailable' }
[IO.File]::WriteAllBytes($p, [byte[]](0xff,0xfe)); Must-Fail { Invoke-Helper $w }
[IO.File]::WriteAllBytes($p, [byte[]]::new(4097)); Must-Fail { Invoke-Helper $w }
Write-Host "PASS fixtures=$run link=$linkResult junction=$junctionResult"
