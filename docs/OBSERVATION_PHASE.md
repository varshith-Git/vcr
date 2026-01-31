# VCR Observation Phase - Discipline & Guidelines

**Status**: v0.1.0 deployed. Now observing.

**Duration**: 3-6 months minimum before considering expansion.

---

## Core Principle

> "The kernel is frozen. We only improve visibility, not capability."

---

## Acceptable Changes

During observation, ONLY these changes are allowed:

### 1. Better Logs

**Goal**: Understand what VCR is doing, without changing what it does.

**Examples**:
```rust
// ALLOWED: Add structured logging
log::info!("CPG hash computed: {}", hash);
log::debug!("Nodes processed: {}", count);

// FORBIDDEN: Change logic
if should_log {  // ❌ New conditional logic
    compute_hash()
}
```

**Guidelines**:
- Add `log::` statements (info, debug, trace)
- Use structured logging (key=value format)
- Never log inside deterministic kernel (metadata only)
- Log at boundaries (ingest start/end, query run, snapshot save)

---

### 2. Clearer Errors

**Goal**: Make failures diagnosable, not recoverable.

**Examples**:
```rust
// ALLOWED: Better error messages
return Err(format!("Parse failed at line {}: {}", line, reason));

// FORBIDDEN: Error recovery
.unwrap_or_else(|e| {  // ❌ Trying to recover
    default_value()
})
```

**Guidelines**:
- Add context to error messages
- Include file paths, line numbers, node IDs
- Still fail-closed (no recovery attempts)
- Never hide failures

---

### 3. Clearer Configs

**Goal**: Make behavior explicit, not automatic.

**Examples**:
```toml
# ALLOWED: Add config documentation
[io]
# I/O backend mode:
# - "auto": Choose based on platform
# - "hot": Use mmap (fast, more memory)
# - "cold": Use io_uring (Linux only)
mode = "auto"

# FORBIDDEN: Add smart defaults
auto_detect_best_mode = true  # ❌ Magic behavior
```

**Guidelines**:
- Document every config option
- Provide examples in `vtr.toml`
- No auto-discovery or "smart" defaults
- Explicit always wins

---

### 4. Better Docs

**Goal**: Explain what exists, not what could exist.

**Examples**:
```markdown
<!-- ALLOWED: Document actual behavior -->
## How VCR Handles Errors
VCR fails closed. Any hash mismatch crashes immediately.

<!-- FORBIDDEN: Roadmap features -->
## Future: Auto-Recovery (coming soon!)  ❌
```

**Guidelines**:
- Document current behavior only
- Add examples from production
- Update deployment runbook
- No roadmaps or future features

---

## Forbidden Changes

These changes are **PROHIBITED** during observation:

### ❌ New Analyses

```rust
// FORBIDDEN
fn taint_analysis_v2() { ... }
fn control_flow_sensitivity() { ... }
```

**Why**: Changes kernel semantics. Breaks determinism guarantees.

---

### ❌ Optimizations

```rust
// FORBIDDEN
#[cfg(feature = "experimental-simd")]
fn fast_hash() { ... }

// FORBIDDEN
use dashmap::DashMap;  // Concurrent hashmap
```

**Why**: Performance is optional. Correctness is mandatory.

---

### ❌ Heuristics

```rust
// FORBIDDEN
if likely_to_succeed(node) {  // Probabilistic
    analyze(node);
}
```

**Why**: "Probably correct" is not acceptable.

---

### ❌ UI/Dashboard

```rust
// FORBIDDEN
fn web_server() { ... }
fn dashboard_html() { ... }
```

**Why**: Premature. CLI + JSON is sufficient.

---

### ❌ Distributed Execution

```rust
// FORBIDDEN
fn distribute_tasks(cluster: &Cluster) { ... }
```

**Why**: Adds complexity without proven need.

---

### ❌ ML Integration

```rust
// FORBIDDEN
fn ml_based_taint_prediction() { ... }
```

**Why**: Probabilistic ≠ deterministic.

---

## Decision Framework

Before making ANY change, ask:

1. **Does it risk correctness?** → No.
2. **Does it change semantics?** → No.
3. **Does narrow deployment need it?** → Probably no.
4. **Can it wait 6 months?** → Probably yes.

**Default answer: Not yet.**

---

## Monitoring During Observation

Track these metrics in production:

### Success Metrics (Must Maintain)

```bash
# Daily
✅ Completion rate: 100%
✅ Hash consistency: 100%
✅ Manual interventions: 0
✅ Crashes: 0

# Weekly  
✅ Determinism verification: Pass
✅ Snapshot chain validation: Pass
✅ Same input → same hash: Pass
```

### Diagnostic Metrics (For Learning)

```bash
# Track but don't alert
- Execution time (median, p95, p99)
- Memory usage (peak, average)
- I/O patterns (reads, writes)
- Parse tree size distribution
```

**Important**: Performance metrics are informational ONLY. Never optimize based on these during observation.

---

## Logging Strategy

Add structured logs at these boundaries:

### Ingestion
```rust
log::info!("ingest_start", path = %path, config = ?config);
// ... ingest ...
log::info!("ingest_complete", epoch_id = epoch.id, hash = %hash, duration_ms = elapsed);
```

### Snapshot
```rust
log::info!("snapshot_save_start", epoch_id = epoch.id);
// ... save ...
log::info!("snapshot_save_complete", snapshot_id = id, hash = %hash);
```

### Query
```rust
log::debug!("query_execute_start", query = %query_file);
// ... execute ...
log::debug!("query_execute_complete", results = results.len());
```

### Errors
```rust
log::error!("snapshot_verify_failed", 
    path = %path, 
    expected_hash = %expected, 
    actual_hash = %actual,
    error = "hash mismatch");
```

---

## Documentation Updates

During observation, update:

### 1. Runbook
Add actual failure cases encountered:
```markdown
## Observed Failures

### 2026-02-15: Hash mismatch on corrupted NFS
- Symptom: Snapshot verification failed
- Root cause: NFS cache inconsistency
- Resolution: Use local storage for snapshots
- Prevention: Document storage requirements
```

### 2. Deployment Guide
Add lessons from deployment:
```markdown
## Deployment Lessons

- VCR requires stable filesystem (no NFS for snapshots)
- io_uring on kernel < 5.10: use hot-path mode
- Parallel execution: measure first, enable only if needed
```

### 3. FAQ
Based on actual questions:
```markdown
## FAQ

Q: Why did VCR crash instead of continuing?
A: Fail-closed philosophy. Corrupted state is unacceptable.

Q: Can we make VCR faster?
A: Performance is optional. Enable feature flags if needed.
```

---

## Change Review Process

For ANY proposed change:

```markdown
## Change Proposal Template

**Description**: [What changes]

**Category**: 
- [ ] Logging
- [ ] Error clarity
- [ ] Config documentation
- [ ] Documentation
- [ ] OTHER (requires extraordinary justification)

**Risks**:
- Does it touch kernel? [Yes/No]
- Does it change semantics? [Yes/No]
- Could it break determinism? [Yes/No]

**Justification**:
- Why now? [Required for observation/debugging]
- Why not wait? [Blocks production validation]

**Validation**:
- [ ] Determinism tests still pass
- [ ] Hash stability maintained
- [ ] No new feature flags added
```

**Approval required**: If ANY risk is "Yes", reject by default.

---

## Monthly Review

At the end of each month:

### Review Checklist

```markdown
## Month N Review

### Stability
- [ ] 100% uptime
- [ ] Zero hash inconsistencies
- [ ] Zero manual interventions
- [ ] Zero "weird behavior" reports

### Changes Made
- Logging improvements: [count]
- Error message improvements: [count]
- Documentation updates: [count]
- Config clarifications: [count]

### Changes REJECTED
- Feature requests: [count]
- Optimization requests: [count]
- "While we're here" requests: [count]

### Observations
- What did we learn?
- What surprised us?
- What validated our assumptions?

### Decision
- [ ] Continue observation (default)
- [ ] Expand to second environment (requires all stability checks)
- [ ] Stop and debug (if any failures)
```

---

## Expansion Criteria (Post-Observation)

ONLY expand if:

1. ✅ **3+ months** of stable operation
2. ✅ **Zero** hash inconsistencies  
3. ✅ **Zero** crashes requiring intervention
4. ✅ **100%** determinism validation pass rate
5. ✅ Developers **trust** the output
6. ✅ Runbook covers **all** encountered failures

**If any criterion is not met: Do NOT expand.**

---

## What Success Looks Like

After 6 months:

**Not**:
- ❌ "VCR is fast"
- ❌ "VCR has many features"
- ❌ "VCR is widely deployed"

**Instead**:
- ✅ "VCR never lied"
- ✅ "VCR never lost data"
- ✅ "VCR always gives same answer"
- ✅ "We trust VCR's output"

---

## Summary

**Observation phase is about**:
- Proving reliability (not adding features)
- Learning operational behavior (not optimizing)
- Building trust (not expanding reach)

**Acceptable changes**: Visibility improvements only  
**Forbidden changes**: Everything else

**The kernel is frozen. Stay disciplined.**
