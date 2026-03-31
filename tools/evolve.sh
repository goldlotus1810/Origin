#!/bin/bash
# evolve.sh — Olang self-evolution loop
# Chạy 24/7: benchmark → detect → patch → test → deploy
# Nox kiểm tra kết quả mỗi ngày

set -e
cd "$(dirname "$0")/.."

LOG="logs/evolve_$(date +%Y%m%d_%H%M%S).log"
mkdir -p logs

log() { echo "[$(date +%H:%M:%S)] $1" | tee -a "$LOG"; }

BINARY="./origin_new.olang"
GEN=0

log "=== OLANG EVOLUTION STARTED ==="
log "Binary: $BINARY ($(stat -c%s $BINARY) bytes)"

while true; do
    GEN=$((GEN + 1))
    log ""
    log "=== GENERATION $GEN ==="

    # 1. BENCHMARK
    log "Benchmarking..."
    FIB30=$(echo 'fn fib(n) { if n < 2 { return n; }; return fib(n-1) + fib(n-2); }; let t0 = __time(); let r = fib(30); let t1 = __time(); emit t1 - t0;' | timeout 30 "$BINARY" 2>/dev/null | grep -oP '^\d+' | tail -1)

    LOOP=$(echo 'let t0 = __time(); let s = 0; let i = 0; while i < 1000000 { let s = s + i; let i = i + 1; }; let t1 = __time(); emit t1 - t0;' | timeout 30 "$BINARY" 2>/dev/null | grep -oP '^\d+' | tail -1)

    SHA=$(echo 'let t0 = __time(); let h = "test"; let i = 0; while i < 1000 { let h = __sha256(h); let i = i + 1; }; let t1 = __time(); emit t1 - t0;' | timeout 30 "$BINARY" 2>/dev/null | grep -oP '^\d+' | tail -1)

    JIT_FIB=$(echo 'let nf = jit_fib(); let t0 = __time(); let r = __call_native(nf, 30); let t1 = __time(); emit t1 - t0;' | timeout 10 "$BINARY" 2>/dev/null | grep -oP '^\d+' | tail -1)

    log "  fib(30) interp: ${FIB30:-timeout}ms"
    log "  loop 1M:        ${LOOP:-timeout}ms"
    log "  SHA×1000:        ${SHA:-timeout}ms"
    log "  fib(30) JIT:     ${JIT_FIB:-timeout}ms"

    # 2. TEST
    log "Testing..."
    TEST_RESULT=$(bash tests.sh 2>&1 | grep -oP '\d+(?= passed)')
    TEST_TOTAL=$(bash tests.sh 2>&1 | grep -oP 'total: \K\d+')
    log "  Tests: ${TEST_RESULT:-0}/${TEST_TOTAL:-0}"

    # 3. RECORD
    echo "$GEN,$(date +%s),$FIB30,$LOOP,$SHA,$JIT_FIB,$TEST_RESULT,$TEST_TOTAL" >> logs/evolution_data.csv

    # 4. REPORT
    log "  Binary: $(stat -c%s $BINARY) bytes"
    log "  VM ASM: $(wc -l < vm/x86_64/vm_x86_64.S) lines"

    # 5. SLEEP (run every 6 hours — 4 generations per day)
    log "Next generation in 6 hours..."
    sleep 21600
done
