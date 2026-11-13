$ErrorActionPreference = 'Stop'
$project = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$evidence = Join-Path $project 'docs/execution/evidence'
function Read-EvidenceJson([string]$Name) {
    Get-Content -Raw -LiteralPath (Join-Path $evidence $Name) | ConvertFrom-Json
}
function Require([bool]$Condition, [string]$Message) {
    if (!$Condition) { throw $Message }
}
$v2 = Read-EvidenceJson 'p0-smoke-controls3-controls.json'
Require (@($v2 | Where-Object exit_code -ne 0).Count -eq 0) 'v2 control failure'
$chat = ($v2 | Where-Object { ($_.command -join ' ') -eq 'screen' }).reply | ConvertFrom-Json
Require ($chat.screen -eq 'chat') 'v2 never entered chat'
$v2Impulses = @(Select-String -LiteralPath (Join-Path $evidence 'p0-smoke-controls3-stderr.log') -Pattern 'event=impulse').Count
Require ($v2Impulses -eq 1) 'v2 repeat/menu/chat produced incorrect impulse count'
$v4 = Read-EvidenceJson 'p0-smoke-capabilities-green-controls.json'
Require (@($v4 | Where-Object exit_code -ne 0).Count -eq 0) 'v4 control failure'
$states = @($v4 | Where-Object { ($_.command -join ' ') -eq 'state' } | ForEach-Object { $_.reply | ConvertFrom-Json })
Require ($states.Count -eq 5) 'v4 missing physical samples'
Require ($states[1].player.position[1] -gt $states[0].player.position[1]) 'v4 no free rise observed'
foreach ($sample in $states[3..4]) { Require ($sample.player.position[1] -le 79.2001) 'v4 sampled ceiling penetration' }
$inventory = ($v4 | Where-Object { $_.command[0] -eq 'inventory' }).reply | ConvertFrom-Json
Require ($inventory.counts.'allomancy_probe_content:probe_metal' -eq 1) 'custom item not observed'
$v4Log = Get-Content -Raw -LiteralPath (Join-Path $evidence 'p0-smoke-capabilities-green-stderr.log')
foreach ($signal in @('ping_sent=true sound_queued=true', 'event=ping valid=true ack_sent=true',
    'event=ack valid=true', 'event=use_item count=1', 'event=living_death', 'event=loot_drops count=1')) {
    Require ($v4Log.Contains($signal)) "Missing runtime signal: $signal"
}
Require (@(Select-String -LiteralPath (Join-Path $evidence 'p0-smoke-capabilities-green-stderr.log') -Pattern 'event=impulse').Count -eq 2) 'v4 unexpected impulses'
$v4Exit = Read-EvidenceJson 'p0-smoke-capabilities-green-result.json'
Require (!$v4Exit.timed_out -and $v4Exit.exit_code -eq 0) 'v4 did not exit cleanly'
for ($i = 1; $i -le 2; $i++) {
    $exit = Read-EvidenceJson "p0-smoke-storage$i-result.json"
    $report = Read-EvidenceJson "p0-smoke-storage$i.json"
    Require (!$exit.timed_out -and $exit.exit_code -eq 0) "storage$i did not exit cleanly"
    Require ($null -eq $report.error -and $null -ne $report.player) "storage$i never entered the world"
    $log = Get-Content -Raw -LiteralPath (Join-Path $evidence "p0-smoke-storage$i-stderr.log")
    Require ($log.Contains("event=inline_storage previous_count=$($i - 1) write_ok=true")) "storage$i counter not persisted"
}
$v5 = Read-EvidenceJson 'p0-runtime-v5.json'
$sourceHash = (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot 'bundle/api/src/lib.rs') -Algorithm SHA256).Hash
Require ($sourceHash -eq $v5.source_sha256) 'current probe differs from verified v5 source'
Add-Type -AssemblyName System.Drawing
$image = [Drawing.Bitmap]::new((Join-Path $evidence 'p0-smoke-capabilities-green-hud.png'))
try {
    Require ($image.Width -eq 1280 -and $image.Height -eq 720) 'HUD capture resolution changed'
    Require ($image.GetPixel(25, 25).ToArgb() -eq -16711681) 'HUD cyan border not rendered'
    Require ($image.GetPixel(35, 35).ToArgb() -eq -16777216) 'HUD black center not rendered'
} finally { $image.Dispose() }
Write-Output 'P0 runtime evidence verified: controls, samples, item, hooks, network, queue, HUD, inline round-trip and source hash.'
