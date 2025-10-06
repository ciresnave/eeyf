//! Runtime abstraction layer for EEYF
//!
//! This module provides a runtime-agnostic interface that allows EEYF to work
//! with different async runtimes (tokio, async-std, smol) without code changes.
//!
//! # Features
//!
//! - `runtime-tokio` (default): Use Tokio runtime
//! - `runtime-async-std`: Use async-std runtime
//! - `runtime-smol`: Use smol runtime
//!
//! # Examples
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use eeyf::runtime::{Runtime, sleep, spawn};
//!
//! async fn example() {
//!     // Spawn a task (works with any runtime)
//!     let handle = spawn(async {
//!         println!("Running on current runtime");
//!     });
//!
//!     // Sleep (works with any runtime)
//!     sleep(Duration::from_secs(1)).await;
//!
//!     // Wait for task
//!     handle.await.ok();
//! }
//! ```

use std::{future::Future, time::Duration};

/// Runtime abstraction trait
///
/// Provides a uniform interface for different async runtimes
pub trait Runtime: Send + Sync + 'static {
    /// Task join handle type
    type JoinHandle<T>: Future<Output = Result<T, JoinError>> + Send
    where
        T: Send + 'static;

    /// Spawn a task on this runtime
    fn spawn<F, T>(&self, future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    /// Spawn a blocking task
    fn spawn_blocking<F, T>(&self, f: F) -> Self::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;

    /// Sleep for a duration
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;

    /// Get current runtime name
    fn name(&self) -> &'static str;

    /// Check if runtime is available
    fn is_available() -> bool;
}

/// Join error type
#[derive(Debug)]
pub enum JoinError {
    /// Task was cancelled
    Cancelled,
    /// Task panicked
    Panic(Box<dyn std::any::Any + Send>),
    /// Runtime-specific error
    Runtime(String),
}

impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "Task was cancelled"),
            Self::Panic(_) => write!(f, "Task panicked"),
            Self::Runtime(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for JoinError {}

/// Spawn a task on the current runtime
pub fn spawn<F, T>(future: F) -> impl Future<Output = Result<T, JoinError>> + Send
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    #[cfg(feature = "runtime-tokio")]
    {
        tokio_runtime::TokioRuntime.spawn(future)
    }

    #[cfg(all(feature = "runtime-async-std", not(feature = "runtime-tokio")))]
    {
        async_std_runtime::AsyncStdRuntime.spawn(future)
    }

    #[cfg(all(
        feature = "runtime-smol",
        not(feature = "runtime-tokio"),
        not(feature = "runtime-async-std")
    ))]
    {
        smol_runtime::SmolRuntime.spawn(future)
    }
}

/// Spawn a blocking task on the current runtime
pub fn spawn_blocking<F, T>(f: F) -> impl Future<Output = Result<T, JoinError>> + Send
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    #[cfg(feature = "runtime-tokio")]
    {
        tokio_runtime::TokioRuntime.spawn_blocking(f)
    }

    #[cfg(all(feature = "runtime-async-std", not(feature = "runtime-tokio")))]
    {
        async_std_runtime::AsyncStdRuntime.spawn_blocking(f)
    }

    #[cfg(all(
        feature = "runtime-smol",
        not(feature = "runtime-tokio"),
        not(feature = "runtime-async-std")
    ))]
    {
        smol_runtime::SmolRuntime.spawn_blocking(f)
    }
}

/// Sleep for a duration on the current runtime
pub async fn sleep(duration: Duration) {
    #[cfg(feature = "runtime-tokio")]
    {
        tokio_runtime::TokioRuntime.sleep(duration).await
    }

    #[cfg(all(feature = "runtime-async-std", not(feature = "runtime-tokio")))]
    {
        async_std_runtime::AsyncStdRuntime.sleep(duration).await
    }

    #[cfg(all(
        feature = "runtime-smol",
        not(feature = "runtime-tokio"),
        not(feature = "runtime-async-std")
    ))]
    {
        smol_runtime::SmolRuntime.sleep(duration).await
    }
}

/// Get the name of the current runtime
pub fn runtime_name() -> &'static str {
    #[cfg(feature = "runtime-tokio")]
    {
        "tokio"
    }

    #[cfg(all(feature = "runtime-async-std", not(feature = "runtime-tokio")))]
    {
        "async-std"
    }

    #[cfg(all(
        feature = "runtime-smol",
        not(feature = "runtime-tokio"),
        not(feature = "runtime-async-std")
    ))]
    {
        "smol"
    }
}

// Tokio runtime adapter
#[cfg(feature = "runtime-tokio")]
pub mod tokio_runtime {
    use super::*;

    /// Tokio runtime adapter
    pub struct TokioRuntime;

    /// Tokio join handle wrapper
    pub struct TokioJoinHandle<T>(tokio::task::JoinHandle<T>);

    impl<T> Future for TokioJoinHandle<T>
    where
        T: Send + 'static,
    {
        type Output = Result<T, JoinError>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            use std::pin::Pin;
            match Pin::new(&mut self.0).poll(cx) {
                std::task::Poll::Ready(Ok(value)) => std::task::Poll::Ready(Ok(value)),
                std::task::Poll::Ready(Err(e)) if e.is_cancelled() => {
                    std::task::Poll::Ready(Err(JoinError::Cancelled))
                },
                std::task::Poll::Ready(Err(e)) if e.is_panic() => {
                    std::task::Poll::Ready(Err(JoinError::Panic(e.into_panic())))
                },
                std::task::Poll::Ready(Err(e)) => {
                    std::task::Poll::Ready(Err(JoinError::Runtime(e.to_string())))
                },
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }

    impl Runtime for TokioRuntime {
        type JoinHandle<T>
            = TokioJoinHandle<T>
        where
            T: Send + 'static;

        fn spawn<F, T>(&self, future: F) -> Self::JoinHandle<T>
        where
            F: Future<Output = T> + Send + 'static,
            T: Send + 'static,
        {
            TokioJoinHandle(tokio::task::spawn(future))
        }

        fn spawn_blocking<F, T>(&self, f: F) -> Self::JoinHandle<T>
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            TokioJoinHandle(tokio::task::spawn_blocking(f))
        }

        async fn sleep(&self, duration: Duration) {
            tokio::time::sleep(duration).await
        }

        fn name(&self) -> &'static str {
            "tokio"
        }

        fn is_available() -> bool {
            tokio::runtime::Handle::try_current().is_ok()
        }
    }
}

// async-std runtime adapter
#[cfg(feature = "runtime-async-std")]
pub mod async_std_runtime {
    use super::*;

    /// async-std runtime adapter
    pub struct AsyncStdRuntime;

    /// async-std join handle wrapper
    pub struct AsyncStdJoinHandle<T>(async_std::task::JoinHandle<T>);

    impl<T> Future for AsyncStdJoinHandle<T>
    where
        T: Send + 'static,
    {
        type Output = Result<T, JoinError>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            use std::pin::Pin;
            match Pin::new(&mut self.0).poll(cx) {
                std::task::Poll::Ready(value) => std::task::Poll::Ready(Ok(value)),
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }

    impl Runtime for AsyncStdRuntime {
        type JoinHandle<T>
            = AsyncStdJoinHandle<T>
        where
            T: Send + 'static;

        fn spawn<F, T>(&self, future: F) -> Self::JoinHandle<T>
        where
            F: Future<Output = T> + Send + 'static,
            T: Send + 'static,
        {
            AsyncStdJoinHandle(async_std::task::spawn(future))
        }

        fn spawn_blocking<F, T>(&self, f: F) -> Self::JoinHandle<T>
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            AsyncStdJoinHandle(async_std::task::spawn_blocking(f))
        }

        async fn sleep(&self, duration: Duration) {
            async_std::task::sleep(duration).await
        }

        fn name(&self) -> &'static str {
            "async-std"
        }

        fn is_available() -> bool {
            // async-std doesn't have a way to check runtime availability
            true
        }
    }
}

// smol runtime adapter
#[cfg(feature = "runtime-smol")]
pub mod smol_runtime {
    use super::*;

    /// smol runtime adapter
    pub struct SmolRuntime;

    /// smol join handle wrapper
    pub struct SmolJoinHandle<T>(smol::Task<T>);

    impl<T> Future for SmolJoinHandle<T>
    where
        T: Send + 'static,
    {
        type Output = Result<T, JoinError>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            use std::pin::Pin;
            match Pin::new(&mut self.0).poll(cx) {
                std::task::Poll::Ready(value) => std::task::Poll::Ready(Ok(value)),
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }

    impl Runtime for SmolRuntime {
        type JoinHandle<T>
            = SmolJoinHandle<T>
        where
            T: Send + 'static;

        fn spawn<F, T>(&self, future: F) -> Self::JoinHandle<T>
        where
            F: Future<Output = T> + Send + 'static,
            T: Send + 'static,
        {
            SmolJoinHandle(smol::spawn(future))
        }

        fn spawn_blocking<F, T>(&self, f: F) -> Self::JoinHandle<T>
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            SmolJoinHandle(smol::unblock(f))
        }

        async fn sleep(&self, duration: Duration) {
            smol::Timer::after(duration).await
        }

        fn name(&self) -> &'static str {
            "smol"
        }

        fn is_available() -> bool {
            // smol doesn't have a way to check runtime availability
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "runtime-tokio")]
    async fn test_tokio_runtime() {
        assert_eq!(runtime_name(), "tokio");

        let handle = spawn(async { 42 });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);

        sleep(Duration::from_millis(10)).await;
    }

    #[async_std::test]
    #[cfg(feature = "runtime-async-std")]
    async fn test_async_std_runtime() {
        assert_eq!(runtime_name(), "async-std");

        let handle = spawn(async { 42 });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);

        sleep(Duration::from_millis(10)).await;
    }

    #[test]
    #[cfg(feature = "runtime-smol")]
    fn test_smol_runtime() {
        smol::block_on(async {
            assert_eq!(runtime_name(), "smol");

            let handle = spawn(async { 42 });
            let result = handle.await.unwrap();
            assert_eq!(result, 42);

            sleep(Duration::from_millis(10)).await;
        });
    }

    #[tokio::test]
    #[cfg(feature = "runtime-tokio")]
    async fn test_spawn_blocking() {
        let handle = spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        });
        let result = handle.await.unwrap();
        assert_eq!(result, 42);
    }
}
