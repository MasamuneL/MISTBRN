# shellcheck shell=bash
# P5 native scenarios, sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# p5_lane [p4]: the V1 lane, with test commands enabled only where requested.
p5_lane() {
  launch MISTBRN_TEST_SCENARIO="${1:-}" MISTBRN_TEST_FAIL_VIAL=0
  ctl command "difficulty peaceful" >/dev/null
  v1_pull_fixture
  ctl command "gamemode survival" >/dev/null
  ctl look -90 -90 >/dev/null
}

p5_unlock() {
  ctl give mistbrn:vial_iron 1 >/dev/null
  v1_use mistbrn:vial_iron
}

p5_pull() {
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 5 >/dev/null
  ctl look -90 0 >/dev/null
  k_key z 60
  ctl state > "$dir/$1.json"
  k_key g 10
  jq -e '.player.position[0] >= 514.5' "$dir/$1.json" >/dev/null ||
    k_fail "$1 did not move at least two blocks"
}

p5_screenshot() {
  ctl --timeout 30 screenshot "$dir/$1.png" >/dev/null
  [[ -s "$dir/$1.png" ]] || die "missing screenshot $dir/$1.png"
}

# Opening via J starts on the tree (B is Horizon's emote wheel). Keep the frame for both evidence and the GUI dimensions.
p5_tree() {
  k_key j 10
  k_screen "$1" '^"mod"$'
  p5_screenshot "$1"
}

# P7b: the book's targets are clicked by name (`book_click`, tools/p7b-lib.sh).
p5_select_table() {
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 5 >/dev/null
  ctl setblock 514 100 512 mistbrn:alembic >/dev/null
  ctl look-at block 514 100 512 west >/dev/null
  ctl wait 5 >/dev/null
}

# p5_conclude NAME: the book's Conclude button, looking at the alembic (P7b, P3c). For a repeat
# (NAME ending in -repeat) the diary must no longer offer it; L3 checks the server's refusal.
p5_conclude() {
  book_open "$1-open"
  book_click tab.diary
  if [[ "$1" == *-repeat ]]; then
    if [[ "$(book_line)" == *"conclude@"* ]]; then k_fail "$1: Conclude still offered"
    else k_ok "$1: no Conclude once concluded"; fi
  else
    book_click conclude
  fi
  k_key esc 10
  k_screen "$1" '^"playing"$'
}

p5_trial_reach() {
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 5 >/dev/null
  ctl look -90 0 >/dev/null
  ctl command "mistbrn-test burn iron" >/dev/null
  ctl wait 40 >/dev/null
  p4_off
  ctl wait 10 >/dev/null
}

# Two identical purchase requests queued in one call, so they reach the server in the same
# tick (`inject buy.<code>.<rank>.<count>`), as a double click would.
p5_repeat_reach() {
  ctl command "mistbrn-test inject buy.1.1.2" > "$dir/replay-command.json"
  ctl wait 10 >/dev/null
}

# Read the schema from the on-disk ByteArray tag (type 7, name "mistbrn", length, "MBRN",
# then the little-endian u16). The field is read before the copied world can be opened.
p5_disk_schema() {
  gzip -dc "$1" | od -An -v -tx1 | LC_ALL=C awk '
    { for (i = 1; i <= NF; i++) bytes = bytes " " $i }
    END {
      tag = " 07 00 07 6d 69 73 74 62 72 6e( [0-9a-f][0-9a-f]){4} 4d 42 52 4e"
      if (!match(bytes, tag)) exit 1
      split(substr(bytes, RSTART + RLENGTH + 1, 5), schema, " ")
      if (schema[2] != "00" || (schema[1] != "01" && schema[1] != "02" && schema[1] != "03")) exit 1
      print schema[1] + 0
    }'
}

p5_migrate() {
  local src="$RT/runs/v1-12-u1" uuid player before after
  [[ -d "$src/saves/mistbrn-u1" && -s "$src/horizon-player.txt" && -f "$src/l2.log" ]] ||
    die "incomplete V1 migration fixture: $src"
  uuid="$(sed -n 's/^MISTBRN_TEST scenario=report .*last_transaction=1 uuid=\([0-9a-f-]*\) .*/\1/p' "$src/l2.log")"
  [[ "$uuid" =~ ^[0-9a-f-]{36}$ ]] || die "no V1 player UUID in $src/l2.log"
  mkdir -p "$dir/saves"
  cp -R "$src/saves/mistbrn-u1" "$dir/saves/$world"
  cp "$src/horizon-player.txt" "$dir/horizon-player.txt"
  player="$dir/saves/$world/players/data/$uuid.dat"
  before="$(p5_disk_schema "$player")" || die "could not read schema 1 from $player"
  [[ "$before" == 1 ]] || die "expected raw schema 1, got $before"
  k_ok "on-disk schema before launch: $before (uuid $uuid)"
  report; ctl save >/dev/null; quit_game
  expect_log 1 'MISTBRN event=save_loaded state=found'
  expect 1 report state=active "uuid=$uuid" schema=3 unlocked_iron=true unlocked_steel=true \
    last_transaction=1 xp_iron_steel=0 nodes=- research=- pair_points=0 knowledge=0
  after="$(p5_disk_schema "$player")" || die "could not read the schema from $player"
  if [[ "$after" == 3 ]]; then k_ok "on-disk schema after save: $after"
  else k_fail "expected raw schema 3 after save, got $after"; fi
}

# p5_scenario: returns 1 for any other scenario name.
p5_scenario() {
  case "$scenario" in
    t1)
      p5_lane
      ctl give mistbrn:vial_iron 2 >/dev/null
      k_key v 10
      v1_use mistbrn:vial_iron
      ctl save >/dev/null; quit_game
      report
      ctl look -90 -90 >/dev/null
      v1_use mistbrn:vial_iron
      ctl give mistbrn:vial_tin 1 >/dev/null
      v1_use mistbrn:vial_tin
      k_key v 10; k_key z 10
      for item in book book iron_ingot mistbrn:steel_ingot iron_nugget; do ctl give "$item" 1 >/dev/null; done
      ctl wait 10 >/dev/null
      ctl open-inventory >/dev/null
      put minecraft:book 100; put minecraft:iron_ingot 101; put mistbrn:steel_ingot 102
      ctl wait 5 >/dev/null
      ctl screen > "$dir/primer-result.json"
      ctl close >/dev/null; ctl wait 5 >/dev/null
      ctl open-inventory >/dev/null
      put minecraft:book 100; put minecraft:iron_nugget 101
      ctl wait 5 >/dev/null
      ctl screen > "$dir/notebook-result.json"
      ctl close >/dev/null; ctl wait 5 >/dev/null
      ctl inventory > "$dir/notebook-inventory.json"
      quit_game
      expect_log 1 'MISTBRN event=control op=cycle command=pair:tin_pewter allowed=true result=ok'
      expect_log 1 'MISTBRN event=vial_credit metal=iron seq=1 unlocked=true'
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1 via=vial'
      expect 2 report state=active unlocked_iron=true unlocked_steel=true selected_pair=iron_steel \
        last_transaction=1 vials=1 reserve_iron=25000
      expect_log 2 'MISTBRN event=vial_credit metal=iron seq=2 unlocked=false'
      expect_log 2 'MISTBRN event=vial_credit metal=tin seq=3 unlocked=false'
      expect_log 2 'MISTBRN event=control op=toggle_first command=activate:tin allowed=true result=MetalLocked(Tin)'
      jq -e '(.slots | map(select(.slot == 200 and has("item") and .item == null))) | length == 1' \
        "$dir/primer-result.json" >/dev/null || k_fail "the old primer result slot was not empty"
      # P7b: the notebook has no recipe; the book and the nugget go back to the inventory.
      jq -e '(.slots | map(select(.slot == 200 and has("item") and .item == null))) | length == 1' \
        "$dir/notebook-result.json" >/dev/null || k_fail "the old notebook result slot was not empty"
      jq -e '(.counts["mistbrn:notebook"] // 0) == 0 and (.counts["mistbrn:primer_iron_steel"] // 0) == 0
             and .counts["minecraft:book"] == 2 and .counts["minecraft:iron_nugget"] == 1' \
        "$dir/notebook-inventory.json" >/dev/null || k_fail "expected no notebook, two books and the nugget" ;;
    x1)
      local run_no
      p5_lane; p5_unlock
      p4_iron_block 524
      for run_no in 1 2 3 4; do p5_pull "pull-$run_no"; done
      ctl save >/dev/null; quit_game
      # The window is saved: after the relaunch a 5th pull still gives nothing; once 1200
      # game ticks have passed since the first pull, a 6th gives the full 10 again.
      report
      p5_pull pull-5
      ctl wait 1200 >/dev/null
      p5_pull pull-6
      ctl save >/dev/null; quit_game
      expect_log 1 'MISTBRN event=unlock pair=iron_steel seq=1 via=vial'
      k_expect_count "four block acquisitions" 'MISTBRN event=target_acquired kind=block id=minecraft:iron_block ' 4 4
      if awk '
        /MISTBRN event=control .* command=activate:iron allowed=true result=ok / { run++ }
        /MISTBRN event=practice category=anchor_move / {
          count[run]++
          if ((run == 1 && $0 ~ / xp=10 count=1 total_xp=10 /) ||
              (run == 2 && $0 ~ / xp=5 count=2 total_xp=15 /) ||
              (run == 3 && $0 ~ / xp=2 count=3 total_xp=17 /)) rewarded++
        }
        END { exit !(run == 4 && rewarded == 3 && !count[0] &&
                     count[1] == 1 && count[2] == 1 && count[3] == 1 && !count[4]) }
      ' "$(log 1)"; then k_ok "practice per pull: 10, 5, 2, no award"
      else k_fail "expected one practice award per pull (10, 5, 2) and none on pull four"; fi
      expect 2 report state=active xp_iron_steel=17
      # Each award belongs to the pull whose lighting precedes it: none for pull 5, 10 for pull 6.
      if awk '
        /MISTBRN event=control .* command=activate:iron allowed=true result=ok / { run++ }
        /MISTBRN event=practice category=anchor_move / { count[run]++; if (run == 2) last = $0 }
        END { exit !(run == 2 && !count[0] && !count[1] && count[2] == 1 &&
                     last ~ / xp=10 count=1 total_xp=27 /) }
      ' "$(log 2)"; then k_ok "after the relaunch: pull 5 nothing (saved window), pull 6 10 (new window)"
      else k_fail "expected no award for pull 5 and 10 XP for pull 6"; fi ;;
    r1)
      p5_lane; p5_unlock
      p4_iron_block 524
      p5_pull block-pull
      ctl save >/dev/null; quit_game
      report
      # N4's actual ingot pull, with the block removed so the new target cannot be an anchor.
      ctl command "tp @s 512.5 100 512.5" >/dev/null
      ctl setblock 524 101 512 air >/dev/null
      ctl command 'summon item 518.5 100 512.5 {Item:{id:"minecraft:iron_ingot",count:1},PickupDelay:0}' >/dev/null
      ctl look-at 518.5 100.125 512.5 >/dev/null
      k_key z 65; k_key g 10
      ctl inventory > "$dir/pulled-ingot.json"
      p5_tree r1-tree
      book_click tab.diary
      p5_screenshot r1-diary
      book_click reveal
      k_key esc 10
      p5_select_table
      p5_conclude conclude-first
      p5_conclude conclude-repeat
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=evidence research=anchor_principle bit=2 ready=false'
      expect 2 report state=active research=1:e2h0c0 xp_iron_steel=10
      expect_log 2 'MISTBRN event=target_acquired kind=item id=minecraft:iron_ingot '
      expect_log 2 'MISTBRN event=evidence research=anchor_principle bit=1 ready=true'
      jq -e '.counts["minecraft:iron_ingot"] == 1' "$dir/pulled-ingot.json" >/dev/null ||
        k_fail "expected to pick up the pulled ingot"
      expect_log 2 'MISTBRN event=hint_revealed research=anchor_principle index=1 result=ok seq=2'
      # One conclusion (+200); the book offers no second one (P7b).
      if grep -a 'MISTBRN event=conclude research=anchor_principle' "$(log 2)" | awk '
        NR == 1 && / result=ok seq=3/ { first = 1 }
        END { exit !(NR == 1 && first) }'; then k_ok "one conclusion"
      else k_fail "expected exactly one conclusion, seq 3"; fi
      # Exactly once: the block pull's 10 plus the conclusion's 200 (never twice).
      expect 3 report state=active research=1:e3h1c1 knowledge=3 last_transaction=3 xp_iron_steel=210 ;;
    b1)
      local target='MISTBRN event=target_acquired kind=block id=minecraft:iron_block at=529.500,101.500,512.500'
      local reserve fill
      p5_lane p4; p5_unlock
      ctl command "mistbrn-test xp 100" >/dev/null
      p4_iron_block 529
      p5_trial_reach
      ctl save >/dev/null; quit_game
      report; quit_game
      launch MISTBRN_TEST_SCENARIO=p4
      p5_tree b1-tree
      book_click node.1
      k_key esc 10
      k_screen b1-closed '^"playing"$'
      p5_trial_reach
      p5_repeat_reach
      ctl command "mistbrn-test inject buy.2.1" >/dev/null; ctl wait 10 >/dev/null
      ctl command "mistbrn-test evidence 1" >/dev/null
      ctl command "mistbrn-test evidence 2" >/dev/null
      p5_select_table; p5_conclude b1-conclude
      ctl command "mistbrn-test inject buy.17.1" >/dev/null; ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=test_xp granted=100 result=ok'
      expect_log 1 'MISTBRN event=burn metal=iron burn=burning result=ok changed=1'
      k_expect_count "base range missed the 17-block target" 'MISTBRN event=target_acquired ' 0 0
      expect 2 report state=active xp_iron_steel=100 pair_points=1 nodes=- last_transaction=1
      expect_log 3 'MISTBRN event=notebook result=request:Purchase { code: 1, rank: 1 }'
      expect_log 3 'MISTBRN event=purchase node=reach rank=1 result=ok seq=2'
      grep -aFq -- "$target" "$(log 3)" || k_fail "reach 1 did not acquire the original target"
      [[ "$(grep -aFc 'MISTBRN event=purchase node=reach rank=1 result=ok ' "$(log 3)" || true)" == 1 ]] ||
        k_fail "expected exactly one successful reach 1 purchase"
      [[ "$(grep -aFc 'MISTBRN event=purchase node=reach rank=1 result=AlreadyOwned' "$(log 3)" || true)" == 2 ]] ||
        k_fail "expected both same-tick replays to return AlreadyOwned"
      expect_log 3 'MISTBRN event=purchase node=accel rank=1 result=NotEnoughPoints'
      expect_log 3 'MISTBRN event=conclude research=anchor_principle result=ok seq=3'
      expect_log 3 'MISTBRN event=purchase node=capacity rank=1 result=ok seq=4'
      expect 4 report state=active nodes=1:1,17:1 research=1:e3h0c1 knowledge=2 capacity_iron=110000 last_transaction=4
      # The HUD log exposes reserve, not capacity. Fill to the reported cap, reject one
      # more milli, and check the HUD displays the resulting full reserve.
      reserve="$(field 4 report reserve_iron)"
      if [[ ! "$reserve" =~ ^[0-9]+$ ]] || (( reserve >= 110000 )); then
        die "invalid iron reserve for the capacity probe: '$reserve'"
      fi
      fill=$((110000 - reserve))
      launch MISTBRN_TEST_SCENARIO=p4
      ctl command "mistbrn-test credit iron $fill" >/dev/null; ctl wait 15 >/dev/null
      ctl command "mistbrn-test credit iron 1" >/dev/null; ctl wait 15 >/dev/null
      quit_game
      expect_log 5 "MISTBRN event=test_credit metal=iron milli=$fill result=ok"
      expect_log 5 'MISTBRN event=test_credit metal=iron milli=1 result=InvalidReserves(CapacityExceeded(Iron))'
      expect_log 5 'MISTBRN hud pair=iron_steel iron=110000/off' ;;
    b2|b3)
      local after
      p5_lane p4; p5_unlock
      ctl command "mistbrn-test xp 100" >/dev/null
      ctl save >/dev/null
      ctl command "mistbrn-test inject buy.1.1" >/dev/null; ctl wait 10 >/dev/null
      if [[ "$scenario" == b3 ]]; then ctl save >/dev/null; fi
      kill_game
      report; quit_game
      expect_log 1 'MISTBRN event=purchase node=reach rank=1 result=ok seq=2'
      expect 2 report state=active schema=3 unlocked_iron=true unlocked_steel=true xp_iron_steel=100
      after="$(field 2 report nodes) $(field 2 report pair_points) $(field 2 report last_transaction)"
      case "$scenario:$after" in
        b2:"- 1 1"|b2:"1:1 0 2"|b3:"1:1 0 2") k_ok "after cut (nodes, points, transaction): $after" ;;
        *) k_fail "inconsistent state after cut (nodes, points, transaction): '$after'" ;;
      esac ;;
    pd1)
      local key before
      local progress='MISTBRN progress xp_iron_steel=100 nodes=1:1 research=1:e2h0c0'
      p5_lane p4; p5_unlock
      ctl give mistbrn:vial_steel 1 >/dev/null; v1_use mistbrn:vial_steel
      ctl command "mistbrn-test xp 100" >/dev/null
      ctl command "mistbrn-test inject buy.1.1" >/dev/null; ctl wait 10 >/dev/null
      ctl command "mistbrn-test evidence 2" >/dev/null
      ctl save >/dev/null; quit_game
      report
      ctl command "kill @s" >/dev/null; ctl wait 20 >/dev/null
      ctl respawn >/dev/null; ctl wait 40 >/dev/null
      k_screen d1-respawned '^"playing"$'
      ctl save >/dev/null; quit_game
      report; quit_game
      expect 2 report state=active reserve_iron=25000 xp_iron_steel=100 nodes=1:1 research=1:e2h0c0 last_transaction=3
      expect_log 2 'MISTBRN hud pair=iron_steel iron=25000/off steel=25000/off'
      expect_log 2 'MISTBRN event=death_cleared'
      expect_log 2 'MISTBRN event=respawn_death_applied'
      expect_log 2 'MISTBRN hud pair=iron_steel iron=0/off steel=0/off'
      awk -v progress="$progress" '
        /MISTBRN event=respawn_death_applied/ { respawned = 1 }
        index($0, progress) { if (respawned) after = 1; else before = 1 }
        END { exit !(before && after) }
      ' "$(log 2)" || k_fail "expected matching progress snapshots before death and after respawn"
      expect 3 report state=active reserve_iron=0 burn_iron=off burn_steel=off unlocked_iron=true unlocked_steel=true
      expect_log 3 'MISTBRN hud pair=iron_steel iron=0/off steel=0/off'
      expect_log 3 "$progress"
      for key in xp_iron_steel pair_points knowledge nodes research windows selected_pair capacity_iron reduction last_transaction; do
        before="$(field 2 report "$key")"
        [[ -n "$before" ]] || k_fail "missing pre-death $key"
        expect 3 report "$key=$before"
      done ;;
    m1)
      p5_migrate ;;
    q1)
      launch MISTBRN_TEST_SCENARIO=
      p5_tree q1-tree
      book_click tab.diary
      p5_screenshot q1-diary
      k_key esc 10
      k_screen q1-closed '^"playing"$'
      # J opens and a second J closes (its press; releasing the opening J does nothing).
      k_key j 10
      k_screen q1-reopened '^"mod"$'
      k_key j 10
      k_screen q1-closed-by-j '^"playing"$'
      k_key t 10
      k_screen q1-chat 'chat'
      k_key j 10
      k_screen q1-chat-after-j 'chat'
      k_key esc 10
      quit_game
      k_expect_count "two notebook openings" 'MISTBRN event=notebook result=open' 2 2
      k_expect_count "two notebook closings" 'MISTBRN event=notebook result=closed' 2 2
      k_expect_count "one diary tab click" 'MISTBRN event=notebook_click ' 1 1
      # As in K4: the open chat takes the key, so the mapping sees no press at all.
      k_expect_count "J ignored with the chat open" 'MISTBRN event=key key=notebook' 2 2 ;;
    *) return 1 ;;
  esac
}
