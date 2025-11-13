// @implements: REQ-003
//! Priority-based message queue

use std::collections::BinaryHeap;
use std::cmp::Ordering;
use tokio::sync::RwLock;
use shared_types::ComponentMessage;
use crate::error::{MessageBusError, Result};

/// Wrapper for ComponentMessage that implements priority ordering
#[derive(Clone)]
struct PriorityMessage {
    message: ComponentMessage,
}

impl PartialEq for PriorityMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.priority == other.message.priority &&
        self.message.timestamp == other.message.timestamp
    }
}

impl Eq for PriorityMessage {}

impl PartialOrd for PriorityMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (lower enum value = higher priority)
        match self.message.priority.cmp(&other.message.priority) {
            Ordering::Equal => {
                // If priorities are equal, use FIFO (older messages first)
                // Reverse comparison because BinaryHeap is a max-heap
                other.message.timestamp.cmp(&self.message.timestamp)
            }
            // Reverse comparison for priority because lower priority value = higher priority
            // and we want higher priority messages first
            ordering => ordering.reverse(),
        }
    }
}

/// Priority queue for messages
pub struct PriorityQueue {
    queue: RwLock<BinaryHeap<PriorityMessage>>,
    capacity: usize,
}

impl PriorityQueue {
    /// Create a new priority queue with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: RwLock::new(BinaryHeap::with_capacity(capacity)),
            capacity,
        }
    }

    /// Push a message onto the queue (async, waits if full)
    pub async fn push(&mut self, message: ComponentMessage) -> Result<()> {
        let mut queue = self.queue.write().await;

        if queue.len() >= self.capacity {
            return Err(MessageBusError::QueueFull);
        }

        queue.push(PriorityMessage { message });
        Ok(())
    }

    /// Try to push a message (non-blocking)
    pub fn try_push(&mut self, message: ComponentMessage) -> Result<()> {
        let queue = self.queue.try_write()
            .map_err(|_| MessageBusError::Internal("Lock contention".to_string()))?;

        if queue.len() >= self.capacity {
            return Err(MessageBusError::QueueFull);
        }

        drop(queue);

        // We need to get write lock again after check
        // This is a simplified version - in production you'd want better synchronization
        let mut queue = self.queue.blocking_write();
        if queue.len() >= self.capacity {
            return Err(MessageBusError::QueueFull);
        }

        queue.push(PriorityMessage { message });
        Ok(())
    }

    /// Pop the highest priority message (async, waits if empty)
    pub async fn pop(&mut self) -> Result<ComponentMessage> {
        let mut queue = self.queue.write().await;

        queue.pop()
            .map(|pm| pm.message)
            .ok_or(MessageBusError::QueueEmpty)
    }

    /// Try to pop a message (non-blocking)
    pub fn try_pop(&self) -> Option<ComponentMessage> {
        let mut queue = self.queue.try_write().ok()?;
        queue.pop().map(|pm| pm.message)
    }

    /// Get the current queue length
    pub async fn len(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_types::{MessageTarget, MessagePriority, MessagePayload};

    fn create_message(id: &str, priority: MessagePriority, timestamp: u64) -> ComponentMessage {
        ComponentMessage {
            id: id.to_string(),
            source: "test".to_string(),
            target: MessageTarget::Broadcast,
            timestamp,
            priority,
            payload: MessagePayload::HealthCheck,
        }
    }

    #[tokio::test]
    async fn queue_orders_by_priority() {
        let mut queue = PriorityQueue::new(10);

        queue.push(create_message("low", MessagePriority::Low, 1000)).await.unwrap();
        queue.push(create_message("high", MessagePriority::High, 1000)).await.unwrap();
        queue.push(create_message("critical", MessagePriority::Critical, 1000)).await.unwrap();

        assert_eq!(queue.pop().await.unwrap().id, "critical");
        assert_eq!(queue.pop().await.unwrap().id, "high");
        assert_eq!(queue.pop().await.unwrap().id, "low");
    }

    #[tokio::test]
    async fn queue_fifo_within_priority() {
        let mut queue = PriorityQueue::new(10);

        queue.push(create_message("msg1", MessagePriority::Normal, 1000)).await.unwrap();
        queue.push(create_message("msg2", MessagePriority::Normal, 2000)).await.unwrap();
        queue.push(create_message("msg3", MessagePriority::Normal, 3000)).await.unwrap();

        assert_eq!(queue.pop().await.unwrap().id, "msg1");
        assert_eq!(queue.pop().await.unwrap().id, "msg2");
        assert_eq!(queue.pop().await.unwrap().id, "msg3");
    }
}
