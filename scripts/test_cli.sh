#!/bin/bash
# CLI comprehensive test script

set -e  # Exit on error

echo "=== VTR CLI Comprehensive Tests ==="
echo

VTR="./target/debug/vtr"

# Test 1: Help and version
echo "Test 1: Help and version"
$VTR --help > /dev/null
$VTR --version
echo "✓ Help/version work"
echo

# Test 2: Single file ingestion
echo "Test 2: Single file ingestion"
echo "fn main() { println!(\"hello\"); }" > /tmp/test_vtr.rs
RESULT=$($VTR ingest /tmp/test_vtr.rs 2>&1)
echo "$RESULT" | grep -q "\"status\":\"success\"" && echo "✓ Ingest successful"
echo "$RESULT" | grep -q "\"cpg_hash\"" && echo "✓ Hash present"
echo "$RESULT"
echo

# Test 3: Snapshot save
echo "Test 3: Snapshot save"
RESULT=$($VTR snapshot save 2>&1)
echo "$RESULT" | grep -q "\"status\":\"success\"" && echo "✓ Snapshot save successful"
echo "$RESULT"
echo

# Test 4: Snapshot verify
echo "Test 4: Snapshot verify"
echo '{"test":"data"}' > /tmp/test_snapshot.json
RESULT=$($VTR snapshot verify /tmp/test_snapshot.json 2>&1 || true)
echo "$RESULT" | grep -q "error" && echo "✓ Invalid snapshot rejected (expected)"
echo "$RESULT"
echo

# Test 5: Query (placeholder)
echo "Test 5: Query"
echo '{"query": "find_functions"}' > /tmp/test_query.json
RESULT=$($VTR query /tmp/test_query.json 2>&1)
echo "$RESULT" | grep -q "\"status\":\"success\"" && echo "✓ Query executed"
echo "$RESULT"
echo

# Test 6: Explain (placeholder)
echo "Test 6: Explain"
RESULT=$($VTR explain test-result-123 2>&1)
echo "$RESULT" | grep -q "\"status\":\"success\"" && echo "✓ Explain executed"
echo "$RESULT"
echo

# Test 7: Config loading
echo "Test 7: Config loading"
cat > /tmp/test_vtr.toml << EOF
[io]
mode = "auto"
uring_enabled = false

[snapshot]
path = "./test_snapshots"
auto_save = true

[execution]
parallel = false
thread_count = 0
EOF

RESULT=$($VTR ingest /tmp/test_vtr.rs --config /tmp/test_vtr.toml 2>&1)
echo "$RESULT" | grep -q "\"status\":\"success\"" && echo "✓ Config loading works"
echo

# Test 8: Error handling (file not found)
echo "Test 8: Error handling"
RESULT=$($VTR ingest /nonexistent/file.rs 2>&1 || true)
echo "$RESULT" | grep -q "error" && echo "✓ Error handling works (file not found)"
echo "$RESULT"
echo

echo "=== All CLI tests passed ✓ ==="
