# shellcheck shell=bash
# V1 primer and player path scenarios, sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# v1_craft_primer: craft the steel ingot and then the primer at a crafting table.
v1_craft_primer() {
  local px py pz
  read -r px py pz < <(ctl state | jq -r '.player.position | map(floor) | "\(.[0]) \(.[1]) \(.[2])"')
  table
  put minecraft:iron_ingot 100; put minecraft:coal 101 102; take
  put minecraft:book 100; put minecraft:iron_ingot 101; put mistbrn:steel_ingot 102; take
  ctl close >/dev/null
  # Remove the table: a right-click use aimed at it would open it instead of using the item.
  ctl setblock $((px + 2)) "$py" "$pz" air >/dev/null
  ctl wait 10 >/dev/null
}

# v1_use ITEM: finish one held use in Survival.
v1_use() {
  local screen _
  # A use sent with any screen open never starts: close until the game is back to playing.
  for _ in 1 2 3; do
    screen="$(ctl state | jq -r '.screen | if . == null then "playing" elif type == "string" then . else (.kind // .name // tojson) end')"
    [[ "$screen" == playing ]] && break
    ctl close >/dev/null || true; ctl wait 10 >/dev/null
  done
  [[ "$screen" == playing ]] || k_fail "screen '$screen' still open before using $1"
  ctl select "$1" >/dev/null; ctl wait 5 >/dev/null
  ctl inventory > "$dir/before-use-$v1_uses.json" || true
  ctl use-item --hold 1.8 > "$dir/use-$v1_uses.json" || true
  ctl wait 10 >/dev/null
  v1_uses=$((v1_uses + 1))
}
v1_uses=0

# v1_pull_fixture: the K1 pull lane in the current world, with no test commands enabled.
v1_pull_fixture() {
  ctl command "tp @s 512.5 112 512.5" >/dev/null
  ctl wait-for loaded --timeout 120 >/dev/null
  # Far chunks of the box can load a little after the player's own: clear twice.
  ctl command "fill 503 100 508 540 126 516 minecraft:air" >/dev/null
  ctl wait 40 >/dev/null
  ctl command "fill 503 100 508 540 126 516 minecraft:air" >/dev/null
  ctl command "fill 503 99 508 540 99 516 minecraft:stone" >/dev/null
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 5 >/dev/null
}

# v1_scenario: runs U1-U5 and V1; returns 1 for any other scenario name.
v1_scenario() {
  local server hud
  case "$scenario" in
    u1)  # Craft and use one primer; both unlocks survive a save and a relaunch.
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=0
      # Cleared lane: spawn terrain (leaves) can hide the crafting table's faces. Let the player
      # settle after the teleport, or the table is aimed at from below the floor.
      v1_pull_fixture
      ctl wait 20 >/dev/null
      ctl command "gamemode survival" >/dev/null
      ctl give book 1 >/dev/null; ctl give iron_ingot 2 >/dev/null; ctl give coal 2 >/dev/null
      ctl wait 10 >/dev/null
      v1_craft_primer
      # Another pair selected first (a tap of V): studying the primer selects its own pair.
      k_key v 10
      v1_use mistbrn:primer_iron_steel
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1[[:space:]]*$'
      expect_log 1 'MISTBRN event=control op=cycle command=pair:tin_pewter allowed=true result=ok'
      # The HUD's pairs in order: tin/pewter after the tap, iron/steel after the primer, and
      # iron/steel again after the relaunch (the choice is saved with the unlock).
      local pairs
      pairs="$({ grep -a "MISTBRN hud pair=" "$(log 1)" || true; } | sed 's/.*pair=\([a-z_]*\).*/\1/' | uniq | tr '\n' ' ')"
      if [[ "$pairs" == *"tin_pewter iron_steel " && "$pairs" == "iron_steel tin_pewter iron_steel " ]]; then
        k_ok "HUD pairs: $pairs"
      else k_fail "HUD pairs '$pairs', expected 'iron_steel tin_pewter iron_steel'"; fi
      if [[ "$({ grep -a "MISTBRN hud pair=" "$(log 2)" || true; } | tail -1)" == *"pair=iron_steel "* ]]; then
        k_ok "iron/steel still selected after the relaunch"
      else k_fail "the relaunch lost the selected pair"; fi
      expect 2 report state=active unlocked_iron=true unlocked_steel=true primers=0 last_transaction=1 burn_iron=off \
        reserve_iron=0 ;;
    u2)  # A second primer refuses an unlocked pair and stays in the inventory.
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=0
      ctl command "gamemode survival" >/dev/null
      ctl give mistbrn:primer_iron_steel 2 >/dev/null; ctl wait 10 >/dev/null
      v1_use mistbrn:primer_iron_steel
      v1_use mistbrn:primer_iron_steel
      ctl save >/dev/null; quit_game
      report; quit_game
      k_expect_count "one unlock" "MISTBRN event=unlock pair=iron_steel " 1 1
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1[[:space:]]*$'
      expect_log 1 'MISTBRN event=unlock_refused pair=iron_steel reason=AlreadyUnlocked[[:space:]]*$'
      expect 2 report state=active primers=1 last_transaction=1 unlocked_steel=true ;;
    u3)  # A failed unlock rolls back the item and both metals.
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=1
      ctl command "gamemode survival" >/dev/null
      ctl give mistbrn:primer_iron_steel 1 >/dev/null; ctl wait 10 >/dev/null
      v1_use mistbrn:primer_iron_steel
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=unlock_refused pair=iron_steel reason=injected_failure[[:space:]]*$'
      expect 2 report state=active primers=1 unlocked_iron=false unlocked_steel=false last_transaction=0 ;;
    u4|u5)  # Hard cut after a primer: U4 without a save (old or new state, never half),
      # U5 after a save (the new state).
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=0
      ctl command "gamemode survival" >/dev/null
      ctl give mistbrn:primer_iron_steel 1 >/dev/null; ctl wait 10 >/dev/null
      # Saved with the primer first (as C3): the cut can only return to "primer, locked".
      ctl save >/dev/null
      v1_use mistbrn:primer_iron_steel
      if [[ "$scenario" == u5 ]]; then ctl save >/dev/null; fi
      kill_game
      report; quit_game
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1[[:space:]]*$'
      local after
      after="$(field 2 report primers) $(field 2 report unlocked_iron) $(field 2 report unlocked_steel) $(field 2 report last_transaction)"
      case "$scenario:$after" in
        u4:"1 false false 0"|u4:"0 true true 1"|u5:"0 true true 1") k_ok "after the cut: primers unlocked_iron unlocked_steel last_transaction = $after" ;;
        *) k_fail "after the cut: primers unlocked_iron unlocked_steel last_transaction = '$after'" ;;
      esac ;;
    v1)  # Craft from raw ingredients, drink, study, pull with Z and put iron out with G.
      # Glass makes two empties: one is filled and drunk (it comes back), one stays spare.
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=0
      ctl command "difficulty peaceful" >/dev/null
      v1_pull_fixture
      ctl command "gamemode survival" >/dev/null
      ctl give glass 2 >/dev/null; ctl give iron_ingot 3 >/dev/null; ctl give cobblestone 1 >/dev/null
      ctl give coal 2 >/dev/null; ctl give book 1 >/dev/null; ctl wait 10 >/dev/null
      ctl open-inventory >/dev/null
      put minecraft:glass 100; put minecraft:glass 102; take
      put minecraft:iron_ingot 100; put minecraft:cobblestone 101; take
      put mistbrn:vial_empty 100; put mistbrn:iron_flakes 101; take
      # The guide's path: steel ingot and primer are three-ingredient shapeless recipes (2x2).
      put minecraft:iron_ingot 100; put minecraft:coal 101 102; take
      put minecraft:book 100; put minecraft:iron_ingot 101; put mistbrn:steel_ingot 102; take
      ctl close >/dev/null
      ctl wait 10 >/dev/null
      v1_use mistbrn:vial_iron
      v1_use mistbrn:primer_iron_steel
      p4_iron_block 524
      k_key z 60
      if ctl --timeout 30 screenshot "$dir/v1-hud.png" >/dev/null && [[ -s "$dir/v1-hud.png" ]]; then
        k_ok "screenshot $dir/v1-hud.png (local, not versioned)"
      else k_fail "screenshot failed (no rendered frame?)"; fi
      k_key g 20
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=vial_credit metal=iron seq=1[[:space:]]*$'
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=2[[:space:]]*$'
      # Exact fuel before any burn: the HUD sent on the unlock shows the whole vial.
      expect_log 1 'MISTBRN hud pair=iron_steel iron=25000/off steel=0/off'
      # Keys need the game window focused (client playing() && window_focused()).
      if grep -aq "MISTBRN event=key .* result=blocked" "$(log 1)"; then
        k_fail "keys blocked: the game window lost focus (or a screen was open) during the run"
      fi
      k_expect_commands activate:iron=ok off:all=ok
      k_pull_check
      expect 2 report state=active burn_iron=off burn_steel=off unlocked_iron=true unlocked_steel=true \
        vials=0 primers=0 empty_vials=2 last_transaction=2
      server="$(field 2 report reserve_iron)"; hud="$(k_hud_last iron)"
      check_range "reserve consumed but not empty" "$server" 0 25000
      if [[ -n "$hud" && "$hud" == "$server" ]]; then k_ok "HUD reserve $hud = server $server"
      else k_fail "HUD reserve '$hud' != server '$server'"; fi ;;
    *) return 1 ;;
  esac
}
