use anyhow::{Context, Result};
use chrono::{NaiveDateTime, TimeZone, Utc};
use std::path::Path;

use super::Candle;

/// Load OHLCV candles from a CSV file.
///
/// Expected columns (header row required):
/// `timestamp,open,high,low,close,volume`
///
/// `timestamp` format: `%Y-%m-%d %H:%M:%S` (UTC assumed).
pub fn load_csv(path: &Path) -> Result<Vec<Candle>> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("Failed to open CSV file: {}", path.display()))?;

    let mut candles = Vec::new();

    for (i, result) in reader.records().enumerate() {
        let record = result.with_context(|| format!("Error reading CSV row {}", i + 1))?;

        let ts_str = record
            .get(0)
            .with_context(|| format!("Missing timestamp at row {}", i + 1))?;
        let naive = NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S")
            .with_context(|| format!("Invalid timestamp '{}' at row {}", ts_str, i + 1))?;
        let timestamp = Utc.from_utc_datetime(&naive);

        let parse_f64 = |col: usize, name: &str| -> Result<f64> {
            record
                .get(col)
                .with_context(|| format!("Missing {} at row {}", name, i + 1))?
                .parse::<f64>()
                .with_context(|| format!("Invalid {} at row {}", name, i + 1))
        };

        candles.push(Candle::new(
            timestamp,
            parse_f64(1, "open")?,
            parse_f64(2, "high")?,
            parse_f64(3, "low")?,
            parse_f64(4, "close")?,
            parse_f64(5, "volume")?,
        ));
    }

    Ok(candles)
}
