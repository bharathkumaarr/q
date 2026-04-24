use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use serde::Deserialize;

use super::Candle;

/// Raw kline entry from the Binance REST API.
/// The API returns arrays of mixed types, so we deserialize into a helper first.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawKline(
    i64,    // 0  open time (ms)
    String, // 1  open
    String, // 2  high
    String, // 3  low
    String, // 4  close
    String, // 5  volume
    i64,    // 6  close time
    String, // 7  quote asset volume
    u64,    // 8  number of trades
    String, // 9  taker buy base
    String, // 10 taker buy quote
    String, // 11 ignore
);

/// Fetch OHLCV candles from the Binance public REST API.
///
/// # Arguments
/// * `symbol`   – e.g. `"BTCUSDT"`
/// * `interval` – e.g. `"5m"`
/// * `limit`    – number of candles (max 1000)
pub async fn fetch_candles(symbol: &str, interval: &str, limit: usize) -> Result<Vec<Candle>> {
    let url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );

    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch candles from Binance for {symbol}"))?;

    let raw: Vec<RawKline> = resp
        .json()
        .await
        .with_context(|| "Failed to parse Binance kline JSON")?;

    let candles = raw
        .into_iter()
        .map(|k| {
            let ts = Utc
                .timestamp_millis_opt(k.0)
                .single()
                .unwrap_or_else(Utc::now);
            Candle::new(
                ts,
                k.1.parse::<f64>().unwrap_or(0.0),
                k.2.parse::<f64>().unwrap_or(0.0),
                k.3.parse::<f64>().unwrap_or(0.0),
                k.4.parse::<f64>().unwrap_or(0.0),
                k.5.parse::<f64>().unwrap_or(0.0),
            )
        })
        .collect();

    Ok(candles)
}
