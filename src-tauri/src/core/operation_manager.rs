use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

/// Manages active operations and their cancellation tokens
#[derive(Clone)]
pub struct OperationManager {
    operations: Arc<RwLock<HashMap<String, CancellationToken>>>,
}

impl OperationManager {
    /// Create a new operation manager
    pub fn new() -> Self {
        Self {
            operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new operation and get its cancellation token
    pub async fn register_operation(&self, operation_id: String) -> CancellationToken {
        let token = CancellationToken::new();
        let mut ops = self.operations.write().await;
        ops.insert(operation_id.clone(), token.clone());
        tracing::info!("Registered operation: {}", operation_id);
        token
    }

    /// Cancel an operation by ID
    pub async fn cancel_operation(&self, operation_id: &str) -> bool {
        let ops = self.operations.read().await;
        if let Some(token) = ops.get(operation_id) {
            tracing::info!("Cancelling operation: {}", operation_id);
            token.cancel();
            true
        } else {
            tracing::warn!("Operation not found: {}", operation_id);
            false
        }
    }

    /// Unregister an operation (called when operation completes)
    pub async fn unregister_operation(&self, operation_id: &str) {
        let mut ops = self.operations.write().await;
        ops.remove(operation_id);
        tracing::info!("Unregistered operation: {}", operation_id);
    }

    /// Check if an operation is cancelled
    pub async fn is_cancelled(&self, operation_id: &str) -> bool {
        let ops = self.operations.read().await;
        if let Some(token) = ops.get(operation_id) {
            token.is_cancelled()
        } else {
            false
        }
    }

    /// Get all active operation IDs
    pub async fn get_active_operations(&self) -> Vec<String> {
        let ops = self.operations.read().await;
        ops.keys().cloned().collect()
    }

    /// Clear all operations (useful for cleanup)
    pub async fn clear_all(&self) {
        let mut ops = self.operations.write().await;
        for (id, token) in ops.iter() {
            tracing::info!("Cancelling operation during cleanup: {}", id);
            token.cancel();
        }
        ops.clear();
    }
}

impl Default for OperationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_cancel() {
        let manager = OperationManager::new();
        let op_id = "test-op-1".to_string();
        
        let token = manager.register_operation(op_id.clone()).await;
        assert!(!token.is_cancelled());
        
        let cancelled = manager.cancel_operation(&op_id).await;
        assert!(cancelled);
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_unregister() {
        let manager = OperationManager::new();
        let op_id = "test-op-2".to_string();
        
        manager.register_operation(op_id.clone()).await;
        manager.unregister_operation(&op_id).await;
        
        let cancelled = manager.cancel_operation(&op_id).await;
        assert!(!cancelled); // Should return false as operation doesn't exist
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let manager = OperationManager::new();
        
        let token1 = manager.register_operation("op1".to_string()).await;
        let token2 = manager.register_operation("op2".to_string()).await;
        
        manager.cancel_operation("op1").await;
        
        assert!(token1.is_cancelled());
        assert!(!token2.is_cancelled());
    }
}
