# VTR CI Integration Guide - Option A

**Target**: VCR repository (self-analysis)  
**Mode**: Shadow (observational, non-blocking)  
**Timeline**: 8 weeks

---

## Phase 1: Setup (Week 1-2)

### 1. Build Release Binary

```bash
cd /Users/as-mac-0272/Desktop/sass/vcr

# Build optimized binary
cargo build --release --bin vtr

# Binary location
ls -lh target/release/vtr
```

**Size**: ~5-10 MB  
**Dependencies**: None (statically linked)

---

### 2. Create GitHub Actions Workflow

Create `.github/workflows/vtr-analysis.yml`:

```yaml
name: VTR Analysis (Observational)

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  vtr-analysis:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v3
      
      - name: Setup Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable
      
      - name: Build VTR
        run: cargo build --release --bin vtr
      
      - name: Run VTR ingestion
        id: ingest
        run: |
          ./target/release/vtr ingest src/ > vtr-results.json
          cat vtr-results.json
        continue-on-error: true
      
      - name: Verify determinism
        id: verify
        run: |
          ./target/release/vtr ingest src/ > vtr-verify.json
          diff vtr-results.json vtr-verify.json && echo "✓ Determinism verified" || echo "✗ Hash mismatch!"
        continue-on-error: true
      
      - name: Save snapshot
        run: |
          ./target/release/vtr snapshot save > snapshot-info.json
          cat snapshot-info.json
        continue-on-error: true
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: vtr-analysis-results
          path: |
            vtr-results.json
            vtr-verify.json
            snapshot-info.json
          retention-days: 30
      
      - name: Report status
        run: |
          echo "## VTR Analysis Results" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "**Ingestion**: $(jq -r '.status' vtr-results.json)" >> $GITHUB_STEP_SUMMARY
          echo "**CPG Hash**: $(jq -r '.cpg_hash' vtr-results.json)" >> $GITHUB_STEP_SUMMARY
          echo "**Nodes**: $(jq -r '.nodes' vtr-results.json)" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "**Determinism**: $(diff -q vtr-results.json vtr-verify.json && echo '✓ Verified' || echo '✗ Failed')" >> $GITHUB_STEP_SUMMARY
```

**Key points**:
- `continue-on-error: true` - Never blocks CI
- Runs on every PR and push to main
- Uploads results as artifacts
- Verifies determinism on every run

---

### 3. Local Testing First

Before pushing to CI, test locally:

```bash
# Test ingestion
./target/release/vtr ingest src/ > local-test.json
cat local-test.json

# Verify determinism
./target/release/vtr ingest src/ > local-verify.json
diff local-test.json local-verify.json

# Expected output: empty diff (files identical)
```

**Success criteria**:
- JSON output is valid
- Hash is consistent across runs
- No crashes

---

## Phase 2: Shadow Mode (Week 3-4)

### Monitor These Metrics

Create a simple tracking spreadsheet or document:

```markdown
## VTR CI Monitoring Log

### Week 3

| Date | PR# | Status | Hash | Determinism | Notes |
|------|-----|--------|------|-------------|-------|
| 2026-02-01 | #42 | ✓ | abc123... | ✓ | First run |
| 2026-02-02 | #43 | ✓ | def456... | ✓ | - |
| 2026-02-03 | main | ✓ | abc123... | ✓ | Same as PR#42 (expected) |

### Issues
- None

### Week 4
...
```

**What to track**:
- ✅ Completion rate (target: 100%)
- ✅ Hash consistency (same code → same hash)
- ✅ Determinism verification (always pass)
- ✅ CI time impact (should be < 2 minutes)

**Red flags**:
- ❌ Hash mismatch on identical code
- ❌ Crash or hang
- ❌ Non-deterministic output

---

## Phase 3: Observation (Week 5-6)

### Daily Checks

```bash
# Check recent CI runs
gh run list --workflow=vtr-analysis.yml --limit 5

# Download recent results
gh run download [run-id] -n vtr-analysis-results

# Verify hash stability
cat vtr-results.json | jq -r '.cpg_hash'
```

### Weekly Validation

```bash
# Clone fresh, run twice
git clone [repo] /tmp/vtr-test
cd /tmp/vtr-test
./target/release/vtr ingest src/ > run1.json
./target/release/vtr ingest src/ > run2.json

# Must be identical
diff run1.json run2.json
```

**Success**: Empty diff every time

---

## Phase 4: Evaluation (Week 7-8)

### Review Checklist

```markdown
## 8-Week CI Integration Review

### Stability ✓/✗
- [ ] 100% completion rate
- [ ] Zero hash inconsistencies  
- [ ] Zero crashes
- [ ] Zero manual interventions

### Integration ✓/✗
- [ ] CI completes within acceptable time
- [ ] No false positives blocking developers
- [ ] Artifacts successfully uploaded
- [ ] Determinism verified on every run

### Observations
- What did we learn?
- Any unexpected behavior?
- Performance impact?

### Decision
- [ ] Continue observation (extend 3 months)
- [ ] Expand to second repo
- [ ] Stop and debug
```

---

## Monitoring Script

Create `scripts/check-vtr-ci.sh`:

```bash
#!/bin/bash
# Quick CI health check

echo "=== VTR CI Health Check ==="
echo

# Get last 10 runs
echo "Recent runs:"
gh run list --workflow=vtr-analysis.yml --limit 10 --json conclusion,databaseId,createdAt | \
  jq -r '.[] | "\(.databaseId): \(.conclusion) (\(.createdAt))"'

echo
echo "Checking for failures..."
FAILURES=$(gh run list --workflow=vtr-analysis.yml --limit 100 --json conclusion | \
  jq '[.[] | select(.conclusion == "failure")] | length')

echo "Failures in last 100 runs: $FAILURES"

if [ "$FAILURES" -eq "0" ]; then
  echo "✓ All runs successful"
else
  echo "✗ Check failed runs"
fi
```

**Run weekly**: `./scripts/check-vtr-ci.sh`

---

## What Success Looks Like

**After 8 weeks**:

✅ VTR runs on every PR without intervention  
✅ Same code always produces same hash  
✅ Determinism verified 100% of time  
✅ Zero developer complaints ("it's blocking me")  
✅ CI time impact negligible (< 5%)

**Then**: Extend observation to 3 months before considering expansion.

---

## Troubleshooting

### Hash Inconsistency

```bash
# Symptom: diff shows files differ
diff vtr-results.json vtr-verify.json

# Debug
1. Check if input changed (git diff)
2. Check cargo version (rustc --version)
3. Check for non-deterministic dependencies
4. Review recent changes to VTR

# Resolution: STOP. Debug before continuing.
```

### CI Timeout

```bash
# If VTR hangs
# Add timeout to workflow:

- name: Run VTR (with timeout)
  timeout-minutes: 5
  run: ./target/release/vtr ingest src/
```

### Too Slow

```bash
# If VTR takes > 5 minutes

# Option 1: Analyze subset
vtr ingest src/core/  # Just core module

# Option 2: Enable parallel (only if determinism verified)
cargo build --release --features parallel-execution
```

---

## Next Steps After CI Success

**If 8-week trial succeeds**:

1. **Extend to 3 months** (same repo, same workflow)
2. **Document lessons learned** in runbook
3. **Share results** with team (builds trust)

**After 3+ months stable**:

4. **Consider**: Add one more repo (similar codebase)
5. **Still avoid**: New analyses, optimizations, expansion beyond 2-3 repos

---

## Summary

**Setup**: 1 week (build, workflow, local test)  
**Shadow**: 2 weeks (monitor, don't act)  
**Observe**: 2 weeks (daily checks, weekly validation)  
**Evaluate**: 2 weeks (review, decide)  
**Then**: 3+ months stable operation before expansion

**Key principle**: Boring reliability over flashy features.
