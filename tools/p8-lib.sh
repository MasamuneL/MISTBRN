# shellcheck shell=bash
# P8 native scenarios (the skill map and the general nodes), sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# p8_knowledge: the anchor principle concluded at an alembic, in survival: 3 knowledge points
# (and 200 iron and steel XP). Iron and steel are unlocked by test credits first.
p8_knowledge() {
  ctl command "mistbrn-test credit iron 30000" >/dev/null
  ctl command "mistbrn-test credit steel 30000" >/dev/null
  ctl command "mistbrn-test evidence 2" >/dev/null
  ctl command "mistbrn-test evidence 4" >/dev/null
  ctl command "gamemode survival" >/dev/null
  p5_select_table
  ctl command "mistbrn-test inject conclude.1.0" >/dev/null
  ctl wait 20 >/dev/null
}

# p8_buy CODE RANK: the purchase request the book sends (op 6, or op 9 above code 63).
p8_buy() {
  ctl command "mistbrn-test inject buy.$1.$2" >/dev/null
  ctl wait 10 >/dev/null
}

# p8_tp: P4's box (peaceful, creative), tin and pewter unlocked by test credits (30 000 each)
# and selected.
p8_tp() {
  p4_fixture
  ctl command "mistbrn-test credit tin 30000" >/dev/null
  ctl command "mistbrn-test credit pewter 30000" >/dev/null
  ctl command "mistbrn-test inject select_tin_pewter" >/dev/null
  ctl wait 5 >/dev/null
}

# p8_tp_xp AMOUNT: tin and pewter experience (pewter fall practice in past windows, 10 a step).
p8_tp_xp() { ctl command "mistbrn-test xp $1 4" >/dev/null; ctl wait 5 >/dev/null; }

# p8_log_range LABEL PATTERN FIELD MIN MAX: FIELD of the first launch-1 line matching PATTERN
# lies in MIN..MAX (integers; a decimal value is cut at its point).
p8_log_range() {
  local value
  value="$({ grep -a "$2" "$(log 1)" || true; } | head -1 | tr ' ' '\n' | sed -n "s/^$3=//p")"
  value="${value%%.*}"
  if [[ "$value" =~ ^[0-9]+$ ]] && (( value >= $4 && value <= $5 )); then k_ok "$1: $value"
  else k_fail "$1: '${value}' not in $4..$5"; fi
}

# p8_migrate: a real P3c world (schema 3 on disk, a claimed milestone) loads and saves
# schema 4 with everything it had (P8 adds only unopened practice windows).
p8_migrate() {
  local name="${MISTBRN_P3C_FIXTURE:-p3c-fixture-h2}" src uuid player before after old new
  local field value old_windows missing=0
  src="$RT/runs/$name"
  [[ -d "$src/saves/mistbrn-h2" && -s "$src/horizon-player.txt" ]] ||
    die "incomplete P3c migration fixture: $src (an h2 run of a P3c build as $name)"
  old="$({ grep -ah 'MISTBRN_TEST scenario=report state=active ' "$src"/l*.log || true; } | tail -1)"
  uuid="$(tr ' ' '\n' <<< "$old" | sed -n 's/^uuid=//p')"
  [[ "$uuid" =~ ^[0-9a-f-]{36}$ ]] || die "no P3c player UUID in $src"
  mkdir -p "$dir/saves"
  cp -R "$src/saves/mistbrn-h2" "$dir/saves/$world"
  cp "$src/horizon-player.txt" "$dir/horizon-player.txt"
  player="$dir/saves/$world/players/data/$uuid.dat"
  before="$(p5_disk_schema "$player")" || die "could not read the schema from $player"
  [[ "$before" == 3 ]] || die "expected raw schema 3, got $before"
  k_ok "on-disk schema before launch: $before (uuid $uuid)"
  report; ctl save >/dev/null; quit_game
  expect_log 1 'MISTBRN event=save_loaded state=found'
  new="$(grep -ah 'MISTBRN_TEST scenario=report state=active ' "$(log 1)" | tail -1)"
  # Every field the P3c report had is the same, but the schema (3 → 4) and what a load and
  # save change anyway (the revision and the raw bytes' hash).
  for field in $(tr ' ' '\n' <<< "$old" | grep '=' | grep -Ev '^(schema|scenario|revision|raw_hash|windows)='); do
    if ! tr ' ' '\n' <<< "$new" | grep -qxF -- "$field"; then
      k_fail "after migration, $field changed: $(tr ' ' '\n' <<< "$new" | grep "^${field%%=*}=" || echo missing)"
      missing=1
    fi
  done
  [[ "$missing" == 0 ]] && k_ok "every report field kept"
  # The windows list grows with the catalog's categories (tin and pewter's are new, unopened).
  old_windows="$(tr ' ' '\n' <<< "$old" | sed -n 's/^windows=//p')"
  value="$(tr ' ' '\n' <<< "$new" | sed -n 's/^windows=//p')"
  if [[ "$value" == "$old_windows",* ]]; then k_ok "windows kept, new ones after: $value"
  else k_fail "windows changed: $old_windows -> $value"; fi
  value="$(tr ' ' '\n' <<< "$new" | sed -n 's/^schema=//p')"
  if [[ "$value" == 4 ]]; then k_ok "reported schema 4"; else k_fail "reported schema $value"; fi
  after="$(p5_disk_schema "$player")" || die "could not read the schema from $player"
  if [[ "$after" == 4 ]]; then k_ok "on-disk schema after save: $after"
  else k_fail "expected raw schema 4 after save, got $after"; fi
}

# p8_scenario: returns 1 for any other scenario name.
p8_scenario() {
  case "$scenario" in
    g1)  # sectors: locked ones are silhouettes naming their milestone; a pair unlocked by
         # credits shows its two metals' own powers; the map follows the selected pair
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      book_pair
      book_open g1-iron
      k_key esc 10
      ctl command "mistbrn-test credit tin 10000" >/dev/null
      ctl command "mistbrn-test credit pewter 10000" >/dev/null
      ctl command "mistbrn-test inject select_tin_pewter" >/dev/null
      ctl wait 20 >/dev/null
      book_open g1-tin
      book_click innate.pewter; book_shot g1-innate
      k_key esc 10
      quit_game
      expect_log 1 'MISTBRN event=book_view .* tab=tree pan=-92,-92 zoom=1.00 .*locked\.tin_pewter@'
      # Tin and pewter selected: the map opens on its sector, its powers shown, not locked.
      expect_log 1 'MISTBRN event=book_view .* tab=tree pan=92,-92 zoom=1.00 .*innate\.tin@.*innate\.pewter@'
      if grep -a 'MISTBRN event=book_view .* pan=92,-92 ' "$(log 1)" | grep -q 'locked\.tin_pewter@'; then
        k_fail "tin and pewter still drawn locked once unlocked"
      else k_ok "the unlocked sector is no silhouette"; fi
      # From tin and pewter's sector, iron and steel's push branch shows on the left.
      expect_log 1 'MISTBRN event=book_view .* pan=92,-92 .*innate\.steel@'
      k_expect_count "no purchase from looking" 'MISTBRN event=purchase ' 0 0 ;;
    g2)  # the mouse: a drag pans and buys nothing, a still press buys, the wheel zooms
      local from x y line bought
      launch MISTBRN_TEST_SCENARIO=p4
      book_quiet
      p6_place
      book_pair
      book_open g2-open
      # Drag from reach 60 GUI pixels right, through a middle point: the map follows.
      if from="$(book_xy node.1)"; then
        x="${from% *}"; y="${from#* }"
        ctl drag-at "$x" "$y" "$(awk -v x="$x" 'BEGIN { printf "%d", x + 30 }')" "$y" \
          "$(awk -v x="$x" 'BEGIN { printf "%d", x + 60 }')" "$y" >/dev/null
        ctl wait 10 >/dev/null
      fi
      # Right after the drag (live log): the view moved and nothing was bought (Codex r1).
      line="$(book_line)"
      if [[ -n "$line" && "$line" != *" pan=-92,-92 "* ]]; then k_ok "the drag moved the map"
      else k_fail "the map did not move with the drag: ${line:0:160}"; fi
      bought="$(grep -ac 'MISTBRN event=purchase ' "$sockdir/l$launch_no.log" || true)"
      if [[ "$bought" == 0 ]]; then k_ok "the drag bought nothing"
      else k_fail "the drag bought something ($bought purchase lines)"; fi
      book_shot g2-dragged
      # A press that does not move buys reach (100 XP: one point).
      book_click node.1
      book_shot g2-bought
      book_scroll node.2 -2
      book_shot g2-zoomed
      k_key esc 10
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=notebook_click .* target=node.1 dragged=true'
      expect_log 1 'MISTBRN event=notebook_click .* target=node.1 dragged=false'
      k_expect_count "one purchase, from the still press" \
        'MISTBRN event=purchase node=reach rank=1 result=ok' 1 1
      expect_log 1 'MISTBRN event=book_scroll lines=-1 pan=[-0-9]*,[-0-9]* zoom=0.64 '
      expect 2 report state=active nodes=1:1 ;;
    g3)  # op 9 reaches the server; flare cost bought with knowledge lowers the flare's fuel
      local reserve used
      launch MISTBRN_TEST_SCENARIO=p4
      p6_place
      p8_knowledge
      # No node has a code above 63 yet: the request still decodes (op 9) and is refused.
      p8_buy 64 1
      p8_buy 18 1
      p8_buy 16 1
      p8_buy 18 1
      ctl command "gamemode creative" >/dev/null
      ctl command "tp @s 512.5 100 512.5" >/dev/null
      ctl look 0 90 >/dev/null
      ctl command "mistbrn-test burn iron flare" >/dev/null
      ctl wait 400 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=purchase node=unknown rank=1 result=UnknownNode code=64'
      # Flare cost needs economy first.
      expect_log 1 'MISTBRN event=purchase node=flare_cost rank=1 result=MissingRequirement code=18'
      expect_log 1 'MISTBRN event=purchase node=economy rank=1 result=ok seq=[0-9]* code=16'
      expect_log 1 'MISTBRN event=purchase node=flare_cost rank=1 result=ok seq=[0-9]* code=18'
      expect 2 report state=active nodes=16:1,18:1
      # 400 ticks flared (plus up to ~17 of command latency; p8-2 saw 6 on 100) at
      # 10 × 0.95 × 2.7 = 25.65 a tick; × 3 (28.5) would use 11 400 or more.
      reserve="$(field 2 report reserve_iron)"
      used=$((30000 - reserve))
      if (( used >= 10260 && used <= 10700 )); then k_ok "flared fuel used: $used (x2.7)"
      else k_fail "flared fuel used: $used, expected 10260..10700 (x2.7; x3 gives 11400+)"; fi ;;
    g4)  # dosing: an iron vial gives 10 % more with dosing 1
      launch MISTBRN_TEST_SCENARIO=p4
      p6_place
      p8_knowledge
      p8_buy 17 1
      p8_buy 20 1
      ctl command "give @s mistbrn:vial_iron 1" >/dev/null
      ctl select mistbrn:vial_iron >/dev/null
      ctl use-item --hold 1.8 > "$dir/use-item.json"
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=purchase node=capacity rank=1 result=ok seq=[0-9]* code=17'
      expect_log 1 'MISTBRN event=purchase node=dosing rank=1 result=ok seq=[0-9]* code=20'
      expect 2 report state=active reserve_iron=57500 nodes=17:1,20:1 ;;
    m4)
      p8_migrate ;;
    g5)  # simultaneous burning: a third metal is refused, then lit once the node is bought
      p8_tp
      ctl command "mistbrn-test credit iron 30000" >/dev/null
      p8_knowledge
      ctl command "gamemode creative" >/dev/null
      p6_place
      ctl command "mistbrn-test burn tin" >/dev/null
      ctl command "mistbrn-test burn pewter" >/dev/null
      ctl command "mistbrn-test burn iron" >/dev/null
      ctl wait 5 >/dev/null
      p8_buy 17 1
      p8_buy 19 1
      ctl command "mistbrn-test burn iron" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=burn metal=iron burn=burning result=TooManyBurning changed=0'
      expect_log 1 'MISTBRN event=purchase node=simultaneous rank=1 result=ok seq=[0-9]* code=19'
      expect_log 1 'MISTBRN event=burn metal=iron burn=burning result=ok changed=1'
      # Reconnecting puts every metal out: the three lit at once show in the log above.
      expect 2 report state=active nodes=17:1,19:1 ;;
    tp1)  # senses: hidden night vision on and off, farther fog, the stun in daylight, clarity
      p8_tp
      p8_tp_xp 1000
      p8_buy 41 1
      ctl command "time set midnight" >/dev/null
      ctl look -90 0 >/dev/null
      ctl command "mistbrn-test burn tin" >/dev/null
      ctl wait 20 >/dev/null
      book_shot tp1-burning
      ctl command "mistbrn-test burn tin flare" >/dev/null
      ctl wait 20 >/dev/null
      book_shot tp1-flared
      ctl command "time set noon" >/dev/null
      ctl wait 20 >/dev/null
      book_shot tp1-stun
      ctl wait 60 >/dev/null
      ctl command "effect give @s minecraft:darkness 13" >/dev/null
      ctl wait 20 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=purchase node=tin_clarity rank=1 result=ok'
      expect_log 1 'MISTBRN event=power power=night_vision state=on'
      expect_log 1 'MISTBRN event=fog factor=1.50'
      expect_log 1 'MISTBRN event=fog factor=2.00'
      expect_log 1 'MISTBRN event=stun light=1[45] ticks=40'
      # 13 s of darkness (260 ticks) with clarity 1 keeps two thirds.
      expect_log 1 'MISTBRN event=clarity from=2[0-9][0-9] to=1[67][0-9]'
      expect_log 1 'MISTBRN event=evidence research=threshold_principle bit=4 ready=false'
      expect_log 1 'MISTBRN event=power power=night_vision state=off'
      expect_log 1 'MISTBRN event=power_cleared reason=off tin=false pewter=false'
      expect 2 report state=active burn_tin=off nodes=41:1 research=2:e4h0c0 ;;
    tp2)  # hearing: a creature behind a wall is heard, glows for the player, and a hit counts
      p8_tp
      p8_tp_xp 1000
      p8_buy 40 1
      p8_buy 42 1
      ctl command "fill 516 100 508 516 104 516 minecraft:stone" >/dev/null
      ctl command "summon minecraft:cow 519.5 100 512.5 {NoAI:1b,Tags:[heard]}" >/dev/null
      ctl look -90 0 >/dev/null
      ctl command "mistbrn-test burn tin" >/dev/null
      # The cow's own ambient sound is the server's (hurt sounds are played by clients): wait
      # for it, up to 800 ticks.
      for _ in $(seq 40); do
        grep -aq 'MISTBRN event=heard ' "$sockdir/l$launch_no.log" 2>/dev/null && break
        ctl wait 20 >/dev/null
      done
      book_shot tp2-heard
      ctl command "damage @e[tag=heard,limit=1] 1 minecraft:player_attack by @s" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=purchase node=hearing rank=1 result=ok'
      expect_log 1 'MISTBRN event=heard entity=[0-9]* distance=[0-9.]* unseen=true'
      expect_log 1 'MISTBRN event=senses_sent count=1'
      expect_log 1 'MISTBRN event=senses count=1'
      expect_log 1 'MISTBRN event=heard_hit entity='
      expect_log 1 'MISTBRN event=practice category=heard_hit xp=10 count=1'
      expect_log 1 'MISTBRN event=evidence research=threshold_principle bit=1 ready=false'
      # Tin out: the client is sent an empty list.
      expect_log 1 'MISTBRN event=senses count=0'
      expect 2 report state=active nodes=40:1,42:1 research=2:e1h0c0 ;;
    tp3)  # pewter's body: less damage taken, burning and flared, and a softer fall survived
      p8_tp
      ctl command "gamemode survival" >/dev/null
      ctl command "mistbrn-test burn pewter" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "damage @s 4 minecraft:generic" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "tp @s 512.5 115 512.5" >/dev/null
      ctl wait 80 >/dev/null
      ctl command "mistbrn-test burn pewter flare" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "damage @s 4 minecraft:generic" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=power power=pewter_body state=burning'
      expect_log 1 'MISTBRN event=pewter_damage from=4.00 to=3.40'
      expect_log 1 'MISTBRN event=pewter_fall_damage distance=1[45]\.[0-9] multiplier=0.70'
      expect_log 1 'MISTBRN event=pewter_fall distance=1[45]\.[0-9] survived=true'
      expect_log 1 'MISTBRN event=practice category=pewter_fall xp=10 count=1'
      expect_log 1 'MISTBRN event=evidence research=threshold_principle bit=2 ready=false'
      expect_log 1 'MISTBRN event=power power=pewter_body state=flared'
      expect_log 1 'MISTBRN event=pewter_damage from=4.00 to=2.80'
      expect_log 1 'MISTBRN event=power power=pewter_body state=off'
      expect 2 report state=active burn_pewter=off xp_tin_pewter=10 research=2:e2h0c0 ;;
    tp4)  # R flares both lit metals; a long flare drags when it ends
      p8_tp
      ctl command "mistbrn-test burn tin" >/dev/null
      ctl command "mistbrn-test burn pewter" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject flare" >/dev/null
      ctl wait 220 >/dev/null
      ctl command "mistbrn-test inject flare" >/dev/null
      ctl wait 20 >/dev/null
      book_shot tp4-drag
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=control op=flare command=flare_pair:tin_pewter:true .*result=ok changed=2'
      expect_log 1 'MISTBRN event=control op=flare command=flare_pair:tin_pewter:false .*result=ok changed=2'
      # Flared about 220 ticks: half of it, 110 or a little more for command latency.
      p8_log_range "drag after the flare" 'MISTBRN event=drag ticks=' ticks 105 130
      expect 2 report state=active burn_tin=off burn_pewter=off ;;
    tp5)  # Reflexes: refused without the technique or a metal, then a dodge, i-frames, cooldown
      p8_tp
      ctl command "mistbrn-test inject dodge.1" >/dev/null
      ctl wait 5 >/dev/null
      p8_tp_xp 3000
      ctl command "mistbrn-test evidence 1 2" >/dev/null
      ctl command "mistbrn-test evidence 2 2" >/dev/null
      ctl command "gamemode survival" >/dev/null
      p5_select_table
      ctl command "mistbrn-test inject conclude.2.0" >/dev/null
      ctl wait 20 >/dev/null
      for node in 40 42 48 50 56; do p8_buy "$node" 1; done
      ctl command "mistbrn-test burn pewter" >/dev/null
      ctl command "mistbrn-test inject dodge.1" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test burn tin" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test inject dodge.0" >/dev/null
      ctl wait 1 >/dev/null
      ctl command "damage @s 2 minecraft:mob_attack" >/dev/null
      ctl command "mistbrn-test inject dodge.0" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=control op=dodge direction=1 result=NotOwned'
      expect_log 1 'MISTBRN event=conclude research=threshold_principle result=ok'
      expect_log 1 'MISTBRN event=purchase node=reflexes rank=1 result=ok'
      expect_log 1 'MISTBRN event=control op=dodge direction=1 result=NotLit'
      k_expect_count "one dodge" 'MISTBRN event=control op=dodge direction=0 result=ok' 1 1
      expect_log 1 'MISTBRN event=control op=dodge direction=0 result=Cooldown'
      expect_log 1 'MISTBRN event=dodged damage_type=minecraft:mob_attack'
      expect 2 report state=active nodes=40:1,42:1,48:1,50:1,56:1 research=2:e3h0c1 ;;
    tp6)  # pewter's fight practice, and the tin/pewter sector of the map bought by a click
      p8_tp
      ctl command "difficulty easy" >/dev/null
      p8_tp_xp 1000
      ctl command "summon minecraft:zombie 515.5 100 512.5 {NoAI:1b,Tags:[fight]}" >/dev/null
      ctl command "mistbrn-test burn pewter" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "damage @e[tag=fight,limit=1] 200 minecraft:player_attack by @s" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      book_quiet
      book_open tp6-map
      book_click node.48
      book_shot tp6-bought
      k_key esc 10
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=pewter_kill kind=minecraft:zombie'
      expect_log 1 'MISTBRN event=practice category=pewter_fight xp=10 count=1'
      expect_log 1 'MISTBRN event=book_view .* pan=92,-92 .*innate\.tin@.*node\.40@.*node\.42@'
      expect_log 1 'MISTBRN event=purchase node=pewter_strength rank=1 result=ok'
      expect 2 report state=active nodes=48:1 ;;
    *) return 1 ;;
  esac
}
