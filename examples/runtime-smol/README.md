# EEYF with smol Runtime Example

This example demonstrates using EEYF with the smol async runtime.

## Features

- Minimal footprint and binary size
- Lightweight and fast
- Great for embedded systems
- Low memory usage

## Running the Example

```bash
cd examples/runtime-smol
cargo run
```

## Configuration

**Important**: You must disable default features and enable `runtime-smol`:

```toml
[dependencies]
eeyf = { version = "0.1", default-features = false, features = ["runtime-smol"] }
smol = "2.0"
```

## Key Points

1. **Disable Default Features**: Must use `default-features = false`
2. **Enable runtime-smol**: Explicit feature flag required
3. **smol::block_on**: Manually run the executor in main function
4. **Lightweight**: Minimal dependencies and small binary size

## Code Structure

```rust
fn main() -> Result<(), Box<dyn Error>> {
    smol::block_on(async {
        let connector = YahooConnector::new()?;
        let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
        // ...
        Ok(())
    })
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

## smol Advantages

- **Minimal**: Small binary size, few dependencies
- **Fast**: Lightweight executor with low overhead
- **Simple**: Straightforward API and implementation
- **Flexible**: Easy to customize and embed

## Performance

smol provides:
- Excellent latency (minimal overhead)
- Good throughput
- Very low memory usage
- Smallest binary size

## Task Spawning

smol makes it easy to spawn tasks:

```rust
use smol;

let handle = smol::spawn(async {
    // Your async code
});

let result = handle.await;
```

## Blocking Operations

For CPU-bound or blocking operations, use `smol::unblock`:

```rust
let result = smol::unblock(|| {
    // Blocking operation
    std::thread::sleep(Duration::from_secs(1));
    42
}).await;
```

## Use Cases

smol is ideal for:
- **Embedded systems**: Limited resources
- **CLI tools**: Fast startup time
- **Microservices**: Small memory footprint
- **Learning**: Simple and understandable code

## Binary Size Comparison

Approximate release binary sizes:
- **Tokio**: ~2-3 MB
- **async-std**: ~1.5-2 MB
- **smol**: ~1-1.5 MB

## Learn More

- [smol Documentation](https://docs.rs/smol)
- [EEYF Runtime Guide](../../docs/RUNTIME.md)
- [smol GitHub](https://github.com/smol-rs/smol)
