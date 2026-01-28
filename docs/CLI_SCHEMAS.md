# VTR CLI JSON Schemas

**API Contract - v0.1.0**

These schemas are FROZEN. Breaking changes require version bump.

---

## Success Responses

### `vtr ingest`

```json
{
  "status": "success",
  "epoch_id": 1,
  "cpg_hash": "sha256_hex_string",
  "nodes": 42
}
```

**Fields**:
- `status`: Always `"success"`
- `epoch_id`: Ingestion epoch ID (u64)
- `cpg_hash`: SHA-256 hash of CPG (deterministic)
- `nodes`: Parse tree node count

---

### `vtr snapshot save`

```json
{
  "status": "success",
  "snapshot_id": 1,
  "hash": "sha256_hex_string"
}
```

**Fields**:
- `status`: Always `"success"`
- `snapshot_id`: Snapshot identifier (u64)
- `hash`: SHA-256 hash of snapshot (matches CPG hash)

---

### `vtr snapshot load`

```json
{
  "status": "success",
  "hash": "sha256_hex_string",
  "verified": true
}
```

**Fields**:
- `status`: Always `"success"`
- `hash`: Verified snapshot hash
- `verified`: Hash verification result (always true on success)

---

### `vtr snapshot verify`

```json
{
  "status": "success",
  "hash": "sha256_hex_string",
  "valid": true
}
```

**Fields**:
- `status`: Always `"success"`
- `hash`: Snapshot hash
- `valid`: Validation result (always true on success)

---

### `vtr query`

```json
{
  "status": "success",
  "query": "path/to/query.json",
  "results": [],
  "count": 0
}
```

**Fields**:
- `status`: Always `"success"`
- `query`: Query file path
- `results`: Query results array (node IDs or data)
- `count`: Result count

---

### `vtr explain`

```json
{
  "status": "success",
  "result_id": "result_identifier",
  "provenance": ["trace_item_1", "trace_item_2"]
}
```

**Fields**:
- `status`: Always `"success"`
- `result_id`: Result being explained
- `provenance`: Deterministic provenance trace

---

## Error Response

**All failures use this schema**:

```json
{
  "status": "error",
  "message": "Human-readable error description",
  "fatal": true
}
```

**Fields**:
- `status`: Always `"error"`
- `message`: Error description (deterministic)
- `fatal`: Always `true` (fail-closed)

**Examples**:

```json
{"status":"error","message":"Path not found: /invalid/path","fatal":true}
{"status":"error","message":"Snapshot verification failed: hash mismatch","fatal":true}
{"status":"error","message":"Parse failed: syntax error at line 10","fatal":true}
```

---

## 

Contract Rules

1. **Stability**: These schemas are frozen for v0.1.x
2. **Determinism**: Same input → same output (always)
3. **Fail-closed**: Errors always fatal, never partial success
4. **Machine-first**: Optimized for parsing, not humans
5. **No optional fields**: All fields always present

**Breaking changes** (major version bump):
- Removing fields
- Changing field types
- Changing field names

**Non-breaking additions** (minor version bump):
- Adding new fields
- New status values (if backward compatible)
