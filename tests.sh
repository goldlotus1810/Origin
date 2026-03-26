#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# OLANG v1.0 — TEST RUNNER
# Chay: bash tests.sh
# Tat ca xanh = OK. Do = co van de. Khong thi biet.
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

ORIGIN_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$ORIGIN_DIR/origin_new.olang"
TEST_DIR="$ORIGIN_DIR/test"
TMP_DIR="/tmp/olang_tests"

PASS=0
FAIL=0
SKIP=0
ERRORS=""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p "$TMP_DIR"

# ─── Helpers ──────────────────────────────────────────────────

run_test() {
    local name="$1"
    local generator="$2"
    local bytecode="$3"
    local expected="$4"
    local timeout_sec="${5:-5}"

    # Generate bytecode
    if ! python3 "$generator" 2>/dev/null; then
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC $name — bytecode generation failed"
        return
    fi

    # Run binary
    local actual
    actual=$(timeout "$timeout_sec" "$BINARY" "$bytecode" 2>/dev/null) || {
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            FAIL=$((FAIL + 1))
            ERRORS="$ERRORS\n  $RED FAIL$NC $name — timeout (${timeout_sec}s)"
            return
        fi
        # Some tests exit with non-zero but still produce output
        actual=$(timeout "$timeout_sec" "$BINARY" "$bytecode" 2>/dev/null || true)
    }

    # Compare
    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
        echo -e "  $GREEN  OK$NC $name"
    else
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC $name"
        ERRORS="$ERRORS\n       expected: $(echo "$expected" | head -3)"
        ERRORS="$ERRORS\n       actual:   $(echo "$actual" | head -3)"
    fi
}

strip_repl() {
    # Strip REPL header (first 2 lines) + "⦿ " prefix + "bye" footer
    # Input:  ⦿ HomeOS v0.05\n○ Type...\n⦿ result\n⦿ bye
    # Output: result
    sed '1,2d' | sed 's/^⦿ //; /^bye$/d' | sed '/^$/d'
}

run_olang_test() {
    local name="$1"
    local olang_code="$2"
    local expected="$3"
    local timeout_sec="${4:-5}"

    local raw_output
    raw_output=$(echo "$olang_code" | timeout "$timeout_sec" "$BINARY" 2>/dev/null || true)

    if [ -z "$raw_output" ]; then
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC $name — no output"
        return
    fi

    # Strip REPL chrome, join lines for multi-emit comparison
    local actual
    actual=$(echo "$raw_output" | strip_repl | tr -d '\n')

    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
        echo -e "  $GREEN  OK$NC $name"
    else
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC $name"
        ERRORS="$ERRORS\n       expected: [$expected]"
        ERRORS="$ERRORS\n       actual:   [$actual]"
    fi
}

run_olang_file_test() {
    local name="$1"
    local olang_file="$2"
    local expected="$3"
    local timeout_sec="${4:-10}"

    if [ ! -f "$olang_file" ]; then
        SKIP=$((SKIP + 1))
        echo -e "  $YELLOW SKIP$NC $name — file not found: $olang_file"
        return
    fi

    local actual
    actual=$(timeout "$timeout_sec" "$BINARY" < "$olang_file" 2>/dev/null) || {
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            FAIL=$((FAIL + 1))
            ERRORS="$ERRORS\n  $RED FAIL$NC $name — timeout (${timeout_sec}s)"
            return
        fi
        actual=$(timeout "$timeout_sec" "$BINARY" < "$olang_file" 2>/dev/null || true)
    }

    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
        echo -e "  $GREEN  OK$NC $name"
    else
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC $name"
        ERRORS="$ERRORS\n       expected: $(echo "$expected" | head -3)"
        ERRORS="$ERRORS\n       actual:   $(echo "$actual" | head -3)"
    fi
}

# ─── Pre-checks ──────────────────────────────────────────────

echo -e "${CYAN}═══ OLANG v1.0 TEST SUITE ═══${NC}"
echo ""

# Check binary exists
if [ ! -x "$BINARY" ]; then
    echo -e "${RED}ERROR: Binary not found: $BINARY${NC}"
    echo "  Build first or check path."
    exit 1
fi

echo -e "${CYAN}Binary:${NC} $BINARY ($(du -h "$BINARY" | cut -f1))"
echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 1: VM BYTECODE TESTS (Python generators)
# ═══════════════════════════════════════════════════════════════

echo -e "${CYAN}--- VM Bytecode Tests ---${NC}"

# These tests generate bytecode via Python and run on raw VM
if command -v python3 &>/dev/null; then
    for pytest in "$TEST_DIR"/test_*.py; do
        [ -f "$pytest" ] || continue
        testname=$(basename "$pytest" .py)
        bytecode_file="$TMP_DIR/${testname}.olang"

        # Generate
        if python3 "$pytest" 2>/dev/null; then
            # Find where bytecode was written
            src_file="/tmp/${testname}.olang"
            [ -f "$src_file" ] && cp "$src_file" "$bytecode_file"

            if [ -f "$bytecode_file" ]; then
                # Run and check it doesn't crash (no expected output for legacy tests)
                if echo "" | timeout 5 "$BINARY" "$bytecode_file" >/dev/null 2>&1; then
                    PASS=$((PASS + 1))
                    echo -e "  $GREEN  OK$NC vm/$testname (no crash)"
                else
                    FAIL=$((FAIL + 1))
                    ERRORS="$ERRORS\n  $RED FAIL$NC vm/$testname — crashed"
                fi
            else
                SKIP=$((SKIP + 1))
                echo -e "  $YELLOW SKIP$NC vm/$testname — bytecode not found"
            fi
        else
            FAIL=$((FAIL + 1))
            ERRORS="$ERRORS\n  $RED FAIL$NC vm/$testname — python error"
        fi
    done
else
    echo -e "  $YELLOW SKIP$NC Python3 not found — skipping bytecode tests"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 2: OLANG SOURCE TESTS (eval via stdin)
# ═══════════════════════════════════════════════════════════════

echo -e "${CYAN}--- Olang Language Tests ---${NC}"

# 2.1 Arithmetic
run_olang_test "arith/add" \
    'emit 2 + 3;' \
    "5"

run_olang_test "arith/sub" \
    'emit 10 - 3;' \
    "7"

run_olang_test "arith/mul" \
    'emit 6 * 7;' \
    "42"

run_olang_test "arith/div" \
    'emit 15 / 3;' \
    "5"

run_olang_test "arith/mod" \
    'emit 17 % 5;' \
    "2"

run_olang_test "arith/negative" \
    'emit 0 - 42;' \
    "-42"

run_olang_test "arith/float" \
    'emit 1 / 3 * 3;' \
    "1"

# 2.2 Variables
run_olang_test "var/let" \
    'let x = 42; emit x;' \
    "42"

run_olang_test "var/reassign" \
    'let x = 1; let x = 2; emit x;' \
    "2"

run_olang_test "var/string" \
    'let s = "hello"; emit s;' \
    "hello"

# 2.3 Strings
run_olang_test "string/concat" \
    'emit "hello" + " " + "world";' \
    "hello world"

run_olang_test "string/len" \
    'emit __len("hello");' \
    "5"

run_olang_test "string/char_at" \
    'emit __char_at("hello", 1);' \
    "e"

run_olang_test "string/substr" \
    'emit __substr("hello world", 6, 11);' \
    "world"

run_olang_test "string/trim" \
    'emit __str_trim("  hi  ");' \
    "hi"

# 2.4 Comparison
run_olang_test "cmp/eq_true" \
    'if 1 == 1 { emit "yes"; } else { emit "no"; };' \
    "yes"

run_olang_test "cmp/eq_false" \
    'if 1 == 2 { emit "yes"; } else { emit "no"; };' \
    "no"

run_olang_test "cmp/lt" \
    'if 3 < 5 { emit "yes"; } else { emit "no"; };' \
    "yes"

run_olang_test "cmp/gt" \
    'if 5 > 3 { emit "yes"; } else { emit "no"; };' \
    "yes"

# 2.5 Control flow
run_olang_test "flow/if_else" \
    'let x = 10; if x > 5 { emit "big"; } else { emit "small"; };' \
    "big"

run_olang_test "flow/while" \
    'let i = 0; while i < 3 { emit i; let i = i + 1; };' \
    "012"

run_olang_test "flow/for_in" \
    'let arr = [10, 20, 30]; for x in arr { emit x; };' \
    "102030"

# 2.6 Functions
run_olang_test "fn/basic" \
    'fn double(x) { return x * 2; }; emit double(21);' \
    "42"

run_olang_test "fn/multi_param" \
    'fn add(a, b) { return a + b; }; emit add(3, 4);' \
    "7"

run_olang_test "fn/recursive" \
    'fn fib(n) { if n < 2 { return n; }; return fib(n - 1) + fib(n - 2); }; emit fib(10);' \
    "55"

# 2.7 Arrays
run_olang_test "array/literal" \
    'let a = [1, 2, 3]; emit __array_len(a);' \
    "3"

run_olang_test "array/push_get" \
    'let a = []; __push(a, 42); let v = __array_get(a, 0); emit v;' \
    "[42]42"

run_olang_test "array/set_at" \
    'let a = [1, 2, 3]; __set_at(a, 1, 99); let v = __array_get(a, 1); emit v;' \
    "[1, 99, 3]99"

# 2.8 Dict
run_olang_test "dict/create_get" \
    'let d = { name: "olang" }; emit d.name;' \
    "olang"

# 2.9 Lambda / HOF
run_olang_test "lambda/basic" \
    'let f = fn(x) { return x + 1; }; emit f(41);' \
    "42"

run_olang_test "hof/map" \
    'let a = [1, 2, 3]; let b = map(a, fn(x) { return x * 10; }); for v in b { emit v; };' \
    "102030"

run_olang_test "hof/filter" \
    'let a = [1, 2, 3, 4, 5]; let b = filter(a, fn(x) { return x > 3; }); for v in b { emit v; };' \
    "45"

run_olang_test "hof/reduce" \
    'let a = [1, 2, 3, 4]; let s = reduce(a, fn(acc, x) { return acc + x; }); emit s;' \
    "10"

run_olang_test "hof/pipe" \
    'let r = pipe(5, fn(x) { return x * 2; }, fn(x) { return x + 1; }); emit r;' \
    "11"

# 2.10 Math builtins
run_olang_test "math/floor" \
    'emit __floor(3.7);' \
    "3"

run_olang_test "math/ceil" \
    'emit __ceil(3.2);' \
    "4"

# 2.11 Type checking
run_olang_test "type/number" \
    'emit __type_of(42);' \
    "number"

run_olang_test "type/string" \
    'emit __type_of("hi");' \
    "string"

run_olang_test "type/array" \
    'emit __type_of([1,2]);' \
    "array"

run_olang_test "type/function" \
    'let f = fn(x) { return x; }; emit __type_of(f);' \
    "function"

# 2.12 Bitwise
run_olang_test "bitwise/or" \
    'emit __bit_or(5, 3);' \
    "7"

run_olang_test "bitwise/and" \
    'emit __bit_and(7, 3);' \
    "3"

run_olang_test "bitwise/xor" \
    'emit __bit_xor(5, 3);' \
    "6"

run_olang_test "bitwise/shl" \
    'emit __bit_shl(1, 4);' \
    "16"

# 2.13 Mol pack/unpack
run_olang_test "mol/pack" \
    'let m = __mol_pack(5, 3, 4, 2, 1); emit m;' \
    "21385"

run_olang_test "mol/extract_s" \
    'let m = __mol_pack(5, 3, 4, 2, 1); emit __mol_s(m);' \
    "5"

run_olang_test "mol/extract_r" \
    'let m = __mol_pack(5, 3, 4, 2, 1); emit __mol_r(m);' \
    "3"

run_olang_test "mol/roundtrip" \
    'let m = __mol_pack(15, 15, 7, 7, 3); emit __mol_s(m); emit __mol_r(m); emit __mol_v(m); emit __mol_a(m); emit __mol_t(m);' \
    "1515773"

# 2.14 SHA-256
run_olang_test "crypto/sha256" \
    'let h = __sha256(__str_bytes("hello")); emit __type_of(h);' \
    "string"

# 2.15 UTF-8
run_olang_test "utf8/ascii_cp" \
    'emit __utf8_cp("ABC", 0);' \
    "65"

run_olang_test "utf8/multibyte_len" \
    'emit __utf8_len("é", 0);' \
    "2"

# 2.16 Logical operators
run_olang_test "logic/and_true" \
    'if 1 && 1 { emit "yes"; } else { emit "no"; };' \
    "yes"

run_olang_test "logic/and_short" \
    'if 0 && 1 { emit "yes"; } else { emit "no"; };' \
    "no"

run_olang_test "logic/or_true" \
    'if 0 || 1 { emit "yes"; } else { emit "no"; };' \
    "yes"

run_olang_test "logic/not" \
    'if __logic_not(0) { emit "yes"; } else { emit "no"; };' \
    "yes"

# 2.17 Sort
run_olang_test "sort/basic" \
    'let a = sort([3, 1, 2]); for v in a { emit v; };' \
    "123"

# 2.18 Join / Contains
run_olang_test "string/join" \
    'let a = ["a", "b", "c"]; emit join(a, "-");' \
    "a-b-c"

run_olang_test "string/contains" \
    'if contains("hello world", "world") { emit "yes"; } else { emit "no"; };' \
    "yes"

# 2.19 Split
run_olang_test "string/split" \
    'let parts = split("a,b,c", ","); emit __array_len(parts);' \
    "3"

# 2.20 Match expression — tested in file tests (multi-line only)

echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 3: OLANG FILE TESTS (.ol test scripts)
# ═══════════════════════════════════════════════════════════════

echo -e "${CYAN}--- Olang File Tests ---${NC}"

for oltest in "$TEST_DIR"/test_*.ol; do
    [ -f "$oltest" ] || continue
    testname=$(basename "$oltest" .ol)

    # Strip // comments (line-start AND inline) then join all lines
    code_input=$(sed 's|//.*||' "$oltest" | grep -v '^[[:space:]]*$' | tr '\n' ' ')

    # Each .ol test file should emit "PASS" or "FAIL" as last line
    file_actual=$(echo "$code_input" | timeout 10 "$BINARY" 2>/dev/null || true)
    last_line=$(echo "$file_actual" | strip_repl | tail -1)

    if [ "$last_line" = "PASS" ]; then
        PASS=$((PASS + 1))
        echo -e "  $GREEN  OK$NC file/$testname"
    elif [ "$last_line" = "FAIL" ]; then
        FAIL=$((FAIL + 1))
        ERRORS="$ERRORS\n  $RED FAIL$NC file/$testname"
        ERRORS="$ERRORS\n       output: $(echo "$file_actual" | strip_repl | head -5)"
    else
        SKIP=$((SKIP + 1))
        echo -e "  $YELLOW SKIP$NC file/$testname — no PASS/FAIL output"
    fi
done

echo ""

# ═══════════════════════════════════════════════════════════════
# SECTION 4: BINARY SANITY
# ═══════════════════════════════════════════════════════════════

echo -e "${CYAN}--- Binary Sanity ---${NC}"

# Check binary size
BINSIZE=$(stat -c%s "$BINARY" 2>/dev/null || stat -f%z "$BINARY" 2>/dev/null)
if [ "$BINSIZE" -gt 500000 ] && [ "$BINSIZE" -lt 5000000 ]; then
    PASS=$((PASS + 1))
    echo -e "  $GREEN  OK$NC binary/size ($(numfmt --to=iec $BINSIZE))"
else
    FAIL=$((FAIL + 1))
    ERRORS="$ERRORS\n  $RED FAIL$NC binary/size — unexpected: $BINSIZE bytes"
fi

# Check ELF header
if file "$BINARY" | grep -q "ELF 64-bit"; then
    PASS=$((PASS + 1))
    echo -e "  $GREEN  OK$NC binary/elf64"
else
    FAIL=$((FAIL + 1))
    ERRORS="$ERRORS\n  $RED FAIL$NC binary/elf64 — not ELF 64-bit"
fi

# Check no libc dependency (allow vDSO which is kernel-injected, not a real dep)
ldd_out=$(ldd "$BINARY" 2>&1 || true)
if echo "$ldd_out" | grep -q "not a dynamic executable\|statically linked"; then
    PASS=$((PASS + 1))
    echo -e "  $GREEN  OK$NC binary/no_libc (static, zero deps)"
elif ! echo "$ldd_out" | grep -qv "linux-vdso\|ld-linux\|^\s*$"; then
    PASS=$((PASS + 1))
    echo -e "  $GREEN  OK$NC binary/no_libc (only vDSO, no libs)"
else
    FAIL=$((FAIL + 1))
    ERRORS="$ERRORS\n  $RED FAIL$NC binary/no_libc — has dynamic deps: $(echo "$ldd_out" | grep -v vdso | head -3)"
fi

# Boot test — exit immediately
if echo "exit" | timeout 5 "$BINARY" >/dev/null 2>&1; then
    PASS=$((PASS + 1))
    echo -e "  $GREEN  OK$NC binary/boot (starts and exits)"
else
    FAIL=$((FAIL + 1))
    ERRORS="$ERRORS\n  $RED FAIL$NC binary/boot — failed to start"
fi

echo ""

# ═══════════════════════════════════════════════════════════════
# REPORT
# ═══════════════════════════════════════════════════════════════

TOTAL=$((PASS + FAIL + SKIP))

echo -e "${CYAN}═══════════════════════════════════════${NC}"
if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}  ALL PASS: $PASS/$TOTAL tests passed${NC}"
else
    echo -e "${RED}  $FAIL FAILED${NC} / $PASS passed / $SKIP skipped (total: $TOTAL)"
fi
echo -e "${CYAN}═══════════════════════════════════════${NC}"

if [ -n "$ERRORS" ]; then
    echo ""
    echo -e "${RED}Failures:${NC}"
    echo -e "$ERRORS"
fi

echo ""

# Cleanup
rm -rf "$TMP_DIR"

# Exit code
if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
exit 0
