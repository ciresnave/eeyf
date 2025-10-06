# Runtime Selection Guide

EEYF supports multiple async runtimes, allowing you to choose the best runtime for your application.

---

## Table of Contents

1. [Supported Runtimes](#supported-runtimes)
2. [Runtime Selection](#runtime-selection)
3. [Feature Flags](#feature-flags)
4. [Runtime Comparison](#runtime-comparison)
5. [Migration Guide](#migration-guide)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

---

## Supported Runtimes

EEYF supports three popular async runtimes:

### 1. Tokio (Default)

**Best for**: General-purpose applications, production workloads

```toml
[dependencies]
eeyf = { version = "0.1", features = ["runtime-tokio"] }
```

**Pros**:
- Most mature and battle-tested
- Excellent performance
- Rich ecosystem
- Great documentation
- Work-stealing scheduler
- Multi-threaded by default

**Cons**:
- Larger binary size
- More complex API
- Higher memory usage

### 2. async-std

**Best for**: Learning async Rust, simpler applications

```toml
[dependencies]
eeyf = { version = "0.1", features = ["runtime-async-std"] }
```

**Pros**:
- Mirrors std library API (familiar)
- Simple and intuitive
- Good for beginners
- Smaller learning curve

**Cons**:
- Less mature than Tokio
- Smaller ecosystem
- Fewer optimization options

### 3. smol

**Best for**: Embedded systems, resource-constrained environments

```toml
[dependencies]
eeyf = { version = "0.1", features = ["runtime-smol"] }
```

**Pros**:
- Minimal footprint
- Lightweight and fast
- Simple implementation
- Low memory usage
- Great for embedded

**Cons**:
- Smaller ecosystem
- Less documentation
- Manual executor management
- Single-threaded by default

---

## Runtime Selection

### Default Runtime (Tokio)

By default, EEYF uses Tokio:

```toml
[dependencies]
eeyf = "0.1"
```

### Explicit Runtime Selection

Choose a specific runtime:

```toml
[dependencies]
eeyf = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
```

### No Runtime Conflicts

Only one runtime feature should be enabled at a time. EEYF will compile-error if multiple runtime features are active.

---

## Feature Flags

### Runtime Features

| Feature Flag        | Runtime   | Default |
| ------------------- | --------- | ------- |
| `runtime-tokio`     | Tokio     | ✅ Yes   |
| `runtime-async-std` | async-std | ❌ No    |
| `runtime-smol`      | smol      | ❌ No    |

### Usage Examples

#### Tokio (Default)

```toml
[dependencies]
eeyf = "0.1"
```

```rust
use eeyf::YahooConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = YahooConnector::new()?;
    let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
    println!("{:?}", quotes);
    Ok(())
}
```

#### async-std

```toml
[dependencies]
eeyf = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
```

```rust
use eeyf::YahooConnector;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = YahooConnector::new()?;
    let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
    println!("{:?}", quotes);
    Ok(())
}
```

#### smol

```toml
[dependencies]
eeyf = { version = "0.1", default-features = false, features = ["runtime-smol"] }
```

```rust
use eeyf::YahooConnector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    smol::block_on(async {
        let connector = YahooConnector::new()?;
        let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
        println!("{:?}", quotes);
        Ok(())
    })
}
```

---

## Runtime Comparison

### Performance

| Runtime       | Throughput | Latency | Memory | Binary Size |
| ------------- | ---------- | ------- | ------ | ----------- |
| **Tokio**     | ⭐⭐⭐⭐⭐      | ⭐⭐⭐⭐⭐   | ⭐⭐⭐    | ⭐⭐          |
| **async-std** | ⭐⭐⭐⭐       | ⭐⭐⭐⭐    | ⭐⭐⭐⭐   | ⭐⭐⭐         |
| **smol**      | ⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐   | ⭐⭐⭐⭐⭐  | ⭐⭐⭐⭐⭐       |

### Feature Support

| Feature        | Tokio | async-std | smol      |
| -------------- | ----- | --------- | --------- |
| Multi-threaded | ✅ Yes | ✅ Yes     | ⚠️ Manual  |
| Work stealing  | ✅ Yes | ✅ Yes     | ❌ No      |
| Blocking tasks | ✅ Yes | ✅ Yes     | ✅ Yes     |
| Timers         | ✅ Yes | ✅ Yes     | ✅ Yes     |
| Channels       | ✅ Yes | ✅ Yes     | ✅ Yes     |
| Filesystem     | ✅ Yes | ✅ Yes     | ⚠️ Limited |
| Process        | ✅ Yes | ✅ Yes     | ⚠️ Limited |
| Signals        | ✅ Yes | ✅ Yes     | ❌ No      |

### Ecosystem

| Aspect              | Tokio       | async-std | smol   |
| ------------------- | ----------- | --------- | ------ |
| Maturity            | Very High   | High      | Medium |
| Docs                | Excellent   | Good      | Fair   |
| Crates.io downloads | 100M+       | 10M+      | 1M+    |
| Active development  | Very Active | Active    | Active |
| Community size      | Large       | Medium    | Small  |

---

## Migration Guide

### From Tokio to async-std

1. Update `Cargo.toml`:

```diff
[dependencies]
-eeyf = "0.1"
+eeyf = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
-tokio = { version = "1", features = ["full"] }
+async-std = { version = "1", features = ["attributes"] }
```

2. Update main function:

```diff
-#[tokio::main]
+#[async_std::main]
async fn main() {
    // Your code (no changes needed)
}
```

3. Update runtime-specific code:

```diff
-tokio::time::sleep(Duration::from_secs(1)).await;
+async_std::task::sleep(Duration::from_secs(1)).await;

-tokio::spawn(async { /* ... */ });
+async_std::task::spawn(async { /* ... */ });
```

### From Tokio to smol

1. Update `Cargo.toml`:

```diff
[dependencies]
-eeyf = "0.1"
+eeyf = { version = "0.1", default-features = false, features = ["runtime-smol"] }
-tokio = { version = "1", features = ["full"] }
+smol = "2"
```

2. Update main function:

```diff
-#[tokio::main]
-async fn main() {
+fn main() {
+    smol::block_on(async {
        // Your code (no changes needed)
+    })
}
```

3. Update runtime-specific code:

```diff
-tokio::time::sleep(Duration::from_secs(1)).await;
+smol::Timer::after(Duration::from_secs(1)).await;

-tokio::spawn(async { /* ... */ });
+smol::spawn(async { /* ... */ }).detach();
```

### From async-std to smol

1. Update `Cargo.toml`:

```diff
[dependencies]
-eeyf = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
+eeyf = { version = "0.1", default-features = false, features = ["runtime-smol"] }
-async-std = { version = "1", features = ["attributes"] }
+smol = "2"
```

2. Update main function:

```diff
-#[async_std::main]
-async fn main() {
+fn main() {
+    smol::block_on(async {
        // Your code (no changes needed)
+    })
}
```

---

## Best Practices

### 1. Choose Based on Requirements

**Use Tokio if**:
- Building production applications
- Need maximum performance
- Want rich ecosystem
- Using many async crates (most use Tokio)

**Use async-std if**:
- Learning async Rust
- Want familiar std-like API
- Building simpler applications
- Prefer ease of use over performance

**Use smol if**:
- Building embedded applications
- Have memory constraints
- Need minimal binary size
- Want simple, lightweight runtime

### 2. Runtime Abstraction

EEYF provides runtime-agnostic APIs, so your code works with any runtime:

```rust
use eeyf::runtime::{spawn, sleep};
use std::time::Duration;

async fn example() {
    // This works with any runtime!
    let handle = spawn(async {
        sleep(Duration::from_secs(1)).await;
        42
    });

    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

### 3. Avoid Runtime-Specific Code

Don't use runtime-specific APIs directly. Use EEYF's runtime abstraction:

```rust
// ❌ Bad: Runtime-specific
tokio::time::sleep(Duration::from_secs(1)).await;

// ✅ Good: Runtime-agnostic
use eeyf::runtime::sleep;
sleep(Duration::from_secs(1)).await;
```

### 4. Test with Multiple Runtimes

If your application may run on different runtimes:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
async-std = { version = "1", features = ["attributes"] }
smol = "2"
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "runtime-tokio")]
    async fn test_with_tokio() {
        // Test with Tokio
    }

    #[async_std::test]
    #[cfg(feature = "runtime-async-std")]
    async fn test_with_async_std() {
        // Test with async-std
    }

    #[test]
    #[cfg(feature = "runtime-smol")]
    fn test_with_smol() {
        smol::block_on(async {
            // Test with smol
        });
    }
}
```

---

## Troubleshooting

### Multiple Runtime Features Enabled

**Error**:
```
error: Only one runtime feature can be enabled at a time
```

**Solution**: Ensure only one runtime feature is active:

```toml
# ❌ Bad
eeyf = { version = "0.1", features = ["runtime-tokio", "runtime-async-std"] }

# ✅ Good
eeyf = { version = "0.1", features = ["runtime-async-std"] }
```

### Runtime Not Available

**Error**:
```
Cannot spawn task: no runtime available
```

**Solution**: Ensure you're running inside an async runtime:

```rust
// ❌ Bad: No runtime
fn main() {
    let connector = YahooConnector::new().unwrap();
    // Error: Can't call async function
}

// ✅ Good: Inside runtime
#[tokio::main]
async fn main() {
    let connector = YahooConnector::new().unwrap();
    // Works!
}
```

### Mixing Runtimes

**Problem**: Using Tokio types with async-std runtime

**Solution**: Use EEYF's runtime-agnostic APIs:

```rust
// ❌ Bad: Runtime-specific
use tokio::time::sleep;

// ✅ Good: Runtime-agnostic
use eeyf::runtime::sleep;
```

### Performance Issues

**Symptom**: Slow performance with smol

**Solution**: Use multi-threaded executor:

```rust
use smol::Executor;
use std::thread;

fn main() {
    let ex = Executor::new();

    // Spawn worker threads
    for _ in 0..num_cpus::get() {
        thread::spawn({
            let ex = ex.clone();
            move || smol::block_on(ex.run(smol::future::pending::<()>()))
        });
    }

    // Run your app
    smol::block_on(ex.run(async {
        // Your code
    }));
}
```

---

## Runtime Detection

Check which runtime is being used:

```rust
use eeyf::runtime::runtime_name;

println!("Using runtime: {}", runtime_name());
// Outputs: "tokio", "async-std", or "smol"
```

---

## Advanced: Custom Runtime

You can implement the `Runtime` trait for custom runtimes:

```rust
use eeyf::runtime::{Runtime, JoinError};
use std::future::Future;
use std::time::Duration;

struct MyRuntime;

impl Runtime for MyRuntime {
    type JoinHandle<T> = MyJoinHandle<T> where T: Send + 'static;

    fn spawn<F, T>(&self, future: F) -> Self::JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Your spawn implementation
    }

    fn spawn_blocking<F, T>(&self, f: F) -> Self::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        // Your spawn_blocking implementation
    }

    async fn sleep(&self, duration: Duration) {
        // Your sleep implementation
    }

    fn name(&self) -> &'static str {
        "my-runtime"
    }

    fn is_available() -> bool {
        true
    }
}
```

---

## Resources

- [Tokio Documentation](https://tokio.rs)
- [async-std Documentation](https://async.rs)
- [smol Documentation](https://docs.rs/smol)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [EEYF Runtime API](../src/runtime.rs)

---

## Summary

| Choose        | If You Need                                                 |
| ------------- | ----------------------------------------------------------- |
| **Tokio**     | Production-grade performance, rich ecosystem, battle-tested |
| **async-std** | Familiar std-like API, ease of use, learning async Rust     |
| **smol**      | Minimal footprint, embedded systems, resource constraints   |

All three runtimes work seamlessly with EEYF. Choose based on your specific requirements!
