use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

/// Progress information for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub operation_id: String,
    pub phase: String,
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
}

/// Progress tracker for download/extraction operations
pub struct ProgressTracker {
    operation_id: String,
    total: Arc<AtomicU64>,
    current: Arc<AtomicU64>,
    start_time: Instant,
    last_update: Arc<std::sync::Mutex<Instant>>,
    cancelled: Arc<AtomicBool>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(operation_id: String, total: u64) -> Self {
        Self {
            operation_id,
            total: Arc::new(AtomicU64::new(total)),
            current: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            last_update: Arc::new(std::sync::Mutex::new(Instant::now())),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Update progress
    pub fn update(&self, current: u64) {
        self.current.store(current, Ordering::Relaxed);
        *self.last_update.lock().unwrap() = Instant::now();
    }

    /// Increment progress
    pub fn increment(&self, amount: u64) {
        self.current.fetch_add(amount, Ordering::Relaxed);
        *self.last_update.lock().unwrap() = Instant::now();
    }

    /// Get current progress info
    pub fn get_info(&self, phase: &str) -> ProgressInfo {
        let current = self.current.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        
        let percentage = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.start_time.elapsed().as_secs();
        let speed_bps = if elapsed > 0 {
            current / elapsed
        } else {
            0
        };

        let eta_seconds = if speed_bps > 0 && total > current {
            (total - current) / speed_bps
        } else {
            0
        };

        ProgressInfo {
            operation_id: self.operation_id.clone(),
            phase: phase.to_string(),
            current,
            total,
            percentage,
            speed_bps,
            eta_seconds,
        }
    }

    /// Check if operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Cancel the operation
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Get operation ID
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Set total
    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    /// Get current value
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    /// Get total value
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }
}

impl Clone for ProgressTracker {
    fn clone(&self) -> Self {
        Self {
            operation_id: self.operation_id.clone(),
            total: Arc::clone(&self.total),
            current: Arc::clone(&self.current),
            start_time: self.start_time,
            last_update: Arc::clone(&self.last_update),
            cancelled: Arc::clone(&self.cancelled),
        }
    }
}

/// Format duration to human-readable string
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let secs = seconds % 60;
        format!("{}m {}s", minutes, secs)
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    }
}

/// Format speed to human-readable string
pub fn format_speed(bytes_per_second: u64) -> String {
    const UNITS: &[&str] = &["B/s", "KB/s", "MB/s", "GB/s"];
    
    if bytes_per_second == 0 {
        return "0 B/s".to_string();
    }
    
    let mut speed = bytes_per_second as f64;
    let mut unit_index = 0;
    
    while speed >= 1024.0 && unit_index < UNITS.len() - 1 {
        speed /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", speed as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", speed, UNITS[unit_index])
    }
}
