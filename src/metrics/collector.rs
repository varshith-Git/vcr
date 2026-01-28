//! Metrics collection (Step 1.7)
//!
//! Simple in-memory metrics for parse times, scan duration, memory usage.

use crate::types::{EpochMarker, FileId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Metrics collector.
pub struct MetricsCollector {
    /// Parse times per file (in microseconds)
    parse_times: HashMap<FileId, u64>,
    
    /// Total scan duration
    scan_duration: Option<Duration>,
    
    /// Memory usage per epoch (in bytes)
    epoch_memory: HashMap<EpochMarker, usize>,
    
    /// Count of reparsed files
    reparse_count: AtomicUsize,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            parse_times: HashMap::new(),
            scan_duration: None,
            epoch_memory: HashMap::new(),
            reparse_count: AtomicUsize::new(0),
        }
    }

    /// Record a parse time.
    pub fn record_parse_time(&mut self, file_id: FileId, duration_us: u64) {
        self.parse_times.insert(file_id, duration_us);
    }

    /// Record scan duration.
    pub fn record_scan_duration(&mut self, duration: Duration) {
        self.scan_duration = Some(duration);
    }

    /// Record epoch memory usage.
    pub fn record_epoch_memory(&mut self, epoch: EpochMarker, bytes: usize) {
        self.epoch_memory.insert(epoch, bytes);
    }

    /// Increment reparse counter.
    pub fn increment_reparse(&self) {
        self.reparse_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get parse time statistics.
    pub fn parse_time_stats(&self) -> ParseTimeStats {
        let mut times: Vec<u64> = self.parse_times.values().copied().collect();
        
        if times.is_empty() {
            return ParseTimeStats::default();
        }

        times.sort_unstable();
        
        let count = times.len();
        let total: u64 = times.iter().sum();
        let mean = total / count as u64;
        
        let p50 = times[count / 2];
        let p95 = times[(count * 95) / 100];
        let p99 = times[(count * 99) / 100];

        ParseTimeStats {
            count,
            total_us: total,
            mean_us: mean,
            p50_us: p50,
            p95_us: p95,
            p99_us: p99,
        }
    }

    /// Get scan duration.
    pub fn scan_duration(&self) -> Option<Duration> {
        self.scan_duration
    }

    /// Get reparse count.
    pub fn reparse_count(&self) -> usize {
        self.reparse_count.load(Ordering::Relaxed)
    }

    /// Get total epoch memory.
    pub fn total_epoch_memory(&self) -> usize {
        self.epoch_memory.values().sum()
    }

    /// Print a summary report.
    pub fn print_summary(&self) {
        println!("=== Valori Kernel Metrics ===");
        
        if let Some(duration) = self.scan_duration {
            println!("Scan duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
        }

        let stats = self.parse_time_stats();
        if stats.count > 0 {
            println!("\nParse times:");
            println!("  Files parsed: {}", stats.count);
            println!("  Total: {:.2}ms", stats.total_us as f64 / 1000.0);
            println!("  Mean: {:.2}μs", stats.mean_us);
            println!("  P50: {:.2}μs", stats.p50_us);
            println!("  P95: {:.2}μs", stats.p95_us);
            println!("  P99: {:.2}μs", stats.p99_us);
        }

        let reparse_count = self.reparse_count();
        if reparse_count > 0 {
            println!("\nReparses: {}", reparse_count);
        }

        let total_memory = self.total_epoch_memory();
        if total_memory > 0 {
            println!("\nTotal epoch memory: {} bytes", total_memory);
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse time statistics.
#[derive(Debug, Default)]
pub struct ParseTimeStats {
    /// Number of files parsed
    pub count: usize,
    
    /// Total parse time (microseconds)
    pub total_us: u64,
    
    /// Mean parse time (microseconds)
    pub mean_us: u64,
    
    /// P50 latency (microseconds)
    pub p50_us: u64,
    
    /// P95 latency (microseconds)
    pub p95_us: u64,
    
    /// P99 latency (microseconds)
    pub p99_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = MetricsCollector::new();
        
        collector.record_parse_time(FileId::new(1), 100);
        collector.record_parse_time(FileId::new(2), 200);
        collector.record_parse_time(FileId::new(3), 300);

        let stats = collector.parse_time_stats();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.total_us, 600);
        assert_eq!(stats.mean_us, 200);
    }

    #[test]
    fn test_reparse_counter() {
        let collector = MetricsCollector::new();
        
        collector.increment_reparse();
        collector.increment_reparse();
        
        assert_eq!(collector.reparse_count(), 2);
    }
}
