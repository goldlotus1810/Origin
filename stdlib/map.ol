// stdlib/map.ol — Dictionary/Map operations for Olang
// Wraps dict builtins into a module.

pub fn get(m, key) { m.get(key) }
pub fn set(m, key, val) { m.set(key, val) }
pub fn has_key(m, key) { m.has_key(key) }
pub fn keys(m) { m.keys() }
pub fn values(m) { m.values() }
pub fn merge(a, b) { a.merge(b) }
pub fn remove(m, key) { m.remove(key) }
