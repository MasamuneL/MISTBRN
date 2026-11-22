# shellcheck shell=bash
# V1 primer and player path scenarios, sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# v1_craft_primer: craft the steel ingot and then the primer at a crafting table.
v1_craft_primer() {
  table
  put minecraft:iron_ingot 100; put minecraft:coal 101 102; take
  put minecraft:book 100; put minecraft:iron_ingot 101; put mistbrn:steel_ingot 102; take
  ctl close >/dev/null
}

# v1_use ITEM: finish one held use in Survival.
v1_use() {
  ctl select "$1" >/dev/null
  ctl use-item --hold 1.8 >/dev/null; ctl wait 10 >/dev/null
}

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

# v1_scenario: runs U1-U3 and V1; returns 1 for any other scenario name.
v1_scenario() {
  local server hud
  case "$scenario" in
    u1)  # Craft and use one primer; both unlocks survive a save and a relaunch.
      launch MISTBRN_TEST_SCENARIO= MISTBRN_TEST_FAIL_VIAL=0
      ctl command "gamemode survival" >/dev/null
      ctl give book 1 >/dev/null; ctl give iron_ingot 2 >/dev/null; ctl give coal 2 >/dev/null
      ctl wait 10 >/dev/null
      v1_craft_primer
      v1_use mistbrn:primer_iron_steel
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1[[:space:]]*$'
      expect 2 report state=active unlocked_iron=true unlocked_steel=true primers=0 last_transaction=1 burn_iron=off ;;
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
      ctl close >/dev/null
      v1_craft_primer
      # The table sits two blocks east of the player, in the pull lane.
      ctl command "setblock 514 100 512 minecraft:air" >/dev/null
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
