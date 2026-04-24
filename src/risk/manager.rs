use crate::config::AppConfig;

/// Handles position sizing and stop-loss evaluation.
#[derive(Debug, Clone)]
pub struct RiskManager {
    /// Fraction of equity risked per trade.
    pub position_size_pct: f64,
    /// Maximum allowed loss as fraction of entry price.
    pub stop_loss_pct: f64,
}

impl RiskManager {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            position_size_pct: config.risk.position_size_pct,
            stop_loss_pct: config.risk.stop_loss_pct,
        }
    }

    /// Compute the dollar amount to allocate for a new position.
    pub fn position_size(&self, equity: f64) -> f64 {
        equity * self.position_size_pct
    }

    /// Return the stop-loss price for a long position.
    pub fn stop_loss_long(&self, entry_price: f64) -> f64 {
        entry_price * (1.0 - self.stop_loss_pct)
    }

    /// Return the stop-loss price for a short position.
    pub fn stop_loss_short(&self, entry_price: f64) -> f64 {
        entry_price * (1.0 + self.stop_loss_pct)
    }

    /// Check whether a long position's stop-loss has been hit.
    pub fn is_stopped_out_long(&self, entry_price: f64, current_price: f64) -> bool {
        current_price <= self.stop_loss_long(entry_price)
    }

    /// Check whether a short position's stop-loss has been hit.
    pub fn is_stopped_out_short(&self, entry_price: f64, current_price: f64) -> bool {
        current_price >= self.stop_loss_short(entry_price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rm() -> RiskManager {
        RiskManager {
            position_size_pct: 0.05,
            stop_loss_pct: 0.50,
        }
    }

    #[test]
    fn test_position_size() {
        let rm = test_rm();
        assert!((rm.position_size(10_000.0) - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stop_loss_long() {
        let rm = test_rm();
        // 50% stop → at $100 entry, stop at $50
        assert!((rm.stop_loss_long(100.0) - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stop_loss_short() {
        let rm = test_rm();
        // 50% stop → at $100 entry, stop at $150
        assert!((rm.stop_loss_short(100.0) - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_stopped_out_long() {
        let rm = test_rm();
        assert!(rm.is_stopped_out_long(100.0, 49.0));
        assert!(!rm.is_stopped_out_long(100.0, 60.0));
    }

    #[test]
    fn test_stopped_out_short() {
        let rm = test_rm();
        assert!(rm.is_stopped_out_short(100.0, 151.0));
        assert!(!rm.is_stopped_out_short(100.0, 130.0));
    }
}
