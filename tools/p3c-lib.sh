# shellcheck shell=bash
# P3c native scenarios (alchemy), sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# p3c_lane: a fresh creative world on a stone floor at 512.5 100 512.5, looking east.
p3c_lane() {
  launch MISTBRN_TEST_SCENARIO="${1:-}"
  ctl command "difficulty peaceful" >/dev/null
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 20 >/dev/null
  ctl command "fill 500 99 500 540 99 540 minecraft:stone" >/dev/null
  ctl command "fill 500 100 500 540 104 540 minecraft:air" >/dev/null
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl look -90 0 >/dev/null
  ctl wait 10 >/dev/null
}

# p3c_count ITEM: how many ITEM the player holds, from a fresh inventory read.
p3c_count() {
  ctl inventory | jq -r --arg i "$1" '.counts[$i] // 0'
}

# p3c_check LABEL ACTUAL EXPECTED
p3c_check() {
  if [[ "$2" == "$3" ]]; then k_ok "$1: $2"; else k_fail "$1: expected $3, got $2"; fi
}

# p3c_lines LAUNCH PATTERN: how many lines of that launch's log match PATTERN (grep BRE).
p3c_lines() { grep -ac "$2" "$(log "$1")" || true; }

# p3c_break_row X0 X1 Y Z: break X0..X1 at Y in row Z from just north of it (survival).
p3c_break_row() {
  local x
  for x in $(seq "$1" "$2"); do
    ctl command "tp @s $x.5 $(($3 + 1)) $(($4 - 1)).5" >/dev/null
    ctl wait 2 >/dev/null
    ctl break "$x" "$3" "$4" >> "$dir/breaking.jsonl"
  done
}

# p3c_fill X Y Z [SLOT ITEM COUNT]...: put items straight into an alembic's slots (0-2 vials,
# 3 ingredient) with /item, as a player's clicks would leave them.
p3c_fill() {
  local x="$1" y="$2" z="$3"; shift 3
  while (( $# >= 3 )); do
    ctl command "item replace block $x $y $z container.$1 with $2 $3" >/dev/null
    shift 3
  done
}

# p3c_contents X Y Z NAME: the block's Items saved as NAME.json.
p3c_contents() { ctl command "data get block $1 $2 $3 Items" > "$dir/$4.json"; }

# p3c_has NAME ITEM SLOT: NAME.json shows ITEM in SLOT.
p3c_has() {
  if grep -aqE "Slot: ${3}b[^}]*id: \\\\?\"$2|id: \\\\?\"$2\\\\?\"[^}]*Slot: ${3}b" "$dir/$1.json"; then
    k_ok "$1: $2 in slot $3"
  else
    k_fail "$1: no $2 in slot $3 ($(tr -d '\n' < "$dir/$1.json" | head -c 400))"
  fi
}

# p3c_scenario: P3c alchemy (a1-a6) and its art capture (av).
p3c_scenario() {
  case "$scenario" in
    a1)  # an empty vial used on a water source becomes a water vial; the water stays
      p3c_lane
      ctl command "gamemode survival" >/dev/null
      ctl command "setblock 515 99 512 minecraft:water" >/dev/null
      ctl give mistbrn:vial_empty 2 >/dev/null
      ctl wait 10 >/dev/null
      ctl select mistbrn:vial_empty >/dev/null
      # The water is no collision face: aim at a point inside it.
      ctl look-at 515.5 99.9 512.5 >/dev/null
      ctl use-item > "$dir/fill.json"
      ctl wait 10 >/dev/null
      p3c_check "water vials" "$(p3c_count mistbrn:vial_water)" 1
      p3c_check "empty vials left" "$(p3c_count mistbrn:vial_empty)" 1
      # Looking at stone fills nothing.
      ctl look-at block 513 99 512 up >/dev/null
      ctl use-item > "$dir/no-fill.json"
      ctl wait 10 >/dev/null
      p3c_check "water vials after stone" "$(p3c_count mistbrn:vial_water)" 1
      ctl command "data get block 515 99 512" > "$dir/water-left.json"
      ctl save >/dev/null; quit_game
      p3c_check "fills logged" "$(p3c_lines 1 'MISTBRN event=vial_filled')" 1 ;;
    a2)  # salt from salt rock (1-5, Silk Touch keeps the block), beach sand and the furnace
      local salt before hits
      p3c_lane
      ctl command "gamemode survival" >/dev/null
      # Salt rock with a plain pickaxe, then with Silk Touch.
      ctl command "setblock 514 100 512 mistbrn:salt_rock" >/dev/null
      ctl command "setblock 514 100 514 mistbrn:salt_rock" >/dev/null
      ctl give diamond_pickaxe >/dev/null
      ctl wait 5 >/dev/null
      ctl select minecraft:diamond_pickaxe >/dev/null
      ctl break 514 100 512 >> "$dir/breaking.jsonl"
      ctl collect 6 >/dev/null
      salt="$(p3c_count mistbrn:salt)"
      if [[ "$salt" =~ ^[1-5]$ ]]; then k_ok "salt rock gave $salt salt"
      else k_fail "salt rock gave '$salt' salt, expected 1-5"; fi
      ctl command 'item replace entity @s weapon.mainhand with minecraft:diamond_pickaxe[enchantments={"minecraft:silk_touch":1}]' >/dev/null
      ctl break 514 100 514 >> "$dir/breaking.jsonl"
      ctl collect 6 >/dev/null
      p3c_check "silk touch salt rock" "$(p3c_count mistbrn:salt_rock)" 1
      p3c_check "silk touch adds no salt" "$(p3c_count mistbrn:salt)" "$salt"
      # Sand: 100 blocks in a beach biome (1 in 20 each), 40 in plains (never).
      ctl command "fillbiome 518 96 518 530 106 530 minecraft:beach" >/dev/null
      ctl command "fill 520 100 520 529 100 529 minecraft:sand" >/dev/null
      ctl command "fill 520 100 534 529 100 537 minecraft:sand" >/dev/null
      ctl command 'item replace entity @s weapon.mainhand with minecraft:diamond_shovel' >/dev/null
      before="$(p3c_count mistbrn:salt)"
      for z in $(seq 520 529) $(seq 534 537); do p3c_break_row 520 529 100 "$z"; done
      ctl wait 10 >/dev/null
      ctl collect 16 >/dev/null
      echo "beach salt picked up: $(( $(p3c_count mistbrn:salt) - before ))" | tee -a "$dir/report.txt"
      # Furnace: a water vial boils into salt and gives the empty vial back above it.
      ctl command 'setblock 512 100 516 minecraft:furnace{Items:[{Slot:0b,id:"mistbrn:vial_water",count:1},{Slot:1b,id:"minecraft:coal",count:1}]}' >/dev/null
      ctl command "tp @s 512.5 100 514.5" >/dev/null
      ctl wait 240 >/dev/null
      p3c_contents 512 100 516 furnace
      p3c_has furnace mistbrn:salt 2
      ctl collect 6 >/dev/null
      p3c_check "vial back from the furnace" "$(p3c_count mistbrn:vial_empty)" 1
      ctl save >/dev/null; quit_game
      hits="$(p3c_lines 1 'MISTBRN event=beach_salt pos=\[52[0-9], 100, 52[0-9]\]')"
      if (( hits >= 1 && hits <= 15 )); then k_ok "beach sand gave salt $hits times in 100"
      else k_fail "beach sand gave salt $hits times in 100 (expected 1-15)"; fi
      p3c_check "plains sand salt" "$(p3c_lines 1 'MISTBRN event=beach_salt pos=\[52[0-9], 100, 53[4-7]\]')" 0
      expect_log 1 'MISTBRN event=furnace_vial pos=\[512, 100, 516\] dropped=true' ;;
    a3)  # two stages, 3 vials per batch, 600 ticks, no fuel; a batch survives save and relaunch
      p3c_lane
      ctl command "setblock 514 100 512 mistbrn:alembic" >/dev/null
      p3c_fill 514 100 512 0 mistbrn:vial_water 1 1 mistbrn:vial_water 1 2 mistbrn:vial_water 1 3 mistbrn:salt 2
      ctl wait 300 >/dev/null
      ctl save >/dev/null; quit_game
      launch
      ctl wait 330 >/dev/null
      p3c_contents 514 100 512 stage1
      p3c_has stage1 mistbrn:vial_brine 0; p3c_has stage1 mistbrn:vial_brine 1; p3c_has stage1 mistbrn:vial_brine 2
      p3c_has stage1 mistbrn:salt 3
      p3c_fill 514 100 512 3 mistbrn:iron_flakes 1
      ctl wait 590 >/dev/null
      p3c_contents 514 100 512 stage2-early
      ctl wait 30 >/dev/null
      p3c_contents 514 100 512 stage2
      p3c_has stage2 mistbrn:vial_iron 0; p3c_has stage2 mistbrn:vial_iron 1; p3c_has stage2 mistbrn:vial_iron 2
      if grep -aq 'mistbrn:iron_flakes' "$dir/stage2.json"; then k_fail "the flake was not consumed"
      else k_ok "one flake consumed for three vials"; fi
      # A water vial with flakes is the wrong stage: nothing happens.
      p3c_fill 514 100 512 0 mistbrn:vial_water 1 3 mistbrn:tin_flakes 1
      ctl wait 620 >/dev/null
      p3c_contents 514 100 512 wrong-stage
      p3c_has wrong-stage mistbrn:vial_water 0
      p3c_has wrong-stage mistbrn:tin_flakes 3
      ctl save >/dev/null; quit_game
      # The first batch finished in launch 2, ~300 ticks after relaunch (not 600 from zero).
      expect_log 2 'MISTBRN event=alembic_batch pos=\[514, 100, 512\] ingredient=mistbrn:salt vials=3'
      expect_log 2 'MISTBRN event=alembic_batch pos=\[514, 100, 512\] ingredient=mistbrn:iron_flakes vials=3'
      if grep -aq 'vial_brine' "$dir/stage2-early.json" && ! grep -aq 'vial_iron' "$dir/stage2-early.json"; then
        k_ok "a batch takes 600 ticks (none at 590)"
      else k_fail "stage 2 finished before 600 ticks"; fi
      p3c_check "salt batches in launch 1" "$(p3c_lines 1 'event=alembic_batch')" 0 ;;
    a4)  # the basic-metals vial credits only unlocked metals, up to capacity; refused with none
      launch
      ctl command "gamemode survival" >/dev/null
      ctl give mistbrn:vial_basic_metals 2 >/dev/null
      ctl give mistbrn:vial_iron 1 >/dev/null
      ctl save >/dev/null; quit_game
      launch
      ctl select mistbrn:vial_basic_metals >/dev/null
      ctl use-item --hold 1.8 > "$dir/basic-none.json"
      ctl wait 10 >/dev/null
      ctl select mistbrn:vial_iron >/dev/null
      ctl use-item --hold 1.8 > "$dir/iron.json"
      ctl wait 10 >/dev/null
      ctl select mistbrn:vial_basic_metals >/dev/null
      ctl use-item --hold 1.8 > "$dir/basic.json"
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 2 'MISTBRN event=vial_refused metal=basic reason=NothingToCredit'
      expect_log 2 'MISTBRN event=vial_credit metal=iron seq=1 unlocked=true'
      expect_log 2 'MISTBRN event=basic_vial_credit credited=\[25000, 25000, 0, 0, 0, 0, 0, 0\]'
      expect_log 2 'MISTBRN event=vial_credit metal=basic seq=2'
      expect 3 report state=active reserve_iron=50000 reserve_steel=25000 reserve_tin=0 \
        last_transaction=2 basic_vials=1 empty_vials=2 ;;
    a5)  # the old flakes + empty vial recipe is gone; vials made before still drink
      launch
      ctl command "gamemode survival" >/dev/null
      ctl give mistbrn:vial_empty 1 >/dev/null
      ctl give mistbrn:iron_flakes 1 >/dev/null
      ctl give mistbrn:vial_iron 1 >/dev/null
      ctl wait 10 >/dev/null
      ctl open-inventory >/dev/null
      put mistbrn:vial_empty 100; put mistbrn:iron_flakes 101; take
      ctl close >/dev/null
      ctl wait 5 >/dev/null
      p3c_check "vials crafted from flakes" "$(p3c_count mistbrn:vial_iron)" 1
      ctl select mistbrn:vial_iron >/dev/null
      ctl use-item --hold 1.8 > "$dir/old-vial.json"
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=vial_credit metal=iron seq=1'
      expect 2 report state=active reserve_iron=25000 vials=0 last_transaction=1 ;;
    a6)  # salt rock generates on beach sand in new chunks
      local found x z total=0 n dx dz
      launch
      ctl command "locate biome minecraft:beach" > "$dir/locate.json"
      # "The nearest minecraft:beach is at [137, 69, 65] (...)"; Y may also read "~".
      found="$(grep -aoE '\[-?[0-9]+, (-?[0-9]+|~), -?[0-9]+\]' "$dir/locate.json" | head -1 || true)"
      x="$(sed -E 's/^\[(-?[0-9]+), [^,]+, (-?[0-9]+)\]$/\1/' <<< "$found")"
      z="$(sed -E 's/^\[(-?[0-9]+), [^,]+, (-?[0-9]+)\]$/\2/' <<< "$found")"
      [[ "$x" =~ ^-?[0-9]+$ && "$z" =~ ^-?[0-9]+$ ]] || die "no beach located: $(head -c 300 "$dir/locate.json")"
      k_ok "beach at $x,$z"
      ctl command "tp @s $x 90 $z" >/dev/null
      ctl wait 200 >/dev/null
      # /fill ... replace with the same block counts it (≤ 32768 blocks per call).
      for dx in -40 0 40; do
        for dz in -40 0 40; do
          ctl command "fill $((x + dx - 20)) 58 $((z + dz - 20)) $((x + dx + 19)) 77 $((z + dz + 19)) mistbrn:salt_rock replace mistbrn:salt_rock" \
            > "$dir/count-$dx-$dz.json"
          n="$(grep -aoE 'filled [0-9]+' "$dir/count-$dx-$dz.json" | grep -oE '[0-9]+' | head -1 || true)"
          total=$((total + ${n:-0}))
        done
      done
      ctl command "tp @s $x 75 $z" >/dev/null
      ctl look 0 60 >/dev/null
      ctl wait 40 >/dev/null
      p5_screenshot a6-beach
      quit_game
      if (( total >= 1 )); then k_ok "salt rock blocks near the beach: $total"
      else k_fail "no salt rock within 60 blocks of the beach at $x,$z"; fi ;;
    av)  # art capture: blocks in the world, the alembic screen mid-batch, items in the inventory
      local i
      p3c_lane
      ctl command "setblock 515 100 511 mistbrn:salt_rock" >/dev/null
      ctl command "setblock 515 100 512 mistbrn:alembic" >/dev/null
      ctl command "setblock 515 100 513 mistbrn:salt_rock" >/dev/null
      ctl command "setblock 515 101 512 mistbrn:alembic" >/dev/null
      for i in vial_empty vial_water vial_brine vial_basic_metals vial_iron salt salt_rock alembic; do
        ctl give "mistbrn:$i" 3 >/dev/null
      done
      ctl wait 20 >/dev/null
      ctl look-at block 515 100 512 west >/dev/null
      ctl wait 10 >/dev/null
      p5_screenshot av-world
      p3c_fill 515 100 512 0 mistbrn:vial_water 1 1 mistbrn:vial_water 1 3 mistbrn:salt 1
      # Survival shows the player's own inventory; an empty hand opens the block.
      ctl command "gamemode survival" >/dev/null
      ctl select 9 >/dev/null
      ctl wait 200 >/dev/null
      ctl interact 515 100 512 > "$dir/interact.json"
      ctl wait 20 >/dev/null
      ctl menu > "$dir/menu.json"
      p5_screenshot av-alembic
      ctl close >/dev/null
      ctl open-inventory >/dev/null
      ctl wait 10 >/dev/null
      p5_screenshot av-inventory
      ctl close >/dev/null
      quit_game ;;
    *) return 1 ;;
  esac
}
