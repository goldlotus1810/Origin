// stdlib/sort.ol — Sorting algorithms for Olang
// Quicksort (default) + insertion sort (small arrays)

pub fn sort(arr) {
  // Default: ascending numeric sort
  return quicksort(arr, fn(a, b) { return a - b; });
}

pub fn sort_by(arr, key_fn) {
  // Sort by key function
  return quicksort(arr, fn(a, b) {
    return key_fn(a) - key_fn(b);
  });
}

pub fn sort_desc(arr) {
  return quicksort(arr, fn(a, b) { return b - a; });
}

pub fn quicksort(arr, cmp) {
  let n = len(arr);
  if n <= 1 { return arr; }
  if n <= 10 { return insertion_sort(arr, cmp); }

  // Partition around pivot (median of 3)
  let pivot_idx = median_of_three(arr, 0, n / 2, n - 1, cmp);
  let pivot = arr[pivot_idx];

  let less = [];
  let equal = [];
  let greater = [];

  let i = 0;
  while i < n {
    let c = cmp(arr[i], pivot);
    if c < 0 {
      push(less, arr[i]);
    } else {
      if c > 0 {
        push(greater, arr[i]);
      } else {
        push(equal, arr[i]);
      }
    }
    i = i + 1;
  }

  let sorted_less = quicksort(less, cmp);
  let sorted_greater = quicksort(greater, cmp);

  // Concat: less + equal + greater
  let result = [];
  let j = 0;
  while j < len(sorted_less) {
    push(result, sorted_less[j]);
    j = j + 1;
  }
  j = 0;
  while j < len(equal) {
    push(result, equal[j]);
    j = j + 1;
  }
  j = 0;
  while j < len(sorted_greater) {
    push(result, sorted_greater[j]);
    j = j + 1;
  }
  return result;
}

fn insertion_sort(arr, cmp) {
  let n = len(arr);
  let result = [];
  let i = 0;
  while i < n {
    push(result, arr[i]);
    i = i + 1;
  }

  i = 1;
  while i < n {
    let key = result[i];
    let j = i - 1;
    while j >= 0 && cmp(result[j], key) > 0 {
      result.array_set(j + 1, result[j]);
      j = j - 1;
    }
    result.array_set(j + 1, key);
    i = i + 1;
  }
  return result;
}

fn median_of_three(arr, a, b, c, cmp) {
  if cmp(arr[a], arr[b]) < 0 {
    if cmp(arr[b], arr[c]) < 0 { return b; }
    if cmp(arr[a], arr[c]) < 0 { return c; }
    return a;
  } else {
    if cmp(arr[a], arr[c]) < 0 { return a; }
    if cmp(arr[b], arr[c]) < 0 { return c; }
    return b;
  }
}

pub fn is_sorted(arr) {
  let n = len(arr);
  if n <= 1 { return true; }
  let i = 1;
  while i < n {
    if arr[i] < arr[i - 1] { return false; }
    i = i + 1;
  }
  return true;
}

pub fn binary_search(sorted_arr, target) {
  // Returns index or -1
  let lo = 0;
  let hi = len(sorted_arr) - 1;
  while lo <= hi {
    let mid = lo + (hi - lo) / 2;
    if sorted_arr[mid] == target { return mid; }
    if sorted_arr[mid] < target {
      lo = mid + 1;
    } else {
      hi = mid - 1;
    }
  }
  return -1;
}

pub fn reverse(arr) {
  let result = [];
  let i = len(arr) - 1;
  while i >= 0 {
    push(result, arr[i]);
    i = i - 1;
  }
  return result;
}
