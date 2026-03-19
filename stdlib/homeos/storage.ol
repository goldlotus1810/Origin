// stdlib/homeos/storage.ol — Mobile storage layer
// PLAN 7.2.3: Persistent storage for mobile devices.
// Android: /data/data/com.homeos/files/
// iOS: Documents/ (app sandbox)
// Desktop: ~/.homeos/

// ── Path resolution ─────────────────────────────────────────────────

pub fn data_dir() {
  // Detect platform and return appropriate data directory
  let platform = __platform();
  if platform == "android" { return "/data/data/com.homeos/files"; }
  if platform == "ios" { return "Documents"; }
  if platform == "wasm" { return "/homeos"; }
  // Default: home directory
  return ".homeos";
}

pub fn origin_path() {
  return data_dir() + "/origin.olang";
}

pub fn config_path() {
  return data_dir() + "/config.json";
}

pub fn log_path() {
  return data_dir() + "/log.olang";
}

// ── Config (JSON, human-readable) ───────────────────────────────────

pub fn config_load() {
  let path = config_path();
  let data = __file_read(path);
  if len(data) == 0 {
    // Default config
    return {
      version: 1,
      language: "vi",
      dream_enabled: true,
      dream_interval: 55,
      max_stm_size: 512,
      auto_backup: true,
      backup_interval_hours: 24
    };
  }
  return json_parse(data);
}

pub fn config_save(cfg) {
  let path = config_path();
  let data = json_emit(cfg);
  __file_write(path, data);
}

// ── Backup ──────────────────────────────────────────────────────────

pub fn backup_origin() {
  let src = origin_path();
  let dst = data_dir() + "/origin.olang.bak";
  let data = __file_read_bytes(src);
  if len(data) > 0 {
    __file_write_bytes(dst, data);
    return true;
  }
  return false;
}

pub fn restore_from_backup() {
  let bak = data_dir() + "/origin.olang.bak";
  let dst = origin_path();
  let data = __file_read_bytes(bak);
  if len(data) > 0 {
    __file_write_bytes(dst, data);
    return true;
  }
  return false;
}

// ── Storage stats ───────────────────────────────────────────────────

pub fn storage_stats() {
  let origin_size = __file_size(origin_path());
  let log_size = __file_size(log_path());
  let config_size = __file_size(config_path());
  return {
    origin_bytes: origin_size,
    log_bytes: log_size,
    config_bytes: config_size,
    total_bytes: origin_size + log_size + config_size,
    data_dir: data_dir()
  };
}
