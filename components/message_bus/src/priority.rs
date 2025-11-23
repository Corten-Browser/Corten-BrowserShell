//! Priority-based message routing and queuing
//!
//! This module provides:
//! - Priority-based message queuing with multiple priority lanes
//! - Message deadlines and timeout tracking
//! - Priority inversion prevention through age-based boosting
//! - Message routing rules based on message type
//! - Metrics collection for queue monitoring

use crate::types::{ComponentMessage, MessagePriority};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A message with priority and deadline information
#[derive(Debug, Clone)]
pub struct PrioritizedMessage {
    /// The actual message payload
    pub message: ComponentMessage,
    /// Priority level for this message
    pub priority: MessagePriority,
    /// Optional deadline for processing
    pub deadline: Option<Instant>,
    /// When this message was created
    pub created_at: Instant,
    /// Unique message ID for tracking
    pub id: u64,
    /// Target component (None for broadcast)
    pub target: Option<String>,
}

impl PrioritizedMessage {
    /// Create a new prioritized message
    pub fn new(message: ComponentMessage, priority: MessagePriority) -> Self {
        static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            message,
            priority,
            deadline: None,
            created_at: Instant::now(),
            id: MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            target: None,
        }
    }

    /// Create a message with a specific deadline
    pub fn with_deadline(message: ComponentMessage, deadline: Duration) -> Self {
        let mut msg = Self::new(message, MessagePriority::Normal);
        msg.deadline = Some(Instant::now() + deadline);
        msg
    }

    /// Set the target component for this message
    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    /// Check if this message has exceeded its deadline
    pub fn is_expired(&self) -> bool {
        self.deadline.map_or(false, |d| Instant::now() > d)
    }

    /// Get the age of this message
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get time remaining until deadline (None if no deadline or expired)
    pub fn time_remaining(&self) -> Option<Duration> {
        self.deadline.and_then(|d| {
            let now = Instant::now();
            if now < d {
                Some(d - now)
            } else {
                None
            }
        })
    }
}

/// Statistics for a single priority lane
#[derive(Debug, Clone, Default)]
pub struct LaneStats {
    /// Total messages enqueued to this lane
    pub enqueued: u64,
    /// Total messages dequeued from this lane
    pub dequeued: u64,
    /// Total messages that expired in this lane
    pub expired: u64,
    /// Total messages boosted from this lane
    pub boosted: u64,
    /// Current queue depth
    pub current_depth: usize,
}

/// Overall queue statistics
#[derive(Debug, Clone, Default)]
pub struct QueueMetrics {
    /// Statistics per priority lane
    pub lanes: [LaneStats; 4],
    /// Total messages processed
    pub total_processed: u64,
    /// Total messages expired
    pub total_expired: u64,
    /// Average wait time in microseconds
    pub avg_wait_time_us: u64,
    /// Maximum wait time in microseconds
    pub max_wait_time_us: u64,
}

impl QueueMetrics {
    /// Get the total current depth across all lanes
    pub fn total_depth(&self) -> usize {
        self.lanes.iter().map(|l| l.current_depth).sum()
    }

    /// Get statistics for a specific priority
    pub fn lane_stats(&self, priority: MessagePriority) -> &LaneStats {
        &self.lanes[priority_to_index(priority)]
    }
}

/// Convert priority to array index
fn priority_to_index(priority: MessagePriority) -> usize {
    match priority {
        MessagePriority::Critical => 0,
        MessagePriority::High => 1,
        MessagePriority::Normal => 2,
        MessagePriority::Low => 3,
    }
}

/// Configuration for the priority queue
#[derive(Debug, Clone)]
pub struct PriorityQueueConfig {
    /// Maximum age before a message is boosted to higher priority
    pub starvation_threshold: Duration,
    /// Maximum depth per lane (0 = unlimited)
    pub max_lane_depth: usize,
    /// Whether to automatically remove expired messages
    pub auto_expire: bool,
    /// Enable priority boosting for starving messages
    pub enable_boosting: bool,
}

impl Default for PriorityQueueConfig {
    fn default() -> Self {
        Self {
            starvation_threshold: Duration::from_secs(5),
            max_lane_depth: 1000,
            auto_expire: true,
            enable_boosting: true,
        }
    }
}

/// A priority queue with multiple lanes for different priority levels
///
/// Messages are organized into separate lanes by priority, with higher
/// priority messages being dequeued first. Includes support for:
/// - Deadline tracking
/// - Priority boosting for starving messages
/// - Metrics collection
pub struct PriorityQueue {
    /// Separate queues for each priority level
    /// Index 0 = Critical, 1 = High, 2 = Normal, 3 = Low
    lanes: [RwLock<VecDeque<PrioritizedMessage>>; 4],
    /// Queue configuration
    config: PriorityQueueConfig,
    /// Metrics tracking
    metrics: RwLock<QueueMetrics>,
    /// Total wait time accumulator (in microseconds)
    total_wait_time_us: AtomicU64,
    /// Count of messages for average calculation
    message_count: AtomicU64,
}

impl PriorityQueue {
    /// Create a new priority queue with default configuration
    pub fn new() -> Self {
        Self::with_config(PriorityQueueConfig::default())
    }

    /// Create a new priority queue with custom configuration
    pub fn with_config(config: PriorityQueueConfig) -> Self {
        Self {
            lanes: [
                RwLock::new(VecDeque::new()),
                RwLock::new(VecDeque::new()),
                RwLock::new(VecDeque::new()),
                RwLock::new(VecDeque::new()),
            ],
            config,
            metrics: RwLock::new(QueueMetrics::default()),
            total_wait_time_us: AtomicU64::new(0),
            message_count: AtomicU64::new(0),
        }
    }

    /// Enqueue a message with the specified priority
    pub async fn enqueue(&self, message: PrioritizedMessage) -> Result<(), QueueError> {
        let lane_idx = priority_to_index(message.priority);

        let mut lane = self.lanes[lane_idx].write().await;

        // Check lane capacity
        if self.config.max_lane_depth > 0 && lane.len() >= self.config.max_lane_depth {
            return Err(QueueError::LaneFull(message.priority));
        }

        lane.push_back(message);

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.lanes[lane_idx].enqueued += 1;
        metrics.lanes[lane_idx].current_depth = lane.len();

        Ok(())
    }

    /// Dequeue the highest priority message
    ///
    /// Returns messages in priority order: Critical > High > Normal > Low
    /// Within the same priority, messages are returned in FIFO order
    pub async fn dequeue(&self) -> Option<PrioritizedMessage> {
        // First, boost any starving messages if enabled
        if self.config.enable_boosting {
            self.boost_starving_messages().await;
        }

        // Try each lane in priority order
        for lane_idx in 0..4 {
            let mut lane = self.lanes[lane_idx].write().await;

            // Skip expired messages if auto-expire is enabled
            if self.config.auto_expire {
                while let Some(front) = lane.front() {
                    if front.is_expired() {
                        let _ = lane.pop_front();
                        let mut metrics = self.metrics.write().await;
                        metrics.lanes[lane_idx].expired += 1;
                        metrics.total_expired += 1;
                        metrics.lanes[lane_idx].current_depth = lane.len();
                    } else {
                        break;
                    }
                }
            }

            if let Some(message) = lane.pop_front() {
                // Update metrics
                let wait_time_us = message.age().as_micros() as u64;
                self.total_wait_time_us.fetch_add(wait_time_us, Ordering::Relaxed);
                let count = self.message_count.fetch_add(1, Ordering::Relaxed) + 1;

                let mut metrics = self.metrics.write().await;
                metrics.lanes[lane_idx].dequeued += 1;
                metrics.lanes[lane_idx].current_depth = lane.len();
                metrics.total_processed += 1;
                metrics.avg_wait_time_us =
                    self.total_wait_time_us.load(Ordering::Relaxed) / count;
                if wait_time_us > metrics.max_wait_time_us {
                    metrics.max_wait_time_us = wait_time_us;
                }

                return Some(message);
            }
        }

        None
    }

    /// Boost starving messages to higher priority lanes
    async fn boost_starving_messages(&self) {
        let threshold = self.config.starvation_threshold;

        // Check lanes from low to high (skip critical, can't boost higher)
        for source_idx in (1..4).rev() {
            let target_idx = source_idx - 1;

            let mut source_lane = self.lanes[source_idx].write().await;
            let mut to_boost = Vec::new();

            // Find messages that have waited too long
            let mut i = 0;
            while i < source_lane.len() {
                if source_lane[i].age() > threshold {
                    if let Some(mut msg) = source_lane.remove(i) {
                        // Boost to next higher priority
                        msg.priority = match msg.priority {
                            MessagePriority::Low => MessagePriority::Normal,
                            MessagePriority::Normal => MessagePriority::High,
                            MessagePriority::High => MessagePriority::Critical,
                            MessagePriority::Critical => MessagePriority::Critical,
                        };
                        to_boost.push(msg);
                    }
                } else {
                    i += 1;
                }
            }

            // Update source lane metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.lanes[source_idx].boosted += to_boost.len() as u64;
                metrics.lanes[source_idx].current_depth = source_lane.len();
            }

            // Move boosted messages to target lane
            if !to_boost.is_empty() {
                drop(source_lane); // Release source lock before acquiring target
                let mut target_lane = self.lanes[target_idx].write().await;
                for msg in to_boost {
                    target_lane.push_back(msg);
                }
                let mut metrics = self.metrics.write().await;
                metrics.lanes[target_idx].current_depth = target_lane.len();
            }
        }
    }

    /// Get current queue metrics
    pub async fn metrics(&self) -> QueueMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        for lane in &self.lanes {
            if !lane.read().await.is_empty() {
                return false;
            }
        }
        true
    }

    /// Get the total number of messages across all lanes
    pub async fn len(&self) -> usize {
        let mut total = 0;
        for lane in &self.lanes {
            total += lane.read().await.len();
        }
        total
    }

    /// Clear all messages from the queue
    pub async fn clear(&self) {
        for lane in &self.lanes {
            lane.write().await.clear();
        }
        let mut metrics = self.metrics.write().await;
        for lane_stats in &mut metrics.lanes {
            lane_stats.current_depth = 0;
        }
    }

    /// Peek at the next message without removing it
    pub async fn peek(&self) -> Option<PrioritizedMessage> {
        for lane_idx in 0..4 {
            let lane = self.lanes[lane_idx].read().await;
            if let Some(msg) = lane.front() {
                if !self.config.auto_expire || !msg.is_expired() {
                    return Some(msg.clone());
                }
            }
        }
        None
    }
}

impl Default for PriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during queue operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueError {
    /// The specified priority lane is full
    LaneFull(MessagePriority),
    /// Message deadline has expired
    DeadlineExpired,
    /// Invalid routing target
    InvalidTarget(String),
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::LaneFull(priority) => {
                write!(f, "Priority lane {:?} is full", priority)
            }
            QueueError::DeadlineExpired => write!(f, "Message deadline has expired"),
            QueueError::InvalidTarget(target) => {
                write!(f, "Invalid routing target: {}", target)
            }
        }
    }
}

impl std::error::Error for QueueError {}

/// Message router that determines priority and routing based on message type
pub struct MessageRouter {
    /// Default priority for messages without explicit priority
    default_priority: MessagePriority,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {
            default_priority: MessagePriority::Normal,
        }
    }

    /// Determine the priority for a message based on its type
    pub fn determine_priority(&self, message: &ComponentMessage) -> MessagePriority {
        match message {
            // Critical priority: User input and window management
            ComponentMessage::KeyboardShortcut(_) => MessagePriority::Critical,
            ComponentMessage::CloseWindow(_) => MessagePriority::Critical,

            // High priority: Navigation and tab operations
            ComponentMessage::CreateWindow(_) => MessagePriority::High,
            ComponentMessage::CreateTab(_, _) => MessagePriority::High,
            ComponentMessage::CloseTab(_) => MessagePriority::High,
            ComponentMessage::NavigateTab(_, _) => MessagePriority::High,

            // Normal priority: UI updates
            ComponentMessage::UpdateAddressBar(_, _) => MessagePriority::Normal,
            ComponentMessage::UpdateTitle(_, _) => MessagePriority::Normal,
        }
    }

    /// Determine the target thread type for a message
    pub fn determine_target(&self, message: &ComponentMessage) -> RoutingTarget {
        match message {
            // UI operations go to the UI thread
            ComponentMessage::UpdateAddressBar(_, _) => RoutingTarget::UI,
            ComponentMessage::UpdateTitle(_, _) => RoutingTarget::UI,

            // Tab operations go to the appropriate tab thread
            ComponentMessage::CreateTab(_, _) => RoutingTarget::TabManager,
            ComponentMessage::CloseTab(_) => RoutingTarget::TabManager,
            ComponentMessage::NavigateTab(_, _) => RoutingTarget::Network,

            // Window operations go to the window manager
            ComponentMessage::CreateWindow(_) => RoutingTarget::WindowManager,
            ComponentMessage::CloseWindow(_) => RoutingTarget::WindowManager,

            // Input goes to the message bus for distribution
            ComponentMessage::KeyboardShortcut(_) => RoutingTarget::MessageBus,
        }
    }

    /// Create a prioritized message with automatic priority determination
    pub fn route(&self, message: ComponentMessage) -> PrioritizedMessage {
        let priority = self.determine_priority(&message);
        PrioritizedMessage::new(message, priority)
    }

    /// Create a prioritized message with a deadline
    pub fn route_with_deadline(
        &self,
        message: ComponentMessage,
        deadline: Duration,
    ) -> PrioritizedMessage {
        let priority = self.determine_priority(&message);
        let mut msg = PrioritizedMessage::new(message, priority);
        msg.deadline = Some(Instant::now() + deadline);
        msg
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Target for message routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingTarget {
    /// Main UI thread
    UI,
    /// Message bus for distribution
    MessageBus,
    /// Tab management
    TabManager,
    /// Window management
    WindowManager,
    /// Network operations
    Network,
    /// IO operations
    IO,
}

/// Priority queue handle for thread-safe sharing
pub type SharedPriorityQueue = Arc<PriorityQueue>;

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{KeyboardShortcut, WindowConfig, WindowId};

    #[tokio::test]
    async fn test_prioritized_message_creation() {
        let msg = ComponentMessage::CreateWindow(WindowConfig::default());
        let pm = PrioritizedMessage::new(msg, MessagePriority::High);

        assert_eq!(pm.priority, MessagePriority::High);
        assert!(pm.deadline.is_none());
        assert!(!pm.is_expired());
    }

    #[tokio::test]
    async fn test_message_with_deadline() {
        let msg = ComponentMessage::CreateWindow(WindowConfig::default());
        let pm = PrioritizedMessage::with_deadline(msg, Duration::from_millis(100));

        assert!(pm.deadline.is_some());
        assert!(!pm.is_expired());

        // Wait for deadline to pass
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(pm.is_expired());
    }

    #[tokio::test]
    async fn test_time_remaining() {
        let msg = ComponentMessage::CreateWindow(WindowConfig::default());
        let pm = PrioritizedMessage::with_deadline(msg, Duration::from_millis(500));

        let remaining = pm.time_remaining();
        assert!(remaining.is_some());
        assert!(remaining.unwrap() <= Duration::from_millis(500));
        assert!(remaining.unwrap() > Duration::from_millis(400));
    }

    #[tokio::test]
    async fn test_priority_queue_basic_operations() {
        let queue = PriorityQueue::new();

        // Enqueue messages
        let msg1 = PrioritizedMessage::new(
            ComponentMessage::UpdateTitle(shared_types::TabId::new(), "Test".to_string()),
            MessagePriority::Low,
        );
        let msg2 = PrioritizedMessage::new(
            ComponentMessage::CreateWindow(WindowConfig::default()),
            MessagePriority::High,
        );

        queue.enqueue(msg1).await.unwrap();
        queue.enqueue(msg2).await.unwrap();

        assert_eq!(queue.len().await, 2);

        // High priority should come first
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.priority, MessagePriority::High);

        // Then low priority
        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.priority, MessagePriority::Low);

        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = PriorityQueue::new();

        // Enqueue in reverse priority order
        let priorities = [
            MessagePriority::Low,
            MessagePriority::Normal,
            MessagePriority::High,
            MessagePriority::Critical,
        ];

        for priority in priorities {
            let msg = PrioritizedMessage::new(
                ComponentMessage::CreateWindow(WindowConfig::default()),
                priority,
            );
            queue.enqueue(msg).await.unwrap();
        }

        // Should dequeue in priority order
        let expected = [
            MessagePriority::Critical,
            MessagePriority::High,
            MessagePriority::Normal,
            MessagePriority::Low,
        ];

        for expected_priority in expected {
            let msg = queue.dequeue().await.unwrap();
            assert_eq!(msg.priority, expected_priority);
        }
    }

    #[tokio::test]
    async fn test_fifo_within_priority() {
        let queue = PriorityQueue::new();

        // Enqueue three messages with same priority
        for i in 0..3 {
            let msg = PrioritizedMessage::new(
                ComponentMessage::UpdateTitle(shared_types::TabId::new(), format!("Title {}", i)),
                MessagePriority::Normal,
            );
            queue.enqueue(msg).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Should dequeue in FIFO order within same priority
        let msg1 = queue.dequeue().await.unwrap();
        let msg2 = queue.dequeue().await.unwrap();
        let msg3 = queue.dequeue().await.unwrap();

        // First message should be older than second, which should be older than third
        assert!(msg1.created_at < msg2.created_at);
        assert!(msg2.created_at < msg3.created_at);
    }

    #[tokio::test]
    async fn test_expired_message_handling() {
        let config = PriorityQueueConfig {
            auto_expire: true,
            ..Default::default()
        };
        let queue = PriorityQueue::with_config(config);

        // Add a message with very short deadline
        let msg = PrioritizedMessage::with_deadline(
            ComponentMessage::CreateWindow(WindowConfig::default()),
            Duration::from_millis(10),
        );
        queue.enqueue(msg).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Should not return expired message
        assert!(queue.dequeue().await.is_none());

        // Check metrics
        let metrics = queue.metrics().await;
        assert_eq!(metrics.total_expired, 1);
    }

    #[tokio::test]
    async fn test_priority_boosting() {
        let config = PriorityQueueConfig {
            starvation_threshold: Duration::from_millis(50),
            enable_boosting: true,
            ..Default::default()
        };
        let queue = PriorityQueue::with_config(config);

        // Add a low priority message
        let msg = PrioritizedMessage::new(
            ComponentMessage::CreateWindow(WindowConfig::default()),
            MessagePriority::Low,
        );
        queue.enqueue(msg).await.unwrap();

        // Wait for starvation threshold
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Dequeue should boost the message
        let dequeued = queue.dequeue().await.unwrap();

        // Should have been boosted from Low to Normal
        assert_eq!(dequeued.priority, MessagePriority::Normal);
    }

    #[tokio::test]
    async fn test_lane_full_error() {
        let config = PriorityQueueConfig {
            max_lane_depth: 2,
            ..Default::default()
        };
        let queue = PriorityQueue::with_config(config);

        // Fill the lane
        for _ in 0..2 {
            let msg = PrioritizedMessage::new(
                ComponentMessage::CreateWindow(WindowConfig::default()),
                MessagePriority::Normal,
            );
            queue.enqueue(msg).await.unwrap();
        }

        // Third should fail
        let msg = PrioritizedMessage::new(
            ComponentMessage::CreateWindow(WindowConfig::default()),
            MessagePriority::Normal,
        );
        let result = queue.enqueue(msg).await;
        assert!(matches!(result, Err(QueueError::LaneFull(MessagePriority::Normal))));
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let queue = PriorityQueue::new();

        // Enqueue some messages
        for priority in [MessagePriority::High, MessagePriority::Normal, MessagePriority::Low] {
            let msg = PrioritizedMessage::new(
                ComponentMessage::CreateWindow(WindowConfig::default()),
                priority,
            );
            queue.enqueue(msg).await.unwrap();
        }

        // Dequeue all
        while queue.dequeue().await.is_some() {}

        let metrics = queue.metrics().await;
        assert_eq!(metrics.total_processed, 3);
        assert_eq!(metrics.lane_stats(MessagePriority::High).dequeued, 1);
        assert_eq!(metrics.lane_stats(MessagePriority::Normal).dequeued, 1);
        assert_eq!(metrics.lane_stats(MessagePriority::Low).dequeued, 1);
    }

    #[tokio::test]
    async fn test_peek() {
        let queue = PriorityQueue::new();

        let msg = PrioritizedMessage::new(
            ComponentMessage::CreateWindow(WindowConfig::default()),
            MessagePriority::High,
        );
        let original_id = msg.id;
        queue.enqueue(msg).await.unwrap();

        // Peek should return the message without removing it
        let peeked = queue.peek().await.unwrap();
        assert_eq!(peeked.id, original_id);

        // Queue should still have the message
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let queue = PriorityQueue::new();

        for _ in 0..5 {
            let msg = PrioritizedMessage::new(
                ComponentMessage::CreateWindow(WindowConfig::default()),
                MessagePriority::Normal,
            );
            queue.enqueue(msg).await.unwrap();
        }

        assert_eq!(queue.len().await, 5);

        queue.clear().await;

        assert!(queue.is_empty().await);
        assert_eq!(queue.len().await, 0);
    }

    #[test]
    fn test_message_router_priority_determination() {
        let router = MessageRouter::new();

        // Critical priority
        assert_eq!(
            router.determine_priority(&ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT)),
            MessagePriority::Critical
        );
        assert_eq!(
            router.determine_priority(&ComponentMessage::CloseWindow(WindowId::new())),
            MessagePriority::Critical
        );

        // High priority
        assert_eq!(
            router.determine_priority(&ComponentMessage::CreateWindow(WindowConfig::default())),
            MessagePriority::High
        );
        assert_eq!(
            router.determine_priority(&ComponentMessage::NavigateTab(
                shared_types::TabId::new(),
                "https://example.com".to_string()
            )),
            MessagePriority::High
        );

        // Normal priority
        assert_eq!(
            router.determine_priority(&ComponentMessage::UpdateTitle(
                shared_types::TabId::new(),
                "Test".to_string()
            )),
            MessagePriority::Normal
        );
    }

    #[test]
    fn test_message_router_target_determination() {
        let router = MessageRouter::new();

        assert_eq!(
            router.determine_target(&ComponentMessage::UpdateTitle(
                shared_types::TabId::new(),
                "Test".to_string()
            )),
            RoutingTarget::UI
        );

        assert_eq!(
            router.determine_target(&ComponentMessage::CreateWindow(WindowConfig::default())),
            RoutingTarget::WindowManager
        );

        assert_eq!(
            router.determine_target(&ComponentMessage::NavigateTab(
                shared_types::TabId::new(),
                "https://example.com".to_string()
            )),
            RoutingTarget::Network
        );

        assert_eq!(
            router.determine_target(&ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT)),
            RoutingTarget::MessageBus
        );
    }

    #[test]
    fn test_message_router_route() {
        let router = MessageRouter::new();
        let msg = ComponentMessage::KeyboardShortcut(KeyboardShortcut::CtrlT);

        let routed = router.route(msg);
        assert_eq!(routed.priority, MessagePriority::Critical);
    }

    #[tokio::test]
    async fn test_queue_metrics_total_depth() {
        let queue = PriorityQueue::new();

        for priority in [
            MessagePriority::Critical,
            MessagePriority::High,
            MessagePriority::Normal,
            MessagePriority::Low,
        ] {
            let msg = PrioritizedMessage::new(
                ComponentMessage::CreateWindow(WindowConfig::default()),
                priority,
            );
            queue.enqueue(msg).await.unwrap();
        }

        let metrics = queue.metrics().await;
        assert_eq!(metrics.total_depth(), 4);
    }

    #[test]
    fn test_queue_error_display() {
        let err = QueueError::LaneFull(MessagePriority::High);
        assert!(err.to_string().contains("High"));

        let err = QueueError::DeadlineExpired;
        assert!(err.to_string().contains("expired"));

        let err = QueueError::InvalidTarget("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
    }

    #[tokio::test]
    async fn test_message_target() {
        let msg = ComponentMessage::CreateWindow(WindowConfig::default());
        let pm = PrioritizedMessage::new(msg, MessagePriority::Normal)
            .with_target("window_manager".to_string());

        assert_eq!(pm.target, Some("window_manager".to_string()));
    }
}
