# shellcheck shell=bash
# P8 native scenarios (the skill map, the general nodes and the pairs' powers), sourced by
# native-test.sh.
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

# p8_zb: P4's box walled in glass (creatures stay on it), zinc and brass unlocked by test
# credits and selected, survival at night on easy (creatures only target survival players),
# with resistance V so nothing they do ends the run.
p8_zb() {
  p4_fixture
  ctl command "fill 502 99 507 541 104 517 minecraft:glass outline" >/dev/null
  ctl command "mistbrn-test credit zinc 30000" >/dev/null
  ctl command "mistbrn-test credit brass 30000" >/dev/null
  ctl command "mistbrn-test inject select_zinc_brass" >/dev/null
  ctl command "time set midnight" >/dev/null
  ctl command "difficulty easy" >/dev/null
  ctl command "gamemode survival" >/dev/null
  ctl command "effect give @s minecraft:resistance 9999 4 true" >/dev/null
  ctl wait 5 >/dev/null
}

# p8_zb_xp AMOUNT: zinc and brass experience (riot practice in past windows, 10 a step).
p8_zb_xp() { ctl command "mistbrn-test xp $1 6" >/dev/null; ctl wait 5 >/dev/null; }

# p8_summon KIND X TAG [NBT]: a creature on the box's middle line, held still by slowness so the
# view finds it where it was put (its target choice is not touched).
p8_summon() {
  ctl command "summon minecraft:$1 $2 100 512.5 {Tags:[$3]${4:+,$4}}" >/dev/null
  ctl command "effect give @e[tag=$3] minecraft:slowness 9999 255 true" >/dev/null
  ctl wait 2 >/dev/null
}

# p8_until PATTERN TICKS: waits (20 ticks a step) until the live log matches PATTERN.
p8_until() {
  local _
  for _ in $(seq $(( $2 / 20 ))); do
    grep -aq "$1" "$sockdir/l$launch_no.log" 2>/dev/null && return 0
    ctl wait 20 >/dev/null
  done
}

# p8_count_re LABEL REGEX MIN MAX: launch-1 lines matching REGEX lie in MIN..MAX.
p8_count_re() {
  local n; n="$(grep -ac -- "$2" "$(log 1)" || true)"
  if (( n >= $3 && n <= $4 )); then k_ok "$1 ($n)"; else k_fail "$1: $n lines of '$2', expected $3..$4"; fi
}

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
      # Code 64 (zinc range) is bought with op 9: it reaches the server, which refuses it
      # while zinc and brass are locked.
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
      expect_log 1 'MISTBRN event=purchase node=zinc_range rank=1 result=PairLocked code=64'
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
      # A free cow in a pen behind the wall (a still one makes no ambient sound).
      for wall in "517 100 510 517 100 514" "521 100 510 521 100 514" \
        "517 100 510 521 100 510" "517 100 514 521 100 514"; do
        ctl command "fill $wall minecraft:oak_fence" >/dev/null
      done
      ctl command "summon minecraft:cow 519.5 100 512.5 {Tags:[heard]}" >/dev/null
      ctl look -90 0 >/dev/null
      ctl command "mistbrn-test burn tin" >/dev/null
      # Hearing follows motion (the pinned game asks sound hooks only on the client): a hit
      # moves the cow at once; its wandering is heard too. Up to 800 ticks.
      ctl command "damage @e[tag=heard,limit=1] 1 minecraft:generic" >/dev/null
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
    tp4)  # R flares both lit metals; a long flare drags when it ends; a potion's night vision
         # outlives tin (Codex r1)
      p8_tp
      ctl command "effect give @s minecraft:night_vision 60" >/dev/null
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
      ctl command "effect clear @s minecraft:night_vision" > "$dir/tp4-night-vision.json"
      if grep -qi 'removed' "$dir/tp4-night-vision.json"; then k_ok "the potion's night vision stayed"
      else k_fail "the potion's night vision is gone: $(head -c 300 "$dir/tp4-night-vision.json")"; fi
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
      ctl wait 20 >/dev/null
      # The i-frames are over: the next hit lands (pewter takes 15 % off).
      ctl command "damage @s 2 minecraft:mob_attack" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=control op=dodge direction=1 result=NotOwned'
      expect_log 1 'MISTBRN event=conclude research=threshold_principle result=ok'
      expect_log 1 'MISTBRN event=purchase node=reflexes rank=1 result=ok'
      expect_log 1 'MISTBRN event=control op=dodge direction=1 result=NotLit'
      k_expect_count "one dodge" 'MISTBRN event=control op=dodge direction=0 result=ok' 1 1
      # Spent once, 500 of pewter, and moved (direction 0 is the look's left).
      k_expect_count "one spend" 'MISTBRN event=dodge spent=500 ' 1 1
      k_expect_count "one hit dodged" 'MISTBRN event=dodged damage_type=minecraft:mob_attack' 1 1
      expect_log 1 'MISTBRN event=pewter_damage from=2.00 to=1.70'
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
    zb1)  # Riot: a calm zombified piglin turns on the player and comes close; zinc out lets it
         # go once its after-time ends
      local piglin
      p8_zb
      p8_summon zombified_piglin 524.5 zb
      ctl look-at 524.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "effect clear @e[tag=zb] minecraft:slowness" >/dev/null
      p8_until 'MISTBRN event=provoke_near ' 400
      book_shot zb1-provoked
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 130 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=burn metal=zinc burn=burning result=ok changed=1'
      expect_log 1 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:zombified_piglin boss=false'
      expect_log 1 'MISTBRN event=evidence research=emotion_principle bit=1 ready=false'
      expect_log 1 'MISTBRN event=provoke_near entity='
      expect_log 1 'MISTBRN event=practice category=provoke xp=10 count=1'
      expect_log 1 'MISTBRN event=power_cleared reason=off zinc=false brass=false'
      piglin="$({ grep -ao 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:zombified_piglin' "$(log 1)" || true; } | head -1 | sed 's/.*entity=\([0-9]*\).*/\1/')"
      expect_log 1 "MISTBRN event=zb_end metal=zinc entity=${piglin:-x} reason=time cleared=true"
      expect 2 report state=active burn_zinc=off research=3:e1h0c0 ;;
    zb2)  # Soothe: a husk chasing the player lets go and cannot take them back, stays soothed;
         # once it ends the husk ignores brass a while; an angry zombified piglin calms
      local anger husk
      p8_zb
      p8_summon husk 518.5 zb
      ctl wait 100 >/dev/null
      ctl look-at 518.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn brass" >/dev/null
      ctl wait 140 >/dev/null
      book_shot zb2-soothed
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 120 >/dev/null
      # Immune for 200 ticks: brass again on the same husk does nothing.
      ctl command "mistbrn-test burn brass" >/dev/null
      ctl wait 20 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl command "kill @e[tag=zb]" >/dev/null
      p8_summon zombified_piglin 506.5 zp
      ctl command "damage @e[tag=zp,limit=1] 1 minecraft:player_attack by @s" >/dev/null
      ctl wait 10 >/dev/null
      ctl look-at 506.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn brass" >/dev/null
      ctl wait 40 >/dev/null
      ctl command "data get entity @e[tag=zp,limit=1] anger_end_time" > "$dir/zb2-anger.json"
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=brass_soothe entity=[0-9]* kind=minecraft:husk boss=false chaser=true'
      expect_log 1 'MISTBRN event=evidence research=emotion_principle bit=2 ready=false'
      p8_count_re "the soothed husk tried to take the player back" 'MISTBRN event=brass_block ' 1 999
      expect_log 1 'MISTBRN event=soothe_held entity='
      expect_log 1 'MISTBRN event=practice category=soothe xp=10 count=1'
      expect_log 1 'MISTBRN event=zb_end metal=brass entity=[0-9]* reason=time'
      p8_count_re "the immune husk soothed only once" 'MISTBRN event=brass_soothe entity=[0-9]* kind=minecraft:husk' 1 1
      husk="$({ grep -ao 'MISTBRN event=brass_soothe entity=[0-9]* kind=minecraft:husk' "$(log 1)" || true; }| head -1 | sed 's/.*entity=\([0-9]*\).*/\1/')"
      expect_log 1 "MISTBRN event=zb_immune metal=brass entity=${husk:-none} kind=minecraft:husk"
      expect_log 1 'MISTBRN event=brass_calm entity=[0-9]* changed=true'
      anger="$({ grep -ao 'anger_end_time[^,}]*\|[0-9-]*L' "$dir/zb2-anger.json" || true; } | head -3 | tr '\n' ' ')"
      if grep -aq 'has the following entity data: 0L\|: 0L\|found no elements\|Found no elements' "$dir/zb2-anger.json"; then
        k_ok "the zombified piglin's anger ended: $anger"
      else k_fail "the zombified piglin still angry: $(head -c 300 "$dir/zb2-anger.json")"; fi
      expect 2 report state=active burn_brass=off research=3:e2h0c0 ;;
    zb3)  # bosses: zinc holds an elder guardian 40 ticks at most while zinc burns, then it is
         # immune (still in view, zinc still lit); a brain-driven piglin is refused
      local boss start end again
      p8_zb
      p8_summon elder_guardian 520.5 boss 'NoAI:1b'
      ctl look-at 520.5 101 512.5 >/dev/null
      # Zinc stays lit and on the boss: cut at 40 ticks, refused while immune, taken again
      # the tick its 600 ticks run out.
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 720 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl command "kill @e[tag=boss]" >/dev/null
      p8_summon piglin 520.5 brain 'NoAI:1b'
      ctl look-at 520.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      p8_count_re "the boss provoked twice: before the cut and after its immunity" 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:elder_guardian boss=true' 2 2
      boss="$({ grep -ao 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:elder_guardian' "$(log 1)" || true; }| head -1 | sed 's/.*entity=\([0-9]*\).*/\1/')"
      start="$({ grep -ao "MISTBRN event=zinc_provoke entity=${boss:-x} .* tick=[0-9]*" "$(log 1)" || true; }| head -1 | sed 's/.*tick=//')"
      end="$({ grep -ao "MISTBRN event=zb_end metal=zinc entity=${boss:-x} reason=time cleared=[a-z]* tick=[0-9]*" "$(log 1)" || true; }| head -1 | sed 's/.*tick=//')"
      if [[ -n "$start" && -n "$end" ]] && (( end - start >= 38 && end - start <= 42 )); then
        k_ok "the boss let go after $((end - start)) ticks, zinc still lit"
      else k_fail "boss hold from tick '${start}' to '${end}', expected 40"; fi
      expect_log 1 "MISTBRN event=zb_immune metal=zinc entity=${boss:-none} kind=minecraft:elder_guardian"
      again="$({ grep -ao "MISTBRN event=zinc_provoke entity=${boss:-x} .* tick=[0-9]*" "$(log 1)" || true; }| sed -n 2p | sed 's/.*tick=//')"
      if [[ -n "$again" && -n "$end" ]] && (( again - end >= 600 && again - end <= 602 )); then
        k_ok "the same boss provoked again $((again - end)) ticks after the cut"
      else k_fail "second provocation at '${again}', cut at '${end}': expected 600 later"; fi
      expect_log 1 'MISTBRN event=zb_incompatible metal=zinc entity=[0-9]* kind=minecraft:piglin'
      k_expect_count "the piglin never provoked" 'kind=minecraft:piglin boss=' 0 0
      expect 2 report state=active burn_zinc=off ;;
    zb4)  # Courage on a tamed wolf and a built golem; the allies attack what the player hits
         # (practice); Retreat sends a soothed husk away
      local uuid husk
      p8_zb
      p8_zb_xp 1500
      for node in 64 66 72 74; do p8_buy "$node" 1; done
      uuid="$(ctl command "data get entity @s UUID" | grep -o '\[I; *-\?[0-9]*, *-\?[0-9]*, *-\?[0-9]*, *-\?[0-9]*\]' | head -1)"
      [[ -n "$uuid" ]] || k_fail "no player UUID read"
      p8_summon wolf 516.5 w "Owner:$uuid"
      ctl look-at 516.5 100.4 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      p8_summon iron_golem 507.5 g 'PlayerCreated:1b'
      ctl look-at 507.5 101.3 512.5 >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl command "effect clear @e[tag=w] minecraft:slowness" >/dev/null
      ctl command "effect clear @e[tag=g] minecraft:slowness" >/dev/null
      p8_summon husk 512.5 h
      ctl command "tp @e[tag=h] 512.5 100 515.5" >/dev/null
      ctl command "damage @e[tag=h,limit=1] 1 minecraft:player_attack by @s" >/dev/null
      p8_until 'MISTBRN event=ally_hit ' 300
      book_shot zb4-allies
      ctl command "kill @e[tag=w]" >/dev/null
      ctl command "kill @e[tag=g]" >/dev/null
      ctl command "kill @e[tag=h]" >/dev/null
      p8_summon husk 515.5 r
      ctl command "effect clear @e[tag=r] minecraft:slowness" >/dev/null
      ctl look-at 515.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn brass" >/dev/null
      p8_until 'MISTBRN event=retreat_reached ' 300
      ctl command "mistbrn-test off" >/dev/null
      ctl command "kill @e[tag=r]" >/dev/null
      # Courage on a fresh wolf, then the game closes with it running: the saved pulse runs out
      # within 2 s of the next load.
      p8_summon wolf 516.5 w2 "Owner:$uuid"
      ctl look-at 516.5 100.4 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      # `report` waits 40 ticks after the load; 20 more make 60 (a 40-tick pulse is gone).
      report
      ctl wait 20 >/dev/null
      ctl command "data get entity @e[tag=w2,limit=1] active_effects" > "$dir/zb4-courage-reload.json"
      quit_game
      if grep -aq 'minecraft:strength\|minecraft:speed' "$dir/zb4-courage-reload.json"; then
        k_fail "courage outlived the reload: $(head -c 300 "$dir/zb4-courage-reload.json")"
      elif grep -aqi 'found no elements\|entity data' "$dir/zb4-courage-reload.json"; then
        k_ok "courage gone 60 ticks after the reload"
      else k_fail "no reading of the reloaded wolf: $(head -c 300 "$dir/zb4-courage-reload.json")"; fi
      expect_log 1 'MISTBRN event=purchase node=courage rank=1 result=ok'
      expect_log 1 'MISTBRN event=purchase node=retreat rank=1 result=ok'
      p8_count_re "courage on the wolf, the golem and the second wolf" 'MISTBRN event=courage entity=[0-9]* level=1 ticks=200 strength=true speed=true' 3 3
      expect_log 1 'MISTBRN event=ally_hit entity='
      expect_log 1 'MISTBRN event=practice category=ally xp=10 count=1'
      husk="$({ grep -ao 'MISTBRN event=brass_soothe entity=[0-9]* kind=minecraft:husk' "$(log 1)" || true; }| head -1 | sed 's/.*entity=\([0-9]*\).*/\1/')"
      expect_log 1 "MISTBRN event=retreat entity=${husk:-none} distance=[0-9.]* to=.* ok=true"
      expect_log 1 "MISTBRN event=retreat_reached entity=${husk:-none} distance="
      expect 2 report state=active nodes=64:1,66:1,72:1,74:1 ;;
    zb5)  # Selective influence: concluded the emotion principle, both metals: the husk in view is
         # provoked, the two near the player soothed
      p8_zb
      p8_zb_xp 3000
      ctl command "mistbrn-test evidence 1 3" >/dev/null
      ctl command "mistbrn-test evidence 2 3" >/dev/null
      p5_select_table
      ctl command "mistbrn-test inject conclude.3.0" >/dev/null
      ctl wait 20 >/dev/null
      ctl command "setblock 514 100 512 minecraft:air" >/dev/null
      for node in 64 66 72 74 80; do p8_buy "$node" 1; done
      p8_summon husk 520.5 view
      p8_summon husk 509.5 side
      ctl command "tp @e[tag=side,limit=1] 509.5 100 510.5" >/dev/null
      p8_summon husk 509.5 side2
      ctl command "tp @e[tag=side2,limit=1] 509.5 100 514.5" >/dev/null
      ctl look-at 520.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl command "mistbrn-test burn brass" >/dev/null
      ctl wait 40 >/dev/null
      book_shot zb5-influence
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=conclude research=emotion_principle result=ok'
      expect_log 1 'MISTBRN event=purchase node=selective_influence rank=1 result=ok'
      p8_count_re "the husk in view provoked" 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:husk' 1 1
      p8_count_re "the two near soothed by influence" 'MISTBRN event=influence entity=[0-9]* kind=minecraft:husk' 2 2
      p8_count_re "brass soothed only those two" 'MISTBRN event=brass_soothe ' 2 2
      expect 2 report state=active nodes=64:1,66:1,72:1,74:1,80:1 research=3:e3h0c1 ;;
    zb6)  # the third evidence, a click on the map buys code 64 (op 9), the general bridge adds
         # 2 blocks to iron's reach, and death, a dimension change and logging out let every
         # creature go
      local husk again third
      p8_zb
      p8_summon zombified_piglin 520.5 zb 'NoAI:1b'
      ctl look-at 520.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "mistbrn-test burn brass" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "mistbrn-test off" >/dev/null
      p8_zb_xp 350
      book_quiet
      book_open zb6-map
      book_click node.64
      book_shot zb6-bought
      k_key esc 10
      p8_knowledge
      ctl command "mistbrn-test evidence 1 3" >/dev/null
      ctl command "mistbrn-test inject conclude.3.0" >/dev/null
      ctl wait 20 >/dev/null
      ctl command "setblock 514 100 512 minecraft:air" >/dev/null
      ctl command "kill @e[tag=zb]" >/dev/null
      p8_buy 72 1
      p8_buy 1 1
      p4_iron_block 531
      p5_trial_reach
      p8_buy 27 1
      p5_trial_reach
      # Zinc held on a creature, then death; again, then the Nether.
      # Persistent: death sends the player to the world spawn, far enough to despawn it.
      p8_summon husk 518.5 d 'NoAI:1b,PersistenceRequired:1b'
      ctl look-at 518.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test credit zinc 30000" >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      ctl command "kill @s" >/dev/null
      ctl wait 20 >/dev/null; ctl respawn >/dev/null; ctl wait 40 >/dev/null
      ctl command "tp @s 512.5 100 512.5" >/dev/null
      ctl command "effect give @s minecraft:resistance 9999 4 true" >/dev/null
      ctl wait 10 >/dev/null
      ctl look-at 518.5 101.2 512.5 >/dev/null
      ctl wait 20 >/dev/null
      ctl command "mistbrn-test credit zinc 30000" >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 20 >/dev/null
      ctl command "execute in minecraft:the_nether run tp @s 512.5 120 512.5" >/dev/null
      ctl wait 40 >/dev/null
      ctl command "execute in minecraft:overworld run tp @s 512.5 100 512.5" >/dev/null
      ctl wait 20 >/dev/null
      # Zinc held on the husk again, then the game closes with it lit.
      ctl look-at 518.5 101.2 512.5 >/dev/null
      ctl command "mistbrn-test credit zinc 30000" >/dev/null
      ctl command "mistbrn-test burn zinc" >/dev/null
      ctl wait 10 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=evidence research=emotion_principle bit=4 ready='
      expect_log 1 'MISTBRN event=purchase node=zinc_range rank=1 result=ok'
      expect_log 1 'MISTBRN event=purchase node=brass_range rank=1 result=ok'
      expect_log 1 'MISTBRN event=purchase node=outer_reach rank=1 result=ok'
      # Reach 1 (18 blocks) misses the block's face 18.5 away; the bridge's 2 more reach it.
      p8_count_re "only the bridge reached the far block" 'MISTBRN event=target_acquired kind=block id=minecraft:iron_block at=531.500,101.500,512.500' 1 1
      expect_log 1 'MISTBRN event=conclude research=emotion_principle result=ok'
      expect_log 1 'MISTBRN event=power_cleared reason=death power=zinc_brass'
      expect_log 1 'MISTBRN event=power_cleared reason=dimension power=zinc_brass'
      # The husk held each time is let go on its own level's pass, for the reason.
      # The husk is reloaded with a new id once the player's death unloads its chunk: the
      # death clear names the first id, the dimension clear the id of the second hold.
      husk="$({ grep -ao 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:husk' "$(log 1)" || true; } | sed -n 1p | sed 's/.*entity=\([0-9]*\).*/\1/')"
      again="$({ grep -ao 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:husk' "$(log 1)" || true; } | sed -n 2p | sed 's/.*entity=\([0-9]*\).*/\1/')"
      p8_count_re "the husk provoked anew after each clear" 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:husk' 3 3
      third="$({ grep -ao 'MISTBRN event=zinc_provoke entity=[0-9]* kind=minecraft:husk' "$(log 1)" || true; } | sed -n 3p | sed 's/.*entity=\([0-9]*\).*/\1/')"
      # Each hold was seen working before its clear (ids may repeat: counted per hold).
      p8_count_re "zinc verified on each husk hold" "MISTBRN event=zinc_verified entity=\\(${husk:-x}\\|${again:-x}\\|${third:-x}\\) " 3 3
      expect_log 1 "MISTBRN event=zb_end metal=zinc entity=${husk:-x} reason=death cleared=true"
      expect_log 1 "MISTBRN event=zb_end metal=zinc entity=${again:-x} reason=dimension cleared=true"
      expect_log 1 'MISTBRN event=power_cleared reason=logout power=zinc_brass'
      expect 2 report state=active nodes=1:1,27:1,64:1,72:1 ;;
    *) return 1 ;;
  esac
}
