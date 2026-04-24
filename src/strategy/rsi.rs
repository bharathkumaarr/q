/// Compute the Relative Strength Index using Wilder's smoothing method.
///
/// Returns a vector of RSI values. The first `period` close prices are consumed
/// to produce the initial RSI, so the output length is `closes.len() - period`.
///
/// # Panics
/// Does not panic; returns an empty vec if there are not enough data points.
pub fn compute_rsi(closes: &[f64], period: usize) -> Vec<f64> {
    if closes.len() <= period || period == 0 {
        return Vec::new();
    }

    let mut gains = Vec::with_capacity(closes.len() - 1);
    let mut losses = Vec::with_capacity(closes.len() - 1);

    for i in 1..closes.len() {
        let change = closes[i] - closes[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    // Initial average gain/loss (simple average of first `period` changes)
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    let mut rsi_values = Vec::with_capacity(closes.len() - period);

    // First RSI value
    rsi_values.push(rsi_from_avg(avg_gain, avg_loss));

    // Subsequent values use Wilder's smoothing
    for i in period..gains.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;
        rsi_values.push(rsi_from_avg(avg_gain, avg_loss));
    }

    rsi_values
}

/// Convert average gain/loss to RSI.
fn rsi_from_avg(avg_gain: f64, avg_loss: f64) -> f64 {
    if avg_loss == 0.0 {
        return 100.0;
    }
    let rs = avg_gain / avg_loss;
    100.0 - (100.0 / (1.0 + rs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        // 15 prices → 14 changes → 1 RSI value minimum with period=14
        let closes = vec![
            44.0, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28,
        ];
        let rsi = compute_rsi(&closes, 14);
        assert!(!rsi.is_empty());
        assert!(rsi[0] > 0.0 && rsi[0] < 100.0);
    }

    #[test]
    fn test_rsi_not_enough_data() {
        let closes = vec![1.0, 2.0, 3.0];
        let rsi = compute_rsi(&closes, 14);
        assert!(rsi.is_empty());
    }

    #[test]
    fn test_rsi_all_gains() {
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let rsi = compute_rsi(&closes, 14);
        assert!(!rsi.is_empty());
        // All gains → RSI should be 100
        assert!((rsi[0] - 100.0).abs() < f64::EPSILON);
    }
}
