// stdlib/homeos/spline.ol — Temporal Dimension (FE.4-5)
// Ported from Rust: crates/olang/src/mol/spline.rs (411 LOC)
// T dimension = temporal pattern. TimeHistory = append-only spline knots.

// ═══ FE.4: TimeMode (T: 0-3) ═══

pub fn time_name(t) {
  if t == 0 { return "timeless"; }    // No temporal pattern
  if t == 1 { return "sequential"; }  // One after another
  if t == 2 { return "cyclical"; }    // Repeating pattern
  if t == 3 { return "rhythmic"; }    // Regular rhythm
  return "timeless";
}

// ═══ FE.4: SplineKnot (observation point) ═══
// Each knot = 1 observation on the temporal spline
// Fields: timestamp, amplitude, frequency, phase, duration

pub fn knot_new(ts, amp, freq, phase, dur) {
  return { ts: ts, amp: amp, freq: freq, phase: phase, dur: dur };
}

pub fn knot_from_text(text, ts) {
  // Create knot from text input
  // Amplitude ∝ text length (normalized to 0-1000)
  let amp = len(text) * 10;
  if amp > 1000 { amp = 1000; }
  // Frequency = 1 (single occurrence)
  // Phase = 0 (default)
  // Duration = amplitude (longer text = longer duration)
  return knot_new(ts, amp, 1, 0, amp);
}

pub fn knot_from_sensor(value, freq, ts) {
  return knot_new(ts, value, freq, 0, 1000 / freq);
}

// ═══ FE.5: TimeHistory (append-only) ═══

pub fn history_new() {
  return { knots: [], count: 0 };
}

pub fn history_observe_text(h, text, ts) {
  let k = knot_from_text(text, ts);
  push(h.knots, k);
  h.count = h.count + 1;
  return k;
}

pub fn history_observe_sensor(h, value, freq, ts) {
  let k = knot_from_sensor(value, freq, ts);
  push(h.knots, k);
  h.count = h.count + 1;
  return k;
}

// Amplitude at time t (linear interpolation between knots)
pub fn history_amplitude_at(h, t) {
  let n = h.count;
  if n == 0 { return 0; }
  if n == 1 { return h.knots[0].amp; }

  // Find surrounding knots
  let i = 0;
  while i < n - 1 {
    let k0 = h.knots[i];
    let k1 = h.knots[i + 1];
    if t >= k0.ts {
      if t <= k1.ts {
        // Linear interpolation
        let dt = k1.ts - k0.ts;
        if dt == 0 { return k0.amp; }
        let frac = (t - k0.ts) * 1000 / dt;
        return k0.amp + (k1.amp - k0.amp) * frac / 1000;
      }
    }
    i = i + 1;
  }
  // Past last knot → return last amplitude
  return h.knots[n - 1].amp;
}

// Predict behavior metrics from history
pub fn history_predict(h) {
  let n = h.count;
  if n == 0 {
    return { familiarity: 0, learning_rate: 1000, avg_amp: 0, periodicity: 0 };
  }

  // Familiarity: saturates at 10 observations (0-1000)
  let fam = n * 100;
  if fam > 1000 { fam = 1000; }

  // Learning rate: 1/√n (approximated as 1000/n for n≥3)
  let lr = 1000;
  if n >= 3 { lr = 1000 / n; }
  if lr < 50 { lr = 50; }

  // Average amplitude
  let sum_amp = 0;
  let i = 0;
  while i < n {
    sum_amp = sum_amp + h.knots[i].amp;
    i = i + 1;
  }
  let avg = sum_amp / n;

  // Periodicity: coefficient of variation in inter-arrival deltas
  let periodicity = 0;
  if n >= 3 {
    let deltas = [];
    let j = 1;
    while j < n {
      let dt = h.knots[j].ts - h.knots[j - 1].ts;
      push(deltas, dt);
      j = j + 1;
    }
    // Mean delta
    let sum_dt = 0;
    let k = 0;
    while k < len(deltas) {
      sum_dt = sum_dt + deltas[k];
      k = k + 1;
    }
    let mean_dt = sum_dt / len(deltas);
    // Variance
    if mean_dt > 0 {
      let sum_sq = 0;
      let m = 0;
      while m < len(deltas) {
        let diff = deltas[m] - mean_dt;
        sum_sq = sum_sq + diff * diff;
        m = m + 1;
      }
      let variance = sum_sq / len(deltas);
      // CV = std/mean ≈ sqrt(variance)/mean
      // High CV = irregular, Low CV = periodic
      periodicity = 1000 - (variance * 100 / (mean_dt * mean_dt));
      if periodicity < 0 { periodicity = 0; }
      if periodicity > 1000 { periodicity = 1000; }
    }
  }

  return { familiarity: fam, learning_rate: lr, avg_amp: avg, periodicity: periodicity };
}
