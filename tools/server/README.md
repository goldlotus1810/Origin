# server

> Terminal REPL for HomeOS — reads stdin, processes through `HomeRuntime`, prints responses to stdout.

## Dependencies
- ucd
- olang (with `std` feature)
- silk (with `std` feature)
- context (with `std` feature)
- agents (with `std` feature)
- memory (with `std` feature)
- runtime (with `std` feature)

## Files
| File | Purpose |
|------|---------|
| src/main.rs | REPL loop: boot `HomeRuntime`, read lines from stdin, call `process_text()`, print response; loads `origin.olang` if present |

## Key API
```rust
// main() pipeline:
// 1. Boot: load origin.olang if exists, otherwise HomeRuntime::new()
//    let rt = HomeRuntime::with_file(session_id, Some(bytes));
// 2. REPL loop:
//    print "○ " prompt
//    read line from stdin
//    let response = rt.process_text(input, now_ns());
//    println!("{}", response.text);
// 3. On exit: print session stats (turn count, f(x))
```

## Rules
- No external framework — uses only `std::io` for stdin/stdout
- Boots from `origin.olang` in current directory if the file exists; otherwise boots from nothing
- Supports `exit` and `quit` commands, plus Ctrl+C / EOF to terminate
- Prints UCD table size at startup; warns if table is empty
- Session stats (turns, f(x)) are printed on exit

## Test
```bash
cargo test -p server
```
