use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use crate::config::AppConfig;
use crate::data::{self, Candle};
use crate::execution::{Account, Side};
use crate::risk::RiskManager;
use crate::strategy::{compute_rsi, generate_signal, Signal};

/// Process a single candle tick: compute RSI, generate signal, manage risk, execute.
///
/// `candle_history` must include enough prior candles for the RSI calculation.
fn process_tick(
    candle_history: &[Candle],
    account: &mut Account,
    risk: &RiskManager,
    config: &AppConfig,
) {
    let closes: Vec<f64> = candle_history.iter().map(|c| c.close).collect();
    let rsi_values = compute_rsi(&closes, config.rsi.period);

    if rsi_values.is_empty() {
        return;
    }

    let latest_rsi = *rsi_values.last().unwrap();
    let latest_candle = candle_history.last().unwrap();
    let price = latest_candle.close;
    let ts = latest_candle.timestamp;

    tracing::debug!(
        rsi = format!("{:.2}", latest_rsi),
        price = format!("{:.2}", price),
        "Tick"
    );

    // ── Stop-loss check ──
    if let Some(ref pos) = account.position {
        let stopped = match pos.side {
            Side::Long => risk.is_stopped_out_long(pos.entry_price, price),
            Side::Short => risk.is_stopped_out_short(pos.entry_price, price),
        };
        if stopped {
            tracing::warn!(
                side = ?pos.side,
                entry = pos.entry_price,
                price = price,
                "🛑 Stop-loss triggered"
            );
            account.close_position(price, ts);
            return;
        }
    }

    // ── Signal ──
    let signal = generate_signal(latest_rsi, account.is_long(), account.is_short(), config);

    match signal {
        Signal::Long => {
            let alloc = risk.position_size(account.balance);
            tracing::info!(rsi = format!("{:.2}", latest_rsi), alloc = format!("{:.2}", alloc), "🟢 LONG signal");
            account.open_position(Side::Long, price, alloc, ts);
        }
        Signal::Short => {
            let alloc = risk.position_size(account.balance);
            tracing::info!(rsi = format!("{:.2}", latest_rsi), alloc = format!("{:.2}", alloc), "🔴 SHORT signal");
            account.open_position(Side::Short, price, alloc, ts);
        }
        Signal::CloseLong | Signal::CloseShort => {
            tracing::info!(rsi = format!("{:.2}", latest_rsi), "🔒 Close signal");
            account.close_position(price, ts);
        }
        Signal::Hold => {}
    }
}

// ────────────────────────────────────────────────
// Backtest
// ────────────────────────────────────────────────

/// Run the strategy over historical candle data loaded from CSV.
pub fn run_backtest(csv_path: &Path, config: &AppConfig) -> Result<Account> {
    let candles = data::load_csv(csv_path)?;
    tracing::info!(candles = candles.len(), "Starting backtest");

    let risk = RiskManager::new(config);
    let mut account = Account::new(config.trading.initial_balance);

    let period = config.rsi.period;

    // We need at least `period + 1` candles to produce the first RSI.
    for i in (period + 1)..=candles.len() {
        let window = &candles[..i];
        process_tick(window, &mut account, &risk, config);
    }

    // Close any remaining position at the last price
    if account.has_position() {
        let last = candles.last().unwrap();
        account.close_position(last.close, last.timestamp);
    }

    tracing::info!("Backtest complete");
    print_backtest_report(&account);
    Ok(account)
}

fn print_backtest_report(account: &Account) {
    println!("\n╔══════════════════════════════════════════╗");
    println!("║        📊 BACKTEST REPORT                ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  Initial Balance:  ${:>12.2}           ║", account.initial_balance);
    println!("║  Final Balance:    ${:>12.2}           ║", account.balance);
    println!("║  Total Return:     {:>+12.2}%          ║", (account.total_pnl() / account.initial_balance) * 100.0);
    println!("║  Total PnL:        ${:>12.2}           ║", account.total_pnl());
    println!("║  Trades:           {:>12}            ║", account.trade_history.len());
    println!("║  Win Rate:         {:>11.1}%           ║", account.win_rate() * 100.0);
    println!("║  Max Drawdown:     {:>11.2}%           ║", account.max_drawdown() * 100.0);
    println!("║  Sharpe Ratio:     {:>12.4}           ║", account.sharpe_ratio());
    println!("╚══════════════════════════════════════════╝\n");
}

// ────────────────────────────────────────────────
// Live paper trading
// ────────────────────────────────────────────────

/// Run the live paper-trading loop.
///
/// Fetches candles from Binance every interval, processes the latest tick,
/// and loops until interrupted.
pub async fn run_live(
    config: AppConfig,
    account: Arc<Mutex<Account>>,
) -> Result<()> {
    let risk = RiskManager::new(&config);
    let symbol = &config.trading.symbol;
    let interval = &config.trading.interval;
    let limit = config.trading.candle_limit;

    tracing::info!(symbol = symbol, interval = interval, "Starting live paper trading");

    // Parse interval string (e.g. "5m") into a sleep duration
    let sleep_secs = parse_interval_secs(interval);

    loop {
        match data::fetch_candles(symbol, interval, limit).await {
            Ok(candles) => {
                if candles.len() > config.rsi.period {
                    let mut acc = account.lock().await;
                    process_tick(&candles, &mut acc, &risk, &config);
                } else {
                    tracing::warn!(count = candles.len(), "Not enough candles for RSI");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to fetch candles");
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
    }
}

/// Parse an interval string like "1m", "5m", "15m", "1h" into seconds.
fn parse_interval_secs(interval: &str) -> u64 {
    let s = interval.trim();
    if let Some(mins) = s.strip_suffix('m') {
        mins.parse::<u64>().unwrap_or(5) * 60
    } else if let Some(hrs) = s.strip_suffix('h') {
        hrs.parse::<u64>().unwrap_or(1) * 3600
    } else if let Some(days) = s.strip_suffix('d') {
        days.parse::<u64>().unwrap_or(1) * 86400
    } else {
        300 // default 5 minutes
    }
}
