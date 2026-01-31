//! VTR CLI - wiring, not product
//!
//! Zero magic. Explicit config. Machine-readable output.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;
use std::fs;

/// Load config from file or use defaults
fn load_config(config_path: Option<PathBuf>) -> vcr::config::ValoriConfig {
    if let Some(path) = config_path {
        // Load from specified path
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| {
                eprintln!("{{\"status\":\"error\",\"message\":\"Failed to read config: {}\",\"fatal\":true}}", e);
                process::exit(1);
            });
        
        toml::from_str(&content)
            .unwrap_or_else(|e| {
                eprintln!("{{\"status\":\"error\",\"message\":\"Failed to parse config: {}\",\"fatal\":true}}", e);
                process::exit(1);
            })
    } else if PathBuf::from("./vtr.toml").exists() {
        // Try default location
        let content = fs::read_to_string("./vtr.toml").unwrap();
        toml::from_str(&content).unwrap_or_default()
    } else {
        // Use built-in defaults
        vcr::config::ValoriConfig::default()
    }
}

#[derive(Parser)]
#[command(name = "vcr")]
#[command(about = "Valori Code Replay - deterministic code analysis")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest repository and build CPG
    Ingest {
        /// Path to repository or file
        path: PathBuf,
        
        /// Config file (default: ./vtr.toml)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// Snapshot operations
    Snapshot {
        #[command(subcommand)]
        operation: SnapshotOp,
    },
    
    /// Run query on CPG
    Query {
        /// Path to query file (JSON)
        query_file: PathBuf,
    },
    
    /// Explain result provenance
    Explain {
        /// Result ID to explain
        result_id: String,
    },
}

#[derive(Subcommand)]
enum SnapshotOp {
    /// Save current CPG snapshot
    Save,
    
    /// Load CPG snapshot
    Load {
        /// Snapshot ID or path
        id: String,
    },
    
    /// Verify snapshot integrity
    Verify {
        /// Snapshot path
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Ingest { path, config } => cmd_ingest(path, config),
        Commands::Snapshot { operation } => match operation {
            SnapshotOp::Save => cmd_snapshot_save(),
            SnapshotOp::Load { id } => cmd_snapshot_load(id),
            SnapshotOp::Verify { path } => cmd_snapshot_verify(path),
        },
        Commands::Query { query_file } => cmd_query(query_file),
        Commands::Explain { result_id } => cmd_explain(result_id),
    };
    
    match result {
        Ok(output) => {
            println!("{}", output);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{{\"status\":\"error\",\"message\":\"{}\",\"fatal\":true}}", e);
            process::exit(1);
        }
    }
}

fn cmd_ingest(path: PathBuf, config: Option<PathBuf>) -> Result<String, String> {
    use vcr::parse::IncrementalParser;
    use vcr::types::{Language, FileId};
    use vcr::io::MmappedFile;
    
    let _config = load_config(config);
    
    // For now: simple single-file ingestion
    // Full repo traversal would go here
    
    if !path.exists() {
        return Err(format!("Path not found: {}", path.display()));
    }
    
    if path.is_file() {
        // Single file ingestion
        let file_id = FileId::new(1);
        let mmap = MmappedFile::open(&path, file_id)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut parser = IncrementalParser::new(Language::Rust)
            .map_err(|e| format!("Failed to create parser: {}", e))?;
        
        let parsed = parser.parse(&mmap, None)
            .map_err(|e| format!("Parse failed: {}", e))?;
        
        // Build CPG (simplified - full pipeline would include semantic analysis)
        let cpg = vcr::cpg::model::CPG::new();
        let hash = cpg.compute_hash();
        
        Ok(format!("{{\"status\":\"success\",\"epoch_id\":1,\"cpg_hash\":\"{}\",\"nodes\":{}}}", 
            hash, parsed.tree.root_node().child_count()))
    } else {
        Err("Directory ingestion not yet implemented - TODO".to_string())
    }
}

fn cmd_snapshot_save() -> Result<String, String> {
    use vcr::storage::CPGSnapshot;
    use vcr::cpg::model::CPG;
    use std::path::PathBuf;
    
    // For now: save empty CPG as demo
    // Full implementation would get current CPG from global state
    let cpg = CPG::new();
    
    let temp_path = PathBuf::from("/tmp/vcr-snapshot-demo.bin");
    
    let snapshot_id = CPGSnapshot::save(&cpg, &temp_path)
        .map_err(|e| format!("Snapshot save failed: {}", e))?;
    
    let hash = cpg.compute_hash();
    
    Ok(format!("{{\"status\":\"success\",\"snapshot_id\":{},\"hash\":\"{}\"}}", 
        snapshot_id.0, hash))
}

fn cmd_snapshot_load(id: String) -> Result<String, String> {
    use vcr::storage::CPGSnapshot;
    use std::path::Path;
    
    // Load from path (id is treated as path for now)
    let path = Path::new(&id);
    
    if !path.exists() {
        return Err(format!("Snapshot not found: {}", id));
    }
    
    // Verify first
    let hash = CPGSnapshot::verify(path)
        .map_err(|e| format!("Snapshot verification failed: {}", e))?;
    
    // Load
    let _cpg = CPGSnapshot::load(path)
        .map_err(|e| format!("Snapshot load failed: {}", e))?;
    
    Ok(format!("{{\"status\":\"success\",\"hash\":\"{}\",\"verified\":true}}", hash))
}

fn cmd_snapshot_verify(path: PathBuf) -> Result<String, String> {
    // TODO: Wire to CPGSnapshot::verify
    use vcr::storage::CPGSnapshot;
    
    match CPGSnapshot::verify(&path) {
        Ok(hash) => Ok(format!("{{\"status\":\"success\",\"hash\":\"{}\",\"valid\":true}}", hash)),
        Err(e) => Err(format!("Snapshot verification failed: {}", e)),
    }
}

fn cmd_query(query_file: PathBuf) -> Result<String, String> {
    use vcr::cpg::model::CPG;
    use vcr::query::primitives::QueryPrimitives;
    use vcr::cpg::model::CPGNodeKind;
    
    // For now: simple hardcoded query demo
    // Full implementation would parse query file (JSON DSL)
    
    if !query_file.exists() {
        return Err(format!("Query file not found: {}", query_file.display()));
    }
    
    // Demo: empty CPG, find all functions
    let cpg = CPG::new();
    let results = QueryPrimitives::find_nodes(&cpg, CPGNodeKind::Function);
    
    Ok(format!("{{\"status\":\"success\",\"query\":\"{}\",\"results\":[],\"count\":{}}}", 
        query_file.display(), results.len()))
}

fn cmd_explain(result_id: String) -> Result<String, String> {
    // Deterministic provenance trace
    // For now: placeholder implementation
    // Full version would:
    // 1. Load result metadata from store
    // 2. Trace back through CPG to origin nodes
    // 3. Output complete provenance chain
    
    Ok(format!("{{\"status\":\"success\",\"result_id\":\"{}\",\"provenance\":[\"TODO: trace origin\"]}}", 
        result_id))
}
