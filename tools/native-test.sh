#!/usr/bin/env bash
# Run one in-game scenario against the dev instance's native build and check its report lines.
# Usage: tools/native-test.sh <scenario> [run-name]
#
# Each run gets its own game folder (the game's working directory): its saves, logs and caches
# never touch the player's own Horizon instance. Each launch logs beside its own control socket;
# the mod reports `MISTBRN_TEST scenario=<name> key=value ...` and this script compares them.
source "$(dirname "$0")/lib.sh"
require jq grep cksum

scenario="${1:-}"; [[ -n "$scenario" ]] || die "usage: $0 <scenario> [run-name]"
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
  echo "git_head=$(git -C "$REPO_ROOT" rev-parse HEAD) git_dirty=$([[ -n "$(git -C "$REPO_ROOT" status --porcelain -- mod)" ]] && echo yes || echo no)"
} | tee "$dir/report.txt"
# Unix socket paths are limited to ~104 bytes: keep the sockets in a short temp folder and copy
# each launch's log (written beside its socket) into the run folder.
sockdir="$(mktemp -d "${TMPDIR:-/tmp}/mbrn.XXXXXX")"
trap 'cp "$sockdir"/*.log "$dir"/ 2>/dev/null || true; rm -rf "$sockdir"' EXIT

launch_no=0
sock=""
# launch [ENV=VALUE...]: start the game in this run's folder and wait until the player is in.
launch() {
  launch_no=$((launch_no + 1)); sock="$sockdir/l$launch_no.sock"
  ( cd "$dir" && env HORIZON_NO_AUDIO=1 "$@" "$ctl_bin" --socket "$sock" --timeout 600 \
      launch --game "$client" --resources "$resources" --world "$world" --creative --wait \
      > "$dir/l$launch_no.launch.json" ) || die "launch $launch_no failed: $(tail -c 600 "$dir/l$launch_no.launch.json")"
}
ctl() { "$ctl_bin" --socket "$sock" --timeout 120 "$@"; }
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
  local worlds; worlds="$(ls -1 "$dir/saves" 2>/dev/null | tr '\n' ' ')"
  if [[ "$worlds" == "$world " ]]; then echo "isolation: saves=$worlds"
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
# field LAUNCH MOD_SCENARIO KEY: the value of KEY on that report line.
field() {
  grep -a "MISTBRN_TEST scenario=$2 " "$(log "$1")" | tail -1 | tr ' ' '\n' | sed -n "s/^$3=//p"
}
# expect_log LAUNCH TEXT: TEXT appears in that launch's log.
expect_log() {
  if grep -aq "$2" "$(log "$1")"; then echo "launch $1: log has '$2'"
  else echo "  FAIL launch $1 log lacks '$2'"; fails=$((fails + 1)); fi
}
check() { if eval "$1"; then echo "check: $2"; else echo "  FAIL $2"; fails=$((fails + 1)); fi; }
# The nugget the "drink" scenarios consume; creative give fills hotbar slot 0 first.
give3() { ctl give iron_nugget 3 >/dev/null; }
report() { launch MISTBRN_TEST_SCENARIO=report; ctl wait 40 >/dev/null; }

case "$scenario" in
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
    expect 2 report state=active reserve_iron=25000 unlocked_iron=true last_transaction=1 nuggets=2 ;;
  s2)  # failure inside the transaction rolls back inventory and save together
    launch MISTBRN_TEST_SCENARIO=drink-fail; give3; ctl wait 40 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink-fail result=err:injected_failure reserve_iron=0 last_transaction=0 nuggets=3
    expect 2 report state=active reserve_iron=0 last_transaction=0 nuggets=3 ;;
  s3)  # hard cut after the credit, before any save: the old state, whole
    launch; give3; ctl wait 20 >/dev/null; ctl save >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=drink; ctl wait 40 >/dev/null; kill_game
    report; quit_game
    expect 2 drink result=ok seq=1
    expect 3 report state=active reserve_iron=0 last_transaction=0 nuggets=3 ;;
  s4)  # hard cut after a save: the new state, whole
    launch; give3; ctl wait 20 >/dev/null; ctl save >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=drink; ctl wait 40 >/dev/null; ctl save >/dev/null; kill_game
    report; quit_game
    expect 2 drink result=ok seq=1
    expect 3 report state=active reserve_iron=25000 last_transaction=1 nuggets=2 ;;
  s5)  # death clears reserves, keeps unlocks and the transaction counter
    launch MISTBRN_TEST_SCENARIO=drink; give3; ctl wait 40 >/dev/null
    ctl command "kill @s" >/dev/null; ctl wait 20 >/dev/null; ctl respawn >/dev/null; ctl wait 40 >/dev/null
    ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 drink result=ok seq=1
    expect_log 1 "MISTBRN event=death_cleared"
    expect 2 report state=active reserve_iron=0 burn_iron=off unlocked_iron=true last_transaction=1 ;;
  s6)  # reconnect stops burns and keeps the (partly consumed) reserve
    launch MISTBRN_TEST_SCENARIO=burn; give3; ctl wait 60 >/dev/null; ctl save >/dev/null; quit_game
    report; quit_game
    expect 1 burn result=ok seq=1
    expect 2 report state=active burn_iron=off unlocked_iron=true last_transaction=1
    r="$(field 2 report reserve_iron)"
    check "[[ -n '$r' ]] && (( r > 0 && r < 25000 ))" "reserve consumed but not empty ($r)" ;;
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
  *) die "unknown scenario $scenario" ;;
esac
for n in $(seq 1 "$launch_no"); do check_load "$n"; done
check_isolation

grep -a "MISTBRN" "$dir"/l*.log >> "$dir/report.txt" || true
if (( fails == 0 )); then echo "PASS $scenario ($run)"; else echo "FAIL $scenario: $fails ($run)"; exit 1; fi
