# Basic EEYF Application Template

A simple template for building applications with the EEYF library.

## Project Structure

```
basic-app/
├── Cargo.toml
├── src/
│   └── main.rs
└── README.md
```

## Getting Started

1. **Copy this template**:
   ```bash
   cp -r templates/basic-app my-app
   cd my-app
   ```

2. **Update `Cargo.toml`** with your project details

3. **Build and run**:
   ```bash
   cargo run
   ```

## Features Used

This template uses the basic EEYF features:
- Quote fetching
- Symbol search
- Error handling
- Basic rate limiting

## Example Usage

The template demonstrates:
1. Fetching latest quotes
2. Getting historical data
3. Searching for symbols
4. Handling errors gracefully

## Customization

To add more features, update `Cargo.toml`:

```toml
[dependencies]
eeyf = { version = "0.1.0", features = ["decimal", "metrics", "tracing"] }
```

Available features:
- `decimal` - High-precision decimal numbers
- `metrics` - Performance metrics
- `tracing` - Distributed tracing
- `performance-cache` - Advanced caching
- `websocket-streaming` - Real-time data streaming

## Next Steps

- Check out the [enterprise template](../enterprise-app/) for production features
- Read the [full documentation](../../docs/)
- Explore more [examples](../../examples/)
