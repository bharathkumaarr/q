use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single OHLCV candle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Candle open time (UTC).
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Candle {
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}
