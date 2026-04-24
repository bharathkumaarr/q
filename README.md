# 🤖 Crypto Trading Bot

A CLI-based RSI mean-reversion crypto paper trading bot written in Rust.

## Features

- **RSI Mean Reversion Strategy** — 14-period RSI on 5-minute candles
- **Live Paper Trading** — Real-time data from Binance public API
- **Backtesting** — Run strategy on historical CSV data with performance metrics
- **Risk Management** — Position sizing (5% equity) and stop-loss (-50%)
- **Webhook Listener** — Accept TradingView-style alerts via HTTP
- **Comprehensive Logging** — Console + file output via `tracing`

## Architecture

```
src/
├── main.rs          # CLI entry point (clap)
├── config/          # TOML config loader
├── data/            # Binance REST client + CSV loader
├── strategy/        # RSI calculation + signal generation
├── risk/            # Position sizing + stop-loss
├── execution/       # Paper trading account + PnL tracking
├── engine/          # Trading engine (live + backtest runner)
├── logger/          # Tracing setup (console + file)
└── webhook/         # Optional HTTP webhook listener (axum)
```

## Quick Start

### Build

```bash
cargo build --release
```

### Configure

Edit `config.toml`:

```toml
[trading]
symbol = "BTCUSDT"
initial_balance = 10000.0
interval = "5m"

[rsi]
period = 14
oversold = 30.0
overbought = 70.0
neutral = 50.0

[risk]
position_size_pct = 0.05
stop_loss_pct = 0.50
```

### Run Live Paper Trading

```bash
cargo run -- run --config config.toml
```

With webhook listener:

```bash
# Enable webhook in config.toml first: [webhook] enabled = true
cargo run -- run --config config.toml --webhook
```

### Backtest

```bash
cargo run -- backtest --config config.toml -f data/sample.csv
```

### Status

```bash
cargo run -- status --config config.toml
```

## Backtesting

### CSV Format

```csv
timestamp,open,high,low,close,volume
2025-01-01 00:00:00,42000.00,42100.00,41900.00,42050.00,150.5
```

### Output Metrics

- Total return (%)
- Sharpe ratio (annualised)
- Max drawdown (%)
- Win rate
- Trade count

## Webhook

POST to `http://localhost:3030/webhook`:

```json
{
  "signal": "long",
  "price": 50000.0
}
```

Signals: `"long"` or `"short"`.

## Strategy Rules

| Condition | Action |
|-----------|--------|
| RSI < 30 | Enter LONG |
| RSI > 70 | Enter SHORT |
| RSI → 50 | Close position |
| Loss ≥ 50% | Stop-loss triggered |

- Only one open position at a time
- 5% of equity per trade

## Tests

```bash
cargo test
```

## License

MIT
