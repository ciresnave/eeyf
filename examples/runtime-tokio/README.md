# EEYF with Tokio Runtime Example

This example demonstrates using EEYF with the Tokio async runtime (default).

## Features

- Default runtime (no special configuration needed)
- Multi-threaded work-stealing scheduler
- Excellent performance and ecosystem support
- Battle-tested in production

## Running the Example

```bash
cd examples/runtime-tokio
cargo run
```

## Key Points

1. **Default Runtime**: Tokio is EEYF's default runtime, so no special features needed
2. **Multi-threaded**: Tokio uses a work-stealing scheduler for optimal performance
3. **#[tokio::main]**: Use this attribute macro for your main function
4. **Runtime Abstraction**: The example shows how to use EEYF's runtime abstraction layer

## Code Structure

```rust
#[tokio::main]
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

## Performance

Tokio provides:
- Excellent throughput
- Low latency
- Efficient resource usage
- Production-ready

## Learn More

- [Tokio Documentation](https://tokio.rs)
- [EEYF Runtime Guide](../../docs/RUNTIME.md)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
