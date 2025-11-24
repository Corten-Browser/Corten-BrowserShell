//! Performance monitoring infrastructure for the Browser Shell.
//!
//! This module provides tools for tracking and monitoring performance metrics,
//! including frame timing, FPS calculations, throughput measurement, and memory usage.
//!
//! # Performance Targets
//!
//! - **UI Target**: 60 FPS (< 16.67ms frame time)
//! - **Throughput Target**: 100,000+ messages/second
//!
//! # Example
//!
//! ```rust
//! use shared_types::performance::{PerformanceMonitor, PerformanceMetrics};
//!
//! let mut monitor = PerformanceMonitor::new();
//!
//! // In your render loop
//! monitor.start_frame();
//! // ... do rendering work ...
//! monitor.end_frame();
//!
//! let metrics = monitor.get_metrics();
//! println!("FPS: {:.1}, Frame Time: {:.2}ms", metrics.fps(), metrics.frame_time_ms());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Target frame rate for smooth UI rendering (60 FPS).
pub const TARGET_FPS: f64 = 60.0;

/// Target frame time in milliseconds for 60 FPS (~16.67ms).
pub const TARGET_FRAME_TIME_MS: f64 = 1000.0 / TARGET_FPS;

/// Target message throughput per second.
pub const TARGET_THROUGHPUT: u64 = 100_000;

/// Number of frames to use for rolling average calculations.
const FRAME_HISTORY_SIZE: usize = 60;

/// Performance metrics snapshot.
///
/// Contains current performance statistics including FPS, frame timing,
/// memory usage, and throughput measurements.
///
/// # Example
///
/// ```rust
/// use shared_types::performance::PerformanceMetrics;
///
/// let metrics = PerformanceMetrics::new(60.0, 16.5, 1024 * 1024 * 100, 150_000);
///
/// assert!(metrics.is_meeting_fps_target());
/// assert!(metrics.is_meeting_throughput_target());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Current frames per second
    fps: f64,
    /// Current frame time in milliseconds
    frame_time_ms: f64,
    /// Memory usage in bytes
    memory_usage_bytes: u64,
    /// Messages processed per second
    throughput: u64,
    /// Number of frames sampled for metrics
    frame_count: u64,
    /// Number of dropped frames (exceeded target frame time)
    dropped_frames: u64,
    /// Peak frame time in milliseconds
    peak_frame_time_ms: f64,
    /// Minimum frame time in milliseconds
    min_frame_time_ms: f64,
}

impl PerformanceMetrics {
    /// Creates new performance metrics with the given values.
    ///
    /// # Arguments
    ///
    /// * `fps` - Frames per second
    /// * `frame_time_ms` - Average frame time in milliseconds
    /// * `memory_usage_bytes` - Current memory usage in bytes
    /// * `throughput` - Messages processed per second
    pub fn new(fps: f64, frame_time_ms: f64, memory_usage_bytes: u64, throughput: u64) -> Self {
        Self {
            fps,
            frame_time_ms,
            memory_usage_bytes,
            throughput,
            frame_count: 0,
            dropped_frames: 0,
            peak_frame_time_ms: frame_time_ms,
            min_frame_time_ms: frame_time_ms,
        }
    }

    /// Returns the current frames per second.
    #[inline]
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Returns the average frame time in milliseconds.
    #[inline]
    pub fn frame_time_ms(&self) -> f64 {
        self.frame_time_ms
    }

    /// Returns the memory usage in bytes.
    #[inline]
    pub fn memory_usage_bytes(&self) -> u64 {
        self.memory_usage_bytes
    }

    /// Returns the memory usage in megabytes.
    #[inline]
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Returns the current message throughput (messages per second).
    #[inline]
    pub fn throughput(&self) -> u64 {
        self.throughput
    }

    /// Returns the total number of frames processed.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the number of dropped frames.
    #[inline]
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    /// Returns the peak (worst) frame time in milliseconds.
    #[inline]
    pub fn peak_frame_time_ms(&self) -> f64 {
        self.peak_frame_time_ms
    }

    /// Returns the minimum (best) frame time in milliseconds.
    #[inline]
    pub fn min_frame_time_ms(&self) -> f64 {
        self.min_frame_time_ms
    }

    /// Returns true if the current FPS meets the 60 FPS target.
    #[inline]
    pub fn is_meeting_fps_target(&self) -> bool {
        self.fps >= TARGET_FPS
    }

    /// Returns true if the frame time is under the target (< 16.67ms).
    #[inline]
    pub fn is_meeting_frame_time_target(&self) -> bool {
        self.frame_time_ms <= TARGET_FRAME_TIME_MS
    }

    /// Returns true if the throughput meets the 100k messages/sec target.
    #[inline]
    pub fn is_meeting_throughput_target(&self) -> bool {
        self.throughput >= TARGET_THROUGHPUT
    }

    /// Returns the percentage of target FPS achieved (capped at 100%).
    pub fn fps_percentage(&self) -> f64 {
        (self.fps / TARGET_FPS * 100.0).min(100.0)
    }

    /// Returns the drop rate as a percentage.
    pub fn drop_rate(&self) -> f64 {
        if self.frame_count == 0 {
            return 0.0;
        }
        (self.dropped_frames as f64 / self.frame_count as f64) * 100.0
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time_ms: 0.0,
            memory_usage_bytes: 0,
            throughput: 0,
            frame_count: 0,
            dropped_frames: 0,
            peak_frame_time_ms: 0.0,
            min_frame_time_ms: f64::MAX,
        }
    }
}

/// Timer for tracking individual frame timing.
///
/// Use this to measure the duration of a single frame or operation.
///
/// # Example
///
/// ```rust
/// use shared_types::performance::FrameTimer;
/// use std::time::Duration;
///
/// let timer = FrameTimer::start();
/// // ... do some work ...
/// std::thread::sleep(Duration::from_millis(5));
/// let elapsed = timer.elapsed();
///
/// assert!(elapsed >= Duration::from_millis(5));
/// ```
#[derive(Debug, Clone)]
pub struct FrameTimer {
    start_time: Instant,
}

impl FrameTimer {
    /// Starts a new frame timer.
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    /// Returns the elapsed time since the timer was started.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Returns the elapsed time in milliseconds.
    #[inline]
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Returns the elapsed time in microseconds.
    #[inline]
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed().as_micros() as u64
    }

    /// Resets the timer to the current instant.
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    /// Returns true if the elapsed time exceeds the target frame time (16.67ms).
    #[inline]
    pub fn exceeded_frame_budget(&self) -> bool {
        self.elapsed_ms() > TARGET_FRAME_TIME_MS
    }

    /// Returns the remaining time budget for this frame.
    ///
    /// Returns `Duration::ZERO` if the budget has been exceeded.
    pub fn remaining_budget(&self) -> Duration {
        let target = Duration::from_secs_f64(TARGET_FRAME_TIME_MS / 1000.0);
        let elapsed = self.elapsed();
        if elapsed >= target {
            Duration::ZERO
        } else {
            target - elapsed
        }
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::start()
    }
}

/// Performance monitor for tracking ongoing performance metrics.
///
/// Tracks frame timing, calculates FPS, monitors memory usage, and measures
/// message throughput over time using rolling averages.
///
/// # Example
///
/// ```rust
/// use shared_types::performance::PerformanceMonitor;
///
/// let mut monitor = PerformanceMonitor::new();
///
/// // Simulate a frame
/// monitor.start_frame();
/// // ... render ...
/// monitor.end_frame();
///
/// // Record message throughput
/// monitor.record_messages(1000);
///
/// let metrics = monitor.get_metrics();
/// println!("Current FPS: {:.1}", metrics.fps());
/// ```
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Current frame timer (active during frame)
    current_frame_timer: Option<FrameTimer>,
    /// History of frame times for rolling average
    frame_times: VecDeque<f64>,
    /// Total frames processed
    frame_count: u64,
    /// Total dropped frames
    dropped_frames: u64,
    /// Peak frame time observed
    peak_frame_time_ms: f64,
    /// Minimum frame time observed
    min_frame_time_ms: f64,
    /// Current memory usage in bytes
    memory_usage_bytes: u64,
    /// Messages processed in current window
    messages_in_window: u64,
    /// Window start time for throughput calculation
    throughput_window_start: Instant,
    /// Last calculated throughput
    last_throughput: u64,
    /// Time of last FPS calculation
    last_fps_calculation: Instant,
    /// Frames since last FPS calculation
    frames_since_last_calc: u64,
    /// Last calculated FPS
    last_fps: f64,
}

impl PerformanceMonitor {
    /// Creates a new performance monitor.
    pub fn new() -> Self {
        Self {
            current_frame_timer: None,
            frame_times: VecDeque::with_capacity(FRAME_HISTORY_SIZE),
            frame_count: 0,
            dropped_frames: 0,
            peak_frame_time_ms: 0.0,
            min_frame_time_ms: f64::MAX,
            memory_usage_bytes: 0,
            messages_in_window: 0,
            throughput_window_start: Instant::now(),
            last_throughput: 0,
            last_fps_calculation: Instant::now(),
            frames_since_last_calc: 0,
            last_fps: 0.0,
        }
    }

    /// Starts timing a new frame.
    ///
    /// Call this at the beginning of your render loop.
    pub fn start_frame(&mut self) {
        self.current_frame_timer = Some(FrameTimer::start());
    }

    /// Ends the current frame and records timing data.
    ///
    /// Call this at the end of your render loop.
    /// Returns the frame time in milliseconds.
    pub fn end_frame(&mut self) -> f64 {
        let frame_time = if let Some(timer) = self.current_frame_timer.take() {
            timer.elapsed_ms()
        } else {
            0.0
        };

        // Record frame time
        self.record_frame_time(frame_time);

        frame_time
    }

    /// Records a frame time directly (useful for external timing).
    pub fn record_frame_time(&mut self, frame_time_ms: f64) {
        self.frame_count += 1;
        self.frames_since_last_calc += 1;

        // Track dropped frames
        if frame_time_ms > TARGET_FRAME_TIME_MS {
            self.dropped_frames += 1;
        }

        // Update min/max
        if frame_time_ms > self.peak_frame_time_ms {
            self.peak_frame_time_ms = frame_time_ms;
        }
        if frame_time_ms < self.min_frame_time_ms {
            self.min_frame_time_ms = frame_time_ms;
        }

        // Add to rolling history
        if self.frame_times.len() >= FRAME_HISTORY_SIZE {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time_ms);

        // Update FPS calculation every second
        let elapsed = self.last_fps_calculation.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            self.last_fps = self.frames_since_last_calc as f64 / elapsed;
            self.frames_since_last_calc = 0;
            self.last_fps_calculation = Instant::now();
        }
    }

    /// Records messages processed for throughput calculation.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of messages processed
    pub fn record_messages(&mut self, count: u64) {
        self.messages_in_window += count;

        // Update throughput calculation every second
        let elapsed = self.throughput_window_start.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            self.last_throughput = (self.messages_in_window as f64 / elapsed) as u64;
            self.messages_in_window = 0;
            self.throughput_window_start = Instant::now();
        }
    }

    /// Updates the current memory usage.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Current memory usage in bytes
    pub fn update_memory_usage(&mut self, bytes: u64) {
        self.memory_usage_bytes = bytes;
    }

    /// Returns the current performance metrics.
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let avg_frame_time = self.average_frame_time();
        let fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            self.last_fps
        };

        PerformanceMetrics {
            fps: fps.max(self.last_fps),
            frame_time_ms: avg_frame_time,
            memory_usage_bytes: self.memory_usage_bytes,
            throughput: self.last_throughput,
            frame_count: self.frame_count,
            dropped_frames: self.dropped_frames,
            peak_frame_time_ms: self.peak_frame_time_ms,
            min_frame_time_ms: if self.min_frame_time_ms == f64::MAX {
                0.0
            } else {
                self.min_frame_time_ms
            },
        }
    }

    /// Returns the average frame time from the rolling history.
    pub fn average_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.frame_times.iter().sum();
        sum / self.frame_times.len() as f64
    }

    /// Returns the current FPS estimate.
    pub fn current_fps(&self) -> f64 {
        let avg = self.average_frame_time();
        if avg > 0.0 {
            (1000.0 / avg).max(self.last_fps)
        } else {
            self.last_fps
        }
    }

    /// Returns true if currently timing a frame.
    #[inline]
    pub fn is_frame_active(&self) -> bool {
        self.current_frame_timer.is_some()
    }

    /// Returns the total number of frames recorded.
    #[inline]
    pub fn total_frames(&self) -> u64 {
        self.frame_count
    }

    /// Returns the number of dropped frames.
    #[inline]
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    /// Resets all metrics to initial state.
    pub fn reset(&mut self) {
        self.current_frame_timer = None;
        self.frame_times.clear();
        self.frame_count = 0;
        self.dropped_frames = 0;
        self.peak_frame_time_ms = 0.0;
        self.min_frame_time_ms = f64::MAX;
        self.memory_usage_bytes = 0;
        self.messages_in_window = 0;
        self.throughput_window_start = Instant::now();
        self.last_throughput = 0;
        self.last_fps_calculation = Instant::now();
        self.frames_since_last_calc = 0;
        self.last_fps = 0.0;
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility to estimate current process memory usage.
///
/// Note: This is a platform-dependent operation. On unsupported platforms,
/// returns 0.
#[cfg(target_os = "linux")]
pub fn get_process_memory_usage() -> u64 {
    use std::fs;

    // Read from /proc/self/statm for memory info
    if let Ok(statm) = fs::read_to_string("/proc/self/statm") {
        let parts: Vec<&str> = statm.split_whitespace().collect();
        if let Some(rss_pages) = parts.get(1) {
            if let Ok(pages) = rss_pages.parse::<u64>() {
                // Each page is typically 4KB
                return pages * 4096;
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
pub fn get_process_memory_usage() -> u64 {
    // Return 0 for unsupported platforms
    // Could be extended with platform-specific implementations
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // PerformanceMetrics tests

    #[test]
    fn test_performance_metrics_new() {
        let metrics = PerformanceMetrics::new(60.0, 16.5, 1024 * 1024 * 100, 150_000);

        assert_eq!(metrics.fps(), 60.0);
        assert_eq!(metrics.frame_time_ms(), 16.5);
        assert_eq!(metrics.memory_usage_bytes(), 104_857_600);
        assert_eq!(metrics.throughput(), 150_000);
    }

    #[test]
    fn test_performance_metrics_memory_mb() {
        let metrics = PerformanceMetrics::new(60.0, 16.5, 1024 * 1024 * 100, 150_000);
        assert!((metrics.memory_usage_mb() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_performance_metrics_fps_target() {
        let meeting_target = PerformanceMetrics::new(60.0, 16.0, 0, 0);
        assert!(meeting_target.is_meeting_fps_target());

        let below_target = PerformanceMetrics::new(55.0, 18.0, 0, 0);
        assert!(!below_target.is_meeting_fps_target());
    }

    #[test]
    fn test_performance_metrics_frame_time_target() {
        let meeting_target = PerformanceMetrics::new(60.0, 16.0, 0, 0);
        assert!(meeting_target.is_meeting_frame_time_target());

        let over_budget = PerformanceMetrics::new(55.0, 20.0, 0, 0);
        assert!(!over_budget.is_meeting_frame_time_target());
    }

    #[test]
    fn test_performance_metrics_throughput_target() {
        let meeting_target = PerformanceMetrics::new(60.0, 16.0, 0, 100_000);
        assert!(meeting_target.is_meeting_throughput_target());

        let below_target = PerformanceMetrics::new(60.0, 16.0, 0, 50_000);
        assert!(!below_target.is_meeting_throughput_target());
    }

    #[test]
    fn test_performance_metrics_fps_percentage() {
        let full = PerformanceMetrics::new(60.0, 16.0, 0, 0);
        assert!((full.fps_percentage() - 100.0).abs() < 0.001);

        let half = PerformanceMetrics::new(30.0, 33.0, 0, 0);
        assert!((half.fps_percentage() - 50.0).abs() < 0.001);

        let over = PerformanceMetrics::new(120.0, 8.0, 0, 0);
        assert!((over.fps_percentage() - 100.0).abs() < 0.001); // Capped at 100%
    }

    #[test]
    fn test_performance_metrics_drop_rate() {
        let mut metrics = PerformanceMetrics::default();
        metrics.frame_count = 100;
        metrics.dropped_frames = 5;
        assert!((metrics.drop_rate() - 5.0).abs() < 0.001);

        let zero_frames = PerformanceMetrics::default();
        assert_eq!(zero_frames.drop_rate(), 0.0);
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.fps(), 0.0);
        assert_eq!(metrics.frame_time_ms(), 0.0);
        assert_eq!(metrics.memory_usage_bytes(), 0);
        assert_eq!(metrics.throughput(), 0);
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = PerformanceMetrics::new(60.0, 16.5, 1024 * 1024, 100_000);
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, deserialized);
    }

    // FrameTimer tests

    #[test]
    fn test_frame_timer_start() {
        let timer = FrameTimer::start();
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_frame_timer_elapsed_ms() {
        let timer = FrameTimer::start();
        thread::sleep(Duration::from_millis(15));
        let elapsed_ms = timer.elapsed_ms();
        assert!(elapsed_ms >= 15.0);
    }

    #[test]
    fn test_frame_timer_elapsed_us() {
        let timer = FrameTimer::start();
        thread::sleep(Duration::from_millis(5));
        let elapsed_us = timer.elapsed_us();
        assert!(elapsed_us >= 5000);
    }

    #[test]
    fn test_frame_timer_reset() {
        let mut timer = FrameTimer::start();
        thread::sleep(Duration::from_millis(20));
        timer.reset();
        let elapsed = timer.elapsed();
        assert!(elapsed < Duration::from_millis(10));
    }

    #[test]
    fn test_frame_timer_exceeded_budget() {
        let timer = FrameTimer::start();
        // Should not exceed budget immediately
        assert!(!timer.exceeded_frame_budget());

        // Sleep longer than target frame time
        thread::sleep(Duration::from_millis(20));
        assert!(timer.exceeded_frame_budget());
    }

    #[test]
    fn test_frame_timer_remaining_budget() {
        let timer = FrameTimer::start();
        let remaining = timer.remaining_budget();
        // Should have most of the budget remaining
        assert!(remaining > Duration::from_millis(10));

        // After exceeding budget
        thread::sleep(Duration::from_millis(20));
        let remaining = timer.remaining_budget();
        assert_eq!(remaining, Duration::ZERO);
    }

    #[test]
    fn test_frame_timer_default() {
        let timer = FrameTimer::default();
        thread::sleep(Duration::from_millis(5));
        assert!(timer.elapsed() >= Duration::from_millis(5));
    }

    // PerformanceMonitor tests

    #[test]
    fn test_performance_monitor_new() {
        let monitor = PerformanceMonitor::new();
        assert!(!monitor.is_frame_active());
        assert_eq!(monitor.total_frames(), 0);
        assert_eq!(monitor.dropped_frames(), 0);
    }

    #[test]
    fn test_performance_monitor_frame_timing() {
        let mut monitor = PerformanceMonitor::new();

        monitor.start_frame();
        assert!(monitor.is_frame_active());

        thread::sleep(Duration::from_millis(5));
        let frame_time = monitor.end_frame();

        assert!(!monitor.is_frame_active());
        assert!(frame_time >= 5.0);
        assert_eq!(monitor.total_frames(), 1);
    }

    #[test]
    fn test_performance_monitor_record_frame_time() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_frame_time(10.0);
        monitor.record_frame_time(12.0);
        monitor.record_frame_time(11.0);

        assert_eq!(monitor.total_frames(), 3);
        let avg = monitor.average_frame_time();
        assert!((avg - 11.0).abs() < 0.001);
    }

    #[test]
    fn test_performance_monitor_dropped_frames() {
        let mut monitor = PerformanceMonitor::new();

        // Record frame times: some under budget, some over
        monitor.record_frame_time(10.0); // OK
        monitor.record_frame_time(15.0); // OK
        monitor.record_frame_time(20.0); // Dropped (> 16.67ms)
        monitor.record_frame_time(25.0); // Dropped

        assert_eq!(monitor.total_frames(), 4);
        assert_eq!(monitor.dropped_frames(), 2);
    }

    #[test]
    fn test_performance_monitor_min_max_frame_time() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_frame_time(10.0);
        monitor.record_frame_time(5.0);
        monitor.record_frame_time(20.0);
        monitor.record_frame_time(15.0);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.peak_frame_time_ms(), 20.0);
        assert_eq!(metrics.min_frame_time_ms(), 5.0);
    }

    #[test]
    fn test_performance_monitor_memory_usage() {
        let mut monitor = PerformanceMonitor::new();

        monitor.update_memory_usage(1024 * 1024 * 50); // 50 MB

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.memory_usage_bytes(), 52_428_800);
        assert!((metrics.memory_usage_mb() - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_performance_monitor_reset() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_frame_time(10.0);
        monitor.record_frame_time(20.0);
        monitor.update_memory_usage(1024);

        monitor.reset();

        assert_eq!(monitor.total_frames(), 0);
        assert_eq!(monitor.dropped_frames(), 0);
        assert_eq!(monitor.average_frame_time(), 0.0);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.memory_usage_bytes(), 0);
    }

    #[test]
    fn test_performance_monitor_get_metrics() {
        let mut monitor = PerformanceMonitor::new();

        // Simulate some frames at ~16ms each (60 FPS target)
        for _ in 0..10 {
            monitor.record_frame_time(16.0);
        }
        monitor.update_memory_usage(1024 * 1024 * 100);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.frame_count(), 10);
        assert!((metrics.frame_time_ms() - 16.0).abs() < 0.001);
        assert_eq!(metrics.memory_usage_bytes(), 104_857_600);
    }

    #[test]
    fn test_performance_monitor_rolling_average() {
        let mut monitor = PerformanceMonitor::new();

        // Fill history beyond capacity
        for i in 0..100 {
            monitor.record_frame_time(i as f64);
        }

        // Should only keep last FRAME_HISTORY_SIZE frames
        // Average of 40-99 = (40+99)*60/2/60 = 69.5
        let avg = monitor.average_frame_time();
        assert!(avg > 60.0); // Recent frames are larger
    }

    #[test]
    fn test_performance_monitor_fps_calculation() {
        let mut monitor = PerformanceMonitor::new();

        // Record frames at exactly 16.67ms (60 FPS)
        for _ in 0..60 {
            monitor.record_frame_time(TARGET_FRAME_TIME_MS);
        }

        let fps = monitor.current_fps();
        assert!((fps - 60.0).abs() < 1.0);
    }

    #[test]
    fn test_performance_monitor_default() {
        let monitor = PerformanceMonitor::default();
        assert_eq!(monitor.total_frames(), 0);
    }

    // Constants tests

    #[test]
    fn test_constants() {
        assert_eq!(TARGET_FPS, 60.0);
        assert!((TARGET_FRAME_TIME_MS - 16.666666666666668).abs() < 0.0001);
        assert_eq!(TARGET_THROUGHPUT, 100_000);
    }

    // Integration test

    #[test]
    fn test_full_performance_monitoring_workflow() {
        let mut monitor = PerformanceMonitor::new();

        // Simulate a few frames
        for _ in 0..5 {
            monitor.start_frame();
            thread::sleep(Duration::from_millis(10));
            monitor.end_frame();
        }

        // Record some message throughput
        monitor.record_messages(1000);

        // Update memory
        monitor.update_memory_usage(50 * 1024 * 1024);

        let metrics = monitor.get_metrics();

        // Verify metrics make sense
        assert_eq!(metrics.frame_count(), 5);
        assert!(metrics.frame_time_ms() >= 10.0);
        assert_eq!(metrics.memory_usage_bytes(), 52_428_800);

        // Should be meeting FPS target with 10ms frames
        assert!(metrics.is_meeting_frame_time_target());
    }
}
