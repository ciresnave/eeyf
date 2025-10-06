---
name: Bug Report
about: Create a report to help us improve EEYF
title: '[BUG] '
labels: bug
assignees: ''
---

## Bug Description

A clear and concise description of what the bug is.

## To Reproduce

Steps to reproduce the behavior:

1. Create client with `...`
2. Call method `...`
3. With parameters `...`
4. See error

## Code Example

```rust
// Minimal reproducible example
use eeyf::YahooFinanceClient;

#[tokio::main]
async fn main() {
    let client = YahooFinanceClient::builder()
        .build()
        .await
        .unwrap();
    
    // Your code that produces the bug
}
```

## Expected Behavior

A clear and concise description of what you expected to happen.

## Actual Behavior

What actually happened, including any error messages.

```
Error message here
```

## Environment

- **EEYF Version**: [e.g., 0.1.5]
- **Rust Version**: [e.g., 1.75.0]
- **OS**: [e.g., Windows 11, Ubuntu 22.04, macOS 14]
- **Features Enabled**: [e.g., `default`, `decimal`, `phase5`]
- **Async Runtime**: [e.g., tokio 1.35, async-std 1.12]

## Additional Context

Add any other context about the problem here:
- Stack trace (if available)
- Related issues
- Workarounds you've tried
- Frequency of the issue

## Checklist

- [ ] I have searched existing issues to ensure this is not a duplicate
- [ ] I have provided a minimal reproducible example
- [ ] I have included my environment details
- [ ] I have checked the documentation and troubleshooting guide
