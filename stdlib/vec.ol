// stdlib/vec.ol — Dynamic array (Vec) operations for Olang
// Wraps array builtins into a module.

pub fn new() { [] }
pub fn len(v) { v.len() }
pub fn push(v, val) { v.push(val) }
pub fn pop(v) { v.pop() }
pub fn get(v, idx) { v[idx] }
pub fn set(v, idx, val) { v.array_set(idx, val) }
pub fn slice(v, start, end) { v.slice(start, end) }
pub fn reverse(v) { v.reverse() }
pub fn contains(v, val) { v.contains(val) }
pub fn join(v, sep) { v.join(sep) }
pub fn map(v, f) { v.map(f) }
pub fn filter(v, f) { v.filter(f) }
pub fn fold(v, init, f) { v.fold(init, f) }
pub fn any(v, f) { v.any(f) }
pub fn all(v, f) { v.all(f) }
pub fn find(v, f) { v.find(f) }
pub fn enumerate(v) { v.enumerate() }
pub fn count(v, f) { v.count(f) }
