# EEYF CLI Tool

Command-line interface for the EEYF Yahoo Finance library.

## Installation

Build the CLI tool with:

```bash
cargo build --bin eeyf --features "cli-tool,decimal" --release
```

The binary will be available at `target/release/eeyf` (or `eeyf.exe` on Windows).

## Usage

### Get Quote Data

Fetch current or historical quote data for a stock symbol:

```bash
# Get latest 5 days of daily quotes (default)
eeyf quote AAPL

# Get 1 month of hourly quotes
eeyf quote AAPL -i 1h -r 1mo

# Get quotes in JSON format
eeyf quote MSFT -f json

# Export to CSV file
eeyf quote GOOGL -f csv -o quotes.csv
```

### Search for Symbols

Search for ticker symbols by company name:

```bash
# Search for Apple
eeyf search Apple

# Limit results
eeyf search "Microsoft" -l 5
```

### Get Company Information

Get detailed information about a company:

```bash
eeyf info AAPL
```

### Export Historical Data

Export historical data to a file:

```bash
# Export to CSV
eeyf export AAPL -s 2024-01-01 -e 2024-12-31 -f csv -o aapl_2024.csv

# Export to JSON
eeyf export MSFT -s 2023-01-01 -e 2023-12-31 -f json -o msft_2023.json
```

### Test Rate Limiting

Test the library's rate limiting capabilities:

```bash
# Make 20 requests and see rate limiting in action
eeyf rate-limit -c 20 -s AAPL
```

### Interactive Mode

Enter interactive mode for exploring data:

```bash
eeyf interactive
```

In interactive mode, you can use these commands:
- `quote <SYMBOL> [interval] [range]` - Fetch quotes
- `search <QUERY>` - Search for symbols
- `info <SYMBOL>` - Get symbol information
- `help` - Show available commands
- `exit` - Exit interactive mode

## Examples

### Example 1: Daily Analysis

Get the last 30 days of daily data and save to CSV:

```bash
eeyf quote AAPL -i 1d -r 1mo -f csv -o aapl_daily.csv
```

### Example 2: Intraday Trading

Get minute-by-minute data for the current day:

```bash
eeyf quote SPY -i 1m -r 1d -f table
```

### Example 3: Batch Export

Export multiple symbols (using a shell script):

```bash
for symbol in AAPL MSFT GOOGL AMZN; do
  eeyf export $symbol -s 2024-01-01 -e 2024-12-31 -o "${symbol}_2024.csv"
done
```

### Example 4: Market Screening

Use interactive mode to quickly explore multiple stocks:

```bash
eeyf interactive

> quote AAPL
> quote MSFT
> search tech
> exit
```

## Command Reference

### `quote` - Fetch Quote Data

```bash
eeyf quote <SYMBOL> [OPTIONS]
```

**Options:**
- `-i, --interval <INTERVAL>` - Time interval (1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo) [default: 1d]
- `-r, --range <RANGE>` - Time range (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max) [default: 5d]
- `-f, --format <FORMAT>` - Output format (json, csv, table) [default: table]
- `-o, --output <FILE>` - Output file (optional)

### `search` - Search for Symbols

```bash
eeyf search <QUERY> [OPTIONS]
```

**Options:**
- `-l, --limit <LIMIT>` - Maximum number of results [default: 10]

### `export` - Export Historical Data

```bash
eeyf export <SYMBOL> [OPTIONS]
```

**Options:**
- `-s, --start <DATE>` - Start date (YYYY-MM-DD)
- `-e, --end <DATE>` - End date (YYYY-MM-DD)
- `-f, --format <FORMAT>` - Output format (csv, json) [default: csv]
- `-o, --output <FILE>` - Output file

### `rate-limit` - Test Rate Limiting

```bash
eeyf rate-limit [OPTIONS]
```

**Options:**
- `-c, --count <COUNT>` - Number of requests to make [default: 10]
- `-s, --symbol <SYMBOL>` - Symbol to fetch [default: AAPL]

### `info` - Get Symbol Information

```bash
eeyf info <SYMBOL>
```

### `interactive` - Interactive Mode

```bash
eeyf interactive
```

### `cache-stats` - Get Cache Statistics

```bash
eeyf cache-stats
```

Note: Requires `performance-cache` feature to be enabled.

### `cache-clear` - Clear Cache

```bash
eeyf cache-clear
```

Note: Requires `performance-cache` feature to be enabled.

### `circuit-status` - Get Circuit Breaker Status

```bash
eeyf circuit-status [SERVICE]
```

Note: Requires enterprise features to be enabled.

## Output Formats

### Table Format (default)

Human-readable table format:

```
Date         Open       High       Low        Close      Volume
----------------------------------------------------------------------
2024-01-02   180.50     182.30     179.80     181.90     52000000
2024-01-03   182.00     184.50     181.50     183.75     54000000
```

### CSV Format

Comma-separated values for spreadsheet import:

```csv
timestamp,date,open,high,low,close,volume,adjclose
1704153600,2024-01-02,180.50,182.30,179.80,181.90,52000000,181.90
```

### JSON Format

Machine-readable JSON format:

```json
[
  {
    "timestamp": 1704153600,
    "open": 180.50,
    "high": 182.30,
    "low": 179.80,
    "close": 181.90,
    "volume": 52000000,
    "adjclose": 181.90
  }
]
```

## Tips

1. **Use table format** for quick visual inspection
2. **Use CSV format** for Excel/spreadsheet analysis
3. **Use JSON format** for programmatic processing
4. **Test rate limits** before bulk operations
5. **Use interactive mode** for exploration

## Troubleshooting

### Rate Limiting

If you see rate limit errors, reduce the number of requests or add delays between calls.

### Network Errors

Check your internet connection and firewall settings.

### Invalid Symbol

Make sure the symbol exists using the `search` command first.

## See Also

- [EEYF Library Documentation](../README.md)
- [API Reference](https://docs.rs/eeyf)
- [Examples](../examples/)
