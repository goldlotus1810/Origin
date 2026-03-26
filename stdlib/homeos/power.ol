// stdlib/homeos/power.ol — Power management for mobile devices
// PLAN 7.2.4: Battery-aware behavior.
// Battery low → reduce Dream frequency.
// Screen off → sleep mode (ISL only).
// Charging → full Dream cycle + compaction.

// ── Power states ────────────────────────────────────────────────────

pub fn state_normal()   { return 0; }  // Plugged in or high battery
pub fn state_saving()   { return 1; }  // Battery < 30%
pub fn state_critical() { return 2; }  // Battery < 10%
pub fn state_sleep()    { return 3; }  // Screen off

pub fn state_name(state) {
  if state == 0 { return "normal"; }
  if state == 1 { return "saving"; }
  if state == 2 { return "critical"; }
  if state == 3 { return "sleep"; }
  return "unknown";
}

// ── Power-aware configuration ───────────────────────────────────────

pub fn power_config(state) {
  // Return configuration based on power state
  if state == 0 {
    // Normal: full features
    return {
      dream_enabled: true,
      dream_interval: 55,        // Fib[10] = 55 turns
      silk_maintenance: true,
      compaction_enabled: true,
      max_stm_size: 512,
      isl_heartbeat_ms: 5000
    };
  }
  if state == 1 {
    // Power saving: reduce background work
    return {
      dream_enabled: true,
      dream_interval: 89,        // Fib[11] = 89 turns (less frequent)
      silk_maintenance: false,    // Skip pruning
      compaction_enabled: false,
      max_stm_size: 256,          // Smaller STM
      isl_heartbeat_ms: 15000     // Less frequent heartbeat
    };
  }
  if state == 2 {
    // Critical: minimal operation
    return {
      dream_enabled: false,       // No Dream at all
      dream_interval: 0,
      silk_maintenance: false,
      compaction_enabled: false,
      max_stm_size: 128,
      isl_heartbeat_ms: 60000     // 1 minute heartbeat
    };
  }
  // Sleep: ISL only
  return {
    dream_enabled: false,
    dream_interval: 0,
    silk_maintenance: false,
    compaction_enabled: false,
    max_stm_size: 64,
    isl_heartbeat_ms: 0           // No heartbeat in sleep
  };
}

// ── Battery level detection ─────────────────────────────────────────

pub fn detect_power_state() {
  let battery = __battery_level();  // 0-100, or -1 if unknown
  let charging = __is_charging();   // true/false
  let screen_on = __is_screen_on(); // true/false

  if !screen_on { return state_sleep(); }
  if charging { return state_normal(); }
  if battery < 0 { return state_normal(); }  // Unknown = assume normal
  if battery < 10 { return state_critical(); }
  if battery < 30 { return state_saving(); }
  return state_normal();
}

// ── Power event handler ─────────────────────────────────────────────

pub fn on_power_change(old_state, new_state) {
  // Called when power state transitions
  if old_state == new_state { return; }

  if new_state == state_sleep() {
    // Entering sleep: flush pending writes, save state
    emit "Power: entering sleep mode\n";
    return { action: "flush_and_sleep" };
  }
  if old_state == state_sleep() && new_state == state_normal() {
    // Waking up with power: run deferred Dream cycle
    emit "Power: waking up, running deferred Dream\n";
    return { action: "run_dream" };
  }
  if new_state == state_critical() {
    emit "Power: critical battery, minimal mode\n";
    return { action: "reduce_activity" };
  }
  return { action: "reconfigure" };
}
