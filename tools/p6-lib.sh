# shellcheck shell=bash
# P6 native scenarios (milestones), sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

# p6_place: the player on a stone floor at 512.5 100 512.5, looking east (chunks loaded first).
p6_place() {
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 20 >/dev/null
  ctl command "fill 508 99 508 518 99 518 minecraft:stone" >/dev/null
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl look -90 0 >/dev/null
  ctl wait 10 >/dev/null
}

# p6_lane [MOD_SCENARIO]: a fresh creative world, iron and steel locked, bosses allowed.
p6_lane() {
  launch MISTBRN_TEST_SCENARIO="${1:-}"
  ctl command "difficulty easy" >/dev/null
  p6_place
}

# p6_kill KIND TAG [WAIT] [Z]: summon a still boss three blocks east (at Z, default 512.5) and
# kill it with one hit that names the player as its attacker, as a melee kill does.
p6_kill() {
  ctl command "summon $1 515.5 100 ${4:-512.5} {NoAI:1b,Tags:[$2]}" > "$dir/summon-$2.json"
  ctl wait 5 >/dev/null
  ctl command "damage @e[tag=$2,limit=1] 2000 minecraft:player_attack by @s" > "$dir/damage-$2.json"
  ctl wait "${3:-40}" >/dev/null
}

# p6_chest X Y Z TABLE: an unopened chest with TABLE east of the player, opened by hand.
p6_chest() {
  ctl command "setblock $1 $2 $3 chest{LootTable:\"$4\"}" >/dev/null
  ctl wait 5 >/dev/null
  ctl look-at block "$1" "$2" "$3" west >/dev/null
  ctl use-item > "$dir/chest-$3.json"
  ctl wait 10 >/dev/null
  ctl close >/dev/null
  ctl wait 10 >/dev/null
}

# p6_wait_line NAME: the newest `MISTBRN_TEST scenario=NAME` line of the running launch, read
# from its live log beside the socket (logs can lag; the run folder gets a copy at exit).
p6_wait_line() {
  local line=""
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    line="$(grep -a "MISTBRN_TEST scenario=$1 " "$sockdir/l$launch_no.log" 2>/dev/null | tail -1 || true)"
    [[ -n "$line" ]] && break
    ctl wait 20 >/dev/null
  done
  printf '%s\n' "$line"
}

# p6_value LINE KEY
p6_value() { tr ' ' '\n' <<< "$1" | sed -n "s/^$2=//p"; }

# p6_milestone_lines KEY: how often launch 1 logged a claim attempt for KEY.
p6_milestone_lines() { grep -ac "MISTBRN event=milestone key=$1 " "$(log 1)" || true; }

# p6_migrate: a real P5 world (schema 2 on disk) loads with no milestones and saves schema 3.
# Fixture: run t1 with a P5 build (main before P6) as $MISTBRN_P5_FIXTURE (default p5-fixture-t1).
p6_migrate() {
  local name="${MISTBRN_P5_FIXTURE:-p5-fixture-t1}" src uuid player before after
  src="$RT/runs/$name"
  [[ -d "$src/saves/mistbrn-t1" && -s "$src/horizon-player.txt" && -f "$src/l1.log" ]] ||
    die "incomplete P5 migration fixture: $src (run t1 with a P5 build as $name)"
  uuid="$({ grep -ah 'MISTBRN_TEST scenario=report ' "$src"/l*.log || true; } | tail -1 |
    tr ' ' '\n' | sed -n 's/^uuid=//p')"
  [[ "$uuid" =~ ^[0-9a-f-]{36}$ ]] || die "no P5 player UUID in $src"
  mkdir -p "$dir/saves"
  cp -R "$src/saves/mistbrn-t1" "$dir/saves/$world"
  cp "$src/horizon-player.txt" "$dir/horizon-player.txt"
  player="$dir/saves/$world/players/data/$uuid.dat"
  before="$(p5_disk_schema "$player")" || die "could not read the schema from $player"
  [[ "$before" == 2 ]] || die "expected raw schema 2, got $before"
  k_ok "on-disk schema before launch: $before (uuid $uuid)"
  report; ctl save >/dev/null; quit_game
  expect_log 1 'MISTBRN event=save_loaded state=found'
  # t1 leaves iron and steel unlocked by the first vial, three credits (iron, iron, tin) and
  # no progress; P6 adds nothing but an empty milestone mask.
  expect 1 report state=active "uuid=$uuid" schema=3 unlocked_iron=true unlocked_steel=true \
    last_transaction=3 milestones=0 unlocked_pairs=iron_steel xp_iron_steel=0 nodes=- research=-
  after="$(p5_disk_schema "$player")" || die "could not read the schema from $player"
  if [[ "$after" == 3 ]]; then k_ok "on-disk schema after save: $after"
  else k_fail "expected raw schema 3 after save, got $after"; fi
}

# p6_scenario: P6 milestones (h1-h5) and the P5 world migration (m3).
p6_scenario() {
  case "$scenario" in
    h1)  # bastion: entering a real one claims once; leaving and coming back claims nothing
      local found x z probe px py pz
      p6_lane p4
      ctl command "execute in minecraft:the_nether run tp @s 0.5 100 0.5" >/dev/null
      ctl wait 60 >/dev/null
      ctl command "mistbrn-test bastion-find" >/dev/null
      found="$(p6_wait_line bastion-find)"
      x="$(p6_value "$found" x)"; z="$(p6_value "$found" z)"
      [[ "$x" =~ ^-?[0-9]+$ && "$z" =~ ^-?[0-9]+$ ]] || die "no bastion found: '$found'"
      ctl command "execute in minecraft:the_nether run tp @s $x 120 $z" >/dev/null
      ctl wait 100 >/dev/null
      ctl command "mistbrn-test bastion-probe" >/dev/null
      probe="$(p6_wait_line bastion-probe)"
      px="$(p6_value "$probe" x)"; py="$(p6_value "$probe" y)"; pz="$(p6_value "$probe" z)"
      [[ "$px" =~ ^-?[0-9]+$ && "$py" =~ ^[0-9]+$ && "$pz" =~ ^-?[0-9]+$ ]] ||
        die "no bastion piece near $x,$z: '$probe'"
      k_ok "bastion start $x,$z; piece block $px,$py,$pz"
      # Whole coordinates: a negative "x.5" floors to the next block, outside the probed one.
      ctl command "execute in minecraft:the_nether run tp @s $px $py $pz" >/dev/null
      ctl wait 60 >/dev/null
      ctl command "execute in minecraft:the_nether run tp @s $((px + 400)) 120 $pz" >/dev/null
      ctl wait 60 >/dev/null
      ctl command "execute in minecraft:the_nether run tp @s $px $py $pz" >/dev/null
      ctl wait 60 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=milestone key=bastion result=ok seq=1 unlocks=tin_pewter'
      if [[ "$(p6_milestone_lines bastion)" == 1 ]]; then k_ok "one bastion claim after re-entering"
      else k_fail "expected one bastion claim, got $(p6_milestone_lines bastion)"; fi
      expect 2 report state=active milestones=1 unlocked_pairs=tin_pewter selected_pair=iron_steel \
        last_transaction=1 ;;
    h2)  # elder guardian: the killer claims once; a second kill and a kill with no player do not
      p6_lane
      p6_kill minecraft:elder_guardian p6a
      p6_kill minecraft:elder_guardian p6b
      ctl command "summon minecraft:elder_guardian 515.5 100 515.5 {NoAI:1b,Tags:[p6c]}" >/dev/null
      ctl wait 5 >/dev/null
      ctl command "kill @e[tag=p6c]" >/dev/null
      ctl wait 40 >/dev/null
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=milestone key=elder_guardian result=ok seq=1 unlocks=zinc_brass'
      expect_log 1 'MISTBRN event=milestone key=elder_guardian result=AlreadyClaimed'
      expect_log 1 'milestone=2 claimants=0'
      if [[ "$(p6_milestone_lines elder_guardian)" == 2 ]]; then k_ok "two guardian claim attempts"
      else k_fail "expected two guardian claim attempts, got $(p6_milestone_lines elder_guardian)"; fi
      expect 2 report state=active milestones=2 unlocked_pairs=zinc_brass unlocked_iron=false \
        selected_pair=iron_steel last_transaction=1 ;;
    h3)  # ancient city: opening an untouched chest claims once; other tables claim nothing
      p6_lane
      p6_chest 514 100 512 minecraft:chests/ancient_city
      p6_chest 514 100 510 minecraft:chests/simple_dungeon
      p6_chest 514 100 514 minecraft:chests/ancient_city_ice_box
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=milestone key=ancient_city result=ok seq=1 unlocks=copper_bronze'
      expect_log 1 'MISTBRN event=milestone key=ancient_city result=AlreadyClaimed'
      if [[ "$(p6_milestone_lines ancient_city)" == 2 ]]; then k_ok "two city claim attempts"
      else k_fail "expected two city claim attempts, got $(p6_milestone_lines ancient_city)"; fi
      expect 2 report state=active milestones=4 unlocked_pairs=copper_bronze last_transaction=1 ;;
    h4)  # dragon: no death hook; its health reaching 0 after a player's hit claims knowledge once
      p6_lane
      p6_kill minecraft:ender_dragon p6a 80
      ctl command "data get entity @e[tag=p6a,limit=1] Health" > "$dir/health-p6a.json"
      p6_kill minecraft:ender_dragon p6b 80 506.5
      ctl command "data get entity @e[tag=p6b,limit=1] Health" > "$dir/health-p6b.json"
      ctl save >/dev/null; quit_game
      report; quit_game
      expect_log 1 'MISTBRN event=milestone key=dragon result=ok seq=1 unlocks=none'
      expect_log 1 'MISTBRN event=milestone key=dragon result=AlreadyClaimed'
      expect 2 report state=active milestones=8 knowledge=3 unlocked_pairs=- last_transaction=1 ;;
    h5)  # hard cut after a claim: unsaved gives the old save whole, saved the new one whole
      p6_lane
      p6_kill minecraft:elder_guardian p6a
      kill_game
      report
      p6_place
      p6_kill minecraft:elder_guardian p6b
      ctl save >/dev/null
      kill_game
      report; quit_game
      expect_log 1 'MISTBRN event=milestone key=elder_guardian result=ok seq=1 unlocks=zinc_brass'
      expect 2 report state=active milestones=0 unlocked_pairs=- last_transaction=0
      expect_log 2 'MISTBRN event=milestone key=elder_guardian result=ok seq=1 unlocks=zinc_brass'
      expect 3 report state=active milestones=2 unlocked_pairs=zinc_brass last_transaction=1 ;;
    m3)
      p6_migrate ;;
    *) return 1 ;;
  esac
}
