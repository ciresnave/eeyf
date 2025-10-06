//! Graceful shutdown handling
//!
//! This module provides graceful shutdown capabilities to ensure all
//! pending operations complete and resources are cleaned up properly.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::timeout;

/// Shutdown signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSignal {
    /// SIGTERM or Ctrl+C
    Terminate,
    /// SIGINT
    Interrupt,
    /// Manual shutdown request
    Manual,
}

/// Shutdown state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownState {
    /// Running normally
    Running,
    /// Shutdown initiated, draining requests
    Draining,
    /// Shutdown complete
    Stopped,
}

/// Shutdown coordinator for graceful shutdown
pub struct ShutdownCoordinator {
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<ShutdownSignal>,
    
    /// Current shutdown state
    state: Arc<RwLock<ShutdownState>>,
    
    /// Pending operations count
    pending_operations: Arc<RwLock<usize>>,
    
    /// Shutdown timeout
    timeout: Duration,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new(timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(100);
        
        Self {
            shutdown_tx,
            state: Arc::new(RwLock::new(ShutdownState::Running)),
            pending_operations: Arc::new(RwLock::new(0)),
            timeout,
        }
    }
    
    /// Subscribe to shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownSignal> {
        self.shutdown_tx.subscribe()
    }
    
    /// Initiate graceful shutdown
    pub async fn shutdown(&self, signal: ShutdownSignal) -> Result<(), String> {
        // Update state to draining
        {
            let mut state = self.state.write().await;
            if *state != ShutdownState::Running {
                return Err("Shutdown already in progress".to_string());
            }
            *state = ShutdownState::Draining;
        }
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(signal);
        
        // Wait for pending operations to complete with timeout
        let drain_result = timeout(self.timeout, self.drain_pending_operations()).await;
        
        match drain_result {
            Ok(_) => {
                let mut state = self.state.write().await;
                *state = ShutdownState::Stopped;
                Ok(())
            }
            Err(_) => {
                let mut state = self.state.write().await;
                *state = ShutdownState::Stopped;
                Err(format!(
                    "Shutdown timeout exceeded, {} operations incomplete",
                    *self.pending_operations.read().await
                ))
            }
        }
    }
    
    /// Wait for all pending operations to complete
    async fn drain_pending_operations(&self) {
        loop {
            let pending = *self.pending_operations.read().await;
            if pending == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Register a new pending operation
    pub async fn register_operation(&self) -> bool {
        let state = self.state.read().await;
        if *state != ShutdownState::Running {
            return false;
        }
        
        let mut pending = self.pending_operations.write().await;
        *pending += 1;
        true
    }
    
    /// Unregister a completed operation
    pub async fn unregister_operation(&self) {
        let mut pending = self.pending_operations.write().await;
        if *pending > 0 {
            *pending -= 1;
        }
    }
    
    /// Get current shutdown state
    pub async fn state(&self) -> ShutdownState {
        *self.state.read().await
    }
    
    /// Get pending operations count
    pub async fn pending_operations(&self) -> usize {
        *self.pending_operations.read().await
    }
    
    /// Check if shutdown is in progress
    pub async fn is_shutting_down(&self) -> bool {
        let state = self.state.read().await;
        *state != ShutdownState::Running
    }
}

/// RAII guard for automatic operation registration/unregistration
pub struct OperationGuard {
    coordinator: Arc<ShutdownCoordinator>,
}

impl OperationGuard {
    /// Create a new operation guard
    pub async fn new(coordinator: Arc<ShutdownCoordinator>) -> Option<Self> {
        if coordinator.register_operation().await {
            Some(Self { coordinator })
        } else {
            None
        }
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let coordinator = self.coordinator.clone();
        tokio::spawn(async move {
            coordinator.unregister_operation().await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_shutdown_coordinator_creation() {
        let coordinator = ShutdownCoordinator::new(Duration::from_secs(5));
        assert_eq!(coordinator.state().await, ShutdownState::Running);
        assert_eq!(coordinator.pending_operations().await, 0);
    }
    
    #[tokio::test]
    async fn test_operation_registration() {
        let coordinator = ShutdownCoordinator::new(Duration::from_secs(5));
        
        assert!(coordinator.register_operation().await);
        assert_eq!(coordinator.pending_operations().await, 1);
        
        coordinator.unregister_operation().await;
        assert_eq!(coordinator.pending_operations().await, 0);
    }
    
    #[tokio::test]
    async fn test_shutdown_blocks_new_operations() {
        let coordinator = Arc::new(ShutdownCoordinator::new(Duration::from_secs(5)));
        
        // Start shutdown
        let shutdown_handle = {
            let coordinator = coordinator.clone();
            tokio::spawn(async move {
                coordinator.shutdown(ShutdownSignal::Manual).await
            })
        };
        
        // Wait a bit for shutdown to initiate
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // New operations should be rejected
        assert!(!coordinator.register_operation().await);
        
        shutdown_handle.await.unwrap().unwrap();
    }
    
    #[tokio::test]
    async fn test_shutdown_waits_for_operations() {
        let coordinator = Arc::new(ShutdownCoordinator::new(Duration::from_secs(5)));
        
        // Register operations
        assert!(coordinator.register_operation().await);
        assert!(coordinator.register_operation().await);
        assert_eq!(coordinator.pending_operations().await, 2);
        
        // Start shutdown in background
        let shutdown_coordinator = coordinator.clone();
        let shutdown_handle = tokio::spawn(async move {
            shutdown_coordinator.shutdown(ShutdownSignal::Manual).await
        });
        
        // Shutdown should be draining
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(coordinator.state().await, ShutdownState::Draining);
        
        // Complete operations
        coordinator.unregister_operation().await;
        coordinator.unregister_operation().await;
        
        // Shutdown should complete
        let result = shutdown_handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(coordinator.state().await, ShutdownState::Stopped);
    }
    
    #[tokio::test]
    async fn test_shutdown_timeout() {
        let coordinator = Arc::new(ShutdownCoordinator::new(Duration::from_millis(200)));
        
        // Register operation that won't complete
        assert!(coordinator.register_operation().await);
        
        // Shutdown should timeout
        let result = coordinator.shutdown(ShutdownSignal::Manual).await;
        assert!(result.is_err());
        assert_eq!(coordinator.state().await, ShutdownState::Stopped);
    }
    
    #[tokio::test]
    async fn test_shutdown_signal_broadcast() {
        let coordinator = ShutdownCoordinator::new(Duration::from_secs(5));
        let mut receiver = coordinator.subscribe();
        
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = coordinator.shutdown(ShutdownSignal::Terminate).await;
        });
        
        let signal = receiver.recv().await.unwrap();
        assert_eq!(signal, ShutdownSignal::Terminate);
    }
    
    #[tokio::test]
    async fn test_multiple_shutdown_attempts() {
        let coordinator = Arc::new(ShutdownCoordinator::new(Duration::from_secs(5)));
        
        // First shutdown should succeed
        let result = coordinator.shutdown(ShutdownSignal::Manual).await;
        assert!(result.is_ok());
        
        // Second shutdown should fail
        let coordinator2 = coordinator.clone();
        let result = coordinator2.shutdown(ShutdownSignal::Manual).await;
        assert!(result.is_err());
    }
}
