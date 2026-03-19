// stdlib/homeos/arena.ol — Arena allocator for per-turn memory management
// PLAN 5.3.1: Allocate freely within a turn, free ALL at turn boundary.
// O(1) reset — just move pointer back to base.

pub fn arena_new(capacity) {
  // Create arena with given capacity (bytes)
  // In native VM: backed by mmap. In Olang VM: backed by array.
  return {
    base: 0,
    ptr: 0,
    capacity: capacity,
    allocations: 0,
    peak: 0
  };
}

pub fn arena_alloc(arena, size) {
  // Bump allocator: O(1) allocation
  let new_ptr = arena.ptr + size;
  if new_ptr > arena.capacity {
    return -1;  // OOM sentinel
  }
  let result = arena.ptr;
  arena.ptr = new_ptr;
  arena.allocations = arena.allocations + 1;
  if arena.ptr > arena.peak {
    arena.peak = arena.ptr;
  }
  return result;
}

pub fn arena_reset(arena) {
  // O(1) reset — all allocations freed at once
  // This is the key advantage: no per-object deallocation needed
  arena.ptr = arena.base;
  arena.allocations = 0;
}

pub fn arena_used(arena) {
  return arena.ptr - arena.base;
}

pub fn arena_remaining(arena) {
  return arena.capacity - arena.ptr;
}

pub fn arena_stats(arena) {
  return {
    used: arena_used(arena),
    remaining: arena_remaining(arena),
    peak: arena.peak,
    allocations: arena.allocations,
    capacity: arena.capacity
  };
}

// Promote: copy data from arena to persistent storage
// Called when data needs to survive arena_reset (STM, QR)
pub fn arena_promote(arena, offset, size, persistent) {
  // In real VM: memcpy from arena region to persistent heap
  // Returns new offset in persistent storage
  let dest = persistent.ptr;
  persistent.ptr = persistent.ptr + size;
  return dest;
}
