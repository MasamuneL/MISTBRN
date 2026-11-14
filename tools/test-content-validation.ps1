param([string]$Validator = (Join-Path $PSScriptRoot 'validate-content.ps1'))
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$run = Join-Path $root ('target/content-validation-tests/' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run | Out-Null
$cases = @('baseline','duplicate','missing','zero','negative','fraction','cycle','order','schema','malformed','claim_string','warden_string','source_string','trade_string','reachable_cycle','unknown_grant','grant_true','grant_false','grant_null','grant_zero','grant_empty')
$failed = 0
foreach ($case in $cases) {
    $dir = Join-Path $run $case
    New-Item -ItemType Directory -Path $dir | Out-Null
    Copy-Item (Join-Path $root 'content/*.json') $dir
    $mp = Join-Path $dir 'metals.json'; $rp = Join-Path $dir 'recipes.json'; $lp = Join-Path $dir 'loot.json'
    $m = Get-Content $mp -Raw | ConvertFrom-Json; $r = Get-Content $rp -Raw | ConvertFrom-Json; $l = Get-Content $lp -Raw | ConvertFrom-Json
    switch ($case) {
        duplicate { $m.items[1].id = $m.items[0].id }
        missing { $r.recipes[0].ingredients[0].id = 'minecraft:missing' }
        zero { $r.recipes[0].result.count = 0 }
        negative { $r.recipes[0].result.count = -1 }
        fraction { $r.recipes[0].result.count = 0.5 }
        cycle { $r.recipes[0].ingredients[0].id = 'allomancy:steel_ingot' }
        order { $m.metals[0].order = 1 }
        schema { $m.schema_version = 99 }
        claim_string { $m.milestones[0].claim_once_per_player = 'True' }
        warden_string { $m.milestones[0].requires_warden_kill = 'False' }
        source_string { $l.sources[0].verified = 'False' }
        trade_string { $l.trades[0].verified = 'False' }
        unknown_grant { $m.milestones[3].grants = 'allomancy:unknown' }
        grant_true { $m.milestones[3].grants = $true }
        grant_false { $m.milestones[3].grants = $false }
        grant_null { $m.milestones[3].grants = $null }
        grant_zero { $m.milestones[3].grants = 0 }
        grant_empty { $m.milestones[3].grants = '' }
        reachable_cycle { $r.recipes += [pscustomobject]@{ id='allomancy:recycle_iron'; ingredients=@([pscustomobject]@{ id='allomancy:steel_ingot'; count=1 }); result=[pscustomobject]@{ id='allomancy:iron_ingot'; count=1 } } }
    }
    $m | ConvertTo-Json -Depth 30 | Set-Content $mp -Encoding utf8
    $r | ConvertTo-Json -Depth 30 | Set-Content $rp -Encoding utf8
    $l | ConvertTo-Json -Depth 30 | Set-Content $lp -Encoding utf8
    if ($case -eq 'malformed') { Set-Content $rp '{' }
    $output = & pwsh -NoProfile -File $Validator -ContentRoot $dir 2>&1
    $code = $LASTEXITCODE
    $output | Out-String | Set-Content (Join-Path $dir 'result.log') -Encoding utf8
    $expected = @{ duplicate='Duplicate'; missing='Missing vanilla source'; zero='positive u32'; negative='positive u32'; fraction='positive u32'; cycle='Unreachable recipes'; order='Wrong metal order'; schema='Unknown schema'; malformed='Content validation failed'; claim_string='Invalid milestone'; warden_string='Invalid milestone'; source_string='Invalid source status'; trade_string='Trade must'; reachable_cycle='Recipe dependency cycle'; unknown_grant='Unknown milestone grant'; grant_true='Unknown milestone grant'; grant_false='Unknown milestone grant'; grant_null='Unknown milestone grant'; grant_zero='Unknown milestone grant'; grant_empty='Unknown milestone grant' }
    $ok = if ($case -eq 'baseline') { $code -eq 0 } else { $code -ne 0 -and "$output" -like ('*' + $expected[$case] + '*') }
    if (-not $ok) { $failed++; Write-Host "FAIL $case exit=$code" } else { Write-Host "PASS $case exit=$code" }
}
Write-Host "Fixtures: $run"
if ($failed) { exit 1 }
