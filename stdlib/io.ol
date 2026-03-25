// ─── stdlib/io.ol ───────────────────────────────────────────────────────────
// IO module: file operations (append-only per QT9) + console output.
//
// Usage:
//   use io;
//   io.println("hello");
//   let data = io.read_file("config.olang");
//   io.append_file("log.olang", data);
// ────────────────────────────────────────────────────────────────────────────

mod io;

// ── Console output ──────────────────────────────────────────────────────────

// Print a value to console (no newline).
pub fn print(msg) {
    __print(msg);
}

// Print a value to console with newline.
pub fn println(msg) {
    __println(msg);
}

// ── File operations (append-only per QT9) ───────────────────────────────────

// Read entire file contents as string.
// Returns file contents or empty on failure.
pub fn read_file(path) {
    __file_read(path)
}

// Append data to file (QT9: append-only, never overwrite).
// Returns 1 on success.
pub fn append_file(path, data) {
    __file_append(path, data)
}

// Write data to file (use append_file when possible — QT9).
// Returns 1 on success.
pub fn write_file(path, data) {
    __file_write(path, data)
}
