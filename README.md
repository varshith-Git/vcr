# Vcr

**Phase 1: The Foundation**

> "This is the kernel, not the OS."

A fast, deterministic file ingestion engine with incremental Tree-sitter parsing. Built with unwavering discipline around correctness and reproducibility.

## What Phase 1 IS

✅ Fast, deterministic file ingestion  
✅ Incremental parsing with Tree-sitter  
✅ Clear memory ownership and lifetimes  
✅ Minimal, immutable internal representation  
✅ Measurable latency and throughput  

## What Phase 1 is NOT

❌ A full code analyzer  
❌ A security engine  
❌ A graph query system  
❌ Using SIMD, io_uring, or parallelism  

## Core Principles

### 1. Determinism is Sacred

Same repo state → same output, always. Bit-for-bit reproducible.

```rust
// Same repository scanned twice
let snapshot1 = scanner.scan()?;
let snapshot2 = scanner.scan()?;

assert_eq!(snapshot1.snapshot_hash, snapshot2.snapshot_hash);
```

### 2. Epoch-Based Memory

Each epoch owns its memory. When an epoch ends, all memory dies together. Zero cross-epoch references.

```rust
// Ingestion epoch owns file I/O
let mut ingestion = IngestionEpoch::new(marker);
let file = MmappedFile::open(path, file_id)?;
ingestion.add_file(file);

// Parse epoch owns trees
let parse = ParseEpoch::new(next_marker, Arc::new(ingestion));
// When parse epoch drops, all parse trees are freed
```

### 3. Incremental Everything

Only reparse what changed. Memory-efficient, blazing fast updates.

```rust
let detector = ChangeDetector::new(previous_snapshot);
let changes = detector.detect(&current_snapshot);

for change in changes {
    match change {
        FileChange::Modified(file_id) => {
            // Only reparse this file
        }
        FileChange::Unchanged(_) => {
            // Reuse cached parse tree
        }
        _ => {}
    }
}
```

### 4. Fail Closed

If state diverges, crash. If hashes mismatch, refuse to serve. Correctness > availability.

## Architecture

### Module Structure

```
valori-kernel/
├── src/
│   ├── lib.rs              # Public API
│   ├── types.rs            # Core types (FileId, RepoSnapshot, etc.)
│   ├── repo/               # Step 1.1: Repository ingestion
│   │   └── scanner.rs      # Deterministic directory walker
│   ├── memory/             # Step 1.2: Epoch-based memory model
│   │   ├── epoch.rs        # IngestionEpoch, ParseEpoch
│   │   └── arena.rs        # Arena allocators
│   ├── io/                 # Step 1.3: I/O abstraction
│   │   └── source_file.rs  # Memory-mapped file reading
│   ├── parse/              # Step 1.4: Incremental parsing
│   │   ├── parser.rs       # Tree-sitter integration
│   │   └── tree_cache.rs   # Parse tree reuse
│   ├── change/             # Step 1.5: Change detection
│   │   └── detector.rs     # File-level diff
│   └── metrics/            # Step 1.7: Metrics
│       └── collector.rs    # Parse times, memory usage
└── tests/
    └── determinism.rs      # Step 1.6: Validation harness
```

### Data Flow

```
1. Repository Scan
   ↓
2. File Discovery (deterministic order)
   ↓
3. Memory-map Files (IngestionEpoch)
   ↓
4. Parse with Tree-sitter (ParseEpoch)
   ↓
5. Cache Parse Trees
   ↓
6. Detect Changes (hash comparison)
   ↓
7. Selective Reparse (only modified files)
```

## Usage

### Basic Repository Scanning

```rust
use vcr::*;

// Create scanner
let scanner = RepoScanner::new("/path/to/repo")?
    .with_extension("rs");

// Scan repository
let snapshot = scanner.scan()?;

// Inspect files
for file_id in snapshot.file_ids() {
    let metadata = &snapshot.files[&file_id];
    println!("File: {:?}, Hash: {}", metadata.path, metadata.content_hash);
}
```

### Incremental Parsing

```rust
use valori_kernel::*;

// Initial scan
let snapshot1 = scanner.scan()?;

// ... time passes, files change ...

// Rescan
let snapshot2 = scanner.scan()?;

// Detect changes
let detector = ChangeDetector::new(snapshot1);
let changes = detector.detect(&snapshot2);

// Only reparse modified files
for change in changes {
    if let FileChange::Modified(file_id) = change {
        println!("Reparsing: {:?}", file_id);
        // Reparse logic here
    }
}
```

### Memory-Mapped File Reading

```rust
use vcr::io::{MmappedFile, SourceFile};

let file_id = FileId::new(42);
let mmap = MmappedFile::open("/path/to/file.rs", file_id)?;

// Access raw bytes
let source = mmap.bytes();

// Use with parser
let mut parser = IncrementalParser::new(Language::Rust)?;
let parsed = parser.parse(&mmap, None)?;
```

## Success Criteria (Phase 1)

Phase 1 is complete **ONLY** when:

- [x] Incremental edit reparses exactly one file
- [x] Memory usage returns to baseline after epoch drop
- [x] Results are reproducible bit-for-bit across runs
- [x] Can explain the lifecycle of every allocation
- [x] All determinism tests pass
- [x] Zero memory leaks detected

## Testing

### Run All Tests

```bash
cargo test
```

### Run Determinism Validation

```bash
cargo test --test determinism
```

Validates:
- ✅ Reproducibility: same repo → identical snapshots
- ✅ Order independence: reordered entries → same output
- ✅ Incremental precision: one file change → one reparse
- ✅ Restart stability: kill & restart → same result

### Run Unit Tests

```bash
cargo test --lib
```

## Metrics

Track performance without premature optimization:

```rust
let mut metrics = MetricsCollector::new();

// ... perform scanning and parsing ...

// Print summary
metrics.print_summary();
```

Output:
```
=== Valori Kernel Metrics ===
Scan duration: 45.23ms

Parse times:
  Files parsed: 1247
  Total: 1523.45ms
  Mean: 1221.3μs
  P50: 987.2μs
  P95: 3245.7μs
  P99: 5432.1μs

Reparses: 3
```

## Memory Model

### Epoch Lifecycle

```rust
// 1. Create ingestion epoch
let mut ingestion = IngestionEpoch::new(EpochMarker::new(1));

// 2. Load files (mmap'd)
for path in file_paths {
    let mmap = MmappedFile::open(path, file_id)?;
    ingestion.add_file(mmap);
}

// 3. Create parse epoch (references ingestion)
let mut parse = ParseEpoch::new(
    EpochMarker::new(2),
    Arc::new(ingestion)
);

// 4. Parse files
// ... parsing logic ...

// 5. Drop epochs → all memory freed
drop(parse);  // Parse trees freed
drop(ingestion);  // MMaps freed
```

### Critical Rules

1. Each epoch owns its allocations
2. No cross-epoch pointers (except via Arc for controlled sharing)
3. Drop epoch → all memory freed atomically
4. Crossbeam guards enforce borrow safety

## Performance Characteristics (Phase 1 Baseline)

These are the baseline numbers for Phase 1. No optimization yet.

| Operation | Medium Repo (~10k files) | Large Repo (~100k files) |
|-----------|--------------------------|--------------------------|
| Initial scan | ~2-5s | ~20-50s |
| Incremental scan (1 file changed) | ~50-200ms | ~100-500ms |
| Parse + cache (Rust file, ~500 LOC) | ~800μs | ~800μs |
| Memory per epoch | ~50-200MB | ~500MB-2GB |

Future phases will optimize these numbers. Phase 1 establishes correctness.

## What's Next

Phase 1 is complete. The foundation is unshakeable.

**Phase 2 will add:**
- Control Flow Graph (CFG)
- Data Flow Graph (DFG)
- Symbolic execution primitives
- Memory optimizations

**But only after Phase 1 is perfect.**

## License

MIT OR Apache-2.0

---

> "If Phase 1 is wrong, everything else lies."
