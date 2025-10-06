# Trading Bot Template

A production-ready algorithmic trading bot template built with EEYF.

## Features

- ✅ Real-time market data via WebSocket
- ✅ Strategy pattern for easy algorithm swapping
- ✅ Risk management and position sizing
- ✅ Performance tracking and reporting
- ✅ PostgreSQL for trade history
- ✅ Prometheus metrics
- ✅ Graceful shutdown handling
- ✅ Comprehensive error handling
- ✅ Structured logging with tracing

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Prometheus (optional, for metrics)

### Setup

1. **Clone and navigate**:
   ```bash
   cp -r templates/trading-bot ~/my-trading-bot
   cd ~/my-trading-bot
   ```

2. **Create environment file**:
   ```bash
   cp .env.example .env
   nano .env
   ```

3. **Setup database**:
   ```bash
   createdb trading_bot
   sqlx migrate run
   ```

4. **Run the bot**:
   ```bash
   cargo run --release
   ```

## Configuration

### Environment Variables (.env)

```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/trading_bot

# EEYF Settings
EEYF_TIMEOUT_SECS=30
EEYF_MAX_RETRIES=5
EEYF_ENABLE_CACHING=true
EEYF_CACHE_TTL_SECS=60

# Trading Settings
INITIAL_CAPITAL=10000.00
MAX_POSITION_SIZE=0.10  # 10% of capital
STOP_LOSS_PCT=0.02      # 2% stop loss
TAKE_PROFIT_PCT=0.05    # 5% take profit

# Symbols to trade (comma-separated)
TRADING_SYMBOLS=AAPL,GOOGL,MSFT,AMZN,TSLA

# Logging
RUST_LOG=info,trading_bot=debug
```

### Configuration File (config/default.toml)

```toml
[eeyf]
timeout_secs = 30
max_retries = 5
enable_caching = true
cache_ttl_secs = 60
requests_per_second = 10

[trading]
initial_capital = 10000.00
max_position_size = 0.10
stop_loss_pct = 0.02
take_profit_pct = 0.05
symbols = ["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"]

[risk]
max_daily_loss = 500.00
max_positions = 5
min_profit_ratio = 1.5  # Risk:Reward ratio

[strategy]
name = "momentum"
parameters = { lookback = 20, threshold = 0.02 }
```

## Project Structure

```
trading-bot/
├── Cargo.toml
├── README.md
├── .env.example
├── config/
│   └── default.toml
├── src/
│   ├── main.rs              # Entry point
│   ├── config.rs            # Configuration
│   ├── client.rs            # EEYF client setup
│   ├── strategy/
│   │   ├── mod.rs           # Strategy trait
│   │   ├── momentum.rs      # Momentum strategy
│   │   └── mean_reversion.rs  # Mean reversion strategy
│   ├── risk.rs              # Risk management
│   ├── portfolio.rs         # Portfolio tracking
│   ├── execution.rs         # Order execution (simulated)
│   ├── database.rs          # Database operations
│   └── error.rs             # Error types
├── migrations/
│   └── 001_initial.sql      # Database schema
└── tests/
    └── integration_test.rs
```

## Architecture

### Flow Diagram

```
┌─────────────┐
│   Yahoo    │
│  Finance   │ ◄──── WebSocket Connection
└─────┬───────┘
      │
      │ Price Updates
      ▼
┌─────────────────┐
│   Data Feed     │
│   (Real-time)   │
└─────┬───────────┘
      │
      │ Quote Data
      ▼
┌─────────────────┐       ┌──────────────┐
│   Strategy      │◄──────│  Historical  │
│   Engine        │       │    Data      │
└─────┬───────────┘       └──────────────┘
      │
      │ Signals (Buy/Sell)
      ▼
┌─────────────────┐       ┌──────────────┐
│  Risk Manager   │◄──────│  Portfolio   │
│                 │───────►│   State      │
└─────┬───────────┘       └──────────────┘
      │
      │ Approved Orders
      ▼
┌─────────────────┐       ┌──────────────┐
│   Execution     │───────►│  Database    │
│   Engine        │       │  (Postgres)  │
└─────────────────┘       └──────────────┘
```

## Usage Examples

### Basic Usage

```rust
use trading_bot::{TradingBot, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env()?;
    
    // Create bot
    let bot = TradingBot::new(config).await?;
    
    // Run bot
    bot.run().await?;
    
    Ok(())
}
```

### Custom Strategy

```rust
use trading_bot::strategy::{Strategy, Signal};
use eeyf::Quote;

pub struct MyStrategy;

#[async_trait]
impl Strategy for MyStrategy {
    async fn analyze(&self, quotes: &[Quote]) -> anyhow::Result<Signal> {
        // Your strategy logic here
        
        if /* buy condition */ {
            Ok(Signal::Buy)
        } else if /* sell condition */ {
            Ok(Signal::Sell)
        } else {
            Ok(Signal::Hold)
        }
    }
}

// Use your strategy
let bot = TradingBot::new(config)
    .with_strategy(MyStrategy)
    .await?;
```

### Risk Management

```rust
use trading_bot::risk::{RiskManager, Position};

let risk_manager = RiskManager::new(config.risk);

// Check if trade is allowed
if risk_manager.can_open_position(&quote, position_size).await? {
    // Execute trade
    let position = executor.buy(symbol, size).await?;
    
    // Set stop loss and take profit
    risk_manager.set_stop_loss(&position, 0.02).await?;
    risk_manager.set_take_profit(&position, 0.05).await?;
}
```

## Built-in Strategies

### 1. Momentum Strategy

Buys when price momentum is positive, sells when negative.

```toml
[strategy]
name = "momentum"
parameters = { lookback = 20, threshold = 0.02 }
```

### 2. Mean Reversion Strategy

Buys when price is below moving average, sells when above.

```toml
[strategy]
name = "mean_reversion"
parameters = { period = 50, deviation = 2.0 }
```

## Database Schema

```sql
-- Positions table
CREATE TABLE positions (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    entry_price DECIMAL(10, 2) NOT NULL,
    quantity INTEGER NOT NULL,
    entry_time TIMESTAMP NOT NULL,
    exit_time TIMESTAMP,
    exit_price DECIMAL(10, 2),
    profit_loss DECIMAL(10, 2),
    status VARCHAR(20) NOT NULL
);

-- Trades table
CREATE TABLE trades (
    id SERIAL PRIMARY KEY,
    position_id INTEGER REFERENCES positions(id),
    symbol VARCHAR(10) NOT NULL,
    action VARCHAR(10) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    quantity INTEGER NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

-- Performance table
CREATE TABLE performance (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL UNIQUE,
    total_pnl DECIMAL(10, 2) NOT NULL,
    win_rate DECIMAL(5, 2),
    sharpe_ratio DECIMAL(5, 2),
    max_drawdown DECIMAL(5, 2)
);
```

## Monitoring

### Metrics

The bot exposes Prometheus metrics at `localhost:9090/metrics`:

- `trading_bot_positions_total` - Total open positions
- `trading_bot_pnl_total` - Total profit/loss
- `trading_bot_trades_total` - Total trades executed
- `trading_bot_win_rate` - Win rate percentage
- `trading_bot_api_latency` - API call latency

### Logging

Structured logging with tracing:

```rust
tracing::info!(
    symbol = %symbol,
    price = %price,
    quantity = quantity,
    "Executed buy order"
);
```

### Dashboard

Use Grafana with provided dashboard JSON (see `grafana/dashboard.json`).

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

```bash
cargo test --test integration_test
```

### Backtesting

```bash
cargo run --bin backtest -- --start 2023-01-01 --end 2023-12-31
```

## Deployment

### Docker

```bash
docker build -t trading-bot .
docker run -d \
    --env-file .env \
    -p 9090:9090 \
    trading-bot
```

### Systemd Service

```ini
[Unit]
Description=Trading Bot
After=network.target postgresql.service

[Service]
Type=simple
User=trader
WorkingDirectory=/opt/trading-bot
ExecStart=/opt/trading-bot/target/release/trading-bot
Restart=always

[Install]
WantedBy=multi-user.target
```

## Safety & Disclaimers

⚠️ **Important Warnings**:

1. **This is a TEMPLATE** - Not financial advice
2. **Test thoroughly** - Use paper trading first
3. **Risk management** - Never risk more than you can afford to lose
4. **Simulated execution** - This template simulates trades (no real orders)
5. **Market conditions** - Past performance doesn't indicate future results
6. **Compliance** - Ensure regulatory compliance in your jurisdiction

## Customization

### Adding New Strategies

1. Create file in `src/strategy/your_strategy.rs`
2. Implement the `Strategy` trait
3. Add to `strategy/mod.rs`
4. Configure in `config/default.toml`

### Connecting Real Broker

Replace `src/execution.rs` with your broker's API:

- Interactive Brokers
- Alpaca
- TD Ameritrade
- etc.

## Performance Tips

1. **Enable all EEYF features** for production
2. **Use connection pooling** for database
3. **Cache historical data** to reduce API calls
4. **Batch database writes** for better performance
5. **Monitor metrics** to catch issues early

## Troubleshooting

See [TROUBLESHOOTING.md](../../docs/TROUBLESHOOTING.md)

## License

MIT OR Apache-2.0

## Support

- GitHub Issues: Report bugs
- Discord: #trading-bots channel
- Documentation: Full API docs

---

**Remember**: Trading involves risk. This is a template for educational purposes. Always test thoroughly and never risk money you can't afford to lose.
