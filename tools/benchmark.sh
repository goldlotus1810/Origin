#!/bin/bash
# Olang Benchmark — Compare against C, Rust, Go, Python, Julia, Node.js
# Usage: bash tools/benchmark.sh
# Mỗi test: chạy 3 lần, lấy best time

set -e
cd "$(dirname "$0")/.."

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

TMP="/tmp/olang_bench"
mkdir -p "$TMP"

echo -e "${CYAN}═══ OLANG BENCHMARK ═══${NC}"
echo ""

# ── Helper: run N times, report best ──
best_of_3() {
    local cmd="$1"
    local best=999999
    for i in 1 2 3; do
        local start=$(date +%s%N)
        eval "$cmd" >/dev/null 2>&1
        local end=$(date +%s%N)
        local ms=$(( (end - start) / 1000000 ))
        if [ "$ms" -lt "$best" ]; then best=$ms; fi
    done
    echo "$best"
}

# ══════════════════════════════════════════════════════════════
# TEST 1: fib(30) — recursive, measures function call overhead
# ══════════════════════════════════════════════════════════════
echo -e "${BOLD}TEST 1: fib(30) — recursive${NC}"

# C
cat > "$TMP/fib.c" << 'EOF'
#include <stdio.h>
int fib(int n) { if (n < 2) return n; return fib(n-1) + fib(n-2); }
int main() { printf("%d\n", fib(30)); return 0; }
EOF
gcc -O2 -o "$TMP/fib_c" "$TMP/fib.c"
t=$(best_of_3 "$TMP/fib_c")
echo -e "  C (gcc -O2):    ${GREEN}${t}ms${NC}"
C_FIB=$t

# Rust
cat > "$TMP/fib.rs" << 'EOF'
fn fib(n: i32) -> i32 { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
fn main() { println!("{}", fib(30)); }
EOF
rustc -O -o "$TMP/fib_rust" "$TMP/fib.rs" 2>/dev/null
t=$(best_of_3 "$TMP/fib_rust")
echo -e "  Rust (rustc -O): ${GREEN}${t}ms${NC}"

# Go
cat > "$TMP/fib.go" << 'EOF'
package main
import "fmt"
func fib(n int) int { if n < 2 { return n }; return fib(n-1) + fib(n-2) }
func main() { fmt.Println(fib(30)) }
EOF
go build -o "$TMP/fib_go" "$TMP/fib.go" 2>/dev/null
t=$(best_of_3 "$TMP/fib_go")
echo -e "  Go:              ${GREEN}${t}ms${NC}"

# Node.js
cat > "$TMP/fib.js" << 'EOF'
function fib(n) { if (n < 2) return n; return fib(n-1) + fib(n-2); }
console.log(fib(30));
EOF
t=$(best_of_3 "node $TMP/fib.js")
echo -e "  Node.js (V8):    ${GREEN}${t}ms${NC}"

# Julia
cat > "$TMP/fib.jl" << 'EOF'
fib(n) = n < 2 ? n : fib(n-1) + fib(n-2)
println(fib(30))
EOF
t=$(best_of_3 "julia $TMP/fib.jl")
echo -e "  Julia:           ${GREEN}${t}ms${NC}"
JULIA_FIB=$t

# Python
cat > "$TMP/fib.py" << 'EOF'
def fib(n):
    if n < 2: return n
    return fib(n-1) + fib(n-2)
print(fib(30))
EOF
t=$(best_of_3 "python3 $TMP/fib.py")
echo -e "  Python 3:        ${YELLOW}${t}ms${NC}"
PY_FIB=$t

# Olang
t=$(best_of_3 "echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; emit fib(30);' | timeout 120 ./origin_new.olang 2>/dev/null")
echo -e "  Olang:           ${RED}${t}ms${NC}"
OL_FIB=$t

echo -e "  ${CYAN}Olang/Python = ${NC}$(echo "scale=1; $OL_FIB / $PY_FIB" | bc)x"
echo ""

# ══════════════════════════════════════════════════════════════
# TEST 2: loop 10M — measure basic iteration speed
# ══════════════════════════════════════════════════════════════
echo -e "${BOLD}TEST 2: loop 10M iterations${NC}"

# C
cat > "$TMP/loop.c" << 'EOF'
#include <stdio.h>
int main() { long s = 0; for (int i = 0; i < 10000000; i++) s += i; printf("%ld\n", s); return 0; }
EOF
gcc -O2 -o "$TMP/loop_c" "$TMP/loop.c"
t=$(best_of_3 "$TMP/loop_c")
echo -e "  C (gcc -O2):    ${GREEN}${t}ms${NC}"

# Python
cat > "$TMP/loop.py" << 'EOF'
s = 0
for i in range(10000000): s += i
print(s)
EOF
t=$(best_of_3 "python3 $TMP/loop.py")
echo -e "  Python 3:        ${YELLOW}${t}ms${NC}"
PY_LOOP=$t

# Node.js
cat > "$TMP/loop.js" << 'EOF'
let s = 0; for (let i = 0; i < 10000000; i++) s += i; console.log(s);
EOF
t=$(best_of_3 "node $TMP/loop.js")
echo -e "  Node.js:         ${GREEN}${t}ms${NC}"

# Olang
t=$(best_of_3 "echo 'let s = 0; let i = 0; while i < 10000000 { let s = s + i; let i = i + 1; }; emit s;' | timeout 120 ./origin_new.olang 2>/dev/null")
echo -e "  Olang:           ${RED}${t}ms${NC}"
OL_LOOP=$t

echo -e "  ${CYAN}Olang/Python = ${NC}$(echo "scale=1; $OL_LOOP / $PY_LOOP" | bc 2>/dev/null || echo "?")x"
echo ""

# ══════════════════════════════════════════════════════════════
# TEST 3: string operations — SHA-256 hash
# ══════════════════════════════════════════════════════════════
echo -e "${BOLD}TEST 3: SHA-256 × 1000${NC}"

# Python
cat > "$TMP/sha.py" << 'EOF'
import hashlib
h = "hello"
for i in range(1000): h = hashlib.sha256(h.encode()).hexdigest()
print(h[:16])
EOF
t=$(best_of_3 "python3 $TMP/sha.py")
echo -e "  Python 3:        ${GREEN}${t}ms${NC}"
PY_SHA=$t

# Node.js
cat > "$TMP/sha.js" << 'EOF'
const crypto = require('crypto');
let h = "hello";
for (let i = 0; i < 1000; i++) h = crypto.createHash('sha256').update(h).digest('hex');
console.log(h.slice(0,16));
EOF
t=$(best_of_3 "node $TMP/sha.js")
echo -e "  Node.js:         ${GREEN}${t}ms${NC}"

# Olang
t=$(best_of_3 "echo 'let h = \"hello\"; let i = 0; while i < 1000 { let h = __sha256(h); let i = i + 1; }; emit substr(h, 0, 16);' | timeout 120 ./origin_new.olang 2>/dev/null")
echo -e "  Olang:           ${RED}${t}ms${NC}"
OL_SHA=$t

echo -e "  ${CYAN}Olang/Python = ${NC}$(echo "scale=1; $OL_SHA / $PY_SHA" | bc 2>/dev/null || echo "?")x"
echo ""

# ══════════════════════════════════════════════════════════════
# TEST 4: file read + process — read 3.2MB, count lines
# ══════════════════════════════════════════════════════════════
echo -e "${BOLD}TEST 4: Read 3.2MB file + count lines${NC}"

# Python
cat > "$TMP/readfile.py" << 'EOF'
with open("data/cuon_theo_chieu_gio.txt") as f:
    lines = f.read().count('\n')
print(lines)
EOF
t=$(best_of_3 "python3 $TMP/readfile.py")
echo -e "  Python 3:        ${GREEN}${t}ms${NC}"
PY_READ=$t

# Olang
t=$(best_of_3 "echo 'let b = __file_read(\"data/cuon_theo_chieu_gio.txt\"); let l = __line_offsets(b); emit __array_len(l) / 2;' | timeout 30 ./origin_new.olang 2>/dev/null")
echo -e "  Olang:           ${GREEN}${t}ms${NC}"
OL_READ=$t

echo -e "  ${CYAN}Olang/Python = ${NC}$(echo "scale=1; $OL_READ / $PY_READ" | bc 2>/dev/null || echo "?")x"
echo ""

# ══════════════════════════════════════════════════════════════
# TEST 5: AES encrypt/decrypt
# ══════════════════════════════════════════════════════════════
echo -e "${BOLD}TEST 5: AES-256 encrypt × 1000${NC}"

# Python
cat > "$TMP/aes.py" << 'EOF'
from hashlib import sha256
# Simulate AES with SHA-256 (Python's AES needs pycryptodome)
h = "0123456789abcdef" * 2
for i in range(1000): h = sha256(h.encode()).hexdigest()
print(h[:16])
EOF
t=$(best_of_3 "python3 $TMP/aes.py")
echo -e "  Python (sha256): ${GREEN}${t}ms${NC}"

# Olang
t=$(best_of_3 "echo 'let k = \"0123456789abcdef0123456789abcdef\"; let d = \"hello world 1234\"; let i = 0; while i < 1000 { let d = __aes_encrypt(k, d); let i = i + 1; }; emit __len(d);' | timeout 120 ./origin_new.olang 2>/dev/null")
echo -e "  Olang (AES-NI):  ${GREEN}${t}ms${NC}"
echo ""

# ══════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════
echo -e "${CYAN}═══ SUMMARY ═══${NC}"
echo -e "  fib(30):    Olang ${OL_FIB}ms vs Python ${PY_FIB}ms = $(echo "scale=1; $OL_FIB / $PY_FIB" | bc 2>/dev/null || echo "?")x"
echo -e "  loop 10M:   Olang ${OL_LOOP}ms vs Python ${PY_LOOP}ms = $(echo "scale=1; $OL_LOOP / $PY_LOOP" | bc 2>/dev/null || echo "?")x"
echo -e "  SHA×1000:   Olang ${OL_SHA}ms vs Python ${PY_SHA}ms = $(echo "scale=1; $OL_SHA / $PY_SHA" | bc 2>/dev/null || echo "?")x"
echo -e "  file read:  Olang ${OL_READ}ms vs Python ${PY_READ}ms = $(echo "scale=1; $OL_READ / $PY_READ" | bc 2>/dev/null || echo "?")x"
echo ""
echo -e "  ${BOLD}Target: Olang/Python ≤ 1.0x cho mỗi test${NC}"
echo -e "  ${BOLD}Nghĩa là: Olang phải NHANH HƠN hoặc BẰNG Python${NC}"
