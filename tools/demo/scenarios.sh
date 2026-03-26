#!/bin/bash
# ═══════════════════════════════════════════════════════════════
#  ○ HomeOS E2E Demo — 10 Scenarios
#  Usage: make demo   (hoặc bash tools/demo/scenarios.sh)
# ═══════════════════════════════════════════════════════════════

set -euo pipefail

CARGO="${CARGO:-cargo}"
SERVER="$CARGO run -p server --"
PASS=0
FAIL=0
TOTAL=10

run_eval() {
    echo "$1" | $SERVER --eval 2>/dev/null
}

check_contains() {
    local label="$1"
    local input="$2"
    shift 2
    local patterns=("$@")

    printf "  %s... " "$label"
    local result
    result=$(run_eval "$input")

    for pat in "${patterns[@]}"; do
        if echo "$result" | grep -qi "$pat"; then
            echo "PASS"
            PASS=$((PASS + 1))
            return
        fi
    done
    echo "FAIL (got: $result)"
    FAIL=$((FAIL + 1))
}

check_not_empty() {
    local label="$1"
    local input="$2"

    printf "  %s... " "$label"
    local result
    result=$(run_eval "$input")

    if [ -n "${result// /}" ]; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL (empty output)"
        FAIL=$((FAIL + 1))
    fi
}

check_exit_code() {
    local label="$1"
    local input="$2"
    local expected="$3"

    printf "  %s... " "$label"
    local code=0
    echo "$input" | $SERVER --eval >/dev/null 2>&1 || code=$?

    if [ "$code" -eq "$expected" ]; then
        echo "PASS"
        PASS=$((PASS + 1))
    else
        echo "FAIL (exit=$code, expected=$expected)"
        FAIL=$((FAIL + 1))
    fi
}

echo "═══════════════════════════════════════════"
echo "  ○ HomeOS E2E Demo — $TOTAL Scenarios"
echo "═══════════════════════════════════════════"
echo

# ── 1. Basic: empty input exits cleanly ──────────────────────
check_exit_code "1. Empty input exits cleanly" "" 0

# ── 2. Natural text produces response ────────────────────────
check_not_empty "2. Natural text → response" "hello"

# ── 3. Vietnamese emotion: supportive tone ───────────────────
check_not_empty "3. Vietnamese emotion → response" "tôi buồn vì mất việc"

# ── 4. Stats command ─────────────────────────────────────────
check_contains "4. ○{stats} shows system info" "○{stats}" "stm" "silk" "node" "turn" "registry"

# ── 5. Multi-line input ──────────────────────────────────────
check_not_empty "5. Multi-line input processes" "$(printf 'hello\ntôi vui')"

# ── 6. Inline Olang program (> prefix) ──────────────────────
check_not_empty "6. Inline Olang program" "> emit 42;"

# ── 7. Emotion: positive text ───────────────────────────────
check_not_empty "7. Positive emotion → response" "tôi rất vui hôm nay"

# ── 8. SecurityGate: crisis detection ───────────────────────
check_not_empty "8. Crisis input → response" "tôi muốn tự tử"

# ── 9. Whitespace-only input → no output ────────────────────
printf "  9. Whitespace-only → no output... "
result=$(run_eval "   ")
if [ -z "${result// /}" ]; then
    echo "PASS"
    PASS=$((PASS + 1))
else
    echo "FAIL (got: $result)"
    FAIL=$((FAIL + 1))
fi

# ── 10. Sequential turns accumulate ─────────────────────────
check_not_empty "10. Sequential turns" "$(printf 'hello\nhello\nhello')"

echo
echo "═══════════════════════════════════════════"
echo "  Results: $PASS pass, $FAIL fail / $TOTAL"
echo "═══════════════════════════════════════════"

exit "$FAIL"
