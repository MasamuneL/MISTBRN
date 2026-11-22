#!/usr/bin/env bash
# Run one in-game scenario against the dev instance's native build and check its report lines.
# Usage: tools/native-test.sh <scenario> [run-name]
#
# Each run gets its own game folder (the game's working directory): its saves, logs and caches
# never touch the player's own Horizon instance. Each launch logs beside its own control socket;
# the mod reports `MISTBRN_TEST scenario=<name> key=value ...` and this script compares them.
source "$(dirname "$0")/lib.sh"
# shellcheck source=tools/p4-lib.sh
source "$(dirname "$0")/p4-lib.sh"
# shellcheck source=tools/p7a-lib.sh
source "$(dirname "$0")/p7a-lib.sh"
require jq grep cksum

scenario="${1:-}"; [[ -n "$scenario" ]] || die "usage: $0 <scenario> [run-name]"
[[ "$scenario" == s11 ]] && require python3
run="${2:-$scenario-$(date +%Y%m%d%H%M%S)}"
RT="$(runtime_root)"; dir="$RT/runs/$run"
[[ ! -e "$dir" ]] || die "run $run exists: $dir"
mkdir -p "$dir"
world="mistbrn-$scenario"

LH="$(launcher_home)"
# The build the dev instance plays (the launcher selects it after native-build).
build="$("$(launcher)" --json instance list | jq -r --arg i "$DEV_INSTANCE" '.[] | select(.id == $i) | .version // empty')"
[[ "$build" == native-* ]] || die "instance $DEV_INSTANCE plays '$build', not a native build: run tools/build-native.sh"
V="$LH/versions/$build"
[[ -d "$V" ]] || die "build folder missing: $V"
resources="$LH/versions/$HORIZON_VERSION/.local/horizon-client/26.3"
[[ -f "$resources/horizon-resources.json" ]] || die "prepared resources missing: $resources"
case "$(os_kind)" in windows) client="$V/horizon-client.exe"; ctl_bin="$V/horizon-ctl.exe" ;;
  *) client="$V/horizon-client"; ctl_bin="$V/horizon-ctl" ;; esac
lib="$(find "$V" \( -name 'libmistbrn.dylib' -o -name 'libmistbrn.so' -o -name 'mistbrn.dll' \) -print -quit)"
{
  echo "run=$run scenario=$scenario build=$build world=$world"
  echo "mod_library=${lib##*/} sha256=$( [[ -n "$lib" ]] && sha256 "$lib" || echo none)"
  echo "git_head=$(git -C "$REPO_ROOT" rev-parse HEAD) git_dirty=$([[ -n "$(git -C "$REPO_ROOT" status --porcelain -- mod tools)" ]] && echo yes || echo no)"
} | tee "$dir/report.txt"
# Unix socket paths are limited to ~104 bytes: keep the sockets in a short temp folder and copy
# each launch's log (written beside its socket) into the run folder.
sockdir="$(mktemp -d "${TMPDIR:-/tmp}/mbrn.XXXXXX")"

launch_no=0
sock=""
cleanup() {
  if [[ -S "$sock" || -f "$sock" ]]; then
    "$ctl_bin" --socket "$sock" --timeout 5 quit >/dev/null 2>&1 || true
  fi
  cp "$sockdir"/*.log "$dir"/ 2>/dev/null || true
  rm -rf "$sockdir"
}
trap cleanup EXIT
# launch [ENV=VALUE...]: start the game in this run's folder and wait until the player is in.
launch() {
  launch_no=$((launch_no + 1)); sock="$sockdir/l$launch_no.sock"
  ( cd "$dir" && env HORIZON_NO_AUDIO=1 "$@" "$ctl_bin" --socket "$sock" --timeout 600 \
      launch --game "$client" --resources "$resources" --world "$world" --creative --wait \
      > "$dir/l$launch_no.launch.json" ) || die "launch $launch_no failed: $(tail -c 600 "$dir/l$launch_no.launch.json")"
}
ctl() {
  local result code
  if result="$("$ctl_bin" --socket "$sock" --timeout 120 "$@")"; then
    printf '%s\n' "$result"
  else
    code=$?
    printf 'horizon-ctl %s: %s\n' "$*" "$result" >&2
    return "$code"
  fi
}
game_pid() { jq -r '.launch.pid' "$dir/l$launch_no.launch.json"; }
quit_game() { ctl quit >/dev/null; wait_exit; }
# Hard cut: the process dies without saving (SIGKILL / taskkill /F).
kill_game() { proc_kill_hard "$(game_pid)"; wait_exit; }
wait_exit() {
  local pid; pid="$(game_pid)"
  for _ in $(seq 1 120); do
    if ! proc_alive "$pid"; then cp "$sockdir/l$launch_no.log" "$dir/" 2>/dev/null || true; return 0; fi
    sleep 1
  done
  die "game pid $pid did not exit"
}
log() { echo "$dir/l$1.log"; }
# G3 line the adapter prints for its first three ticks per world open.
TICK_LINE="MISTBRN event=player_tick state="
# check_load N: G2 (mod loaded) and G3 (three server ticks reached the core) for launch N.
check_load() {
  local f; f="$(log "$1")"
  if grep -aq "MISTBRN event=loaded version=" "$f"; then echo "launch $1: loaded"
  else echo "  FAIL launch $1 not loaded"; fails=$((fails + 1)); fi
  local ticks; ticks="$(grep -ac "$TICK_LINE" "$f" || true)"
  if (( ticks >= 3 )); then echo "launch $1: player_tick x$ticks"
  else echo "  FAIL launch $1 ticks=$ticks"; fails=$((fails + 1)); fi
}
# check_isolation: G4, the run folder holds only this scenario's world.
check_isolation() {
  local expected="$world " worlds
  [[ "$scenario" != c7 ]] || expected="$world $world-b "
  worlds="$(ls -1 "$dir/saves" 2>/dev/null | tr '\n' ' ')"
  if [[ "$worlds" == "$expected" ]]; then echo "isolation: saves=$worlds"
  else echo "  FAIL isolation saves='$worlds'"; fails=$((fails + 1)); fi
}
fails=0
# expect LAUNCH MOD_SCENARIO key=value... : every pair must appear on that scenario's report line.
expect() {
  local n="$1" name="$2"; shift 2
  local line; line="$(grep -a "MISTBRN_TEST scenario=$name " "$(log "$n")" | tail -1 || true)"
  echo "launch $n: ${line:-<no $name report>}"
  for pair in "$@"; do
    if [[ " $line " != *" $pair "* ]]; then echo "  FAIL missing $pair"; fails=$((fails + 1)); fi
  done
}
# field LAUNCH MOD_SCENARIO KEY: the value of KEY on that report line (empty if absent).
field() {
  { grep -a "MISTBRN_TEST scenario=$2 " "$(log "$1")" || true; } | tail -1 | tr ' ' '\n' | sed -n "s/^$3=//p"
}
# expect_log LAUNCH TEXT: TEXT appears in that launch's log.
expect_log() {
  if grep -aq "$2" "$(log "$1")"; then echo "launch $1: log has '$2'"
  else echo "  FAIL launch $1 log lacks '$2'"; fails=$((fails + 1)); fi
}
# check_range LABEL VALUE MIN MAX: MIN < VALUE < MAX for an integer VALUE (no eval).
check_range() {
  if [[ "$2" =~ ^[0-9]+$ ]] && (( $2 > $3 && $2 < $4 )); then echo "check: $1 ($2)"
  else echo "  FAIL $1 ($2 not in ($3, $4))"; fails=$((fails + 1)); fi
}
# The filled vial the "drink" scenarios consume; creative give fills hotbar slot 0 first.
give3() { ctl give mistbrn:vial_iron 3 >/dev/null; }
report() { launch MISTBRN_TEST_SCENARIO=report; ctl wait 40 >/dev/null; }
# ore_fixture [EXTRA_IRON_Z]: first launch places copper (z=3, z=5 deepslate) and iron (z=7, plus
# EXTRA_IRON_Z if given) rows and a loot chest in an existing chunk, then saves; the next launch mines.
ore_fixture() {
  launch
  ctl command "difficulty peaceful" >/dev/null
  ctl command "tp @s 0.5 165 2.5" >/dev/null
  ctl wait-for loaded --timeout 120 >/dev/null
  ctl command "fill -1 160 1 11 165 12 minecraft:air" >/dev/null
  ctl command "fill -1 159 1 11 159 12 minecraft:stone" >/dev/null
  ctl command "fill 0 160 3 7 160 3 minecraft:copper_ore" >/dev/null
  ctl command "fill 0 160 5 7 160 5 minecraft:deepslate_copper_ore" >/dev/null
  ctl command "fill 0 160 7 7 160 7 minecraft:deepslate_iron_ore" >/dev/null
  [[ -z "${1:-}" ]] || ctl command "fill 0 160 $1 7 160 $1 minecraft:iron_ore" >/dev/null
  ctl command 'setblock 9 160 10 minecraft:chest{LootTable:"minecraft:chests/simple_dungeon",LootTableSeed:1L}' >/dev/null
  ctl give diamond_pickaxe >/dev/null
  ctl command "tp @s 0.5 160 2.5" >/dev/null
  ctl command "gamemode survival" >/dev/null
  ctl save >/dev/null; quit_game
}
# mine_row Z: break x=0..7 at y=160 in row Z, collect, and save the inventory as mining-Z.json.
mine_row() {
  for x in $(seq 0 7); do
    ctl command "tp @s $x.5 160 $(($1 - 1)).5" >/dev/null
    ctl wait 3 >/dev/null
    ctl break "$x" 160 "$1" >> "$dir/mining.jsonl"
  done
  ctl collect 12 >> "$dir/mining.jsonl"
  ctl inventory > "$dir/mining-$1.json"
}
# slot_of ITEM: the first inventory slot (0-35) holding ITEM, from a fresh inventory read.
slot_of() {
  ctl inventory | jq -er --arg i "$1" '[.hotbar[], .main[]] | map(select(.item == $i)) | .[0].slot'
}
# put ITEM GRID...: place one ITEM from the inventory in each crafting GRID slot of the open screen.
put() {
  local item="$1" src; shift; src="$(slot_of "$item")" || die "no $item to craft with"
  ctl click-slot "$src" >/dev/null
  for g in "$@"; do ctl click-slot "$g" right >/dev/null; done
  ctl click-slot "$src" >/dev/null
}
# take: move the crafting result into the inventory.
take() { ctl wait 5 >/dev/null; ctl click-slot 200 shift >/dev/null; ctl wait 5 >/dev/null; }
# table: place a crafting table beside the player and open it.
table() {
  local px py pz
  read -r px py pz < <(ctl state | jq -r '.player.position | map(floor) | "\(.[0]) \(.[1]) \(.[2])"')
  ctl setblock $((px + 2)) "$py" "$pz" crafting_table >/dev/null
  ctl interact $((px + 2)) "$py" "$pz" >/dev/null
}

case "$scenario" in
  n1)  # Pull toward an iron block 12 blocks away and stop at its collision face.
    p4_fixture
    p4_iron_block 524
    p4_burn iron
    ctl wait 100 >/dev/null
    p4_off; quit_game
    p4_check_log ;;
  n2)  # Push away from the iron block and stop against the glass wall.
    p4_fixture
    p4_iron_block 524
    ctl command "fill 508 100 511 508 103 513 minecraft:glass" >/dev/null
    p4_burn steel
    ctl wait 100 >/dev/null
    p4_off; quit_game
    p4_check_log ;;
  n3)  # Push downward on a grounded clip in Survival, hover, then fall with steel off.
    p4_fixture
    ctl command "tp @s 510.5 100 512.5" >/dev/null
    ctl command 'summon item 512.5 100 512.5 {Item:{id:"mistbrn:clip",count:1},PickupDelay:32767}' >/dev/null
    ctl wait 10 >/dev/null
    ctl command "gamemode survival" >/dev/null
    ctl command "mistbrn-test credit steel 25000" >/dev/null
    ctl command "tp @s 512.5 102 512.5" >/dev/null
    ctl look 0 90 >/dev/null
    ctl command "mistbrn-test burn steel" >/dev/null
    ctl wait 160 >/dev/null
    p4_off
    ctl wait 60 >/dev/null
    ctl state > "$dir/after-off.json"
    quit_game
    p4_check_log "$(jq -r '.player.position[1] // .position[1] // "missing"' "$dir/after-off.json")" ;;
  n4)  # Pull in an iron ingot six blocks away without displacing the player.
    p4_fixture
    ctl command 'summon item 518.5 100 512.5 {Item:{id:"minecraft:iron_ingot",count:1},PickupDelay:0}' >/dev/null
    ctl look-at 518.5 100.125 512.5 >/dev/null
    p4_burn iron
    ctl wait 80 >/dev/null
    p4_off
    ctl inventory > "$dir/pickup-inventory.json"
    quit_game
    p4_check_log
    ingots="$(jq -r '.counts["minecraft:iron_ingot"] // 0' "$dir/pickup-inventory.json")"
    echo "check: ingots_collected=$ingots" | tee -a "$dir/report.txt"
    if [[ "$ingots" != 1 ]]; then p4_fail "expected to collect one iron ingot"; fi ;;
  n11)  # N4 in the Nether: powers run per level, so a parked level's entities are reachable.
    p4_fixture
    ctl command "execute in minecraft:the_nether run tp @s 512.5 112 512.5" >/dev/null
    ctl wait-for loaded --timeout 120 >/dev/null; ctl wait 40 >/dev/null
    for _ in 1 2; do
      ctl command "execute in minecraft:the_nether run fill 503 100 508 540 116 516 minecraft:air" >/dev/null
      ctl wait 20 >/dev/null
    done
    ctl command "execute in minecraft:the_nether run fill 503 99 508 540 99 516 minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_nether run tp @s 512.5 100 512.5" >/dev/null
    ctl wait 5 >/dev/null
    ctl command 'execute in minecraft:the_nether run summon item 518.5 100 512.5 {Item:{id:"minecraft:iron_ingot",count:1},PickupDelay:0}' >/dev/null
    ctl look-at 518.5 100.125 512.5 >/dev/null
    p4_burn iron
    ctl wait 80 >/dev/null
    p4_off
    ctl inventory > "$dir/pickup-inventory.json"
    ctl state > "$dir/nether-state.json"
    quit_game
    p4_check_log
    ingots="$(jq -r '.counts["minecraft:iron_ingot"] // 0' "$dir/pickup-inventory.json")"
    dimension="$(jq -r '.dimension // .player.dimension // .world.dimension // "missing"' "$dir/nether-state.json")"
    echo "check: ingots_collected=$ingots dimension=$dimension" | tee -a "$dir/report.txt"
    if [[ "$ingots" != 1 ]]; then p4_fail "expected to collect one iron ingot in the Nether"; fi
    if [[ "$dimension" != minecraft:the_nether ]]; then p4_fail "expected to be in the Nether"; fi ;;
  n5)  # Push a single clip through a stationary zombie at capped projectile speed.
    p4_fixture
    ctl command "time set midnight" >/dev/null
    ctl command "difficulty normal" >/dev/null
    ctl command 'summon zombie 520.5 100 512.5 {NoAI:1b,Tags:["mistbrn_p4"]}' >/dev/null
    ctl command 'summon item 514 100 512.5 {Item:{id:"mistbrn:clip",count:1},PickupDelay:32767}' >/dev/null
    ctl look-at 514 100.125 512.5 >/dev/null
    p4_burn steel
    ctl wait 30 >/dev/null
    p4_off
    quit_game
    p4_check_log ;;
  n5w)  # The same shot with glass between the clip and the zombie: the wall takes it.
    p4_fixture
    ctl command "time set midnight" >/dev/null
    ctl command "difficulty normal" >/dev/null
    # Glass at x=516..517; the zombie at 517.5 is within one 2-block coin move of the glass face.
    ctl command 'summon zombie 517.5 100 512.5 {NoAI:1b,Tags:["mistbrn_p4"]}' >/dev/null
    ctl command "fill 516 100 511 516 103 514 minecraft:glass" >/dev/null
    ctl command 'summon item 514 100 512.5 {Item:{id:"mistbrn:clip",count:1},PickupDelay:32767}' >/dev/null
    ctl look-at 514 100.125 512.5 >/dev/null
    p4_burn steel
    ctl wait 30 >/dev/null
    p4_off
    quit_game
    p4_check_log ;;
  n6)  # Reach an anchor 20 blocks away while flared and spend triple the normal rate.
    p4_fixture
    p4_iron_block 532
    p4_burn iron 25000 flare
    ctl wait 100 >/dev/null
    p4_off
    ctl save >/dev/null; quit_game
    report; quit_game
    expect 2 report state=active
    p4_check_log "$(field 2 report reserve_iron)" ;;
  n7)  # Spend 200 milli in exactly 20 ticks and stop reporting motion on exhaustion.
    p4_fixture
    p4_iron_block 524
    p4_burn iron 200
    ctl wait 60 >/dev/null
    quit_game
    p4_check_log ;;
  n8)  # Die during an iron pull, respawn with an empty reserve, and never resume the pull.
    p4_fixture
    p4_iron_block 524
    p4_burn iron
    ctl wait 15 >/dev/null  # long enough for a measurable pull before the stop
    ctl command "kill @s" >/dev/null
    ctl wait 20 >/dev/null; ctl respawn >/dev/null; ctl wait 40 >/dev/null
    ctl save >/dev/null; quit_game
    report; quit_game
    p4_check_log
    expect 2 report state=active reserve_iron=0 ;;
  n9)  # Change dimension during an iron pull and verify that iron stops in the same session.
    p4_fixture
    p4_iron_block 524
    p4_burn iron
    ctl wait 15 >/dev/null  # long enough for a measurable pull before the stop
    ctl command "execute in minecraft:the_nether run tp @s 0 100 0" >/dev/null
    ctl wait-for loaded --timeout 120 >/dev/null; ctl wait 40 >/dev/null
    ctl state > "$dir/nether-state.json"
    quit_game
    p4_check_log
    dimension="$(jq -r '.dimension // .player.dimension // .world.dimension // "missing"' "$dir/nether-state.json")"
    echo "check: dimension=$dimension" | tee -a "$dir/report.txt"
    if [[ "$dimension" != minecraft:the_nether ]]; then p4_fail "expected to reach the Nether"; fi ;;
  n10)  # Remove the iron block during a pull and clear the invalid target.
    p4_fixture
    p4_iron_block 524
    p4_burn iron
    ctl wait 15 >/dev/null  # long enough for a measurable pull before the stop
    ctl command "setblock 524 101 512 minecraft:air" >/dev/null
    ctl wait 40 >/dev/null
    p4_off; quit_game
    p4_check_log ;;
  vial-smoke)
    launch
    ctl command "gamemode survival" >/dev/null
    ctl give glass 2 >/dev/null
    ctl open-inventory >/dev/null
    ctl click-slot 0 >/dev/null
    ctl click-slot 100 right >/dev/null
    ctl click-slot 102 >/dev/null
    ctl wait 10 >/dev/null
    ctl screen > "$dir/craft-screen.json"
    ctl click-slot 200 shift >/dev/null
    ctl close >/dev/null
    ctl inventory > "$dir/craft-inventory.json"
    quit_game
    jq -e '.counts["mistbrn:vial_empty"] == 2 and (.counts["minecraft:glass"] // 0) == 0' \
      "$dir/craft-inventory.json" >/dev/null || die "expected two crafted vials and no glass" ;;
  d1)  # craft every P3b accessory: four in the 2x2 grid, two at a crafting table
    launch
    ctl command "gamemode survival" >/dev/null
    for i in glass stick stick stick string copper_ingot gold_ingot iron_ingot oak_log; do ctl give "$i" 1 >/dev/null; done
    ctl give gray_wool 8 >/dev/null; ctl give mistbrn:bronze_ingot 1 >/dev/null; ctl wait 10 >/dev/null
    ctl open-inventory >/dev/null
    put minecraft:glass 100; put minecraft:stick 102; take
    put mistbrn:bronze_ingot 100; put minecraft:string 101; take
    # One ingot alone still makes vanilla nuggets: no mod recipe shadows them.
    put minecraft:copper_ingot 100; take
    put minecraft:gold_ingot 100; take
    put minecraft:iron_ingot 100; take
    put minecraft:copper_nugget 100; take
    put minecraft:gold_nugget 100; take
    ctl close >/dev/null
    table
    put minecraft:oak_log 101; put minecraft:stick 104 107; take
    put minecraft:gray_wool 100 102 103 104 105 106 107 108; take
    ctl close >/dev/null
    ctl inventory > "$dir/crafted-inventory.json"
    quit_game
    jq -e '.counts == {"minecraft:copper_nugget": 8, "minecraft:gold_nugget": 8, "minecraft:iron_nugget": 9,
        "mistbrn:boxing": 1, "mistbrn:bronze_earring": 1, "mistbrn:clip": 1, "mistbrn:dueling_cane": 1,
        "mistbrn:glass_dagger": 1, "mistbrn:mistcloak": 1}' \
      "$dir/crafted-inventory.json" >/dev/null || die "crafted accessories or leftovers differ: $(jq -c .counts "$dir/crafted-inventory.json")"
    echo "check: six accessories crafted, vanilla nuggets intact, exact leftovers" ;;
  d2)  # wear the cloak and the earring; both survive a save and a relaunch
    launch
    ctl command "gamemode survival" >/dev/null
    ctl give mistbrn:mistcloak 1 >/dev/null; ctl give mistbrn:bronze_earring 1 >/dev/null; ctl wait 10 >/dev/null
    ctl select mistbrn:mistcloak >/dev/null; ctl use-item >/dev/null; ctl wait 5 >/dev/null
    ctl select mistbrn:bronze_earring >/dev/null; ctl use-item >/dev/null; ctl wait 5 >/dev/null
    ctl key f5 tap >/dev/null 2>&1 || true; ctl wait 10 >/dev/null
    ctl screenshot "$dir/third-person.png" >/dev/null 2>&1 || true
    ctl save >/dev/null; quit_game
    launch; ctl inventory > "$dir/worn-inventory.json"; quit_game
    jq -e '[.armor[] | select(.item != null) | {part, item}] | sort_by(.part)
        == [{part: "chest", item: "mistbrn:mistcloak"}, {part: "head", item: "mistbrn:bronze_earring"}]' \
      "$dir/worn-inventory.json" >/dev/null || die "worn accessories differ: $(jq -c .armor "$dir/worn-inventory.json")"
    jq -e '([.hotbar[], .main[]] | map(select(.item != null)) | length) == 0
        and .counts == {"mistbrn:bronze_earring": 1, "mistbrn:mistcloak": 1}' \
      "$dir/worn-inventory.json" >/dev/null || die "equipping left a copy in the inventory"
    echo "check: cloak on chest and earring on head after relaunch" ;;
  d3)  # a full reserve refuses the fifth vial with a translated message and keeps it
    launch
    ctl command "gamemode survival" >/dev/null
    ctl give mistbrn:vial_iron 5 >/dev/null; ctl wait 10 >/dev/null
    ctl select mistbrn:vial_iron >/dev/null
    for _ in 1 2 3 4 5; do ctl use-item --hold 1.8 >/dev/null; ctl wait 10 >/dev/null; done
    ctl messages > "$dir/messages.json"
    ctl save >/dev/null; quit_game
    report; quit_game
    [[ "$(grep -ac "MISTBRN event=vial_credit metal=iron" "$(log 1)")" == 4 ]] || die "expected four credits"
    expect_log 1 "MISTBRN event=vial_refused metal=iron reason=InvalidReserves(CapacityExceeded(Iron))"
    [[ "$(grep -ac "MISTBRN event=vial_refused" "$(log 1)")" == 1 ]] || die "expected exactly one refusal"
    jq -e '[.styled[] | select(.translate? == "message.mistbrn.vial.reserve_full")] | length == 1' \
      "$dir/messages.json" >/dev/null || die "missing reserve-full translation key in chat"
    grep -q "that reserve is full" "$dir/messages.json" || die "reserve-full key did not resolve to its text"
    expect 2 report state=active reserve_iron=100000 last_transaction=4 vials=1 empty_vials=4 ;;
  c1)  # vial chain in the 2x2 grid: glass -> empty vials, ingot + stone -> flakes, vial + flakes
    launch
    ctl command "gamemode survival" >/dev/null
    for i in glass glass iron_ingot cobblestone; do ctl give "$i" 1 >/dev/null; done
    ctl wait 10 >/dev/null
    ctl open-inventory >/dev/null
    put minecraft:glass 100; put minecraft:glass 102; take
    put minecraft:iron_ingot 100; put minecraft:cobblestone 101; take
    put mistbrn:vial_empty 100; put mistbrn:iron_flakes 101; take
    ctl close >/dev/null
    ctl inventory > "$dir/craft-inventory.json"
    quit_game
    jq -e '.counts == {"mistbrn:iron_flakes": 1, "mistbrn:vial_empty": 1, "mistbrn:vial_iron": 1}' \
      "$dir/craft-inventory.json" >/dev/null || die "crafting ledger differs: $(jq -c .counts "$dir/craft-inventory.json")" ;;
  c2|c3|c4|c5)
    # Seed and save an inventory, then use the actual item in Survival.
    launch
    ctl command "gamemode survival" >/dev/null
    give3; ctl save >/dev/null; quit_game
    if [[ "$scenario" == c5 ]]; then launch MISTBRN_TEST_FAIL_VIAL=1
    else launch; fi
    ctl select mistbrn:vial_iron >/dev/null
    if [[ "$scenario" == c2 ]]; then
      ctl mouse right down >/dev/null
      ctl wait 23 >/dev/null
      ctl status > "$dir/drinking-status.json"
      ctl mouse right up >/dev/null
      ctl wait 35 >/dev/null
      ctl inventory > "$dir/short-use-inventory.json"
    fi
    ctl use-item --hold 1.8 > "$dir/use-item.json"
    ctl wait 10 >/dev/null
    case "$scenario" in
      c3) kill_game ;;
      c4) ctl save >/dev/null; kill_game ;;
      *) ctl save >/dev/null; quit_game ;;
    esac
    report; quit_game
    if [[ "$scenario" == c5 ]]; then
      expect_log 2 "MISTBRN event=vial_refused metal=iron reason=injected_failure"
    else
      expect_log 2 "MISTBRN event=vial_credit metal=iron seq=1"
    fi
    case "$scenario" in
      c2|c4) expect 3 report state=active reserve_iron=25000 last_transaction=1 vials=2 empty_vials=1 ;;
      c3|c5) expect 3 report state=active reserve_iron=0 last_transaction=0 vials=3 empty_vials=0 ;;
    esac
    if [[ "$scenario" == c2 ]]; then
      jq -e '.using[0] == 0 and .using[1] >= 23 and .using[1] < 32' \
        "$dir/drinking-status.json" >/dev/null || die "expected the main-hand drink to still be in progress"
      jq -e '.counts["mistbrn:vial_iron"] == 3 and (.counts["mistbrn:vial_empty"] // 0) == 0' \
        "$dir/short-use-inventory.json" >/dev/null || die "an interrupted drink consumed the vial"
    fi ;;
  c2b)
    launch
    ctl command "gamemode survival" >/dev/null
    ctl command "item replace entity @s weapon.offhand with mistbrn:vial_iron 1" >/dev/null
    ctl use-item --hold 1.8 >/dev/null
    ctl wait 10 >/dev/null
    ctl inventory > "$dir/offhand-inventory.json"
    quit_game
    expect_log 1 "MISTBRN event=vial_credit metal=iron seq=1"
    jq -e '.offhand.item == "mistbrn:vial_empty" and .offhand.count == 1 and (.counts["mistbrn:vial_iron"] // 0) == 0' \
      "$dir/offhand-inventory.json" >/dev/null || die "the last filled offhand vial should become an empty vial" ;;
  c6f)  # Fortune III multiplies the samples deterministically (core test c6_fortune_fixture_counts)
    ore_fixture 9
    launch
    ctl select diamond_pickaxe >/dev/null
    ctl command "enchant @s minecraft:fortune 3" > "$dir/fortune-command.json"
    ctl wait 10 >/dev/null
    ctl inventory > "$dir/fortune-tool.json"
    jq -e '.hotbar | any(.item == "minecraft:diamond_pickaxe" and ((.components // "") | contains("minecraft:fortune")))' \
      "$dir/fortune-tool.json" >/dev/null || die "the mining tool did not receive Fortune"
    mine_row 3; mine_row 9
    quit_game
    jq -e '.counts["mistbrn:raw_tin"] == 4' "$dir/mining-3.json" >/dev/null ||
      die "expected four tin samples with Fortune III: $(jq -c .counts "$dir/mining-3.json")"
    jq -e '.counts["mistbrn:raw_tin"] == 4 and .counts["mistbrn:raw_zinc"] == 9' "$dir/mining-9.json" >/dev/null ||
      die "expected nine zinc samples with Fortune III (three without): $(jq -c .counts "$dir/mining-9.json")"
    echo "check: Fortune III gave 4 tin (1 without) and 9 zinc (3 without) at the fixed positions" ;;
  c6)
    # Place the fixture in an existing chunk; only the second launch mines it.
    ore_fixture
    launch
    ctl select diamond_pickaxe >/dev/null
    for z in 3 7 5; do
      if [[ "$z" == 5 ]]; then
        ctl command "enchant @s minecraft:silk_touch 1" > "$dir/silk-touch-command.json"
        ctl wait 10 >/dev/null
        ctl inventory > "$dir/silk-touch-tool.json"
        jq -e '.hotbar | any(.item == "minecraft:diamond_pickaxe" and ((.components // "") | contains("minecraft:silk_touch")))' \
          "$dir/silk-touch-tool.json" >/dev/null || die "the mining tool did not receive Silk Touch"
      fi
      for x in $(seq 0 7); do
        ctl command "tp @s $x.5 160 $((z - 1)).5" >/dev/null
        ctl wait 3 >/dev/null
        ctl break "$x" 160 "$z" >> "$dir/mining.jsonl"
      done
      ctl collect 12 >> "$dir/mining.jsonl"
      ctl inventory > "$dir/mining-$z.json"
    done
    ctl command "tp @s 9.5 160 9.5" >/dev/null
    ctl wait 3 >/dev/null
    ctl interact 9 160 10 > "$dir/chest-open.json"
    ctl menu > "$dir/chest-menu.json"
    ctl close >/dev/null; quit_game
    # Copper: x=5 at z=3. Iron: x=1,6,7 at z=7. The z=5 row includes
    # eligible positions too, but Silk Touch must leave both sample totals unchanged.
    jq -e '.counts["mistbrn:raw_tin"] == 1' "$dir/mining-3.json" >/dev/null ||
      die "expected exactly one tin sample from copper"
    jq -e '.counts["mistbrn:raw_tin"] == 1 and .counts["mistbrn:raw_zinc"] == 3 and .counts["minecraft:deepslate_copper_ore"] == 8' \
      "$dir/mining-5.json" >/dev/null || die "zinc samples or Silk Touch drops differ from expected"
    jq -e '[.. | objects | select(.item? == "mistbrn:vial_tin") | .count] == [1]' "$dir/chest-menu.json" >/dev/null ||
      die "expected one discovery vial in the previously unopened loot chest" ;;
  c7)
    launch MISTBRN_TEST_SCENARIO=report
    ctl wait 40 >/dev/null
    ctl command "gamemode survival" >/dev/null
    give3; ctl wait 10 >/dev/null; ctl select mistbrn:vial_iron >/dev/null
    ctl use-item --hold 1.8 >/dev/null
    ctl wait 10 >/dev/null
    ctl pause >/dev/null
    ctl click-button "Save and Quit" >/dev/null
    ctl wait-for screen title --timeout 120 >/dev/null
    # This boundary is still in the first process, before world B has opened.
    live_log="$sockdir/l1.log"
    [[ "$(grep -ac 'MISTBRN event=save_loaded' "$live_log")" == 1 ]] ||
      die "expected one player load before leaving world A"
    grep -aq 'MISTBRN event=vial_credit metal=iron seq=1' "$live_log" ||
      die "expected a vial credit in world A"
    world_b_line=$(( $(wc -l < "$live_log") + 1 ))
    ctl open "$world-b" 42 creative >/dev/null
    ctl wait-for loaded --timeout 120 >/dev/null
    ctl wait 40 >/dev/null
    ctl inventory > "$dir/second-world-inventory.json"
    quit_game
    world_b_loads="$(tail -n "+$world_b_line" "$(log 1)" | grep -a 'MISTBRN event=save_loaded' || true)"
    [[ "$world_b_loads" == 'MISTBRN event=save_loaded state=fresh' ]] ||
      die "expected one fresh player load after opening world B"
    [[ "$(grep -ac "$TICK_LINE" "$(log 1)")" == 6 ]] ||
      die "expected three tick reports for each world"
    jq -e '(.counts["mistbrn:vial_iron"] // 0) == 0 and (.counts["mistbrn:vial_empty"] // 0) == 0' \
      "$dir/second-world-inventory.json" >/dev/null || die "vials from world A appeared in world B"
    # Reopen B in a fresh process to report exactly what the first process saved there.
    world_a="$world"; world="$world-b"
    report; quit_game
    world="$world_a"
    expect_log 2 "MISTBRN event=save_loaded state=found"
    expect 2 report state=active reserve_iron=0 last_transaction=0 vials=0 empty_vials=0 \
      "uuid=$(field 1 report uuid)" ;;
  pd-roundtrip)
    launch MISTBRN_TEST_SCENARIO=pd-roundtrip; ctl wait 40 >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=pd-roundtrip; ctl wait 40 >/dev/null; quit_game
    expect 1 pd-roundtrip phase=write ok=true
    expect 2 pd-roundtrip phase=read value=1,2,3 ;;
  tx-abort)
    launch; give3; ctl wait 20 >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=tx-abort; ctl wait 40 >/dev/null; quit_game
    expect 2 tx-abort result=deliberate_abort count_before=3 inside_count=2 inside_pd=1 count_after=3 pd_unchanged=true ;;
  raw-root)
    launch MISTBRN_TEST_SCENARIO=raw-root; ctl wait 40 >/dev/null; quit_game
    expect 1 raw-root raw_nbt=some ;;
  s1)  # round trip of one transactional credit
    launch MISTBRN_TEST_SCENARIO=drink; give3; ctl wait 40 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink result=ok seq=1
    expect 2 report state=active reserve_iron=25000 unlocked_iron=true last_transaction=1 vials=2 empty_vials=1 ;;
  s2)  # failure inside the transaction rolls back inventory and save together
    launch MISTBRN_TEST_SCENARIO=drink-fail; give3; ctl wait 40 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink-fail result=err:injected_failure reserve_iron=0 last_transaction=0 vials=3 empty_vials=0
    expect 2 report state=active reserve_iron=0 last_transaction=0 vials=3 empty_vials=0 ;;
  s3)  # hard cut after the credit, before any save: the old state, whole
    launch; give3; ctl wait 20 >/dev/null; ctl save >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=drink; ctl wait 40 >/dev/null; kill_game
    report; quit_game
    expect 2 drink result=ok seq=1
    expect 3 report state=active reserve_iron=0 last_transaction=0 vials=3 empty_vials=0 ;;
  s4)  # hard cut after a save: the new state, whole
    launch; give3; ctl wait 20 >/dev/null; ctl save >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=drink; ctl wait 40 >/dev/null; ctl save >/dev/null; kill_game
    report; quit_game
    expect 2 drink result=ok seq=1
    expect 3 report state=active reserve_iron=25000 last_transaction=1 vials=2 empty_vials=1 ;;
  s5)  # death clears reserves, keeps unlocks and the transaction counter
    launch MISTBRN_TEST_SCENARIO=drink; give3; ctl wait 40 >/dev/null
    ctl command "kill @s" >/dev/null; ctl wait 20 >/dev/null; ctl respawn >/dev/null; ctl wait 40 >/dev/null
    ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink result=ok seq=1
    expect_log 1 "MISTBRN event=death_cleared"
    expect 2 report state=active reserve_iron=0 burn_iron=off unlocked_iron=true last_transaction=1 ;;
  s5b)  # leaving from the death screen must not keep the reserves
    launch MISTBRN_TEST_SCENARIO=drink; give3; ctl wait 40 >/dev/null
    ctl command "kill @s" >/dev/null; ctl wait 20 >/dev/null; quit_game
    launch; ctl respawn >/dev/null; ctl wait 40 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink result=ok seq=1
    expect_log 2 "MISTBRN event=respawn_death_applied"
    expect 3 report state=active reserve_iron=0 burn_iron=off unlocked_iron=true last_transaction=1 ;;
  s6)  # reconnect stops burns and keeps the (partly consumed) reserve
    launch MISTBRN_TEST_SCENARIO=burn; give3; ctl wait 60 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 burn result=ok seq=1
    expect 2 report state=active burn_iron=off unlocked_iron=true last_transaction=1
    check_range "reserve consumed but not empty" "$(field 2 report reserve_iron)" 0 25000 ;;
  s7)  # a dimension change keeps everything
    launch MISTBRN_TEST_SCENARIO=drink; give3; ctl wait 40 >/dev/null
    ctl command "execute in minecraft:the_nether run tp @s 0 100 0" >/dev/null; ctl wait 60 >/dev/null
    ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink result=ok seq=1
    expect 2 report state=active reserve_iron=25000 last_transaction=1 dimension=minecraft:the_nether ;;
  s8)  # burning to exhaustion turns the metal off and empties it exactly
    launch MISTBRN_TEST_SCENARIO=burn-small; give3; ctl wait 160 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 burn-small result=ok seq=1 reserve_iron=1000
    expect_log 1 "MISTBRN event=exhausted metal=iron"
    expect 2 report state=active reserve_iron=0 burn_iron=off remainder_iron=0 last_transaction=1 ;;
  s9|s10)  # an unreadable save blocks the player and is never overwritten
    plant=corrupt-crc; reason=crc
    [[ "$scenario" == s10 ]] && { plant=future-schema; reason=unsupported_version:2; }
    launch MISTBRN_TEST_SCENARIO=$plant; ctl wait 40 >/dev/null; ctl save >/dev/null; quit_game
    report; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 $plant written=true
    h="$(field 1 $plant raw_hash)"
    expect_log 2 "MISTBRN event=save_blocked reason=$reason"
    expect 2 report state=blocked reason=$reason raw_hash=$h
    expect 3 report state=blocked reason=$reason raw_hash=$h ;;
  s11)  # a non-compound NeoForgeData root on disk (edited with the game closed)
    report; ctl save >/dev/null; quit_game
    # Edit exactly the controlled player's file: a world can hold other player files.
    uuid="$(field 1 report uuid)"
    file="$dir/saves/$world/players/data/$uuid.dat"
    [[ -n "$uuid" && -f "$file" ]] || die "player file for uuid '$uuid' not found in $world"
    python3 "$REPO_ROOT/tools/nbt-poke.py" "$file" NeoForgeData 7 | tee -a "$dir/report.txt"
    report; quit_game
    state="$(field 2 report state)"; why="$(field 2 report reason)"
    echo "S11 outcome: state=$state reason=${why:-none}" | tee -a "$dir/report.txt"
    expect 2 report state=blocked reason=root_not_compound uuid=$uuid ;;
  k[1-8])  # P7a keys and HUD (tools/p7a-lib.sh)
    k_scenario ;;
  *) die "unknown scenario $scenario" ;;
esac
for n in $(seq 1 "$launch_no"); do check_load "$n"; done
check_isolation

grep -a "MISTBRN" "$dir"/l*.log >> "$dir/report.txt" || true
if (( fails == 0 )); then echo "PASS $scenario ($run)"; else echo "FAIL $scenario: $fails ($run)"; exit 1; fi
