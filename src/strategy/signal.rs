use crate::config::AppConfig;

/// Trading signal produced by the strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// Enter a long position.
    Long,
    /// Enter a short position.
    Short,
    /// Close an existing long position (RSI returned to neutral).
    CloseLong,
    /// Close an existing short position (RSI returned to neutral).
    CloseShort,
    /// No action.
    Hold,
}

/// Determine the trading signal from the latest RSI value
/// and the current position state.
///
/// # Arguments
/// * `rsi`            – latest RSI reading
/// * `is_long`        – whether a long position is currently open
/// * `is_short`       – whether a short position is currently open
/// * `config`         – application config (RSI thresholds)
pub fn generate_signal(rsi: f64, is_long: bool, is_short: bool, config: &AppConfig) -> Signal {
    let oversold = config.rsi.oversold;
    let overbought = config.rsi.overbought;
    let neutral = config.rsi.neutral;

    // ── Exit logic first ──
    if is_long && rsi >= neutral {
        return Signal::CloseLong;
    }
    if is_short && rsi <= neutral {
        return Signal::CloseShort;
    }

    // ── Entry logic (only when flat) ──
    if !is_long && !is_short {
        if rsi < oversold {
            return Signal::Long;
        }
        if rsi > overbought {
            return Signal::Short;
        }
    }

    Signal::Hold
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn test_config() -> AppConfig {
        let toml_str = r#"
            [trading]
            symbol = "BTCUSDT"
            initial_balance = 10000.0

            [rsi]
            period = 14
            oversold = 30.0
            overbought = 70.0
            neutral = 50.0

            [risk]
            position_size_pct = 0.05
            stop_loss_pct = 0.50

            [logging]
            level = "info"
            file = "test.log"
        "#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_long_entry() {
        let cfg = test_config();
        assert_eq!(generate_signal(25.0, false, false, &cfg), Signal::Long);
    }

    #[test]
    fn test_short_entry() {
        let cfg = test_config();
        assert_eq!(generate_signal(75.0, false, false, &cfg), Signal::Short);
    }

    #[test]
    fn test_close_long() {
        let cfg = test_config();
        assert_eq!(generate_signal(50.0, true, false, &cfg), Signal::CloseLong);
    }

    #[test]
    fn test_close_short() {
        let cfg = test_config();
        assert_eq!(generate_signal(50.0, false, true, &cfg), Signal::CloseShort);
    }

    #[test]
    fn test_hold() {
        let cfg = test_config();
        assert_eq!(generate_signal(45.0, false, false, &cfg), Signal::Hold);
    }
}
