use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Top-level application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub trading: TradingConfig,
    pub rsi: RsiConfig,
    pub risk: RiskConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub webhook: WebhookConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradingConfig {
    /// Trading pair symbol, e.g. "BTCUSDT"
    pub symbol: String,
    /// Initial paper-trading balance in USD
    pub initial_balance: f64,
    /// Candle interval, e.g. "5m"
    #[serde(default = "default_interval")]
    pub interval: String,
    /// Number of candles to fetch per request
    #[serde(default = "default_candle_limit")]
    pub candle_limit: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RsiConfig {
    /// RSI look-back period (default 14)
    #[serde(default = "default_rsi_period")]
    pub period: usize,
    /// RSI oversold threshold — go LONG below this
    #[serde(default = "default_oversold")]
    pub oversold: f64,
    /// RSI overbought threshold — go SHORT above this
    #[serde(default = "default_overbought")]
    pub overbought: f64,
    /// RSI neutral level — close position at this level
    #[serde(default = "default_neutral")]
    pub neutral: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    /// Fraction of equity to risk per trade (e.g. 0.05 = 5%)
    #[serde(default = "default_position_size_pct")]
    pub position_size_pct: f64,
    /// Stop-loss as a fraction of entry price (e.g. 0.50 = 50%)
    #[serde(default = "default_stop_loss_pct")]
    pub stop_loss_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Path to the log file
    #[serde(default = "default_log_file")]
    pub file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    /// Enable webhook listener
    #[serde(default)]
    pub enabled: bool,
    /// Port for the webhook HTTP server
    #[serde(default = "default_webhook_port")]
    pub port: u16,
}

// ── defaults ──

fn default_interval() -> String {
    "5m".into()
}
fn default_candle_limit() -> usize {
    100
}
fn default_rsi_period() -> usize {
    14
}
fn default_oversold() -> f64 {
    30.0
}
fn default_overbought() -> f64 {
    70.0
}
fn default_neutral() -> f64 {
    50.0
}
fn default_position_size_pct() -> f64 {
    0.05
}
fn default_stop_loss_pct() -> f64 {
    0.50
}
fn default_log_level() -> String {
    "info".into()
}
fn default_log_file() -> String {
    "crypto_bot.log".into()
}
fn default_webhook_port() -> u16 {
    3030
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_webhook_port(),
        }
    }
}

impl AppConfig {
    /// Load config from a TOML file at the given path.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: AppConfig =
            toml::from_str(&contents).with_context(|| "Failed to parse config TOML")?;
        Ok(config)
    }
}
