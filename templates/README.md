# EEYF Project Templates

Ready-to-use project templates for common EEYF use cases. Each template provides a complete starting point with best practices built-in.

## Available Templates

### 1. Trading Bot Template
A production-ready algorithmic trading bot with real-time data processing, strategy execution, and risk management.

**Location**: `templates/trading-bot/`

**Features**:
- Real-time WebSocket data streaming
- Strategy pattern for easy algorithm swapping
- Risk management and position sizing
- Performance tracking and reporting
- Graceful shutdown handling

**Tech Stack**: EEYF (full features), Tokio, PostgreSQL, Prometheus

---

### 2. Portfolio Tracker Template
A comprehensive portfolio tracking application with real-time valuations and performance analytics.

**Location**: `templates/portfolio-tracker/`

**Features**:
- Multi-portfolio support
- Real-time position valuation
- Historical performance tracking
- Dividend tracking
- Tax lot tracking

**Tech Stack**: EEYF, Axum, SQLite, Handlebars templates

---

### 3. Market Screener Template
A powerful stock screener with customizable criteria and real-time filtering.

**Location**: `templates/market-screener/`

**Features**:
- Custom screening criteria
- Real-time price updates
- Batch symbol processing
- Export to CSV/JSON
- Watchlist management

**Tech Stack**: EEYF, Polars, CSV

---

### 4. Real-Time Dashboard Template
A beautiful real-time dashboard with WebSocket updates and customizable layouts.

**Location**: `templates/realtime-dashboard/`

**Features**:
- WebSocket real-time updates
- Multiple watchlists
- Technical indicators
- Price alerts
- Responsive design

**Tech Stack**: EEYF (WebSocket), Axum, HTMX, TailwindCSS

---

### 5. Data Pipeline Template
An ETL pipeline for collecting, processing, and storing market data at scale.

**Location**: `templates/data-pipeline/`

**Features**:
- Scheduled data collection
- Data transformation and enrichment
- Multiple storage backends
- Error handling and retry logic
- Monitoring and alerting

**Tech Stack**: EEYF, Tokio-cron, PostgreSQL/TimescaleDB, Prometheus

---

### 6. CLI Tool Template
A command-line tool for quick market data queries and analysis.

**Location**: `templates/cli-tool/`

**Features**:
- Interactive and non-interactive modes
- Multiple output formats (JSON, table, CSV)
- Configuration file support
- Shell completion
- Color output

**Tech Stack**: EEYF, clap, prettytable-rs

---

### 7. Microservice Template
A microservice architecture template for building scalable market data APIs.

**Location**: `templates/microservice/`

**Features**:
- RESTful API
- OpenAPI/Swagger documentation
- Authentication and rate limiting
- Health checks
- Docker containerization

**Tech Stack**: EEYF, Axum, Tower, OpenAPI

---

### 8. Research Platform Template
A quantitative research platform with Jupyter notebook integration.

**Location**: `templates/research-platform/`

**Features**:
- Jupyter notebook integration
- Data export to Polars/Arrow
- Backtesting framework
- Statistical analysis tools
- Visualization helpers

**Tech Stack**: EEYF, Polars, Arrow, Jupyter

---

## Quick Start

### Using a Template

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourorg/eeyf.git
   cd eeyf/templates
   ```

2. **Copy your chosen template**:
   ```bash
   cp -r trading-bot/ ~/my-trading-bot/
   cd ~/my-trading-bot/
   ```

3. **Follow the template's README**:
   ```bash
   cat README.md
   ```

### Using cargo-generate (Recommended)

```bash
# Install cargo-generate
cargo install cargo-generate

# Generate project from template
cargo generate --git https://github.com/yourorg/eeyf \
               --name my-project \
               template/trading-bot
```

---

## Template Structure

Each template follows this structure:

```
template-name/
├── Cargo.toml                 # Dependencies and metadata
├── README.md                  # Template-specific documentation
├── .env.example               # Environment variables example
├── config/
│   └── default.toml          # Default configuration
├── src/
│   ├── main.rs               # Application entry point
│   ├── config.rs             # Configuration management
│   ├── client.rs             # EEYF client setup
│   └── ...                   # Template-specific modules
├── tests/
│   └── integration_test.rs   # Integration tests
├── examples/
│   └── basic_usage.rs        # Usage examples
└── docs/
    └── GUIDE.md              # Template guide
```

---

## Customization Guide

### 1. Configuration

All templates use environment variables and config files:

```bash
# Copy and edit .env.example
cp .env.example .env
nano .env
```

```toml
# config/default.toml
[eeyf]
timeout_secs = 30
max_retries = 5
enable_caching = true
cache_ttl_secs = 60

[application]
# Your app-specific settings
```

### 2. Client Setup

Templates include pre-configured EEYF client:

```rust
// src/client.rs
pub async fn create_client(config: &Config) -> Result<YahooFinanceClient> {
    YahooFinanceClient::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .max_retries(config.max_retries)
        .enable_caching(config.enable_caching)
        .cache_ttl(Duration::from_secs(config.cache_ttl_secs))
        .build()
        .await
}
```

### 3. Error Handling

Templates include comprehensive error handling:

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("EEYF error: {0}")]
    Eeyf(#[from] eeyf::YahooFinanceError),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    // Template-specific errors
}
```

### 4. Logging

Templates use structured logging:

```rust
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,my_app=debug")
        .init();
    
    info!("Application starting");
    // ...
}
```

---

## Contributing Templates

Want to contribute a template? Great!

### Template Requirements

1. **Complete and functional** - Must work out of the box
2. **Well-documented** - Clear README and inline comments
3. **Best practices** - Follow Rust and EEYF best practices
4. **Tested** - Include integration tests
5. **Examples** - Provide usage examples
6. **Configuration** - Use config files and environment variables

### Submission Process

1. **Create template** in `templates/your-template-name/`
2. **Add to this README** with description and features
3. **Include comprehensive README** in template directory
4. **Add to CI** for automated testing
5. **Open PR** with template

### Template Checklist

- [ ] Functional out-of-the-box
- [ ] README.md with clear instructions
- [ ] .env.example with all required variables
- [ ] Cargo.toml with correct dependencies
- [ ] src/ with well-structured code
- [ ] tests/ with integration tests
- [ ] examples/ with usage examples
- [ ] Proper error handling
- [ ] Logging/tracing configured
- [ ] Graceful shutdown
- [ ] Configuration management
- [ ] Docker support (if applicable)

---

## Template Maintenance

### Version Compatibility

Templates are tested with:
- **EEYF**: 0.1.x
- **Rust**: 1.75+
- **Tokio**: 1.35+

### Updates

Templates are updated when:
- New EEYF versions are released
- Dependencies have security updates
- Community feedback suggests improvements
- New best practices emerge

### Getting Help

- **Template Issues**: Open issue with `template:` prefix
- **Template Questions**: Ask in Discord #templates channel
- **Template Ideas**: Open discussion in GitHub Discussions

---

## Next Steps

After using a template:

1. **Customize** for your specific needs
2. **Read** the EEYF documentation for advanced features
3. **Test** thoroughly before production use
4. **Monitor** with metrics and logging
5. **Share** your project in the showcase!

---

## License

Templates are licensed under the same terms as EEYF (MIT OR Apache-2.0).
