// File: crates/icn-core/src/shutdown.rs
//
// This module implements a graceful shutdown system for the ICN network node.
// It coordinates the orderly shutdown of all system components, ensuring proper
// cleanup and resource release.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use futures::future::join_all;

use crate::error::{Error, Result};

/// Default timeout duration for component shutdown
const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum number of shutdown signals that can be buffered
const MAX_SHUTDOWN_SIGNALS: usize = 16;

/// Manages the graceful shutdown of system components
#[derive(Debug)]
pub struct ShutdownManager {
    /// Flag indicating if shutdown has been initiated
    shutdown_flag: AtomicBool,
    
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,
    
    /// List of registered shutdown handlers
    handlers: Mutex<Vec<Box<dyn ShutdownHandler>>>,
    
    /// Maximum time to wait for shutdown operations
    shutdown_timeout: Duration,
}

/// Trait for components that need shutdown handling
#[async_trait::async_trait]
pub trait ShutdownHandler: Send + Sync {
    /// Get the name of the component for logging
    fn name(&self) -> &str;
    
    /// Perform shutdown operations for this component
    async fn shutdown(&self) -> Result<()>;
    
    /// Get the shutdown priority (lower numbers shut down first)
    fn priority(&self) -> i32 {
        0 // Default priority
    }
}

impl ShutdownManager {
    /// Creates a new shutdown manager with default settings
    pub fn new() -> Self {
        Self::with_timeout(DEFAULT_SHUTDOWN_TIMEOUT)
    }

    /// Creates a new shutdown manager with a custom timeout duration
    pub fn with_timeout(timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(MAX_SHUTDOWN_SIGNALS);
        
        Self {
            shutdown_flag: AtomicBool::new(false),
            shutdown_tx,
            handlers: Mutex::new(Vec::new()),
            shutdown_timeout: timeout,
        }
    }

    /// Registers a new shutdown handler
    pub async fn register_handler<H>(&self, handler: H) -> Result<()>
    where
        H: ShutdownHandler + 'static,
    {
        let mut handlers = self.handlers.lock().await;
        handlers.push(Box::new(handler));
        
        // Sort handlers by priority
        handlers.sort_by_key(|h| h.priority());
        
        debug!("Registered shutdown handler: {}", handler.name());
        Ok(())
    }

    /// Gets a receiver for shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Checks if shutdown has been initiated
    pub fn is_shutdown_initiated(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }

    /// Initiates system shutdown
    pub async fn initiate_shutdown(&self) -> Result<()> {
        // Check if shutdown already initiated
        if self.shutdown_flag.swap(true, Ordering::SeqCst) {
            debug!("Shutdown already initiated");
            return Ok(());
        }

        info!("Initiating system shutdown");

        // Send shutdown signal to all subscribers
        if let Err(e) = self.shutdown_tx.send(()) {
            warn!("Failed to broadcast shutdown signal: {}", e);
        }

        // Get handlers
        let mut handlers = self.handlers.lock().await;

        // Create shutdown futures for all handlers
        let futures: Vec<_> = handlers.iter().map(|handler| {
            let handler_name = handler.name().to_string();
            async move {
                match timeout(self.shutdown_timeout, handler.shutdown()).await {
                    Ok(result) => {
                        if let Err(e) = result {
                            error!(
                                "Error shutting down component {}: {}",
                                handler_name,
                                e
                            );
                        } else {
                            debug!("Successfully shut down component {}", handler_name);
                        }
                    }
                    Err(_) => {
                        error!(
                            "Timeout shutting down component {} after {:?}",
                            handler_name,
                            self.shutdown_timeout
                        );
                    }
                }
            }
        }).collect();

        // Execute all shutdown futures concurrently
        join_all(futures).await;

        info!("System shutdown complete");
        Ok(())
    }

    /// Waits for a shutdown signal
    pub async fn wait_for_shutdown(&self) -> Result<()> {
        let mut rx = self.subscribe();
        rx.recv().await.map_err(|e| {
            Error::system(format!("Failed to receive shutdown signal: {}", e))
        })?;
        Ok(())
    }

    /// Sets a new shutdown timeout duration
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.shutdown_timeout = timeout;
    }

    /// Gets the current shutdown timeout duration
    pub fn get_timeout(&self) -> Duration {
        self.shutdown_timeout
    }
}

/// Example component implementing a prioritized shutdown
#[derive(Debug)]
pub struct PrioritizedComponent {
    /// Component name
    name: String,
    
    /// Shutdown priority
    priority: i32,
    
    /// Time needed for cleanup
    cleanup_duration: Duration,
}

impl PrioritizedComponent {
    /// Creates a new prioritized component
    pub fn new(
        name: impl Into<String>,
        priority: i32,
        cleanup_duration: Duration,
    ) -> Self {
        Self {
            name: name.into(),
            priority,
            cleanup_duration,
        }
    }
}

#[async_trait::async_trait]
impl ShutdownHandler for PrioritizedComponent {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Starting shutdown for component {} (priority {})", self.name, self.priority);
        tokio::time::sleep(self.cleanup_duration).await;
        info!("Completed shutdown for component {}", self.name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_shutdown_handler_registration() {
        let manager = Arc::new(ShutdownManager::new());
        
        let component = PrioritizedComponent::new(
            "test-component",
            0,
            Duration::from_millis(100)
        );
        
        assert!(manager.register_handler(component).await.is_ok());
        
        let handlers = manager.handlers.lock().await;
        assert_eq!(handlers.len(), 1);
    }

    #[tokio::test]
    async fn test_shutdown_priority_ordering() {
        let manager = Arc::new(ShutdownManager::new());
        
        // Register components with different priorities
        let component1 = PrioritizedComponent::new(
            "high-priority",
            1,
            Duration::from_millis(100)
        );
        let component2 = PrioritizedComponent::new(
            "low-priority",
            2,
            Duration::from_millis(100)
        );
        
        // Register in reverse priority order
        manager.register_handler(component2).await.unwrap();
        manager.register_handler(component1).await.unwrap();
        
        let handlers = manager.handlers.lock().await;
        assert_eq!(handlers[0].priority(), 1); // High priority should be first
        assert_eq!(handlers[1].priority(), 2);
    }

    #[tokio::test]
    async fn test_shutdown_signal_broadcast() {
        let manager = Arc::new(ShutdownManager::new());
        let mut rx1 = manager.subscribe();
        let mut rx2 = manager.subscribe();
        
        // Start shutdown in background
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            manager_clone.initiate_shutdown().await.unwrap();
        });
        
        // Both receivers should get the signal
        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let manager = ShutdownManager::with_timeout(Duration::from_millis(100));
        
        // Register a slow component
        let slow_component = PrioritizedComponent::new(
            "slow-component",
            0,
            Duration::from_secs(1)
        );
        
        manager.register_handler(slow_component).await.unwrap();
        
        // Shutdown should complete despite slow component
        assert!(manager.initiate_shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_shutdown_attempts() {
        let manager = Arc::new(ShutdownManager::new());
        
        // First shutdown should succeed
        assert!(manager.initiate_shutdown().await.is_ok());
        assert!(manager.is_shutdown_initiated());
        
        // Second shutdown should be no-op
        assert!(manager.initiate_shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_shutdown() {
        let manager = Arc::new(ShutdownManager::new());
        let manager_clone = manager.clone();
        
        // Start waiting for shutdown in background
        let wait_handle = tokio::spawn(async move {
            manager_clone.wait_for_shutdown().await
        });
        
        // Initiate shutdown after delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        manager.initiate_shutdown().await.unwrap();
        
        // Wait task should complete
        assert!(wait_handle.await.unwrap().is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_component_shutdown() {
        let manager = Arc::new(ShutdownManager::new());
        
        // Register multiple components with same priority
        for i in 0..3 {
            let component = PrioritizedComponent::new(
                format!("component-{}", i),
                0,
                Duration::from_millis(100)
            );
            manager.register_handler(component).await.unwrap();
        }
        
        let start = std::time::Instant::now();
        manager.initiate_shutdown().await.unwrap();
        
        // All components should shut down concurrently
        assert!(start.elapsed() < Duration::from_millis(300));
    }
}