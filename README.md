# VCR: Valori Code Replay

**Deterministic Code Analysis Infrastructure.**

[![CI Status](https://github.com/varshith-Git/vcr/actions/workflows/vtr-analysis.yml/badge.svg)](https://github.com/varshith-Git/vcr/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![v0.1.0](https://img.shields.io/badge/release-v0.1.0-green)](https://github.com/varshith-Git/vcr/releases/tag/v0.1.0)

> "If VCR says something, we can prove why — forever."

---

## 🏗 What is VCR?

VCR is an **audit-grade** code analysis kernel designed for environments where "probably correct" is not enough.

Unlike traditional linters or LSPs that prioritize speed and heuristics, VCR prioritizes **Determinism** and **Provenance**. It builds a queryable **Code Property Graph (CPG)** that connects control flow, data flow, and syntax into a single, hash-verified structure.

### The Guarantee
| Property | VCR Guarantee |
|----------|---------------|
| **Determinism** | Same input → Exact same hash (always) |
| **Stability** | Parallel execution = Serial results |
| **Safety** | Fail-closed (crashes on corruption) |
| **Auditability** | Snapshots are replayable forever |

---

## ⚡ Quick Start

### Installation

```bash
# From source (v0.1.0)
cargo install --path . --bin vcr
```

### Usage

1. **Ingest Code**: Build the deterministic graph.
   ```bash
   vcr ingest ./src > graph.json
   # Output includes canonical SHA-256 hash of the analysis
   ```

2. **Save Snapshot**: Persist the state for later replay.
   ```bash
   vcr snapshot save
   # Saves to ./vtr-snapshots/ with hash verification
   ```

3. **Verify Integrity**: Prove nothing changed.
   ```bash
   vcr snapshot verify ./vtr-snapshots/snap-1.bin
   ```

---

## 📐 Architecture

VCR is built on a **Kernel/User** model:

### 1. The Kernel (Core)
- **Epoch-Based Memory**: Zero garbage collection pauses, deterministic cleanup.
- **Incremental Parser**: Reuses Tree-sitter trees for unchanged files.
- **Semantic Graph**: Unified AST + CFG + DFG.
- **Taint Analysis**: Bounded, path-sensitive data flow tracking.

### 2. The User (CLI)
- **Minimal Wiring**: The `vcr` binary is a thin wrapper around the kernel.
- **JSON-First**: Output is machine-readable for CI/CD integration.
- **Explicit Config**: No magic defaults. You control `vcr.toml`.

---

## 🔮 Roadmap & Features

### Open Core (Free / Open Source)
*Infrastructure for local truth.*
- ✅ Deterministic Ingestion
- ✅ CPG Construction (AST/CFG/DFG)
- ✅ Local Snapshots & Replay
- ✅ Local Taint Analysis
- ✅ CLI & JSON Output

### Enterprise (Premium / Cloud)
*Infrastructure for team trust.*
- 🔒 **Valori Kernel Integration**: Vector-based semantic memory.
- ☁️ **Long-Term Storage**: Hosted snapshot retention.
- 👥 **Team RBAC**: Access control for sensitive repos.
- 📜 **Compliance Reports**: Automated audit PDFs.
- 🔗 **Cross-Repo Analysis**: Supply chain graphing.

---

## 🛡 Security & Determinism

VCR fails closed. If a hash mismatches, if a bit flips, or if an analysis path is ambiguous, VCR will **crash** rather than return a potentially incorrect result.

See [Why VCR is Deterministic](docs/WHY_DETERMINISTIC.md) for the technical proof.

---

## 🤝 Contributing

The kernel is currently **FROZEN** for the Observation Phase (v0.1.0).
We accept PRs for:
- Logging & Visibility
- Documentation
- Error Message Clarity

We do **not** accept PRs for:
- New Analyses
- Heuristic Optimizations
- Feature Additions

See [Observation Phase Guidelines](docs/OBSERVATION_PHASE.md).

---

*(c) 2026 Valori Systems. Trust through Restraint.*
