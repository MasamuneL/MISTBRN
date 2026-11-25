# shellcheck shell=bash
# P7b native scenarios (the book), sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# book_line: the newest `event=book_view` line of the running launch (its live log).
book_line() {
  grep -a 'MISTBRN event=book_view ' "$sockdir/l$launch_no.log" 2>/dev/null | tail -1 || true
}

# book_at NAME: "X Y", the GUI pixel where the newest book_view line puts target NAME
# (`tab.diary`, `view.4`, `node.1`, `conclude`); status 1 when it is not there.
book_at() {
  local at
  at="$(tr ';= ' '\n\n\n' <<< "$(book_line)" | sed -n "s/^$1@//p" | tail -1)"
  [[ "$at" =~ ^[0-9.-]+,[0-9.-]+$ ]] || return 1
  echo "${at%,*} ${at#*,}"
}

# book_xy NAME: book_at, or a FAIL counted in the scenario's own shell (not a subshell).
book_xy() {
  local xy
  ctl wait 5 >/dev/null
  if xy="$(book_at "$1")"; then echo "$xy"; return 0; fi
  return 1
}

# book_click NAME: click book target NAME.
book_click() {
  local xy
  if ! xy="$(book_xy "$1")"; then k_fail "book target $1 not in: $(book_line | head -c 300)"; return 0; fi
  ctl click-at "${xy% *}" "${xy#* }" >/dev/null
  ctl wait 10 >/dev/null
}

# book_scroll NAME STEPS: turn the wheel over book target NAME (positive scrolls up); with
# the canvas at its top, one notch up moves nothing and only points there (a tooltip).
book_scroll() {
  local xy
  if ! xy="$(book_xy "$1")"; then k_fail "book target $1 not in: $(book_line | head -c 300)"; return 0; fi
  ctl scroll-at "${xy% *}" "${xy#* }" "$2" >/dev/null
  ctl wait 5 >/dev/null
}

# book_open NAME: J opens the book on its trees; the capture NAME shows it.
book_open() {
  k_key j 10
  k_screen "$1" '^"mod"$'
  p5_screenshot "$1"
}

# book_shot NAME: a capture of the book as it is now.
book_shot() { ctl wait 5 >/dev/null; p5_screenshot "$1"; }

# book_quiet: no tutorial toast over the book's top right (captures only; vanilla option).
book_quiet() { ctl options-set tutorialStep none >/dev/null || true; }

# book_pair: iron and steel unlocked (test credits), with 100 XP so reach can be bought.
book_pair() {
  ctl command "mistbrn-test credit iron 30000" >/dev/null
  ctl command "mistbrn-test credit steel 30000" >/dev/null
  ctl command "mistbrn-test xp 100" >/dev/null
  ctl wait 20 >/dev/null
}

# book_table: a crafting table two blocks east of the player at 512.5 100 512.5, looked at.
book_table() {
  p6_place
  ctl setblock 514 100 512 crafting_table >/dev/null
  ctl look-at block 514 100 512 west >/dev/null
  ctl wait 5 >/dev/null
}

# p7b_scenario: returns 1 for any other scenario name.
p7b_scenario() {
  case "$scenario" in
    l1)  # every view, the canvas scroll, a tooltip, the tabs (captures reviewed by hand)
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      book_pair
      book_open l1-iron
      book_click view.1; book_shot l1-tin
      book_click view.4; book_shot l1-general
      book_click view.0
      book_scroll node.2 -5
      book_shot l1-iron-scrolled
      book_scroll node.3 10
      # Reach's tooltip: a click would buy it, so the pointer goes there with the wheel.
      book_scroll node.1 1
      book_shot l1-tooltip
      book_click tab.diary; book_shot l1-diary
      book_click tab.milestones; book_shot l1-milestones
      book_click tab.reserves; book_shot l1-reserves
      k_key esc 10
      quit_game
      for view in 0 1 4; do
        expect_log 1 "MISTBRN event=book_view gui=[0-9.]*x[0-9.]* tab=tree view=$view "
      done
      expect_log 1 'MISTBRN event=book_scroll lines=-1 scroll=56,0'
      expect_log 1 'MISTBRN event=book_scroll lines=1 scroll=0,0'
      for tab in diary milestones reserves; do expect_log 1 "MISTBRN event=book_view .* tab=$tab "; done
      k_expect_count "no purchase from looking" 'MISTBRN event=purchase ' 0 0 ;;
    l2)  # keyboard only: Tab to reach, Enter buys one rank, a held Enter buys nothing more
         # (Page Down has no horizon-ctl key name; its mapping is a unit test)
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      book_pair
      # 450 XP: three points, enough for reach 1 (cost 1) and reach 2 (cost 2).
      ctl command "mistbrn-test xp 350" >/dev/null; ctl wait 10 >/dev/null
      book_open l2-open
      # Focus order: 4 tabs, 5 trees, then reach (the innate power is not focusable).
      for _ in 1 2 3 4 5 6 7 8 9 10; do ctl key tab tap >/dev/null; ctl wait 2 >/dev/null; done
      book_shot l2-focus
      ctl key enter tap >/dev/null; ctl wait 10 >/dev/null
      ctl key enter repeat >/dev/null; ctl wait 20 >/dev/null
      book_shot l2-held
      ctl key enter tap >/dev/null; ctl wait 10 >/dev/null
      book_shot l2-second
      k_key esc 10
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=book_key key=Next repeat=false focus=node.1 '
      expect_log 1 'MISTBRN event=book_key key=Activate repeat=true focus=node.1 '
      expect_log 1 'MISTBRN event=purchase node=reach rank=1 result=ok'
      expect_log 1 'MISTBRN event=purchase node=reach rank=2 result=ok'
      # Two taps, two requests, and none between the held Enter's repeats and the second tap.
      k_expect_count "one request per tap" 'MISTBRN event=notebook result=request:Purchase' 2 2
      if awk '/book_key key=Activate repeat=true/ { held = 1 }
              held && /request:Purchase/ { bad = 1; exit }
              held && /book_key key=Activate repeat=false/ { done = 1; exit }
              END { exit !(done && !bad) }' "$(log 1)"; then k_ok "the held Enter sent nothing"
      else k_fail "a request between the held Enter and the second tap"; fi
      expect 2 report state=active nodes=1:2 ;;
    l3)  # Conclude: refused in creative, without a table in sight, beyond 5 blocks (also from
         # the book, which then says why) and behind a block; once at the table
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      book_pair
      ctl command "mistbrn-test evidence 2" >/dev/null
      ctl command "mistbrn-test evidence 4" >/dev/null
      book_table
      ctl command "mistbrn-test inject conclude.1.0" >/dev/null; ctl wait 10 >/dev/null
      ctl command "gamemode survival" >/dev/null
      ctl look -90 -90 >/dev/null; ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject conclude.1.0" >/dev/null; ctl wait 10 >/dev/null
      # 5.5 blocks from the table's face, still on the stone floor (it starts at x 508).
      ctl command "tp @s 508.5 100 512.5" >/dev/null; ctl wait 10 >/dev/null
      # The table's west face centre, as a point: `look-at block` needs it within reach.
      ctl look-at 514.0 100.5 512.5 >/dev/null; ctl wait 5 >/dev/null
      book_open l3-far
      book_click tab.diary
      book_click conclude; ctl wait 30 >/dev/null
      book_shot l3-refused
      k_key esc 10
      # Glass between the eyes and the table: the ray stops there.
      book_table
      ctl setblock 513 100 512 glass >/dev/null; ctl setblock 513 101 512 glass >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject conclude.1.0" >/dev/null; ctl wait 10 >/dev/null
      ctl setblock 513 100 512 air >/dev/null; ctl setblock 513 101 512 air >/dev/null
      book_table
      book_open l3-open
      book_click tab.diary; book_shot l3-ready
      book_click conclude; book_shot l3-concluded
      k_key esc 10
      ctl command "mistbrn-test inject conclude.1.0" >/dev/null; ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      k_expect_count "refused in creative" 'MISTBRN event=control op=conclude research=1 result=GameMode' 1 1
      k_expect_count "three refusals without a table in reach" \
        'MISTBRN event=control op=conclude research=1 result=NoTable' 3 3
      expect_log 1 'MISTBRN event=notebook result=conclude_refused'
      k_expect_count "two requests from the book" 'MISTBRN event=notebook result=request:Conclude { research: 1 }' 2 2
      expect_log 1 'MISTBRN event=conclude research=anchor_principle result=AlreadyConcluded'
      k_expect_count "one conclusion" 'MISTBRN event=conclude research=anchor_principle result=ok' 1 1
      expect 2 report state=active research=1:e6h0c1 knowledge=3 xp_iron_steel=300 ;;
    l4)  # settings: book at 125 % and 75 %, bars at 125 %, reduce flashing holds the warning
      local pulsing steady
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      ctl command "mistbrn-test credit iron 5000" >/dev/null
      ctl command "mistbrn-test credit steel 5000" >/dev/null
      ctl command "mistbrn-test setting book_scale 2" >/dev/null
      book_open l4-book-125; k_key esc 10
      ctl command "mistbrn-test setting book_scale 0" >/dev/null
      book_open l4-book-75; k_key esc 10
      ctl command "mistbrn-test setting hud_scale 2" >/dev/null
      ctl command "mistbrn-test burn iron" >/dev/null; ctl wait 10 >/dev/null
      book_shot l4-hud-125
      ctl command "mistbrn-test setting hud_scale 1" >/dev/null; ctl wait 10 >/dev/null
      book_shot l4-pulse; ctl wait 40 >/dev/null
      ctl command "mistbrn-test setting reduce_flashes 1" >/dev/null; ctl wait 40 >/dev/null
      book_shot l4-steady
      p4_off
      for key in book_scale hud_scale reduce_flashes; do
        ctl command "mistbrn-test setting $key $([[ $key == reduce_flashes ]] && echo 0 || echo 1)" >/dev/null
      done
      quit_game
      expect_log 1 'MISTBRN event=test_setting key=book_scale value=2 result=true'
      # 125 % does not fit a 427x240 GUI: the book takes the largest scale that does (> 1).
      expect_log 1 'MISTBRN event=book_view .* scale=1\.[0-9]* '
      expect_log 1 'MISTBRN event=book_view .* scale=0.75 '
      expect_log 1 'MISTBRN event=hud_box .* scale=1.25 '
      # The warning's strength as drawn (test runs log each change): several values while it
      # pulses, one and only one with reduce flashing.
      pulsing="$(grep -a 'MISTBRN event=hud_warning alpha=[0-9]* steady=false' "$(log 1)" | sort -u | wc -l)"
      steady="$(grep -a 'MISTBRN event=hud_warning alpha=[0-9]* steady=true' "$(log 1)" | sort -u)"
      if (( pulsing >= 3 )); then k_ok "the low reserve warning pulses ($pulsing strengths)"
      else k_fail "expected the warning to pulse, saw $pulsing strengths"; fi
      if [[ "$steady" == 'MISTBRN event=hud_warning alpha=84 steady=true' ]]; then
        k_ok "reduce flashing: the warning holds at 84"
      else k_fail "expected one steady strength (84), saw: ${steady:-none}"; fi ;;
    l5)  # the Reserves tab against the server's reserves
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      ctl command "mistbrn-test credit iron 30000" >/dev/null
      ctl command "mistbrn-test credit tin 12500" >/dev/null
      ctl command "mistbrn-test burn iron" >/dev/null; ctl wait 10 >/dev/null
      book_open l5-open
      book_click tab.reserves; book_shot l5-reserves
      k_key esc 10
      p4_off
      quit_game
      expect_log 1 'MISTBRN event=book_view .* tab=reserves '
      expect_log 1 'MISTBRN hud pair=iron_steel iron=[0-9]*/burning' ;;
    l6)  # the notebook recipe is gone; a notebook already made still opens the book
      launch MISTBRN_TEST_SCENARIO=
      ctl command "recipe give @s mistbrn:notebook" > "$dir/l6-recipe.json" 2>&1 || true
      ctl give mistbrn:notebook 1 >/dev/null; ctl wait 5 >/dev/null
      ctl select mistbrn:notebook >/dev/null
      ctl look -90 -90 >/dev/null
      ctl use-item > "$dir/l6-use.json"; ctl wait 10 >/dev/null
      k_screen l6-open '^"mod"$'
      p5_screenshot l6-open
      k_key esc 10
      quit_game
      expect_log 1 'MISTBRN event=notebook result=open'
      if [[ -f "$REPO_ROOT/mod/data/mistbrn/recipe/notebook.json" ]]; then
        k_fail "the notebook recipe is still in the pack"
      else k_ok "no notebook recipe in the pack"; fi
      # For the evidence: what the game answers when asked for the recipe (reviewed by hand).
      echo "l6 recipe give: $(tr -d '\n' < "$dir/l6-recipe.json" | head -c 200)" | tee -a "$dir/report.txt" ;;
    *) return 1 ;;
  esac
}
