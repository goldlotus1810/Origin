// stdlib/set.ol — Set collection for Olang
// Unique-value collection based on molecular chain equality.

pub fn new() { Set() }
pub fn insert(s, val) { s.insert(val) }
pub fn contains(s, val) { s.contains(val) }
pub fn remove(s, val) { s.remove(val) }
pub fn len(s) { s.len() }
pub fn union(a, b) { a.union(b) }
pub fn intersection(a, b) { a.intersection(b) }
pub fn difference(a, b) { a.difference(b) }
pub fn to_array(s) { s.to_array() }
