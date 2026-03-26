#!/bin/bash
# nox_check.sh — Nox daily status check
# Run: bash tools/nox_check.sh

cd "$(dirname "$0")/.."

echo "╔═══════════════════════════════════════════╗"
echo "║  NOX DAILY CHECK — $(date +%Y-%m-%d)          ║"
echo "╠═══════════════════════════════════════════╣"

# Binary
echo "║  Binary: $(ls -lh origin_new.olang | awk '{print $5}')"
echo "║  VM ASM: $(wc -l < vm/x86_64/vm_x86_64.S) lines"

# Git
echo "║  Commits: $(git log --oneline | wc -l)"
echo "║  Last: $(git log --oneline -1)"

# Quick benchmark
echo "║"
echo "║  Quick benchmark:"

FIB=$(echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; let t0 = __time(); emit fib(30); let t1 = __time(); emit t1-t0;' | timeout 30 ./origin_new.olang 2>/dev/null | tail -2 | head -1)
echo "║    fib(30) interp: ${FIB}ms"

JIT=$(echo 'let nf = jit_fib(); let t0 = __time(); let r = __call_native(nf, 30); let t1 = __time(); emit t1-t0;' | timeout 10 ./origin_new.olang 2>/dev/null | grep -oP '^\d+' | tail -1)
echo "║    fib(30) JIT:    ${JIT}ms"

# Tests
TESTS=$(bash tests.sh 2>&1 | tail -3 | head -1)
echo "║    Tests: $TESTS"

# Evolution log
if [ -f logs/evolution_data.csv ]; then
    GENS=$(tail -n +2 logs/evolution_data.csv | wc -l)
    echo "║    Generations: $GENS"
    if [ "$GENS" -gt 0 ]; then
        LAST=$(tail -1 logs/evolution_data.csv)
        echo "║    Last: $LAST"
    fi
fi

echo "║"
echo "╚═══════════════════════════════════════════╝"
