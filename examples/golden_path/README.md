# Golden Path Example

**Purpose**: Regression sanity check for v0.1.0

Not for marketing. For validation that VCR still works.

---

## Files

- `simple.rs` - Minimal Rust code (2 functions)
- `query.json` - Simple query
- `expected_output.txt` - Expected deterministic output

---

## Run

```bash
# Ingest
vcr ingest examples/golden_path/simple.rs

# Query
vcr query examples/golden_path/query.json

# Explain
vcr explain test-result-1
```

---

## Expected Behavior

**Ingest output**:
- `status`: `"success"`
- `cpg_hash`: Deterministic SHA-256 (same every time)
- `nodes`: Parse tree node count

**Query output**:
- `status`: `"success"`
- `count`: 0 (empty CPG in current implementation)

**Explain output**:
- `status`: `"success"`
- `provenance`: Deterministic trace

---

## Validation

Run this example before every release:

```bash
# Should produce identical output every time
vcr ingest examples/golden_path/simple.rs > /tmp/out1.json
vcr ingest examples/golden_path/simple.rs > /tmp/out2.json
diff /tmp/out1.json /tmp/out2.json  # Should be empty
```

If hashes change, release is broken.
