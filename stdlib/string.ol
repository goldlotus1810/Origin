// stdlib/string.ol — String builtins for Olang
// Wraps string manipulation functions.

pub fn split(s, delim) { s.str_split(delim) }
pub fn contains(s, sub) { s.str_contains(sub) }
pub fn replace(s, from, to) { s.str_replace(from, to) }
pub fn starts_with(s, prefix) { s.str_starts_with(prefix) }
pub fn ends_with(s, suffix) { s.str_ends_with(suffix) }
pub fn index_of(s, sub) { s.str_index_of(sub) }
pub fn trim(s) { s.str_trim() }
pub fn upper(s) { s.str_upper() }
pub fn lower(s) { s.str_lower() }
pub fn substr(s, start, len) { s.str_substr(start, len) }
pub fn len(s) { s.str_len() }
pub fn concat(a, b) { a.str_concat(b) }
pub fn char_at(s, idx) { s.char_at(idx) }
pub fn repeat(s, n) { s.repeat(n) }
pub fn pad_left(s, len, ch) { s.pad_left(len, ch) }
pub fn pad_right(s, len, ch) { s.pad_right(len, ch) }
pub fn matches(s, pattern) { s.matches(pattern) }
pub fn chars(s) { s.chars() }
