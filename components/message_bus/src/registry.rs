// @implements: REQ-005
//! Component registry for managing registered components

use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use shared_types::ComponentMessage;
use crate::error::{MessageBusError, Result};

/// Component registry that tracks registered components and their message channels
pub struct ComponentRegistry {
    components: RwLock<HashMap<String, mpsc::Sender<ComponentMessage>>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Register a component with its message sender
    pub async fn register(
        &mut self,
        component_id: String,
        sender: mpsc::Sender<ComponentMessage>,
    ) -> Result<()> {
        let mut components = self.components.write().await;

        if components.contains_key(&component_id) {
            return Err(MessageBusError::ComponentAlreadyRegistered(component_id));
        }

        components.insert(component_id, sender);
        Ok(())
    }

    /// Deregister a component
    pub async fn deregister(&mut self, component_id: &str) -> Result<()> {
        let mut components = self.components.write().await;

        if components.remove(component_id).is_some() {
            Ok(())
        } else {
            Err(MessageBusError::ComponentNotFound(component_id.to_string()))
        }
    }

    /// Check if a component is registered
    pub async fn is_registered(&self, component_id: &str) -> bool {
        let components = self.components.read().await;
        components.contains_key(component_id)
    }

    /// Get the sender for a specific component
    pub async fn get_sender(&self, component_id: &str) -> Option<mpsc::Sender<ComponentMessage>> {
        let components = self.components.read().await;
        components.get(component_id).cloned()
    }

    /// List all registered component IDs
    pub async fn list_components(&self) -> Vec<String> {
        let components = self.components.read().await;
        components.keys().cloned().collect()
    }

    /// Get all senders (for broadcast)
    pub async fn get_all_senders(&self) -> Vec<mpsc::Sender<ComponentMessage>> {
        let components = self.components.read().await;
        components.values().cloned().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn registry_basic_operations() {
        let mut registry = ComponentRegistry::new();
        let (tx, _rx) = mpsc::channel(10);

        // Register
        assert!(registry.register("comp1".to_string(), tx.clone()).await.is_ok());
        assert!(registry.is_registered("comp1").await);

        // Duplicate registration fails
        assert!(registry.register("comp1".to_string(), tx).await.is_err());

        // Deregister
        assert!(registry.deregister("comp1").await.is_ok());
        assert!(!registry.is_registered("comp1").await);
    }
}
