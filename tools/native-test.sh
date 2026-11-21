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
TICK_LINE="MISTBRN event=player_tick accepted=true tick_recorded=true"
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
# expect LAUNCH "key=value ..." : every pair must appear on this scenario's report line.
expect() {
  local n="$1"; shift
  local line; line="$(grep -a "MISTBRN_TEST scenario=$scenario " "$(log "$n")" | tail -1 || true)"
  echo "launch $n: ${line:-<no report>}"
  for pair in "$@"; do
    if [[ " $line " != *" $pair "* ]]; then echo "  FAIL missing $pair"; fails=$((fails + 1)); fi
  done
}

case "$scenario" in
  pd-roundtrip)
    launch MISTBRN_TEST_SCENARIO=pd-roundtrip; ctl wait 40 >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=pd-roundtrip; ctl wait 40 >/dev/null; quit_game
    expect 1 phase=write ok=true
    expect 2 phase=read value=1,2,3 ;;
  tx-abort)
    launch; ctl give iron_nugget 3 >/dev/null; ctl wait 20 >/dev/null; quit_game
    launch MISTBRN_TEST_SCENARIO=tx-abort; ctl wait 40 >/dev/null; quit_game
    expect 2 result=deliberate_abort count_before=3 inside_count=2 inside_pd=1 count_after=3 pd_unchanged=true ;;
  raw-root)
    launch MISTBRN_TEST_SCENARIO=raw-root; ctl wait 40 >/dev/null; quit_game
    expect 1 raw_nbt=some ;;
  *) die "unknown scenario $scenario" ;;
esac
for n in $(seq 1 "$launch_no"); do check_load "$n"; done
check_isolation

grep -a "MISTBRN" "$dir"/l*.log >> "$dir/report.txt" || true
if (( fails == 0 )); then echo "PASS $scenario ($run)"; else echo "FAIL $scenario: $fails ($run)"; exit 1; fi
