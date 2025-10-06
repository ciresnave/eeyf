# EEYF with async-std Runtime Example

This example demonstrates using EEYF with the async-std runtime.

## Features

- Familiar std-like API
- Simple and intuitive
- Good for learning async Rust
- Cross-platform support

## Running the Example

```bash
cd examples/runtime-async-std
cargo run
```

## Configuration

**Important**: You must disable default features and enable `runtime-async-std`:

```toml
[dependencies]
eeyf = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
async-std = { version = "1.13", features = ["attributes"] }
```

## Key Points

1. **Disable Default Features**: Must use `default-features = false`
2. **Enable runtime-async-std**: Explicit feature flag required
3. **#[async_std::main]**: Use this attribute macro for your main function
4. **std-like API**: async-std mirrors the standard library's API

## Code Structure

```rust
#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connector = YahooConnector::new()?;
    let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
    // ...
}
```

## Runtime Abstraction

The example demonstrates using EEYF's runtime abstraction:

```rust
use eeyf::runtime;

// Works with any runtime!
let handle = runtime::spawn(async { 42 });
let result = handle.await?;

runtime::sleep(Duration::from_millis(100)).await;
```

## async-std Advantages

- **Familiar**: Mirrors std library API (`task::spawn`, `task::sleep`, etc.)
- **Simple**: Easy to learn and use
- **Cross-platform**: Works everywhere Rust works
- **Good docs**: Clear and beginner-friendly documentation

## Performance

async-std provides:
- Good throughput
- Reasonable latency
- Lower memory usage than Tokio
- Suitable for most applications

## Concurrent Tasks

async-std makes it easy to spawn concurrent tasks:

```rust
use async_std::task;

let handle = task::spawn(async {
    // Your async code
});

let result = handle.await;
```

## Learn More

- [async-std Documentation](https://async.rs)
- [EEYF Runtime Guide](../../docs/RUNTIME.md)
- [async-std Book](https://book.async.rs)
