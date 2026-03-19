// stdlib/iter.ol — Iterator combinators for Olang
// Higher-order functions over arrays.
// Note: vec.ol already has map/filter/fold/any/all/find/enumerate/count
// This module adds: reduce, zip, take, skip, flat_map, chunk, window, range.

pub fn reduce(arr, f) {
  // Reduce without initial value (uses first element)
  let n = len(arr);
  if n == 0 { return 0; }
  let acc = arr[0];
  let i = 1;
  while i < n {
    acc = f(acc, arr[i]);
    i = i + 1;
  }
  return acc;
}

pub fn zip(a, b) {
  // Zip two arrays into array of pairs
  let result = [];
  let n = min(len(a), len(b));
  let i = 0;
  while i < n {
    push(result, [a[i], b[i]]);
    i = i + 1;
  }
  return result;
}

pub fn take(arr, n) {
  // First N elements
  let result = [];
  let count = min(n, len(arr));
  let i = 0;
  while i < count {
    push(result, arr[i]);
    i = i + 1;
  }
  return result;
}

pub fn skip(arr, n) {
  // Skip first N elements
  let result = [];
  let i = n;
  let total = len(arr);
  while i < total {
    push(result, arr[i]);
    i = i + 1;
  }
  return result;
}

pub fn flat_map(arr, f) {
  // Map then flatten one level
  let result = [];
  let i = 0;
  let n = len(arr);
  while i < n {
    let sub = f(arr[i]);
    let j = 0;
    let m = len(sub);
    while j < m {
      push(result, sub[j]);
      j = j + 1;
    }
    i = i + 1;
  }
  return result;
}

pub fn chunk(arr, size) {
  // Split into chunks of given size
  let result = [];
  let i = 0;
  let n = len(arr);
  while i < n {
    let c = [];
    let j = 0;
    while j < size && (i + j) < n {
      push(c, arr[i + j]);
      j = j + 1;
    }
    push(result, c);
    i = i + size;
  }
  return result;
}

pub fn window(arr, size) {
  // Sliding window
  let result = [];
  let n = len(arr);
  if n < size { return result; }
  let i = 0;
  while i <= n - size {
    let w = [];
    let j = 0;
    while j < size {
      push(w, arr[i + j]);
      j = j + 1;
    }
    push(result, w);
    i = i + 1;
  }
  return result;
}

pub fn range(start, end) {
  // Generate [start, start+1, ..., end-1]
  let result = [];
  let i = start;
  while i < end {
    push(result, i);
    i = i + 1;
  }
  return result;
}

pub fn range_step(start, end, step) {
  let result = [];
  let i = start;
  while i < end {
    push(result, i);
    i = i + step;
  }
  return result;
}

pub fn sum(arr) {
  let s = 0;
  let i = 0;
  let n = len(arr);
  while i < n {
    s = s + arr[i];
    i = i + 1;
  }
  return s;
}

pub fn product(arr) {
  let p = 1;
  let i = 0;
  let n = len(arr);
  while i < n {
    p = p * arr[i];
    i = i + 1;
  }
  return p;
}

pub fn max_val(arr) {
  if len(arr) == 0 { return 0; }
  let m = arr[0];
  let i = 1;
  let n = len(arr);
  while i < n {
    if arr[i] > m { m = arr[i]; }
    i = i + 1;
  }
  return m;
}

pub fn min_val(arr) {
  if len(arr) == 0 { return 0; }
  let m = arr[0];
  let i = 1;
  let n = len(arr);
  while i < n {
    if arr[i] < m { m = arr[i]; }
    i = i + 1;
  }
  return m;
}

pub fn index_of(arr, val) {
  let i = 0;
  let n = len(arr);
  while i < n {
    if arr[i] == val { return i; }
    i = i + 1;
  }
  return -1;
}

pub fn unique(arr) {
  let result = [];
  let i = 0;
  let n = len(arr);
  while i < n {
    if index_of(result, arr[i]) < 0 {
      push(result, arr[i]);
    }
    i = i + 1;
  }
  return result;
}

pub fn repeat(val, n) {
  let result = [];
  let i = 0;
  while i < n {
    push(result, val);
    i = i + 1;
  }
  return result;
}
