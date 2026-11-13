param([Parameter(Mandatory)][string]$LogPath)
$ErrorActionPreference = 'Stop'
$expected = 'ALLOMANCY_CORE event=player_tick accepted=true tick_recorded=true'
$matches = @(Select-String -LiteralPath $LogPath -SimpleMatch $expected)
if (!$matches.Count) { throw 'No authoritative player tick reached the core bridge successfully.' }
Write-Output "native_bridge_player_ticks=$($matches.Count)"
