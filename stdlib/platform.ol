// ─── stdlib/platform.ol ─────────────────────────────────────────────────────
// Platform detection module: replaces hal crate for Olang userspace.
//
// Usage:
//   use platform;
//   let a = platform.arch();     // "x86_64", "aarch64", "riscv64", ...
//   let o = platform.os();       // "linux", "macos", "windows", "bare"
//   let m = platform.memory();   // total memory in bytes (0 if unavailable)
// ────────────────────────────────────────────────────────────────────────────

mod platform;

// CPU architecture string.
// Returns: "x86_64", "x86", "aarch64", "arm", "riscv64", "riscv32",
//          "mips", "wasm32", or "unknown"
pub fn arch() {
    platform_arch()
}

// Operating system string.
// Returns: "linux", "macos", "windows", "bare", or "unknown"
pub fn os() {
    platform_os()
}

// Total system memory in bytes.
// Returns 0 if not available (bare metal, WASM, etc.).
pub fn memory() {
    platform_memory()
}
