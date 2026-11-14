param([string]$ContentRoot = (Join-Path (Split-Path $PSScriptRoot -Parent) 'content'))
$ErrorActionPreference = 'Stop'
try {
 function Fail($message) { throw $message }
 function Positive($n, $where) { if (($n -isnot [int] -and $n -isnot [long]) -or $n -lt 1 -or $n -gt 4294967295) { Fail "$where must be a positive u32 integer" } }
 function Id($id, $where) { if ($id -isnot [string] -or $id -cnotmatch '^(allomancy|minecraft):[a-z0-9_]+$') { Fail "Invalid ID in ${where}: $id" } }
 function Unique($values, $where) { $seen = @{}
  foreach ($id in $values) { Id $id $where
  if ($seen.ContainsKey($id)) { Fail "Duplicate $where ID: $id" }
  $seen[$id] = $true } }
 $docs = foreach ($name in @('metals','recipes','loot')) { $d = Get-Content (Join-Path $ContentRoot "$name.json") -Raw | ConvertFrom-Json
  if (($d.schema_version -isnot [int] -and $d.schema_version -isnot [long]) -or $d.schema_version -ne 1 -or $d.catalog_version -cne 'allomancy-content-internal-0.1.0' -or $d.scope -cne 'internal-catalog-not-hmod-not-hnmod') { Fail "Unknown schema, version or scope: $name" }
  $d }
 $m,$r,$l = $docs
 foreach ($list in @($m.metals,$m.pairs,$m.items,$m.milestones,$r.recipes,$r.sample_recipes,$r.filled_vial_recipes,$l.sources,$l.trades)) { if ($null -eq $list -or $list -isnot [array]) { Fail 'Missing catalog array' } }
 $recipes = @($r.recipes) + @($r.sample_recipes) + @($r.filled_vial_recipes)
 Unique @($m.metals.id) 'metal'
  Unique @($m.pairs) 'pair'
  Unique @($m.items.id) 'item'
  Unique @($m.milestones.id) 'milestone'
  Unique @($recipes.id) 'recipe'
  Unique @($l.sources.id) 'source'
  Unique @($l.trades.id) 'trade'
 $names = @('iron','steel','tin','pewter','zinc','brass','copper','bronze')
  $pairs = @('iron_steel','tin_pewter','zinc_brass','copper_bronze')
 if ($m.metals.Count -ne 8 -or $m.pairs.Count -ne 4) { Fail 'Expected eight metals and four pairs' }
 $items = @{}
  foreach ($item in $m.items) { $items[$item.id] = $item
  if (-not $item.kind) { Fail "Missing kind: $($item.id)" } }
  $milestones = @{}
  foreach ($h in $m.milestones) { $milestones[$h.id] = $h }
 for ($i=0; $i -lt 8; $i++) {
  $metal = $m.metals[$i]
  $name = $names[$i]
  $pair = 'allomancy:' + $pairs[[int][math]::Floor($i/2)]
  if ($metal.id -cne "allomancy:$name" -or ($metal.order -isnot [int] -and $metal.order -isnot [long]) -or $metal.order -ne $i -or $metal.pair -cne $pair -or $m.pairs[[int][math]::Floor($i/2)] -cne $pair) { Fail "Wrong metal order or pair at $i" }
  if ($metal.item -cne "allomancy:${name}_ingot" -or $metal.sample -cne "allomancy:sample_$name" -or $items[$metal.item].kind -cne 'metal_material' -or $items[$metal.sample].kind -cne 'metal_sample') { Fail "Invalid metal item/sample mapping: $name" }
  Positive $metal.assay_units_per_sample "assay $name"
  $vial = $items["allomancy:vial_filled_$name"]
  if ($vial.kind -cne 'filled_vial' -or $vial.properties.metal -cne $metal.id -or $vial.properties.state -cne 'filled') { Fail "Invalid vial mapping: $name" }
  Positive $vial.properties.assay_units "vial $name"
  $sr = @($r.sample_recipes | Where-Object { $_.result.id -ceq $metal.sample })
  $vr = @($r.filled_vial_recipes | Where-Object { $_.result.id -ceq $vial.id })
  if ($sr.Count -ne 1 -or @($sr[0].ingredients.id) -cnotcontains $metal.item -or $vr.Count -ne 1 -or @($vr[0].ingredients.id) -cnotcontains $metal.sample -or @($vr[0].ingredients.id) -cnotcontains 'allomancy:empty_vial') { Fail "Invalid recipe mapping: $name" }
 }
 foreach ($h in $m.milestones) { if (-not $items.ContainsKey($h.fragment) -or $items[$h.fragment].milestone -cne $h.id -or ($h.claim_once_per_player -isnot [bool] -or $h.claim_once_per_player -ne $true) -or ($h.requires_warden_kill -isnot [bool] -or $h.requires_warden_kill -ne $false)) { Fail "Invalid milestone: $($h.id)" }
  if ($null -ne $h.unlocks_pair -and @($m.pairs) -cnotcontains $h.unlocks_pair) { Fail "Missing milestone pair: $($h.unlocks_pair)" }
  if ($null -ne $h.PSObject.Properties['grants'] -and ($h.grants -isnot [string] -or $h.grants -cne 'allomancy:final_mastery')) { Fail "Unknown milestone grant: $($h.grants)" } }
 foreach ($item in $m.items) { if ($item.milestone -and -not $milestones.ContainsKey($item.milestone)) { Fail "Missing milestone: $($item.milestone)" }
  foreach ($tag in @($item.properties.metal_tags)) { if ($tag -and @($m.metals.id) -cnotcontains $tag) { Fail "Missing metal tag: $tag" } } }
 $available = @{}
  $vanilla = @{}
 foreach ($source in $l.sources) { if (($source.verified -isnot [bool] -or $source.verified -ne $false) -or ($source.status -cne 'proposed' -and $source.status -cne 'assumed_available') -or -not $source.kind -or @($source.outputs).Count -eq 0) { Fail "Invalid source status: $($source.id)" }
  if ($source.kind -cne 'vanilla_material' -and $source.status -cne 'proposed') { Fail "Source must be proposed: $($source.id)" }
  if ($source.milestone -and -not $milestones.ContainsKey($source.milestone)) { Fail "Missing source milestone: $($source.milestone)" }
  foreach ($o in $source.outputs) { Id $o.id 'source output'
  Positive $o.count 'source count'
  if ($o.id -clike 'allomancy:*' -and -not $items.ContainsKey($o.id)) { Fail "Missing source item: $($o.id)" }
  $available[$o.id] = $true
  if ($o.id -clike 'minecraft:*') { $vanilla[$o.id] = $true } } }
 foreach ($trade in $l.trades) { if ($trade.status -cne 'proposed' -or ($trade.verified -isnot [bool] -or $trade.verified -ne $false)) { Fail "Trade must be proposed and unverified: $($trade.id)" }
  Id $trade.output.id 'trade'
  Positive $trade.output.count 'trade count'
  if (-not $items.ContainsKey($trade.output.id)) { Fail "Missing trade item: $($trade.output.id)" } }
 foreach ($metal in $m.metals) { if ($metal.vanilla_input -and -not $vanilla.ContainsKey($metal.vanilla_input)) { Fail "Missing vanilla source: $($metal.vanilla_input)" } }
 foreach ($recipe in $recipes) { if (@($recipe.ingredients).Count -eq 0 -or $null -eq $recipe.ingredients[0]) { Fail "Empty ingredients: $($recipe.id)" }
  foreach ($entry in @($recipe.ingredients) + @($recipe.result)) { Id $entry.id 'recipe'
  Positive $entry.count "recipe $($recipe.id)"
  if ($entry.id -clike 'allomancy:*' -and -not $items.ContainsKey($entry.id)) { Fail "Missing recipe item: $($entry.id)" }
  if ($entry.id -clike 'minecraft:*' -and -not $vanilla.ContainsKey($entry.id)) { Fail "Missing vanilla source: $($entry.id)" } }
  foreach ($ingredient in $recipe.ingredients) { if ($items[$ingredient.id].kind -ceq 'filled_vial') { Fail "Filled vial ingredient: $($recipe.id)" } } }
 $executable = @{}
  do { $changed = $false
  foreach ($recipe in $recipes) { $ready = $true
  foreach ($ingredient in $recipe.ingredients) { if (-not $available.ContainsKey($ingredient.id)) { $ready = $false } }
  if ($ready) { $executable[$recipe.id] = $true }
  if ($ready -and -not $available.ContainsKey($recipe.result.id)) { $available[$recipe.result.id] = $true
  $changed = $true } } } while ($changed)
 $unreachable = @($recipes | Where-Object { -not $executable.ContainsKey($_.id) })
  if ($unreachable.Count) { Fail "Unreachable recipes from declared sources: $($unreachable.id -join ', ')" }
 # Kahn's algorithm checks ingredient-to-result dependencies independently of sources.
 $edges = @{}
 $indegree = @{}
 foreach ($recipe in $recipes) {
  $result = $recipe.result.id
  if (-not $indegree.ContainsKey($result)) { $indegree[$result] = 0 }
  foreach ($ingredient in $recipe.ingredients) {
   $id = $ingredient.id
   if (-not $indegree.ContainsKey($id)) { $indegree[$id] = 0 }
   if (-not $edges.ContainsKey($id)) { $edges[$id] = @{} }
   if (-not $edges[$id].ContainsKey($result)) {
    $edges[$id][$result] = $true
    $indegree[$result]++
   }
  }
 }
 $queue = [System.Collections.Generic.Queue[string]]::new()
 foreach ($id in $indegree.Keys) { if ($indegree[$id] -eq 0) { $queue.Enqueue($id) } }
 $visited = 0
 while ($queue.Count -gt 0) {
  $id = $queue.Dequeue()
  $visited++
  if ($edges.ContainsKey($id)) {
   foreach ($result in $edges[$id].Keys) {
    $indegree[$result]--
    if ($indegree[$result] -eq 0) { $queue.Enqueue($result) }
   }
  }
 }
 if ($visited -ne $indegree.Count) { Fail 'Recipe dependency cycle detected' }
 Write-Output "Valid internal catalog: metals=$($m.metals.Count) pairs=$($m.pairs.Count) items=$($m.items.Count) milestones=$($m.milestones.Count) recipes=$($recipes.Count) sources=$($l.sources.Count) trades=$($l.trades.Count); declared-source reachable recipes=$($recipes.Count)."
 Write-Output 'WARNING: reachability includes proposed, unverified sources; this does not validate Horizon or SDK compatibility.'
} catch { [Console]::Error.WriteLine("Content validation failed: $($_.Exception.Message)")
  exit 1 }
