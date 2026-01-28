# VTR Narrow Deployment Strategy

**Goal**: Validate operational behavior in production. Not adoption.

---

## Selection Criteria

Pick ONE environment that is:

✅ **Predictable load** - Same tasks, repeatable  
✅ **Controlled input** - Known codebases  
✅ **Observable behavior** - Clear metrics  
✅ **Boring failure modes** - No novel edge cases  
✅ **Rollback-friendly** - Easy to disable  

❌ **Avoid**:
- Public-facing services
- Variable workloads
- Novel/experimental use cases
- Marketing-driven deployments

---

## Recommended Deployment Options

### Option A: Internal CI Job ⭐ (RECOMMENDED)

**Setup**:
```yaml
# .github/workflows/vtr-analysis.yml
name: VTR Code Analysis

on:
  pull_request:
    branches: [main]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run VTR
        run: |
          cargo install --path path/to/vtr
          vtr ingest src/ > vtr-results.json
          
      - name: Verify determinism
        run: |
          vtr ingest src/ > vtr-results2.json
          diff vtr-results.json vtr-results2.json
          
      - name: Save snapshot
        run: vtr snapshot save
```

**Why this works**:
- Same repo, predictable changes
- Clear success metric (CI pass/fail)
- Easy rollback (disable workflow)
- Observable (CI logs)
- Boring (no novel code patterns)

**Success metrics**:
- Runs complete 100% of the time
- Hash stability across runs
- Zero "weird behavior" reports
- CI time impact < 5%

---

### Option B: Nightly Audit Run

**Setup**:
```bash
#!/bin/bash
# nightly-audit.sh

# Ingest critical repositories
for repo in /repos/auth /repos/payments /repos/core; do
  echo "Analyzing $repo..."
  vtr ingest "$repo" > "audit-$(basename $repo)-$(date +%Y%m%d).json"
  
  # Save snapshot
  vtr snapshot save
  
  # Verify determinism
  vtr ingest "$repo" > "/tmp/verify.json"
  diff "audit-$(basename $repo)-$(date +%Y%m%d).json" "/tmp/verify.json"
done

# Run compliance query
vtr query queries/taint-to-external.json > taint-results.json
```

**Cron**:
```cron
0 2 * * * /scripts/nightly-audit.sh
```

**Why this works**:
- Same repos, nightly cadence
- Compliance-focused (clear value)
- Non-blocking (not in critical path)
- Audit trail (snapshots over time)

**Success metrics**:
- Completes every night
- Snapshot hash chain validates
- Determinism verified each run
- Zero manual interventions

---

### Option C: Pre-Merge Semantic Diff

**Setup**:
```bash
#!/bin/bash
# pre-merge-vtr.sh

# Analyze main branch
git checkout main
vtr ingest src/ > main-cpg.json
MAIN_HASH=$(jq -r '.cpg_hash' main-cpg.json)

# Analyze feature branch
git checkout feature-branch
vtr ingest src/ > feature-cpg.json  
FEATURE_HASH=$(jq -r '.cpg_hash' feature-cpg.json)

# Compare
if [ "$MAIN_HASH" != "$FEATURE_HASH" ]; then
  echo "Semantic change detected"
  # Run diff query
  vtr query queries/cpg-diff.json
fi
```

**Why this works**:
- Clear signal (semantic change or not)
- Developer-facing (immediate feedback)
- Repeatable (same diff, same result)
- Valuable (catches unintended changes)

**Success metrics**:
- Developers trust the signal
- False positive rate < 1%
- Analysis time< 30s
- Zero hash inconsistencies

---

## Deployment Timeline

### Week 1-2: Setup

- Install VTR in target environment
- Create basic config (`vtr.toml`)
- Run manual tests
- Verify determinism locally

### Week 3-4: Shadow Mode

- Run VTR alongside existing tools
- Collect outputs, don't act on them
- Monitor for crashes, hangs, errors
- Validate hash stability

### Week 5-6: Observation

- Enable in one repository only
- Monitor logs daily
- Track success rate
- Document "weird behavior" (target: 0)

### Week 7-8: Evaluation

**Success criteria**:
- ✅ 100% completion rate (no crashes)
- ✅ Hash stability (same input → same hash)
- ✅ Zero manual interventions
- ✅ Determinism verified (parallel = serial)

**If ANY criterion fails**: Stop. Debug. Don't expand.

---

## Monitoring Checklist

Track these metrics:

```bash
# Daily check
- [ ] Did VTR complete successfully?
- [ ] Were hashes consistent?
- [ ] Any error logs?
- [ ] Any manual interventions?

# Weekly check
- [ ] Run determinism validation
- [ ] Compare week-over-week hashes
- [ ] Review any anomalies
- [ ] Update runbook if needed
```

**Alert on**:
- Hash mismatch
- Crash/hang
- Timeout
- Manual restart required

**Do NOT alert on**:
- Performance variance (not a guarantee)
- Benign warnings

---

## What NOT to Do

❌ **Don't rush**:
- Skip shadow mode → immediate production issues
- Expand to multiple repos before validation

❌ **Don't add features**:
- "Let's add this analysis while we're here"
- Feature requests during deployment phase

❌ **Don't tolerate anomalies**:
- "Hash mismatch, but let's continue"
- "Crashed once, but it's fine"

**Any violation of guarantees = stop and fix.**

---

## Runbook Template

```markdown
# VTR Deployment Runbook

## Normal Operation

1. VTR runs automatically (CI/cron/pre-commit)
2. Outputs JSON to configured path
3. Snapshots saved to snapshot_dir
4. Hash logged for verification

## Failure Modes

### Hash Mismatch
- **Symptom**: Same input, different hash
- **Action**: STOP. Do not continue. Debug.
- **Escalation**: Review epoch construction

### Crash
- **Symptom**: VTR exits with error
- **Action**: Check logs, verify input
- **Escalation**: Restore from last snapshot

### Hang/Timeout
- **Symptom**: VTR doesn't complete
- **Action**: Kill process, check for unbounded loop
- **Escalation**: Review analysis bounds

### Snapshot Corruption
- **Symptom**: Load fails verification
- **Action**: Revert to previous snapshot
- **Escalation**: Investigate storage integrity

## Recovery

1. Identify failure mode
2. Restore from last valid snapshot
3. Verify hash chain
4. Resume operation

**Never proceed with corrupted state.**
```

---

## Success Definition

**After 3 months**, VTR should:

✅ Run without manual intervention  
✅ Produce stable hashes  
✅ Handle all input without crashes  
✅ Pass determinism checks 100% of time  

**If yes**: Consider expanding to 2-3 similar environments.

**If no**: Do not expand. Fix issues first.

---

## Expansion Criteria (Post-Validation)

Only expand if ALL of:

1. ✅ 3+ months stable in narrow environment
2. ✅ Zero hash inconsistencies
3. ✅ Zero crashes requiring intervention
4. ✅ Developers trust the output
5. ✅ Runbook covers all encountered failures

**Then** consider:
- Adding one more repository
- Enabling one more analysis
- Extending to similar workflow

**Still avoid**:
- Distributed deployment
- Public-facing service
- Variable/novel workloads

---

## Summary

**Narrow deployment means**:

ONE environment.  
ONE repo (or small set).  
ONE workflow.

**Observe for months, not weeks.**

Let VTR prove itself through boring reliability, not flashy features.
