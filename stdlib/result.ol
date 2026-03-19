// stdlib/result.ol — Option/Result patterns for Olang
// Provides ok/err wrapping, unwrap, map, and_then.

// Constructor
pub fn ok(val) { { tag: "ok", val: val } }
pub fn err(msg) { { tag: "err", msg: msg } }
pub fn none() { { tag: "none" } }
pub fn some(val) { { tag: "some", val: val } }

// Type checks
pub fn is_ok(r) { r.tag == "ok" }
pub fn is_err(r) { r.tag == "err" }
pub fn is_some(r) { r.tag == "some" }
pub fn is_none(r) { r.tag == "none" }

// Unwrap
pub fn unwrap(r) {
  if r.tag == "ok" { return r.val; }
  if r.tag == "some" { return r.val; }
  return 0;
}

pub fn unwrap_or(r, default) {
  if r.tag == "ok" { return r.val; }
  if r.tag == "some" { return r.val; }
  return default;
}

pub fn unwrap_err(r) {
  if r.tag == "err" { return r.msg; }
  return "";
}

// Transform
pub fn map_ok(r, f) {
  if r.tag == "ok" { return ok(f(r.val)); }
  return r;
}

pub fn map_err(r, f) {
  if r.tag == "err" { return err(f(r.msg)); }
  return r;
}

pub fn and_then(r, f) {
  if r.tag == "ok" { return f(r.val); }
  return r;
}

pub fn or_else(r, f) {
  if r.tag == "err" { return f(r.msg); }
  return r;
}

// Option helpers
pub fn map_some(opt, f) {
  if opt.tag == "some" { return some(f(opt.val)); }
  return none();
}

pub fn flatten(opt) {
  if opt.tag == "some" {
    if opt.val.tag == "some" { return opt.val; }
  }
  return opt;
}

// Try pattern: wrap a value that could be 0/empty
pub fn try_val(val) {
  if val == 0 { return none(); }
  if val == "" { return none(); }
  return some(val);
}
