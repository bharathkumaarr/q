mod config;
mod data;
mod engine;
mod execution;
mod logger;
mod risk;
mod strategy;
mod webhook;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio::sync::Mutex;

use config::AppConfig;
use execution::Account;

/// 🤖 Crypto Trading Bot — RSI mean-reversion paper trader
#[derive(Parser)]
#[command(name = "crypto_bot", version, about)]
struct Cli {
    /// Path to the TOML config file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start live paper trading using Binance market data
    Run {
        /// Also start the webhook listener
        #[arg(long)]
        webhook: bool,
    },
    /// Run the strategy on historical CSV data
    Backtest {
        /// Path to the CSV candle file
        #[arg(short = 'f', long)]
        csv: PathBuf,
    },
    /// Show current account status and PnL
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(&cli.config)?;

    // Initialise logging (hold the guard so logs flush on exit)
    let _guard = logger::init_logging(&config.logging.level, &config.logging.file)?;

    tracing::info!(symbol = %config.trading.symbol, "Crypto Bot starting");

    match cli.command {
        Commands::Run { webhook } => {
            let account = Arc::new(Mutex::new(Account::new(config.trading.initial_balance)));

            if webhook && config.webhook.enabled {
                let wh_config = config.clone();
                let wh_account = Arc::clone(&account);
                tokio::spawn(async move {
                    if let Err(e) = crate::webhook::start_webhook(wh_config, wh_account).await {
                        tracing::error!(error = %e, "Webhook server failed");
                    }
                });
            }

            engine::run_live(config, account).await?;
        }

        Commands::Backtest { csv } => {
            engine::run_backtest(&csv, &config)?;
        }

        Commands::Status => {
            // In a real system this would read persisted state.
            // For the MVP we show a fresh account status.
            let account = Account::new(config.trading.initial_balance);
            account.print_status();
            println!("  💡 Tip: run `crypto_bot run` first to generate trade data.");
        }
    }

    Ok(())
}
