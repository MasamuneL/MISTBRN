# shellcheck shell=bash
# P7a key and HUD scenarios (K1-K7), sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# k_key KEY [WAIT]: tap one key as a player would, then let the request reach the server.
k_key() { ctl key "$1" tap >/dev/null; ctl wait "${2:-5}" >/dev/null; }

k_credit() { ctl command "mistbrn-test credit $1 ${2:-25000}" >/dev/null; ctl wait 5 >/dev/null; }

k_fail() { echo "  FAIL $*" | tee -a "$dir/report.txt"; fails=$((fails + 1)); }
k_ok() { echo "check: $*" | tee -a "$dir/report.txt"; }

# k_count PATTERN: lines of launch 1 matching the fixed string PATTERN.
k_count() { grep -aFc -- "$1" "$(log 1)" || true; }

# k_expect_count LABEL PATTERN MIN MAX
k_expect_count() {
  local n; n="$(k_count "$2")"
  if (( n >= $3 && n <= $4 )); then k_ok "$1 ($n)"; else k_fail "$1: $n lines of '$2', expected $3..$4"; fi
}

# k_commands: the server's key commands of launch 1, in order, as command=result words.
k_commands() {
  { grep -a "MISTBRN event=control " "$(log 1)" || true; } | sed -n 's/.* command=\([^ ]*\) .* result=\([^ ]*\) .*/\1=\2/p' | tr '\n' ' '
}

# k_expect_commands WORDS...: exactly this sequence of command=result.
k_expect_commands() {
  local got want="$* "
  got="$(k_commands)"
  if [[ "$got" == "$want" ]]; then k_ok "commands: $got"; else k_fail "commands '$got', expected '$want'"; fi
}

# k_screen NAME PATTERN: record the open screen; fail unless its JSON mentions PATTERN.
k_screen() {
  # `state` answers on the game tick; `screen` waits for a rendered frame, which a hidden
  # test window may never draw.
  ctl state > "$dir/$1.json" || true
  local open; open="$(jq -c '.screen // empty' "$dir/$1.json" 2>/dev/null)"
  if grep -Eqi -- "$2" <<< "$open"; then k_ok "$1 open: $open"
  else k_fail "$1: screen '$open' does not look like $2"; fi
}

# k_pull_check: iron pulled by key — force samples between the key's lighting and its off,
# x grew, none after the off.
k_pull_check() {
  local out samples first last after off
  out="$(LC_ALL=C awk '
    /MISTBRN event=control / && / command=activate:iron / && / result=ok / { on = 1; next }
    /MISTBRN event=control / && / command=off:all / && / result=ok / { if (on) off = 1; on = 0; next }
    /MISTBRN p4 tick=/ {
      if (off) { after++; next }
      if (!on) next
      for (i = 1; i <= NF; i++) if ($i ~ /^pos=/) { split(substr($i, 5), p, ","); x = p[1] + 0 }
      if (!n) first = x
      last = x; n++
    }
    END { printf "samples=%d first_x=%.3f last_x=%.3f after_off=%d off=%d\n", n, first, last, after, off }
  ' "$(log 1)")"
  echo "check: $out" | tee -a "$dir/report.txt"
  # shellcheck disable=SC2001  # a regex, not a glob: drop every key= prefix
  read -r samples first last after off <<< "$(echo "$out" | sed 's/[a-z_]*=//g')"
  [[ "$off" == 1 ]] || k_fail "no successful off by key"
  (( ${samples:-0} >= 20 )) || k_fail "expected >= 20 force ticks while iron burned by key"
  awk -v a="$first" -v b="$last" 'BEGIN { exit !(b > a + 3) }' || k_fail "expected the key pull to move x (> 3 blocks)"
  [[ "$after" == 0 ]] || k_fail "force applied after the off key"
}

# k_flare_check: from the client HUD lines, iron spends about three times as much per
# snapshot while flared as while burning.
k_flare_check() {
  local out ratio
  out="$(LC_ALL=C awk '
    /MISTBRN hud pair=iron_steel / {
      for (i = 1; i <= NF; i++) if ($i ~ /^iron=/) { split(substr($i, 6), v, "/"); r = v[1] + 0; s = v[2] }
      if (prev_state == s && (s == "burning" || s == "flared") && prev > r) { sum[s] += prev - r; cnt[s]++ }
      prev = r; prev_state = s
    }
    END {
      b = cnt["burning"] ? sum["burning"] / cnt["burning"] : 0
      f = cnt["flared"] ? sum["flared"] / cnt["flared"] : 0
      printf "burning_per_hud=%.1f flared_per_hud=%.1f ratio=%.2f\n", b, f, b ? f / b : 0
    }
  ' "$(log 1)")"
  echo "check: $out" | tee -a "$dir/report.txt"
  ratio="${out##*ratio=}"
  awk -v r="$ratio" 'BEGIN { exit !(r >= 2.7 && r <= 3.3) }' || k_fail "expected flared spend ~3x burning (ratio $ratio)"
}

# k_hud_last METAL: the last reserve the client HUD showed for METAL in launch 1.
k_hud_last() {
  grep -a "MISTBRN hud pair=" "$(log 1)" | tail -1 | tr ' ' '\n' | sed -n "s/^$1=\([0-9]*\)\/.*/\1/p"
}

# k_gui_size PNG: the GUI size in GUI pixels ("W H") for a window screenshot, as the game
# scales it: options.txt guiScale, or for auto (0) the largest scale that keeps 320x240.
k_gui_size() {
  local hex fw fh scale
  hex="$(od -An -tx1 -j16 -N8 "$1" 2>/dev/null | tr -d '[:space:]')" || return 0
  [[ ${#hex} == 16 ]] || return 0
  fw=$((16#${hex:0:8})); fh=$((16#${hex:8:8}))
  scale="$(sed -n 's/^guiScale://p' "$dir/options.txt" 2>/dev/null | tail -1)"
  if [[ -z "$scale" || "$scale" == 0 ]]; then
    scale=1
    while (( fw / (scale + 1) >= 320 && fh / (scale + 1) >= 240 )); do scale=$((scale + 1)); done
  fi
  awk -v fw="$fw" -v fh="$fh" -v s="$scale" 'BEGIN { printf "%.2f %.2f", fw / s, fh / s }'
}

# k_scenario: runs K1-K9; returns 1 for any other scenario name.
k_scenario() {
  local server hud
  case "$scenario" in
    k1)  # Z lights iron and pulls toward the block; G puts it out and the force stops.
      p4_fixture
      p4_iron_block 524
      k_credit iron
      k_key z 60
      k_key g 20
      quit_game
      k_expect_count "client sent toggle_first" "MISTBRN event=key key=toggle_first" 1 1
      k_expect_commands activate:iron=ok off:all=ok
      k_pull_check ;;
    k2)  # Toggle, flare and switch: Z, R, R, X, Z, Z.
      p4_fixture
      ctl look -90 -90 >/dev/null
      k_credit iron
      k_credit steel
      k_key z 30; k_key r 30; k_key r 30; k_key x 10; k_key z 10; k_key z 10
      quit_game
      k_expect_commands activate:iron=ok flare:iron=ok activate:iron=ok activate:steel=ok activate:iron=ok off:iron=ok
      k_flare_check ;;
    k3)  # V goes through the four pairs; Z acts on the selected pair's first metal.
      p4_fixture
      k_credit iron
      k_key v; k_key z; k_key v; k_key v; k_key v; k_key z; k_key g
      quit_game
      k_expect_commands pair:tin_pewter=ok "activate:tin=MetalLocked(Tin)" pair:zinc_brass=ok \
        pair:copper_bronze=ok pair:iron_steel=ok activate:iron=ok off:all=ok
      k_expect_count "HUD showed tin/pewter" "MISTBRN hud pair=tin_pewter" 1 100 ;;
    k9)  # Hold V: the pair wheel opens; a click on its bottom slot chooses zinc/brass (not the
      # next pair, so a tap read as a cycle cannot pass) and closes the wheel while V is still
      # held; Z then acts on zinc. The GUI size comes from the screenshot (k_gui_size).
      p4_fixture
      k_credit iron
      ctl key v down >/dev/null; ctl wait 5 >/dev/null
      k_screen k9-wheel '"mod"'
      ctl --timeout 30 screenshot "$dir/k9-wheel.png" >/dev/null || k_fail "wheel screenshot failed"
      # The game's log reaches its file only later, so the GUI size comes from the screenshot.
      local gui w h
      gui="$(k_gui_size "$dir/k9-wheel.png")"
      w="${gui% *}"; h="${gui#* }"
      [[ -n "$gui" ]] || k_fail "no GUI size from the screenshot"
      ctl click-at "$(awk -v w="$w" 'BEGIN { printf "%d", w / 2 }')" \
        "$(awk -v h="$h" 'BEGIN { printf "%d", h / 2 + 44 }')" >/dev/null || k_fail "click on the wheel failed"
      ctl wait 5 >/dev/null
      k_screen k9-closed-by-click playing
      ctl key v up >/dev/null; ctl wait 5 >/dev/null
      k_key z; k_key g
      quit_game
      k_expect_count "wheel chose zinc/brass" "MISTBRN event=wheel result=select:zinc_brass" 1 1
      k_expect_count "one wheel choice" "MISTBRN event=wheel result=" 1 1
      k_expect_commands pair:zinc_brass=ok "activate:zinc=MetalLocked(Zinc)" off:all=ok
      if [[ -s "$dir/k9-wheel.png" ]]; then k_ok "screenshot $dir/k9-wheel.png (local, not versioned)"; fi ;;
    k4|k5)  # Nothing lights with the chat (K4) or the inventory (K5) open; it does once closed.
      p4_fixture
      k_credit iron
      if [[ "$scenario" == k4 ]]; then
        k_key t 10; k_screen k4-screen chat
        k_key z 10; k_key x 10; k_key r 10; k_key v 10
        k_key esc 10
      else
        ctl open-inventory >/dev/null; ctl wait 10 >/dev/null; k_screen k5-screen "inventory|creative"
        k_key z 10; k_key x 10; k_key r 10; k_key v 10
        ctl close >/dev/null; ctl wait 10 >/dev/null
      fi
      k_key z 10
      k_key g 10
      quit_game
      k_expect_count "client sent only after closing" "MISTBRN event=key key=" 2 2
      k_expect_commands activate:iron=ok off:all=ok ;;
    k6)  # The HUD follows the server's reserve; a local screenshot for review.
      p4_fixture
      ctl look -90 -90 >/dev/null
      k_credit iron 50000
      k_key z 40
      ctl status > "$dir/k6-status.json" || true
      ctl --timeout 30 screenshot "$dir/k6-hud.png" >/dev/null || k_fail "screenshot failed (no rendered frame?)"
      k_key g 20
      ctl save >/dev/null; quit_game
      report; quit_game
      expect 2 report state=active
      server="$(field 2 report reserve_iron)"; hud="$(k_hud_last iron)"
      if [[ -n "$hud" && "$hud" == "$server" ]]; then k_ok "HUD reserve $hud = server $server"
      else k_fail "HUD reserve '$hud' != server '$server'"; fi
      k_expect_count "HUD updates" "MISTBRN hud pair=iron_steel" 3 1000
      if [[ -s "$dir/k6-hud.png" ]]; then k_ok "screenshot $dir/k6-hud.png (local, not versioned)"
      else k_fail "no screenshot"; fi ;;
    k7)  # Malformed bytes and a burst are dropped and logged; the session goes on.
      p4_fixture
      k_credit iron
      k_key k 80
      # The transport paces a client's packets; flood the server inbox directly as well.
      ctl command "mistbrn-test flood 20" >/dev/null; ctl wait 10 >/dev/null
      k_key z 10
      k_key g 10
      quit_game
      k_expect_count "client sent the fuzz" "MISTBRN event=key key=test_fuzz" 1 1
      k_expect_count "rejected length" "result=rejected:length" 3 3
      k_expect_count "rejected version" "result=rejected:version" 1 1
      k_expect_count "rejected op" "result=rejected:op" 1 1
      k_expect_count "rejected arg" "result=rejected:arg" 1 1
      # flood 20: 16 fit the player's inbox, 8 pass the budget, the other 8 are one rate line.
      k_expect_count "inbox full" "op=any result=inbox_full dropped=4" 1 1
      k_expect_count "rate limited (logged once)" "op=any result=rate" 1 1
      k_expect_count "offs within the budget" "command=off:all allowed=true result=ok changed=0" 9 28
      k_expect_count "session alive: Z still lights" "command=activate:iron allowed=true result=ok changed=1" 1 1 ;;
    k8)  # The server's own check: with a chest open, a request (injected, as a modified
         # client would send it) is refused; putting metals out still passes.
      p4_fixture
      k_credit iron
      ctl command "mistbrn-test inject toggle_first" >/dev/null; ctl wait 5 >/dev/null
      ctl setblock 514 100 512 chest >/dev/null; ctl wait 5 >/dev/null
      ctl interact 514 100 512 >/dev/null; ctl wait 10 >/dev/null
      k_screen k8-screen "chest|generic|container"
      ctl command "mistbrn-test inject toggle_second" >/dev/null; ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject select_tin_pewter" >/dev/null; ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject off" >/dev/null; ctl wait 5 >/dev/null
      ctl close >/dev/null; ctl wait 10 >/dev/null
      ctl command "mistbrn-test inject toggle_first" >/dev/null; ctl wait 10 >/dev/null
      ctl command "mistbrn-test inject off" >/dev/null; ctl wait 5 >/dev/null
      quit_game
      k_expect_count "refused with the chest open" "command=activate:steel allowed=false result=InputCaptured changed=0" 1 1
      k_expect_count "off puts iron out with the chest open" "command=off:all allowed=false result=ok changed=1" 1 1
      k_expect_count "lit with nothing open" "command=activate:iron allowed=true result=ok changed=1" 2 2
      k_expect_commands activate:iron=ok activate:steel=InputCaptured pair:tin_pewter=InputCaptured \
        off:all=ok activate:iron=ok off:all=ok ;;
    *) return 1 ;;
  esac
}
