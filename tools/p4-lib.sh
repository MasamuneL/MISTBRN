# shellcheck shell=bash
# P4 fixtures and checks, sourced by native-test.sh.
# ShellCheck also checks this file directly, without the caller's dir and scenario.
# shellcheck disable=SC2154

p4_fixture() {
  launch MISTBRN_TEST_SCENARIO=p4
  ctl command "difficulty peaceful" >/dev/null
  ctl command "tp @s 512.5 112 512.5" >/dev/null
  ctl wait-for loaded --timeout 120 >/dev/null
  # Far chunks of the box can load a little after the player's own: clear twice.
  ctl command "fill 503 100 508 540 126 516 minecraft:air" >/dev/null
  ctl wait 40 >/dev/null
  ctl command "fill 503 100 508 540 126 516 minecraft:air" >/dev/null
  ctl command "fill 503 99 508 540 99 516 minecraft:stone" >/dev/null
  ctl command "tp @s 512.5 100 512.5" >/dev/null
  ctl wait 5 >/dev/null
}

# The near face is x=$1 and the block center is $1.5.
p4_iron_block() {
  ctl command "setblock $1 101 512 minecraft:iron_block" >/dev/null
  ctl look -90 0 >/dev/null
}

p4_burn() {
  local metal="$1" amount="${2:-25000}" mode="${3:-}"
  ctl command "mistbrn-test credit $metal $amount" >/dev/null
  ctl command "mistbrn-test burn $metal${mode:+ $mode}" >/dev/null
}

p4_off() { ctl command "mistbrn-test off" >/dev/null; }

p4_fail() { echo "  FAIL $*" | tee -a "$dir/report.txt"; fails=$((fails + 1)); }

# Read launch 1 in log order so events and their following samples need no shared tick field.
# Optional observations come from saved state (N6), live player state (N3) or chat (N5).
p4_check_log() {
  local results line
  if ! results="$(LC_ALL=C awk -v scenario="$scenario" -v observed="${1:-}" -v after="${2:-}" '
    function number(value) { return value ~ /^-?[0-9]+([.][0-9]+)?$/ }
    function vector(value, parts) {
      return split(value, parts, ",") == 3 && number(parts[1]) && number(parts[2]) && number(parts[3])
    }
    function abs(value) { return value < 0 ? -value : value }
    function norm(x, y, z) { return sqrt(x * x + y * y + z * z) }
    function check(ok, message) { if (!ok) print "  FAIL " message }
    function metrics(message) { print "check: " message }
    function stop_event() {
      if (!stop_seen) block_samples_before_stop = block_samples
      stop_seen++
    }
    BEGIN {
      steel = scenario ~ /^n([235]|5w)$/
      metal = steel ? "steel" : "iron"
      burn = scenario == "n6" ? "flared" : "burning"
      item = scenario ~ /^n([345]|5w)$/
      kind = item ? "item" : "block"
      id = item ? (scenario == "n4" ? "minecraft:iron_ingot" : "mistbrn:clip") : "minecraft:iron_block"
      reason = scenario == "n8" ? "death" : (scenario == "n9" ? "dimension" : "broken")
    }
    {
      marker = index($0, "MISTBRN ")
      if (!marker) next
      count = split(substr($0, marker + length("MISTBRN ")), tokens, /[[:space:]]+/)
      for (key in fields) delete fields[key]
      for (i = 1; i <= count; i++) {
        equal = index(tokens[i], "=")
        if (equal) fields[substr(tokens[i], 1, equal - 1)] = substr(tokens[i], equal + 1)
      }
    }
    fields["event"] == "burn" && (fields["metal"] == metal || fields["metal"] == "all") && fields["result"] == "ok" {
      if (fields["burn"] == burn) burn_seen++
      if (fields["burn"] == "off") { off_seen++; off_changed = fields["changed"] }
    }
    fields["event"] == "shutdown" && fields["reason"] == "dimension" && fields["result"] == "ok" {
      shutdown_changed = fields["changed"]
    }
    fields["event"] == "target_cleared" && fields["reason"] == "exhausted" { exhausted_clear++ }
    fields["event"] == "target_acquired" && fields["kind"] == kind && fields["id"] == id {
      if (burn_seen && !off_seen && !stop_seen) acquired++
    }
    fields["event"] == "exhausted" && fields["metal"] == metal && scenario == "n7" { stop_event() }
    fields["event"] == "target_cleared" && fields["reason"] == reason && scenario ~ /^n(8|9|10)$/ {
      stop_event()
    }
    fields["event"] == "coin_hit" {
      hits++
      if (fields["entity"] == "minecraft:zombie" && fields["dealt"] == "true") zombie_hits++
      hit_speed = fields["speed"]
      health_before = fields["health_before"]; health_after = fields["health_after"]
    }
    tokens[1] == "p4item" {
      if (!vector(fields["vel"], velocity)) { bad_item++; next }
      item_samples++
      # The mod logs the exact norm: three-decimal components can round above the cap.
      speed = number(fields["speed"]) ? fields["speed"] + 0 : norm(velocity[1], velocity[2], velocity[3])
      if (speed > max_item_speed) max_item_speed = speed
    }
    tokens[1] == "p4" {
      if (stop_seen) {
        after_stop++
        if (fields["target"] == "block") blocks_after_stop++
      }
      if (off_seen) after_off++
      if (off_seen || stop_seen) next
      if (fields["metal"] != metal || fields["tick"] !~ /^[0-9]+$/ ||
          !vector(fields["pos"], pos) || !vector(fields["vel"], velocity)) { bad_player++; next }

      tick = fields["tick"] + 0
      x = pos[1] + 0; y = pos[2] + 0; z = pos[3] + 0
      if (!samples) {
        first_tick = tick
        first_x = min_x = max_x = x
        min_y = max_y = y
        max_vy = velocity[2] + 0
      } else if (tick == last_tick + 1) {
        step = abs(x - final_x)
        if (step > max_step) max_step = step
        # The server sees client moves, sometimes two in one tick: average over 4 ticks.
        if (samples >= 4) {
          window = abs(x - win_x[(samples - 4) % 4]) / 4
          if (window > max_window) max_window = window
        }
      } else {
        gaps++
      }
      if (x < min_x) min_x = x
      if (x > max_x) max_x = x
      if (y < min_y) min_y = y
      if (y > max_y) max_y = y
      if (velocity[2] + 0 > max_vy) max_vy = velocity[2] + 0
      win_x[samples % 4] = x
      speed = norm(velocity[1], velocity[2], velocity[3])
      if (speed > max_speed) max_speed = speed
      displacement = norm(x - 512.5, y - 100, z - 512.5)
      if (displacement > max_displacement) max_displacement = displacement
      if (acquired && fields["target"] == "block") block_samples++
      if (acquired && fields["target"] == "item") item_targets++
      if (fields["target"] == "item" && fields["anchor"] == "true") item_anchors++
      tail_x[samples % 20] = final_x = x
      tail_y[samples % 20] = final_y = y
      last_tick = tick
      samples++
    }
    END {
      metrics(sprintf("samples=%d first_tick=%d last_tick=%d gaps=%d", samples, first_tick, last_tick, gaps))
      check(burn_seen == 1, "expected one successful " burn " " metal " activation")
      check(acquired > 0, "expected to acquire " id)
      check(item ? item_targets > 0 : block_samples > 0, "expected a player tick with target=" kind)
      check(samples >= 2 && !gaps && !bad_player, "expected consecutive valid player ticks")
      if (scenario ~ /^n([1-6]|5w)$/ || scenario == "n10")
        check(off_seen > 0, "expected successful " metal " off")
      metrics(sprintf("p4_after_off=%d", after_off))
      check(!after_off, "found force applied after a successful off")

      if (scenario == "n1") {
        metrics(sprintf("first_x=%.3f final_x=%.3f max_x=%.3f max_step=%.3f max_window=%.3f max_speed=%.3f",
            first_x, final_x, max_x, max_step, max_window, max_speed))
        check(final_x > first_x, "expected x to increase")
        check(max_speed <= 1.05 + 1e-6, "expected player speed <= 1.05")
        check(max_window <= 1.05 + 1e-6, "expected mean x step over 4 ticks <= 1.05")
        check(final_x <= 523.75, "expected final x <= block face - 0.25")
        check(max_x < 524, "player reached or entered the iron block")
        check(samples >= 20 && final_x >= 523.5, "expected the pull to reach the block face (x >= 523.5)")
      } else if (scenario == "n2" || scenario == "n3") {
        low = high = scenario == "n2" ? tail_x[0] : tail_y[0]
        for (i = 1; i < 20 && i < samples; i++) {
          point = scenario == "n2" ? tail_x[i] : tail_y[i]
          if (point < low) low = point
          if (point > high) high = point
        }
        check(samples >= 20, "expected 20 ticks in the final observation window")
        if (scenario == "n2") {
          metrics(sprintf("final_x=%.3f wall_min_x=%.3f wall_max_x=%.3f", final_x, low, high))
          check(low >= 509.25 && high <= 509.35, "expected x to settle at glass face + 0.3 +/- 0.05")
        } else {
          metrics(sprintf("min_y=%.3f max_y=%.3f final_y=%.3f hover_min_y=%.3f hover_max_y=%.3f hover_span=%.3f max_vy=%.3f item_anchors=%d after_off_y=%s",
              min_y, max_y, final_y, low, high, high - low, max_vy, item_anchors, observed))
          check(item_anchors > 0 && max_y > 102, "expected to rise above the starting height using the clip")
          check(low > 100 && high - low <= 1 + 1e-6, "expected airborne hover with y oscillation <= 1")
          check(max_vy <= 1, "expected upward velocity <= 1.0")
          check(number(observed) && observed + 0 < final_y, "expected y to fall after turning steel off")
        }
      } else if (scenario == "n4") {
        metrics(sprintf("max_displacement=%.3f item_samples=%d", max_displacement, item_samples))
        check(!bad_item && item_samples > 0, "expected ingot motion in p4item")
        check(max_displacement <= 0.05 + 1e-6, "expected player displacement <= 0.05")
      } else if (scenario == "n5") {
        metrics(sprintf("max_item_speed=%.6f hit_speed=%s coin_hits=%d zombie_health_before=%s zombie_health_after=%s",
            max_item_speed, hit_speed, hits, health_before, health_after))
        check(!bad_item && item_samples > 0 && max_item_speed <= 2 + 1e-6, "expected clip speed <= 2.0")
        check(hits == 1 && zombie_hits == 1, "expected exactly one coin_hit on the zombie")
        check(number(hit_speed) && hit_speed + 0 <= 2, "expected coin_hit speed <= 2.0")
        check(number(health_before) && number(health_after) && health_before + 0 > health_after + 0,
            "expected zombie health to decrease (mod log)")
      } else if (scenario == "n5w") {
        metrics(sprintf("max_item_speed=%.6f coin_hits=%d", max_item_speed, hits))
        check(!bad_item && item_samples > 0 && max_item_speed <= 2 + 1e-6, "expected clip speed <= 2.0")
        check(hits == 0, "a coin hit a body behind glass")
      } else if (scenario == "n6") {
        consumed = 25000 - observed
        metrics(sprintf("max_step=%.3f max_window=%.3f max_speed=%.3f final_x=%.3f reserve_iron=%s consumed_milli=%d expected_milli=%d",
            max_step, max_window, max_speed, final_x, observed, consumed, samples * 30))
        check(max_speed <= 1.45 + 1e-6, "expected flared player speed <= 1.45")
        check(max_window <= 1.45 + 1e-6, "expected mean flared x step over 4 ticks <= 1.45")
        check(number(observed) && abs(consumed - samples * 30) <= 30, "expected 30 milli per flared tick (+/- one tick)")
        check(samples >= 20 && final_x >= 531.5, "expected the flared pull to reach the block face (x >= 531.5)")
      } else {
        metrics(sprintf("stop_events=%d block_ticks_before=%d p4_after=%d block_ticks_after=%d",
            stop_seen, block_samples_before_stop, after_stop, blocks_after_stop))
        check(stop_seen == 1 && block_samples_before_stop > 0,
            "expected one " (scenario == "n7" ? "iron exhaustion" : "target clear for " reason) " during a block pull")
        if (scenario == "n7") {
          check(samples >= 19 && samples <= 21, "expected iron exhaustion after 20 ticks (+/- one)")
          check(exhausted_clear == 1, "expected target_cleared reason=exhausted")
        }
        if (scenario == "n9") check(shutdown_changed >= 1, "expected the dimension shutdown to put iron out")
        if (scenario == "n10") check(off_changed == 1, "expected iron still burning after the block broke")
        if (scenario == "n10") check(!blocks_after_stop, "found block targeting after reason=broken")
        else check(!after_stop, "found player motion samples after the stop event")
      }
    }
  ' "$(log 1)")"; then
    p4_fail "could not read P4 log from launch 1"
    return
  fi
  while IFS= read -r line; do
    case "$line" in
      "  FAIL "*) p4_fail "${line#  FAIL }" ;;
      *) echo "$line" | tee -a "$dir/report.txt" ;;
    esac
  done <<< "$results"
}
