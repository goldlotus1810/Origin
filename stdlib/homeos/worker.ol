// homeos/worker.ol — Tier 2 agent protocol
// Worker = tế bào thần kinh ngoại vi
// Silent by default. Wake on ISL. Xử lý → sleep.
// Worker gửi molecular chain, KHÔNG gửi raw data (QT rule).

pub fn worker_new(kind, addr) {
  return { kind: kind, addr: addr, active: false };
}

pub fn worker_wake(worker, msg) {
  worker.active = true;
  let result = dispatch(worker, msg);
  worker.active = false;
  return result;
}

fn dispatch(worker, msg) {
  if worker.kind == "sensor" { return sensor_read(msg); }
  if worker.kind == "actuator" { return actuator_write(msg); }
  if worker.kind == "camera" { return camera_capture(msg); }
  if worker.kind == "network" { return network_op(msg); }
  return { ok: false, reason: "unknown worker kind" };
}

fn sensor_read(msg) {
  // Read sensor → encode as molecular chain
  return { ok: true, chain: mol_new(1, 1, 128, 128, 3), raw: false };
}

fn actuator_write(msg) {
  // Execute actuator command
  return { ok: true, action: "executed" };
}

fn camera_capture(msg) {
  // Capture → SDF fitting → molecular chain (not raw pixels)
  return { ok: true, chain: mol_new(1, 1, 128, 128, 3), raw: false };
}

fn network_op(msg) {
  return { ok: true, action: "network_done" };
}
