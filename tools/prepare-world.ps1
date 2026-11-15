[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$WorldRoot,
    [Parameter(Mandatory)][string]$AllowedRoot,
    [switch]$ProvisionNew
)
$ErrorActionPreference = 'Stop'
function Assert-NoLinks([string]$path) {
    $item = Get-Item -LiteralPath $path -Force
    while ($null -ne $item) {
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Reparse point rejected: $($item.FullName)" }
        if ([string]::Equals($item.FullName.TrimEnd('\'), $allowed.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) { return }
        $item = if ($item -is [IO.FileInfo]) { $item.Directory } else { $item.Parent }
    }
    throw 'Path does not reach the authorized root'
}
function Read-Marker([string]$path) {
    Assert-NoLinks $path
    $stream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    try {
        if ($stream.Length -gt 4096) { throw 'Marker exceeds 4096 bytes' }
        $bytes = [byte[]]::new([int]$stream.Length)
        $offset = 0
        while ($offset -lt $bytes.Length) {
            $n = $stream.Read($bytes, $offset, $bytes.Length - $offset)
            if ($n -eq 0) { throw 'Unexpected EOF' }; $offset += $n
        }
    } finally { $stream.Dispose() }
    $text = [Text.UTF8Encoding]::new($false, $true).GetString($bytes)
    $doc = [System.Text.Json.JsonDocument]::Parse($text)
    try {
        if ($doc.RootElement.ValueKind -ne [System.Text.Json.JsonValueKind]::Object) { throw 'Marker must be an object' }
        $names = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
        foreach ($p in $doc.RootElement.EnumerateObject()) {
            if (-not $names.Add($p.Name) -or $p.Name -cnotin @('schema_version','world_id')) { throw 'Duplicate or unknown field' }
            if ($p.Name -ceq 'schema_version') {
                if ($p.Value.ValueKind -ne [System.Text.Json.JsonValueKind]::Number -or $p.Value.GetRawText() -cne '1') { throw 'Invalid schema version' }
            } else {
                if ($p.Value.ValueKind -ne [System.Text.Json.JsonValueKind]::String -or $p.Value.GetString() -cnotmatch '^[0-9a-f]{32}$') { throw 'Invalid world ID' }
                $id = $p.Value.GetString()
            }
        }
        if ($names.Count -ne 2) { throw 'Missing marker field' }
        return @{ Id = $id; Bytes = $bytes }
    } finally { $doc.Dispose() }
}
function Write-New([string]$path, [byte[]]$bytes) {
    $tmp = "$path.tmp"
    $stream = [IO.File]::Open($tmp, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($bytes, 0, $bytes.Length); $stream.Flush($true) } finally { $stream.Dispose() }
    [IO.File]::Move($tmp, $path)
}
if (Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^(cargo|horizon.*)(\.exe)?$' }) { throw 'Close Horizon and Cargo before preparation' }
$allowed = (Get-Item -LiteralPath $AllowedRoot -Force).FullName
$world = (Get-Item -LiteralPath $WorldRoot -Force).FullName
if (-not (Test-Path -LiteralPath $allowed -PathType Container) -or -not (Test-Path -LiteralPath $world -PathType Container)) { throw 'Roots must be existing directories' }
Assert-NoLinks $allowed
Assert-NoLinks $world
$prefix = $allowed.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
if (-not $world.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) { throw 'WorldRoot must be strictly inside AllowedRoot' }
$dir = Join-Path $world 'allomancy'
$primary = Join-Path $dir 'world-identity.json'
$backup = "$primary.bak"
if (Test-Path -LiteralPath $dir) { Assert-NoLinks $dir; if (-not (Test-Path -LiteralPath $dir -PathType Container)) { throw 'allomancy is not a directory' } }
foreach ($temp in @("$primary.tmp", "$backup.tmp")) { if (Test-Path -LiteralPath $temp) { throw "Partial temporary file preserved: $temp" } }
$hasPrimary = Test-Path -LiteralPath $primary
$hasBackup = Test-Path -LiteralPath $backup
$p = if ($hasPrimary) { Read-Marker $primary } else { $null }
$b = if ($hasBackup) { Read-Marker $backup } else { $null }
if ($p -and $b) {
    if ($p.Id -cne $b.Id) { throw 'Marker conflict' }
    Write-Output "Existing: $($p.Id)"; return
}
if ($p -or $b) {
    if ($p) { Write-New $backup $p.Bytes; $source = 'Primary'; $id = $p.Id }
    else { Write-New $primary $b.Bytes; $source = 'Backup'; $id = $b.Id }
    $p = Read-Marker $primary; $b = Read-Marker $backup
    if ($p.Id -cne $b.Id -or $p.Id -cne $id) { throw 'Verification failed' }
    Write-Output "Repaired from ${source}: $id"; return
}
if (-not $ProvisionNew) { throw 'Explicit -ProvisionNew required for a new empty world' }
if (Get-ChildItem -LiteralPath $world -Force | Select-Object -First 1) { throw 'New world must be empty; existing worlds require separate import/audit' }
$id = [guid]::NewGuid().ToString('N')
$bytes = [Text.UTF8Encoding]::new($false).GetBytes("{`"schema_version`":1,`"world_id`":`"$id`"}")
New-Item -ItemType Directory -Path $dir | Out-Null
Assert-NoLinks $dir
Write-New $primary $bytes
Write-New $backup $bytes
$p = Read-Marker $primary; $b = Read-Marker $backup
if ($p.Id -cne $id -or $b.Id -cne $id) { throw 'Verification failed' }
Write-Output "Provisioned: $id"
