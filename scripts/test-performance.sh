#!/bin/bash
set -euo pipefail

# Performance testing script for vex
# Tests: parallel downloads, cache acceleration, version switching speed

echo "=========================================="
echo "vex Performance Tests"
echo "=========================================="
echo ""

PASSED=0
FAILED=0
TMPDIR=$(mktemp -d)

cleanup() {
    rm -rf "$TMPDIR" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

pass() {
    echo "  ✓ $1"
    ((PASSED++))
}

fail() {
    echo "  ✗ $1"
    ((FAILED++))
}

# Test 1: Version switching speed
echo "[ 1. Version Switching Speed ]"
if command -v vex >/dev/null 2>&1; then
    # Install two Node versions for testing
    vex install node@20.11.0 >/dev/null 2>&1 || true
    vex install node@22.0.0 >/dev/null 2>&1 || true

    # Measure switch time
    START=$(date +%s%N)
    vex use node@20.11.0 >/dev/null 2>&1
    END=$(date +%s%N)
    SWITCH_TIME=$(( (END - START) / 1000000 ))  # Convert to milliseconds

    if [ "$SWITCH_TIME" -lt 1000 ]; then
        pass "Version switch completed in ${SWITCH_TIME}ms (< 1s)"
    else
        fail "Version switch took ${SWITCH_TIME}ms (> 1s)"
    fi

    # Test multiple switches
    START=$(date +%s%N)
    for _ in {1..5}; do
        vex use node@20.11.0 >/dev/null 2>&1
        vex use node@22.0.0 >/dev/null 2>&1
    done
    END=$(date +%s%N)
    AVG_TIME=$(( (END - START) / 10000000 ))  # 10 switches, convert to ms

    if [ "$AVG_TIME" -lt 500 ]; then
        pass "Average switch time: ${AVG_TIME}ms (< 500ms)"
    else
        fail "Average switch time: ${AVG_TIME}ms (> 500ms)"
    fi
else
    fail "vex command not found"
fi

# Test 2: Cache effectiveness
echo ""
echo "[ 2. Cache Effectiveness ]"
if command -v vex >/dev/null 2>&1; then
    # Clear cache
    rm -rf ~/.vex/cache/*.json 2>/dev/null || true

    # First call (no cache)
    START=$(date +%s%N)
    vex list-remote node --filter latest >/dev/null 2>&1
    END=$(date +%s%N)
    NO_CACHE_TIME=$(( (END - START) / 1000000 ))

    # Second call (with cache)
    START=$(date +%s%N)
    vex list-remote node --filter latest >/dev/null 2>&1
    END=$(date +%s%N)
    CACHE_TIME=$(( (END - START) / 1000000 ))

    SPEEDUP=$(( NO_CACHE_TIME / (CACHE_TIME + 1) ))

    if [ "$SPEEDUP" -ge 5 ]; then
        pass "Cache speedup: ${SPEEDUP}x (no-cache: ${NO_CACHE_TIME}ms, cached: ${CACHE_TIME}ms)"
    else
        fail "Cache speedup only ${SPEEDUP}x (expected >= 5x)"
    fi

    # Verify cache file exists
    if [ -f ~/.vex/cache/remote-node.json ]; then
        pass "Cache file created successfully"
    else
        fail "Cache file not found"
    fi
else
    fail "vex command not found"
fi

# Test 3: Binary execution speed
echo ""
echo "[ 3. Binary Execution Speed ]"
if command -v vex >/dev/null 2>&1; then
    vex use node@20.11.0 >/dev/null 2>&1 || true

    # Test symlink resolution speed
    START=$(date +%s%N)
    for _ in {1..100}; do
        ~/.vex/bin/node --version >/dev/null 2>&1
    done
    END=$(date +%s%N)
    AVG_EXEC_TIME=$(( (END - START) / 100000 ))  # 100 calls, convert to us

    if [ "$AVG_EXEC_TIME" -lt 10000 ]; then
        pass "Average execution time: ${AVG_EXEC_TIME}μs (< 10ms)"
    else
        fail "Average execution time: ${AVG_EXEC_TIME}μs (> 10ms)"
    fi
else
    fail "vex command not found"
fi

# Test 4: Concurrent operations
echo ""
echo "[ 4. Concurrent Operations ]"
if command -v vex >/dev/null 2>&1; then
    # Test concurrent list-remote calls
    START=$(date +%s%N)
    vex list-remote node --filter latest >/dev/null 2>&1 &
    vex list-remote go --filter latest >/dev/null 2>&1 &
    vex list-remote python --filter latest >/dev/null 2>&1 &
    wait
    END=$(date +%s%N)
    CONCURRENT_TIME=$(( (END - START) / 1000000 ))

    if [ "$CONCURRENT_TIME" -lt 3000 ]; then
        pass "Concurrent operations completed in ${CONCURRENT_TIME}ms (< 3s)"
    else
        fail "Concurrent operations took ${CONCURRENT_TIME}ms (> 3s)"
    fi
else
    fail "vex command not found"
fi

# Test 5: Memory usage
echo ""
echo "[ 5. Memory Usage ]"
if command -v vex >/dev/null 2>&1; then
    # Get memory usage during version switch
    if command -v /usr/bin/time >/dev/null 2>&1; then
        MEM_OUTPUT=$(/usr/bin/time -l vex use node@20.11.0 2>&1 | grep "maximum resident set size" || echo "0")
        MEM_KB=$(echo "$MEM_OUTPUT" | awk '{print $1}')
        MEM_MB=$(( MEM_KB / 1024 ))

        if [ "$MEM_MB" -lt 100 ]; then
            pass "Memory usage: ${MEM_MB}MB (< 100MB)"
        else
            fail "Memory usage: ${MEM_MB}MB (> 100MB)"
        fi
    else
        echo "  ⚠ /usr/bin/time not available, skipping memory test"
    fi
else
    fail "vex command not found"
fi

# Summary
echo ""
echo "=========================================="
echo "Performance Tests: $PASSED passed, $FAILED failed"
echo "=========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
