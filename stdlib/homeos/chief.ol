// homeos/chief.ol — Tier 1 agent protocol
// Chief = tủy sống: xử lý, tổng hợp, quản lý Workers

pub fn chief_new(name, kind) {
  return { name: name, kind: kind, workers: [], inbox: [] };
}

pub fn chief_receive(chief, msg) {
  // Dispatch based on message type
  if msg.type == "report" {
    return process_report(chief, msg);
  }
  if msg.type == "query" {
    return process_query(chief, msg);
  }
  return { action: "ack", from: chief.name };
}

fn process_report(chief, msg) {
  // Aggregate worker reports → summary for AAM
  return {
    action: "summary",
    from: chief.name,
    data: msg.data,
    worker_count: len(chief.workers)
  };
}

fn process_query(chief, msg) {
  // Forward query to appropriate worker
  return { action: "forward", from: chief.name, to: "worker" };
}

pub fn chief_add_worker(chief, worker) {
  push(chief.workers, worker);
}
