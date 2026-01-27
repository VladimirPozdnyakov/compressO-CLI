use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Progress tracking metrics for video compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMetrics {
    /// Time when compression started
    #[serde(skip)]
    pub start_time: Option<Instant>,
    /// Total elapsed time since start
    pub elapsed_time: Duration,
    /// Total duration of video being compressed (in seconds)
    pub total_duration: Option<f64>,
    /// Original file size in bytes
    pub original_size: u64,
    /// Current compression progress (0.0 to 100.0)
    pub current_progress: f64,
}

impl ProgressMetrics {
    /// Create a new ProgressMetrics instance
    pub fn new(original_size: u64, total_duration: Option<f64>) -> Self {
        Self {
            start_time: Some(Instant::now()),
            elapsed_time: Duration::from_secs(0),
            total_duration,
            original_size,
            current_progress: 0.0,
        }
    }

    /// Update elapsed time based on start time
    pub fn update_elapsed(&mut self) {
        if let Some(start) = self.start_time {
            self.elapsed_time = start.elapsed();
        }
    }

    /// Calculate current processing speed in bytes per second
    pub fn calculate_speed(&self) -> f64 {
        let elapsed_secs = self.elapsed_time.as_secs_f64();
        if elapsed_secs > 0.0 && self.current_progress > 0.0 {
            let bytes_processed = (self.original_size as f64 * self.current_progress) / 100.0;
            bytes_processed / elapsed_secs
        } else {
            0.0
        }
    }

    /// Calculate estimated time remaining in seconds
    pub fn calculate_eta(&self) -> Option<f64> {
        if self.current_progress <= 0.0 || self.current_progress >= 100.0 {
            return None;
        }

        let speed = self.calculate_speed();
        if speed <= 0.0 {
            return None;
        }

        let remaining_bytes = self.original_size as f64 * (100.0 - self.current_progress) / 100.0;
        Some(remaining_bytes / speed)
    }

    /// Update current progress percentage
    pub fn update_progress(&mut self, progress: f64) {
        self.current_progress = progress.clamp(0.0, 100.0);
        self.update_elapsed();
    }
}

impl Default for ProgressMetrics {
    fn default() -> Self {
        Self {
            start_time: None,
            elapsed_time: Duration::from_secs(0),
            total_duration: None,
            original_size: 0,
            current_progress: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_progress_metrics() {
        let metrics = ProgressMetrics::new(1000000, Some(60.0));
        assert_eq!(metrics.original_size, 1000000);
        assert_eq!(metrics.total_duration, Some(60.0));
        assert_eq!(metrics.current_progress, 0.0);
        assert!(metrics.start_time.is_some());
    }

    #[test]
    fn test_update_progress() {
        let mut metrics = ProgressMetrics::new(1000000, Some(60.0));
        metrics.update_progress(50.0);
        assert_eq!(metrics.current_progress, 50.0);
    }

    #[test]
    fn test_progress_clamping() {
        let mut metrics = ProgressMetrics::new(1000000, Some(60.0));
        metrics.update_progress(150.0);
        assert_eq!(metrics.current_progress, 100.0);
        metrics.update_progress(-10.0);
        assert_eq!(metrics.current_progress, 0.0);
    }

    #[test]
    fn test_calculate_speed() {
        let mut metrics = ProgressMetrics::new(1000000, Some(60.0));
        thread::sleep(Duration::from_millis(100));
        metrics.update_progress(50.0);
        let speed = metrics.calculate_speed();
        assert!(speed > 0.0);
    }

    #[test]
    fn test_calculate_eta() {
        let mut metrics = ProgressMetrics::new(1000000, Some(60.0));
        thread::sleep(Duration::from_millis(100));
        metrics.update_progress(50.0);
        let eta = metrics.calculate_eta();
        assert!(eta.is_some());
        assert!(eta.unwrap() > 0.0);
    }

    #[test]
    fn test_eta_at_completion() {
        let mut metrics = ProgressMetrics::new(1000000, Some(60.0));
        metrics.update_progress(100.0);
        let eta = metrics.calculate_eta();
        assert!(eta.is_none());
    }

    #[test]
    fn test_eta_at_start() {
        let metrics = ProgressMetrics::new(1000000, Some(60.0));
        let eta = metrics.calculate_eta();
        assert!(eta.is_none());
    }
}
