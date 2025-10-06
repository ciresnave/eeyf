# Fuzzing Setup for EEYF

This document provides instructions for fuzzing the EEYF library to discover edge cases and potential vulnerabilities.

## Overview

Fuzzing is an automated testing technique that provides invalid, unexpected, or random data as inputs to find bugs, crashes, and security vulnerabilities.

## Tool: cargo-fuzz

We use **cargo-fuzz** which leverages LibFuzzer for coverage-guided fuzzing.

### Installation

```powershell
cargo install cargo-fuzz
```

## Project Structure

```
eeyf/
├── fuzz/
│   ├── Cargo.toml
│   └── fuzz_targets/
│       ├── fuzz_json_parsing.rs
│       ├── fuzz_symbol_validation.rs
│       ├── fuzz_error_handling.rs
│       └── fuzz_url_construction.rs
```

## Initialize Fuzzing

```powershell
cargo fuzz init
```

This creates the `fuzz/` directory with initial setup.

## Fuzz Targets

### 1. JSON Parsing Fuzzer

Tests the robustness of JSON parsing against malformed inputs.

**File**: `fuzz/fuzz_targets/fuzz_json_parsing.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as JSON
    if let Ok(s) = std::str::from_utf8(data) {
        let _: Result<serde_json::Value, _> = serde_json::from_str(s);
    }
});
```

**Run**:
```powershell
cargo fuzz run fuzz_json_parsing
```

### 2. Symbol Validation Fuzzer

Tests symbol string handling and validation logic.

**File**: `fuzz/fuzz_targets/fuzz_symbol_validation.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(symbol) = std::str::from_utf8(data) {
        // Test symbol validation logic
        let is_valid = symbol.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-');
        
        // Test URL construction doesn't panic
        let _ = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
        
        // Test symbol trimming/normalization
        let normalized = symbol.trim().to_uppercase();
        let _ = normalized.len();
    }
});
```

**Run**:
```powershell
cargo fuzz run fuzz_symbol_validation
```

### 3. Error Handling Fuzzer

Tests error creation and handling with arbitrary inputs.

**File**: `fuzz/fuzz_targets/fuzz_error_handling.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use eeyf::{YahooError, ErrorContext};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test error creation with arbitrary strings
        let _err1 = YahooError::FetchFailed(s.to_string());
        let _err2 = YahooError::ConnectionFailed(s.to_string());
        let _err3 = YahooError::DeserializeFailed(s.to_string());
        
        // Test error context with arbitrary data
        let context = ErrorContext::new()
            .with_symbol(s)
            .with_endpoint(s)
            .with_request_id(s)
            .with_metadata("key", s);
        
        // Test error display doesn't panic
        let err = YahooError::TooManyRequests(s.to_string());
        let with_context = err.with_context(context);
        let _ = format!("{}", with_context);
    }
});
```

**Run**:
```powershell
cargo fuzz run fuzz_error_handling
```

### 4. URL Construction Fuzzer

Tests URL building logic for potential injection or malformation issues.

**File**: `fuzz/fuzz_targets/fuzz_url_construction.rs`

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test URL construction with arbitrary input
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", s);
        
        // Test URL parsing
        let _ = url::Url::parse(&url);
        
        // Test query parameter encoding
        let encoded = percent_encoding::utf8_percent_encode(
            s, 
            percent_encoding::NON_ALPHANUMERIC
        );
        let _ = encoded.to_string();
    }
});
```

**Run**:
```powershell
cargo fuzz run fuzz_url_construction
```

## Running Fuzz Tests

### Single Target

```powershell
# Run for 60 seconds
cargo fuzz run fuzz_json_parsing -- -max_total_time=60

# Run with specific number of inputs
cargo fuzz run fuzz_symbol_validation -- -runs=1000000

# Run with custom memory limit (MB)
cargo fuzz run fuzz_error_handling -- -rss_limit_mb=2048
```

### All Targets

```powershell
# List all targets
cargo fuzz list

# Run each target for 5 minutes
foreach ($target in (cargo fuzz list)) {
    cargo fuzz run $target -- -max_total_time=300
}
```

## Corpus Management

Fuzzing generates a corpus of interesting inputs in `fuzz/corpus/<target>/`.

### Using the Corpus

```powershell
# Run with existing corpus
cargo fuzz run fuzz_json_parsing

# Minimize corpus (keep only unique coverage-increasing inputs)
cargo fuzz cmin fuzz_json_parsing

# Merge multiple corpora
cargo fuzz cmin -merge fuzz_json_parsing corpus1/ corpus2/
```

### Seeding the Corpus

Create initial interesting inputs:

```powershell
# Create corpus directory
New-Item -ItemType Directory -Path fuzz/corpus/fuzz_json_parsing -Force

# Add seed inputs
@'
{"chart":{"result":[{"meta":{"symbol":"AAPL"}}]}}
'@ | Out-File -Encoding UTF8 fuzz/corpus/fuzz_json_parsing/valid.json

@'
{"invalid json
'@ | Out-File -Encoding UTF8 fuzz/corpus/fuzz_json_parsing/malformed.json
```

## Analyzing Results

### Crash Artifacts

When a crash is found, it's saved to `fuzz/artifacts/<target>/`:

```powershell
# List crashes
Get-ChildItem fuzz/artifacts/fuzz_json_parsing/

# Reproduce a crash
cargo fuzz run fuzz_json_parsing fuzz/artifacts/fuzz_json_parsing/crash-abc123
```

### Coverage Reports

```powershell
# Generate coverage report
cargo fuzz coverage fuzz_json_parsing

# View coverage in browser
cargo fuzz coverage --html fuzz_json_parsing
explorer fuzz/coverage/fuzz_json_parsing/html/index.html
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Fuzzing

on:
  schedule:
    - cron: '0 0 * * *'  # Run daily
  workflow_dispatch:

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_json_parsing
          - fuzz_symbol_validation
          - fuzz_error_handling
          - fuzz_url_construction
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust nightly
        uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          override: true
      
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      
      - name: Run fuzzer
        run: |
          cargo fuzz run ${{ matrix.target }} -- \
            -max_total_time=600 \
            -rss_limit_mb=4096
      
      - name: Upload artifacts
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-artifacts-${{ matrix.target }}
          path: fuzz/artifacts/${{ matrix.target }}/
```

## Best Practices

### 1. Start with Seed Corpus

Provide initial valid inputs to guide fuzzing toward interesting mutations.

### 2. Run Continuously

Fuzzing finds bugs over time. Run it continuously in CI or on dedicated machines.

### 3. Minimize Corpus Regularly

```powershell
cargo fuzz cmin fuzz_json_parsing
```

This keeps the corpus small and fast to run.

### 4. Focus on Parsing and External Input

Prioritize fuzzing any code that:
- Parses external data (JSON, URLs)
- Handles user input (symbols, parameters)
- Performs string manipulation
- Makes decisions based on input

### 5. Monitor Memory Usage

Use `--rss_limit_mb` to prevent OOM:

```powershell
cargo fuzz run fuzz_json_parsing -- -rss_limit_mb=2048
```

### 6. Parallelize

Run multiple fuzz jobs in parallel:

```powershell
cargo fuzz run fuzz_json_parsing -- -jobs=8
```

## Troubleshooting

### "command not found: cargo-fuzz"

Install cargo-fuzz:
```powershell
cargo +nightly install cargo-fuzz
```

### Slow fuzzing

- Simplify the target code
- Use release mode: `cargo fuzz run --release <target>`
- Increase `-jobs` for parallelization

### No crashes found

This is good! But also:
- Add more diverse seed inputs
- Run longer (hours/days)
- Check coverage: `cargo fuzz coverage <target>`

## Advanced Usage

### Dictionary

Create a dictionary file for fuzzing with known keywords:

**File**: `fuzz/dictionaries/json.dict`
```
"chart"
"result"
"meta"
"symbol"
"AAPL"
"timestamp"
"quote"
```

Use it:
```powershell
cargo fuzz run fuzz_json_parsing -- -dict=fuzz/dictionaries/json.dict
```

### Custom Mutators

For structured fuzzing, implement custom mutators in your fuzz target:

```rust
use libfuzzer_sys::arbitrary::{Arbitrary, Unstructured};

#[derive(Debug)]
struct FuzzInput {
    symbol: String,
    interval: String,
    range: String,
}

impl<'a> Arbitrary<'a> for FuzzInput {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, libfuzzer_sys::arbitrary::Error> {
        Ok(FuzzInput {
            symbol: u.arbitrary()?,
            interval: u.choose(&["1m", "5m", "1d", "1wk", "1mo"])?.to_string(),
            range: u.choose(&["1d", "5d", "1mo", "1y"])?.to_string(),
        })
    }
}

fuzz_target!(|input: FuzzInput| {
    // Use structured input
});
```

## Expected Findings

Common issues fuzzing discovers:
- Panic on malformed JSON
- Integer overflow/underflow
- Infinite loops
- Memory leaks
- Assertion failures
- Improper error handling

## Next Steps

1. **Initialize fuzzing**: `cargo fuzz init`
2. **Create targets**: Add fuzz_targets as shown above
3. **Run initial fuzzing**: `cargo fuzz run <target> -- -max_total_time=300`
4. **Fix any crashes**: Reproduce and fix issues found
5. **Add to CI**: Automate fuzzing in GitHub Actions
6. **Run continuously**: Set up long-running fuzz jobs

## Resources

- [cargo-fuzz documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [LibFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [AFL++ (alternative fuzzer)](https://github.com/AFLplusplus/AFLplusplus)
