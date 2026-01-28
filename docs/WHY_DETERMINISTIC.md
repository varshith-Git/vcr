# Why VTR Always Gives the Same Answer

**A technical explanation of VTR's determinism guarantees**

---

## The Problem

Most code analysis tools are non-deterministic:
- Same code analyzed twice → different results
- Parallel execution → race conditions  
- Crashes → corrupted state
- Timestamps → irreproducible behavior
- Heuristics → "usually works"

**This erodes trust.**

You cannot prove what the tool said yesterday.  
You cannot replay past analyses.  
You cannot audit decisions.

---

## VTR's Approach

VTR guarantees: **Same input → same output (always).**

Not "usually". Not "with high probability". **Always.**

---

## 1. Epoch-Based Memory Model

**Core principle**: Every analysis is a pure function from input to output.

### How It Works

```
Input (source code) → ParseEpoch → SemanticEpoch → CPGEpoch → Results
```

**Each epoch is**:
- Self-contained (no shared mutable state)
- Immutable after construction
- Deterministically hashable
- Independently verifiable

**No global state.** No timestamps. No randomness.

### Example

```rust
// Two parse epochs from same source
let epoch1 = ParseEpoch::from_source(source);
let epoch2 = ParseEpoch::from_source(source);

assert_eq!(epoch1.hash(), epoch2.hash());  // Always true
```

**Why this matters**: You can recreate any past analysis exactly.

---

## 2. Deterministic Graph Construction

**Problem**: Graph traversal order affects results if not controlled.

**VTR's solution**: Explicit ordering at every step.

### Control Flow Graph (CFG)

```rust
// Nodes added in syntax order (deterministic)
for stmt in function.statements() {
    cfg.add_node(stmt.id, stmt.kind);  // ID = source position
}

// Edges added in declaration order
for edge in cfg.edges() {
    // Edge order = (from_id, to_id) lexicographic
}
```

**Result**: Same source → same node IDs → same edge order.

### Data Flow Graph (DFG)

```rust
// Values numbered by SSA construction order (deterministic)
let v1 = ssa_builder.phi(v0);  // ID = construction sequence
let v2 = ssa_builder.value(expr);
```

**No** Bloom filters.  
**No** hash maps with random seeds.  
**No** "approximate" structures.

### Code Property Graph (CPG)

Fusion of CFG + DFG + AST:

```rust
cpg.nodes = merge_deterministic(cfg.nodes, dfg.nodes, ast.nodes);
cpg.edges = sort_by_id(all_edges);  // Total order
```

**Hash verification**:
```rust
let hash = cpg.compute_hash();  // SHA-256 over sorted fields
```

Same CPG → same hash. **Always.**

---

## 3. Bounded Analysis

**Problem**: Unbounded analysis (full interprocedural, whole-program) introduces complexity that breaks determinism.

**VTR's choice**: Bounded, conservative analysis with explicit limits.

### Pointer Analysis

```rust
// K-limited context sensitivity
const MAX_CONTEXT_DEPTH: usize = 3;

// Field-sensitive up to N levels
const MAX_FIELD_DEPTH: usize = 2;

// Stop at function boundaries (modular)
```

**Result**: Analysis always terminates. Same code → same points-to sets.

### Taint Analysis

```rust
// Path enumeration with depth limit
const MAX_TAINT_PATH_LENGTH: usize = 10;

// No probabilistic propagation
// No "may-alias" approximations
```

If a path exists, we find it. If we hit the limit, we report incompleteness (deterministically).

**No hidden timeouts.** No "try harder" heuristics.

---

## 4. Deterministic Execution

**Problem**: Parallelism introduces non-determinism via race conditions.

**VTR's solution**: Parallel execution, serial commit.

### Execution Model

```rust
// Parallel task execution (performance)
tasks.par_iter().for_each(|task| {
    let result = execute(task);
    results.lock().insert(task.id, result);  // Concurrent writes
});

// Serial result commit (determinism)
results.sort_by_key(|r| r.task_id);  // Total order
for result in results {
    cpg.commit(result);  // Sequential state updates
}
```

**Guarantee**: Parallel = serial results. **Always.**

Verified by stress tests:
```rust
let serial = execute_serial(plan);
let parallel = execute_parallel(plan);
assert_eq!(serial.hash(), parallel.hash());
```

---

## 5. Fail-Closed Philosophy

**Problem**: Partial failures lead to corrupted state.

**VTR's choice**: Crash on any anomaly.

### Hash Verification

```rust
// Load snapshot
let cpg = CPGSnapshot::load(path)?;
let stored_hash = snapshot.hash;
let computed_hash = cpg.compute_hash();

if stored_hash != computed_hash {
    panic!("Hash mismatch: snapshot corrupted");  // Crash
}
```

**No recovery attempts.** No "best effort". Fail immediately.

### Version Checks

```rust
if snapshot.version != CURRENT_VERSION {
    return Err("Version mismatch");  // Refuse to load
}
```

**No automatic migration.** Explicit versioning only.

---

## 6. Replay Guarantees

**Core guarantee**: Given a snapshot, you can replay to the exact same state.

### Snapshot Structure

```flatbuffers
table CPGSnapshot {
  epoch_id: uint64;
  cpg_hash: string;       // SHA-256 (deterministic)
  timestamp: uint64;      // Recording time (metadata only)
  version: uint32;        // Schema version
  nodes: [CPGNode];       // Sorted by ID
  edges: [CPGEdge];       // Sorted by (from, to, kind)
}
```

**Fields are ordered.** Hash is over sorted representation.

### Replay Process

```rust
// Save
let snapshot = CPGSnapshot::save(&cpg, path)?;
let original_hash = cpg.compute_hash();

// Load (later, different machine)
let restored_cpg = CPGSnapshot::load(path)?;
let restored_hash = restored_cpg.compute_hash();

assert_eq!(original_hash, restored_hash);  // Always true
```

---

## What VTR Does NOT Do

To maintain determinism, VTR explicitly **avoids**:

❌ **Heuristics**: No "probably correct" approximations  
❌ **Timestamps in kernel**: Outside metadata only  
❌ **Floating point in critical paths**: Integer arithmetic only  
❌ **Probabilistic inference**: No ML, no Bayesian models  
❌ **System clocks inside epochs**: No `now()` in analysis  
❌ **Random sampling**: No probabilistic algorithms  
❌ **Unbounded iteration**: All loops have explicit limits  

**Trade-off**: Expressiveness for certainty.

---

## How to Verify VTR's Determinism

**Test 1: Run twice**
```bash
vtr ingest repo/ > out1.json
vtr ingest repo/ > out2.json
diff out1.json out2.json  # Must be empty
```

**Test 2: Enable/disable features**
```bash
# With io_uring
cargo build --features cold-path-uring
vtr ingest repo/ > with_uring.json

# Without io_uring  
cargo build
vtr ingest repo/ > without_uring.json

# Must match
diff with_uring.json without_uring.json
```

**Test 3: Load snapshot**
```bash
# Save
vtr ingest repo/
vtr snapshot save > snapshot.json
HASH1=$(jq -r '.hash' snapshot.json)

# Restart
vtr snapshot load snapshot-id
HASH2=$(vtr snapshot verify snapshot-id | jq -r '.hash')

# Must match
test "$HASH1" = "$HASH2"
```

**Test 4: Parallel vs serial**
```bash
# Serial
cargo build
vtr ingest repo/ > serial.json

# Parallel
cargo build --features parallel-execution
vtr ingest repo/ > parallel.json

# Must match
diff serial.json parallel.json
```

---

## Guarantees Summary

| Property | VTR Guarantee |
|----------|---------------|
| Same source → same CPG | ✅ Always |
| Same query → same results | ✅ Always |
| Parallel = serial | ✅ Always (verified) |
| Snapshot restore | ✅ Exact hash match |
| Feature flags | ✅ Same semantics |
| Cross-platform | ✅ Same hash (byte-for-byte) |
| Across versions | ✅ Within major version |

---

## Why This Matters

**Security audits**: "What did VTR report on 2023-01-15?" → Load snapshot, replay exactly.

**Compliance**: "Prove this code was analyzed correctly." → Hash verification chain.

**Debugging**: "Why did query X return Y?" → Deterministic provenance trace.

**Trust**: "Can I depend on this tool?" → Yes. It will never lie.

---

## The Cost

Determinism is not free:

**What we sacrifice**:
- Unbounded analysis (bounded instead)
- Probabilistic speedups (deterministic execution only)
- Automatic state recovery (crash on corruption)

**What we gain**:
- Reproducible results (always)
- Auditable decisions (provable)
- Trust under stress (fail-closed)

---

## Conclusion

VTR always gives the same answer because:

1. **Epochs are pure functions** (no hidden state)
2. **Graphs have total order** (no random traversal)
3. **Analysis is bounded** (explicit limits)
4. **Execution is serialized** (deterministic commit)
5. **Failures are fatal** (no partial corruption)
6. **Snapshots are replayable** (hash-verified)

**This is infrastructure, not a research prototype.**

Infrastructure doesn't guess. It proves.
