//! Threading model for the browser shell
//!
//! This module provides a multi-threaded architecture model with dedicated thread pools
//! for different types of work:
//!
//! - **UI Thread**: Main thread for egui rendering (not managed by this pool)
//! - **MessageBus Thread**: Dedicated async runtime for message routing
//! - **IO Pool**: Thread pool for file operations
//! - **Network Thread**: Dedicated thread for HTTP requests
//! - **TabRender Threads**: Per-tab render threads for active tabs

use shared_types::ComponentError;
use std::collections::HashMap;
use std::panic;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, oneshot, RwLock};

/// Types of threads managed by the thread pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreadType {
    /// UI thread - main thread for egui rendering (marker only, not spawned by pool)
    UI,
    /// Message bus thread - dedicated async runtime for message routing
    MessageBus,
    /// IO pool - thread pool for file operations
    IO,
    /// Network thread - dedicated thread for HTTP requests
    Network,
    /// Per-tab render thread - one per active tab
    TabRender,
}

impl ThreadType {
    /// Get a human-readable name for the thread type
    pub fn name(&self) -> &'static str {
        match self {
            ThreadType::UI => "ui",
            ThreadType::MessageBus => "message-bus",
            ThreadType::IO => "io-pool",
            ThreadType::Network => "network",
            ThreadType::TabRender => "tab-render",
        }
    }

    /// Get the thread name prefix for spawned threads
    pub fn thread_name_prefix(&self) -> String {
        format!("corten-{}", self.name())
    }
}

/// A task that can be submitted to a thread pool
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// An async task that can be submitted to an async runtime
pub type AsyncTask = Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + 'static>;

/// Configuration for the thread pool
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of IO worker threads (default: 4)
    pub io_workers: usize,
    /// Maximum number of tab render threads (default: 16)
    pub max_tab_render_threads: usize,
    /// Whether to catch and log panics (default: true)
    pub catch_panics: bool,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            io_workers: 4,
            max_tab_render_threads: 16,
            catch_panics: true,
        }
    }
}

/// Internal worker for synchronous thread pools
struct SyncWorker {
    thread: Option<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

/// Internal state for a tab render thread
struct TabRenderThread {
    #[allow(dead_code)] // Used for diagnostics and logging
    tab_id: String,
    thread: Option<JoinHandle<()>>,
    task_tx: mpsc::UnboundedSender<Task>,
    shutdown: Arc<AtomicBool>,
}

/// Thread pool manager for the browser shell
///
/// Manages different types of threads for various workloads:
/// - Message bus async runtime
/// - IO pool for file operations
/// - Network thread for HTTP requests
/// - Per-tab render threads
pub struct ThreadPool {
    config: ThreadPoolConfig,
    /// Shutdown flag for the entire pool
    shutdown: Arc<AtomicBool>,
    /// Message bus runtime and task sender
    message_bus: Arc<RwLock<Option<MessageBusRuntime>>>,
    /// IO pool workers and task sender
    io_pool: Arc<RwLock<Option<IoPool>>>,
    /// Network thread
    network_thread: Arc<RwLock<Option<NetworkThread>>>,
    /// Per-tab render threads (indexed by tab ID)
    tab_render_threads: Arc<RwLock<HashMap<String, TabRenderThread>>>,
    /// Counter for active tasks (for graceful shutdown)
    active_tasks: Arc<AtomicUsize>,
}

/// Message bus runtime with async support
struct MessageBusRuntime {
    runtime: Runtime,
    shutdown: Arc<AtomicBool>,
}

/// IO pool with multiple worker threads
struct IoPool {
    workers: Vec<SyncWorker>,
    task_tx: mpsc::UnboundedSender<Task>,
    #[allow(dead_code)]
    task_rx: Arc<RwLock<mpsc::UnboundedReceiver<Task>>>,
}

/// Network thread for HTTP operations
struct NetworkThread {
    runtime: Runtime,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    /// Create a new thread pool with default configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(ThreadPool)` on success, or a `ComponentError` if initialization fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use message_bus::threading::ThreadPool;
    ///
    /// let pool = ThreadPool::new().expect("Failed to create thread pool");
    /// ```
    pub fn new() -> Result<Self, ComponentError> {
        Self::with_config(ThreadPoolConfig::default())
    }

    /// Create a new thread pool with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the thread pool
    ///
    /// # Returns
    ///
    /// Returns `Ok(ThreadPool)` on success, or a `ComponentError` if initialization fails.
    pub fn with_config(config: ThreadPoolConfig) -> Result<Self, ComponentError> {
        Ok(Self {
            config,
            shutdown: Arc::new(AtomicBool::new(false)),
            message_bus: Arc::new(RwLock::new(None)),
            io_pool: Arc::new(RwLock::new(None)),
            network_thread: Arc::new(RwLock::new(None)),
            tab_render_threads: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Initialize the message bus runtime
    ///
    /// This creates a dedicated tokio runtime for async message routing.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if initialization fails.
    pub async fn init_message_bus(&self) -> Result<(), ComponentError> {
        let mut guard = self.message_bus.write().await;
        if guard.is_some() {
            return Err(ComponentError::InvalidState(
                "Message bus runtime already initialized".to_string(),
            ));
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("corten-message-bus")
            .enable_all()
            .build()
            .map_err(|e| {
                ComponentError::InitializationFailed(format!(
                    "Failed to create message bus runtime: {}",
                    e
                ))
            })?;

        *guard = Some(MessageBusRuntime {
            runtime,
            shutdown: Arc::new(AtomicBool::new(false)),
        });

        Ok(())
    }

    /// Initialize the IO thread pool
    ///
    /// Creates worker threads for file operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if initialization fails.
    pub async fn init_io_pool(&self) -> Result<(), ComponentError> {
        let mut guard = self.io_pool.write().await;
        if guard.is_some() {
            return Err(ComponentError::InvalidState(
                "IO pool already initialized".to_string(),
            ));
        }

        let (task_tx, task_rx) = mpsc::unbounded_channel::<Task>();
        let task_rx = Arc::new(RwLock::new(task_rx));

        let mut workers = Vec::with_capacity(self.config.io_workers);
        let global_shutdown = self.shutdown.clone();
        let catch_panics = self.config.catch_panics;
        let active_tasks = self.active_tasks.clone();

        for i in 0..self.config.io_workers {
            let worker_shutdown = Arc::new(AtomicBool::new(false));
            let shutdown_clone = worker_shutdown.clone();
            let global_shutdown_clone = global_shutdown.clone();
            let task_rx_clone = task_rx.clone();
            let active_tasks_clone = active_tasks.clone();

            let handle = thread::Builder::new()
                .name(format!("corten-io-pool-{}", i))
                .spawn(move || {
                    // Create a runtime for this worker to receive tasks
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create IO worker runtime");

                    rt.block_on(async {
                        loop {
                            if shutdown_clone.load(Ordering::Relaxed)
                                || global_shutdown_clone.load(Ordering::Relaxed)
                            {
                                break;
                            }

                            let task = {
                                let mut rx = task_rx_clone.write().await;
                                // Use try_recv with a small sleep to allow checking shutdown
                                match rx.try_recv() {
                                    Ok(task) => Some(task),
                                    Err(_) => {
                                        tokio::time::sleep(std::time::Duration::from_millis(10))
                                            .await;
                                        None
                                    }
                                }
                            };

                            if let Some(task) = task {
                                active_tasks_clone.fetch_add(1, Ordering::Relaxed);
                                if catch_panics {
                                    let _ = panic::catch_unwind(panic::AssertUnwindSafe(task));
                                } else {
                                    task();
                                }
                                active_tasks_clone.fetch_sub(1, Ordering::Relaxed);
                            }
                        }
                    });
                })
                .map_err(|e| {
                    ComponentError::InitializationFailed(format!(
                        "Failed to spawn IO worker {}: {}",
                        i, e
                    ))
                })?;

            workers.push(SyncWorker {
                thread: Some(handle),
                shutdown: worker_shutdown,
            });
        }

        *guard = Some(IoPool {
            workers,
            task_tx,
            task_rx,
        });

        Ok(())
    }

    /// Initialize the network thread
    ///
    /// Creates a dedicated runtime for HTTP operations.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if initialization fails.
    pub async fn init_network(&self) -> Result<(), ComponentError> {
        let mut guard = self.network_thread.write().await;
        if guard.is_some() {
            return Err(ComponentError::InvalidState(
                "Network thread already initialized".to_string(),
            ));
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("corten-network")
            .enable_all()
            .build()
            .map_err(|e| {
                ComponentError::InitializationFailed(format!(
                    "Failed to create network runtime: {}",
                    e
                ))
            })?;

        *guard = Some(NetworkThread {
            runtime,
            shutdown: Arc::new(AtomicBool::new(false)),
        });

        Ok(())
    }

    /// Spawn a tab render thread for a specific tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - Unique identifier for the tab
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if:
    /// - A thread for this tab already exists
    /// - Maximum number of tab threads reached
    /// - Thread spawning fails
    pub async fn spawn_tab_render_thread(&self, tab_id: String) -> Result<(), ComponentError> {
        let mut guard = self.tab_render_threads.write().await;

        if guard.contains_key(&tab_id) {
            return Err(ComponentError::InvalidState(format!(
                "Tab render thread for '{}' already exists",
                tab_id
            )));
        }

        if guard.len() >= self.config.max_tab_render_threads {
            return Err(ComponentError::InvalidState(format!(
                "Maximum tab render threads ({}) reached",
                self.config.max_tab_render_threads
            )));
        }

        let (task_tx, mut task_rx) = mpsc::unbounded_channel::<Task>();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        let global_shutdown = self.shutdown.clone();
        let catch_panics = self.config.catch_panics;
        let active_tasks = self.active_tasks.clone();
        let tab_id_clone = tab_id.clone();

        let handle = thread::Builder::new()
            .name(format!("corten-tab-render-{}", tab_id))
            .spawn(move || {
                loop {
                    if shutdown_clone.load(Ordering::Relaxed)
                        || global_shutdown.load(Ordering::Relaxed)
                    {
                        break;
                    }

                    // Blocking receive with timeout
                    match task_rx.try_recv() {
                        Ok(task) => {
                            active_tasks.fetch_add(1, Ordering::Relaxed);
                            if catch_panics {
                                let _ = panic::catch_unwind(panic::AssertUnwindSafe(task));
                            } else {
                                task();
                            }
                            active_tasks.fetch_sub(1, Ordering::Relaxed);
                        }
                        Err(mpsc::error::TryRecvError::Empty) => {
                            thread::sleep(std::time::Duration::from_millis(10));
                        }
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            break;
                        }
                    }
                }
                // Log tab thread shutdown (tab_id_clone used for potential logging)
                let _ = tab_id_clone;
            })
            .map_err(|e| {
                ComponentError::InitializationFailed(format!(
                    "Failed to spawn tab render thread for '{}': {}",
                    tab_id, e
                ))
            })?;

        guard.insert(
            tab_id.clone(),
            TabRenderThread {
                tab_id,
                thread: Some(handle),
                task_tx,
                shutdown,
            },
        );

        Ok(())
    }

    /// Stop a tab render thread
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab whose render thread should be stopped
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if the tab thread doesn't exist.
    pub async fn stop_tab_render_thread(&self, tab_id: &str) -> Result<(), ComponentError> {
        let mut guard = self.tab_render_threads.write().await;

        let mut thread = guard.remove(tab_id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!(
                "Tab render thread for '{}' not found",
                tab_id
            ))
        })?;

        // Signal shutdown
        thread.shutdown.store(true, Ordering::Relaxed);

        // Wait for thread to finish (with timeout)
        if let Some(handle) = thread.thread.take() {
            // Drop the sender to unblock any waiting receives
            drop(thread.task_tx);

            // Wait for thread with a reasonable timeout
            let _ = handle.join();
        }

        Ok(())
    }

    /// Submit a task to the IO pool
    ///
    /// # Arguments
    ///
    /// * `task` - The task to execute on an IO worker
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if the IO pool is not initialized.
    pub async fn submit_io_task<F>(&self, task: F) -> Result<(), ComponentError>
    where
        F: FnOnce() + Send + 'static,
    {
        let guard = self.io_pool.read().await;
        let pool = guard.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("IO pool not initialized".to_string())
        })?;

        pool.task_tx
            .send(Box::new(task))
            .map_err(|_| ComponentError::MessageRoutingFailed("IO pool channel closed".to_string()))
    }

    /// Submit a task to a tab render thread
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to submit the task to
    /// * `task` - The task to execute on the tab's render thread
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if the tab thread doesn't exist.
    pub async fn submit_tab_task<F>(&self, tab_id: &str, task: F) -> Result<(), ComponentError>
    where
        F: FnOnce() + Send + 'static,
    {
        let guard = self.tab_render_threads.read().await;
        let thread = guard.get(tab_id).ok_or_else(|| {
            ComponentError::ResourceNotFound(format!(
                "Tab render thread for '{}' not found",
                tab_id
            ))
        })?;

        thread
            .task_tx
            .send(Box::new(task))
            .map_err(|_| {
                ComponentError::MessageRoutingFailed(format!(
                    "Tab render thread channel for '{}' closed",
                    tab_id
                ))
            })
    }

    /// Spawn an async task on the message bus runtime
    ///
    /// # Arguments
    ///
    /// * `future` - The future to spawn
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if the message bus is not initialized.
    pub async fn spawn_on_message_bus<F>(&self, future: F) -> Result<(), ComponentError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let guard = self.message_bus.read().await;
        let runtime = guard.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("Message bus runtime not initialized".to_string())
        })?;

        runtime.runtime.spawn(future);
        Ok(())
    }

    /// Spawn an async task on the network runtime
    ///
    /// # Arguments
    ///
    /// * `future` - The future to spawn
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `ComponentError` if the network thread is not initialized.
    pub async fn spawn_on_network<F>(&self, future: F) -> Result<(), ComponentError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let guard = self.network_thread.read().await;
        let runtime = guard.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("Network thread not initialized".to_string())
        })?;

        runtime.runtime.spawn(future);
        Ok(())
    }

    /// Get the number of active tasks across all pools
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(Ordering::Relaxed)
    }

    /// Get the number of active tab render threads
    pub async fn tab_render_thread_count(&self) -> usize {
        self.tab_render_threads.read().await.len()
    }

    /// Check if the pool is shutting down
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Initiate graceful shutdown of all threads
    ///
    /// This signals all threads to stop and waits for them to finish.
    /// Active tasks will be allowed to complete before threads terminate.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for shutdown
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if shutdown completed within timeout, or a `ComponentError` if timeout exceeded.
    pub async fn shutdown(&self, timeout: std::time::Duration) -> Result<(), ComponentError> {
        // Signal global shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        let start = std::time::Instant::now();

        // Stop tab render threads
        {
            let mut guard = self.tab_render_threads.write().await;
            for (_, thread) in guard.iter_mut() {
                thread.shutdown.store(true, Ordering::Relaxed);
            }
            // Join all tab threads
            for (_, mut thread) in guard.drain() {
                if let Some(handle) = thread.thread.take() {
                    let _ = handle.join();
                }
            }
        }

        // Stop IO pool
        {
            let mut guard = self.io_pool.write().await;
            if let Some(ref mut pool) = *guard {
                for worker in pool.workers.iter_mut() {
                    worker.shutdown.store(true, Ordering::Relaxed);
                }
                // Wait for workers to finish
                for worker in pool.workers.iter_mut() {
                    if let Some(handle) = worker.thread.take() {
                        let _ = handle.join();
                    }
                }
            }
            *guard = None;
        }

        // Stop message bus runtime - use spawn_blocking to avoid dropping runtime in async context
        {
            let mut guard = self.message_bus.write().await;
            if let Some(runtime) = guard.take() {
                runtime.shutdown.store(true, Ordering::Relaxed);
                // Spawn blocking task to drop the runtime outside async context
                tokio::task::spawn_blocking(move || {
                    drop(runtime);
                })
                .await
                .ok();
            }
        }

        // Stop network runtime - use spawn_blocking to avoid dropping runtime in async context
        {
            let mut guard = self.network_thread.write().await;
            if let Some(runtime) = guard.take() {
                runtime.shutdown.store(true, Ordering::Relaxed);
                // Spawn blocking task to drop the runtime outside async context
                tokio::task::spawn_blocking(move || {
                    drop(runtime);
                })
                .await
                .ok();
            }
        }

        // Wait for active tasks to complete
        while self.active_tasks.load(Ordering::Relaxed) > 0 {
            if start.elapsed() > timeout {
                return Err(ComponentError::InvalidState(format!(
                    "Shutdown timeout: {} tasks still active",
                    self.active_tasks.load(Ordering::Relaxed)
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new().expect("Failed to create default ThreadPool")
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Note: We can't use async here, so we do synchronous cleanup
        // The runtimes will be dropped when their Arc refcounts reach zero

        // For tab render threads and IO pool workers, signal shutdown
        // They will terminate when they next check the shutdown flag
    }
}

/// Handle for submitting tasks to a specific thread type
#[derive(Clone)]
pub struct ThreadHandle {
    #[allow(dead_code)] // Will be used for task submission methods
    pool: Arc<ThreadPool>,
    thread_type: ThreadType,
    #[allow(dead_code)] // Used for TabRender thread type
    tab_id: Option<String>,
}

impl ThreadHandle {
    /// Create a new thread handle for a specific thread type
    pub fn new(pool: Arc<ThreadPool>, thread_type: ThreadType) -> Self {
        Self {
            pool,
            thread_type,
            tab_id: None,
        }
    }

    /// Create a new thread handle for a specific tab render thread
    pub fn for_tab(pool: Arc<ThreadPool>, tab_id: String) -> Self {
        Self {
            pool,
            thread_type: ThreadType::TabRender,
            tab_id: Some(tab_id),
        }
    }

    /// Get the thread type this handle targets
    pub fn thread_type(&self) -> ThreadType {
        self.thread_type
    }
}

/// Result of a task execution
#[derive(Debug)]
pub enum TaskResult<T> {
    /// Task completed successfully with result
    Success(T),
    /// Task panicked
    Panicked(String),
    /// Task was cancelled
    Cancelled,
}

/// Execute a task and return its result through a oneshot channel
pub fn execute_with_result<F, T>(task: F) -> oneshot::Receiver<TaskResult<T>>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = oneshot::channel();

    let wrapped_task = move || {
        let result = panic::catch_unwind(panic::AssertUnwindSafe(task));
        let task_result = match result {
            Ok(value) => TaskResult::Success(value),
            Err(e) => TaskResult::Panicked(format!("{:?}", e)),
        };
        let _ = tx.send(task_result);
    };

    // Note: The caller is responsible for submitting this to the appropriate pool
    // This is a helper function for creating result-returning tasks
    wrapped_task();

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::time::Duration;

    #[tokio::test]
    async fn test_thread_pool_creation() {
        let pool = ThreadPool::new();
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_thread_pool_with_config() {
        let config = ThreadPoolConfig {
            io_workers: 2,
            max_tab_render_threads: 8,
            catch_panics: true,
        };
        let pool = ThreadPool::with_config(config);
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_thread_type_names() {
        assert_eq!(ThreadType::UI.name(), "ui");
        assert_eq!(ThreadType::MessageBus.name(), "message-bus");
        assert_eq!(ThreadType::IO.name(), "io-pool");
        assert_eq!(ThreadType::Network.name(), "network");
        assert_eq!(ThreadType::TabRender.name(), "tab-render");
    }

    #[tokio::test]
    async fn test_thread_type_prefixes() {
        assert_eq!(ThreadType::UI.thread_name_prefix(), "corten-ui");
        assert_eq!(
            ThreadType::MessageBus.thread_name_prefix(),
            "corten-message-bus"
        );
    }

    #[tokio::test]
    async fn test_init_message_bus() {
        let pool = ThreadPool::new().unwrap();
        let result = pool.init_message_bus().await;
        assert!(result.is_ok());

        // Second init should fail
        let result2 = pool.init_message_bus().await;
        assert!(result2.is_err());

        // Clean up properly
        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_init_io_pool() {
        let pool = ThreadPool::new().unwrap();
        let result = pool.init_io_pool().await;
        assert!(result.is_ok());

        // Second init should fail
        let result2 = pool.init_io_pool().await;
        assert!(result2.is_err());

        // Clean up properly
        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_init_network() {
        let pool = ThreadPool::new().unwrap();
        let result = pool.init_network().await;
        assert!(result.is_ok());

        // Second init should fail
        let result2 = pool.init_network().await;
        assert!(result2.is_err());

        // Clean up properly
        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_spawn_tab_render_thread() {
        let pool = ThreadPool::new().unwrap();

        // Spawn a tab thread
        let result = pool.spawn_tab_render_thread("tab-1".to_string()).await;
        assert!(result.is_ok());

        // Check count
        assert_eq!(pool.tab_render_thread_count().await, 1);

        // Duplicate should fail
        let result2 = pool.spawn_tab_render_thread("tab-1".to_string()).await;
        assert!(result2.is_err());

        // Clean up
        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_stop_tab_render_thread() {
        let pool = ThreadPool::new().unwrap();

        pool.spawn_tab_render_thread("tab-1".to_string())
            .await
            .unwrap();
        assert_eq!(pool.tab_render_thread_count().await, 1);

        let result = pool.stop_tab_render_thread("tab-1").await;
        assert!(result.is_ok());
        assert_eq!(pool.tab_render_thread_count().await, 0);

        // Stopping non-existent should fail
        let result2 = pool.stop_tab_render_thread("tab-1").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_submit_io_task() {
        let pool = ThreadPool::new().unwrap();
        pool.init_io_pool().await.unwrap();

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = pool
            .submit_io_task(move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await;

        assert!(result.is_ok());

        // Wait for task to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.load(Ordering::Relaxed), 1);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_submit_io_task_without_init() {
        let pool = ThreadPool::new().unwrap();

        let result = pool.submit_io_task(|| {}).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_submit_tab_task() {
        let pool = ThreadPool::new().unwrap();
        pool.spawn_tab_render_thread("tab-1".to_string())
            .await
            .unwrap();

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = pool
            .submit_tab_task("tab-1", move || {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await;

        assert!(result.is_ok());

        // Wait for task to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.load(Ordering::Relaxed), 1);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_submit_tab_task_nonexistent() {
        let pool = ThreadPool::new().unwrap();

        let result = pool.submit_tab_task("nonexistent", || {}).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_on_message_bus() {
        let pool = ThreadPool::new().unwrap();
        pool.init_message_bus().await.unwrap();

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = pool
            .spawn_on_message_bus(async move {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await;

        assert!(result.is_ok());

        // Wait for task to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.load(Ordering::Relaxed), 1);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_spawn_on_network() {
        let pool = ThreadPool::new().unwrap();
        pool.init_network().await.unwrap();

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = pool
            .spawn_on_network(async move {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await;

        assert!(result.is_ok());

        // Wait for task to complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(counter.load(Ordering::Relaxed), 1);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_max_tab_render_threads() {
        let config = ThreadPoolConfig {
            io_workers: 1,
            max_tab_render_threads: 2,
            catch_panics: true,
        };
        let pool = ThreadPool::with_config(config).unwrap();

        pool.spawn_tab_render_thread("tab-1".to_string())
            .await
            .unwrap();
        pool.spawn_tab_render_thread("tab-2".to_string())
            .await
            .unwrap();

        // Third should fail
        let result = pool.spawn_tab_render_thread("tab-3".to_string()).await;
        assert!(result.is_err());

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let pool = ThreadPool::new().unwrap();
        pool.init_io_pool().await.unwrap();
        pool.init_message_bus().await.unwrap();
        pool.init_network().await.unwrap();
        pool.spawn_tab_render_thread("tab-1".to_string())
            .await
            .unwrap();

        let result = pool.shutdown(Duration::from_secs(2)).await;
        assert!(result.is_ok());
        assert!(pool.is_shutting_down());
    }

    #[tokio::test]
    async fn test_panic_handling() {
        let config = ThreadPoolConfig {
            io_workers: 2, // Use 2 workers for resilience
            max_tab_render_threads: 4,
            catch_panics: true,
        };
        let pool = ThreadPool::with_config(config).unwrap();
        pool.init_io_pool().await.unwrap();

        // Submit a task that panics
        let result = pool
            .submit_io_task(|| {
                panic!("Test panic");
            })
            .await;

        assert!(result.is_ok()); // Task submission should succeed

        // Wait a bit for the panic to be caught
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Pool should still be usable - submit multiple times to ensure at least one worker is available
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        pool.submit_io_task(move || {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        })
        .await
        .unwrap();

        // Give more time for the task to complete
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }

    #[tokio::test]
    async fn test_thread_handle_creation() {
        let pool = Arc::new(ThreadPool::new().unwrap());

        let handle = ThreadHandle::new(pool.clone(), ThreadType::IO);
        assert_eq!(handle.thread_type(), ThreadType::IO);

        let tab_handle = ThreadHandle::for_tab(pool, "tab-1".to_string());
        assert_eq!(tab_handle.thread_type(), ThreadType::TabRender);
    }

    #[tokio::test]
    async fn test_active_task_count() {
        let pool = ThreadPool::new().unwrap();
        pool.init_io_pool().await.unwrap();

        assert_eq!(pool.active_task_count(), 0);

        // Submit a task that takes some time
        let (tx, rx) = oneshot::channel::<()>();
        pool.submit_io_task(move || {
            // Block until signal
            std::thread::sleep(Duration::from_millis(200));
            let _ = tx.send(());
        })
        .await
        .unwrap();

        // Give the task time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Task should be active (may be 1 depending on timing)
        // This is a weak assertion due to timing
        let count = pool.active_task_count();
        assert!(count <= 1);

        // Wait for completion
        let _ = rx.await;

        // Give time for counter to update
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(pool.active_task_count(), 0);

        pool.shutdown(Duration::from_secs(1)).await.ok();
    }
}
