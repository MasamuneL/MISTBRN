#!/usr/bin/env bash
# Run one in-game scenario against the dev instance's native build and check its report lines.
# Usage: tools/native-test.sh <scenario> [run-name]
#
# Each run gets its own game folder (the game's working directory): its saves, logs and caches
# never touch the player's own Horizon instance. Each launch logs beside its own control socket;
# the mod reports `MISTBRN_TEST scenario=<name> key=value ...` and this script compares them.
source "$(dirname "$0")/lib.sh"

scenario="${1:-}"; [[ -n "$scenario" ]] || die "usage: $0 <scenario> [run-name]"
run="${2:-$scenario-$(date +%Y%m%d%H%M%S)}"
RT="$(runtime_root)"; dir="$RT/runs/$run"
[[ ! -e "$dir" ]] || die "run $run exists: $dir"
mkdir -p "$dir"
world="mistbrn-$scenario"

LH="$(launcher_home)"
build="$(ls -1t "$LH/versions" | grep '^native-' | head -1)"
[[ -n "$build" ]] || die "no native build installed: run tools/build-native.sh first"
V="$LH/versions/$build"
resources="$LH/versions/$HORIZON_VERSION/.local/horizon-client/26.3"
[[ -f "$resources/horizon-resources.json" ]] || die "prepared resources missing: $resources"
case "$(os_kind)" in windows) client="$V/horizon-client.exe"; ctl_bin="$V/horizon-ctl.exe" ;;
  *) client="$V/horizon-client"; ctl_bin="$V/horizon-ctl" ;; esac
echo "run=$run build=$build world=$world dir=$dir"

launch_no=0
sock=""
# launch [ENV=VALUE...]: start the game in this run's folder and wait until the player is in.
launch() {
  launch_no=$((launch_no + 1)); sock="$dir/l$launch_no.sock"
  ( cd "$dir" && env HORIZON_NO_AUDIO=1 "$@" "$ctl_bin" --socket "$sock" --timeout 600 \
      launch --game "$client" --resources "$resources" --world "$world" --creative --wait \
      > "$dir/l$launch_no.launch.json" ) || die "launch $launch_no failed: $(tail -c 600 "$dir/l$launch_no.launch.json")"
}
ctl() { "$ctl_bin" --socket "$sock" --timeout 120 "$@"; }
quit_game() { ctl quit >/dev/null; wait_exit; }
# The game writes its log beside the socket; wait for the process to leave.
wait_exit() {
  local pid; pid="$(jq -r '.launch.pid' "$dir/l$launch_no.launch.json")"
  for _ in $(seq 1 120); do kill -0 "$pid" 2>/dev/null || return 0; sleep 1; done
  die "game pid $pid did not exit"
}
log() { echo "$dir/l$1.log"; }
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
    expect 2 result=deliberate_abort count_before=3 inside_count=2 count_after=3 pd_unchanged=true ;;
  raw-root)
    launch MISTBRN_TEST_SCENARIO=raw-root; ctl wait 40 >/dev/null; quit_game
    expect 1 raw_nbt=some ;;
  *) die "unknown scenario $scenario" ;;
esac

grep -a "MISTBRN" "$dir"/l*.log > "$dir/report.txt" || true
if (( fails == 0 )); then echo "PASS $scenario ($run)"; else echo "FAIL $scenario: $fails ($run)"; exit 1; fi
