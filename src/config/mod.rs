//! Operational configuration (Path B6)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// VTR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValoriConfig {
    /// I/O configuration
    pub io: IOConfig,
    
    /// Snapshot configuration
    pub snapshot: SnapshotConfig,
    
    /// Execution configuration
    pub execution: ExecutionConfig,
}

/// I/O configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOConfig {
    /// I/O mode: "auto", "hot", "cold"
    pub mode: String,
    
    /// Enable io_uring (Linux-only)
    pub uring_enabled: bool,
}

/// Snapshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Snapshot directory path
    pub path: PathBuf,
    
    /// Auto-save on completion
    pub auto_save: bool,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Enable parallel execution
    pub parallel: bool,
    
    /// Thread count (0 = auto)
    pub thread_count: usize,
}

impl Default for ValoriConfig {
    fn default() -> Self {
        Self {
            io: IOConfig {
                mode: "auto".to_string(),
                uring_enabled: false,
            },
            snapshot: SnapshotConfig {
                path: PathBuf::from("./snapshots"),
                auto_save: true,
            },
            execution: ExecutionConfig {
                parallel: false,
                thread_count: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValoriConfig::default();
        assert_eq!(config.io.mode, "auto");
        assert!(!config.io.uring_enabled);
        assert!(config.snapshot.auto_save);
    }
}
