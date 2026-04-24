use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Side of the position.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Side {
    Long,
    Short,
}

/// An open position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub side: Side,
    pub entry_price: f64,
    pub quantity: f64,
    pub entry_time: DateTime<Utc>,
}

/// A completed trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub side: Side,
    pub entry_price: f64,
    pub exit_price: f64,
    pub quantity: f64,
    pub pnl: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
}

/// Simulated paper trading account.
#[derive(Debug, Clone)]
pub struct Account {
    pub initial_balance: f64,
    pub balance: f64,
    pub position: Option<Position>,
    pub trade_history: Vec<TradeRecord>,
    pub peak_balance: f64,
}

impl Account {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            balance: initial_balance,
            position: None,
            trade_history: Vec::new(),
            peak_balance: initial_balance,
        }
    }

    /// Open a new position.
    pub fn open_position(
        &mut self,
        side: Side,
        price: f64,
        allocation: f64,
        timestamp: DateTime<Utc>,
    ) {
        let quantity = allocation / price;
        self.position = Some(Position {
            side,
            entry_price: price,
            quantity,
            entry_time: timestamp,
        });
        tracing::info!(
            side = ?side,
            price = price,
            quantity = quantity,
            allocation = allocation,
            "📈 Opened position"
        );
    }

    /// Close the current position and record the trade.
    pub fn close_position(&mut self, exit_price: f64, exit_time: DateTime<Utc>) -> Option<f64> {
        let pos = self.position.take()?;

        let pnl = match pos.side {
            Side::Long => (exit_price - pos.entry_price) * pos.quantity,
            Side::Short => (pos.entry_price - exit_price) * pos.quantity,
        };

        self.balance += pnl;
        if self.balance > self.peak_balance {
            self.peak_balance = self.balance;
        }

        let record = TradeRecord {
            side: pos.side,
            entry_price: pos.entry_price,
            exit_price,
            quantity: pos.quantity,
            pnl,
            entry_time: pos.entry_time,
            exit_time,
        };

        tracing::info!(
            side = ?record.side,
            entry = record.entry_price,
            exit = record.exit_price,
            pnl = format!("{:.2}", pnl),
            balance = format!("{:.2}", self.balance),
            "📉 Closed position"
        );

        self.trade_history.push(record);
        Some(pnl)
    }

    /// Whether a position is currently open.
    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    pub fn is_long(&self) -> bool {
        matches!(&self.position, Some(p) if p.side == Side::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(&self.position, Some(p) if p.side == Side::Short)
    }

    /// Total PnL since inception.
    pub fn total_pnl(&self) -> f64 {
        self.balance - self.initial_balance
    }

    /// Win rate (fraction of winning trades).
    pub fn win_rate(&self) -> f64 {
        if self.trade_history.is_empty() {
            return 0.0;
        }
        let wins = self.trade_history.iter().filter(|t| t.pnl > 0.0).count();
        wins as f64 / self.trade_history.len() as f64
    }

    /// Maximum drawdown as a fraction (0.0 – 1.0).
    pub fn max_drawdown(&self) -> f64 {
        let mut peak = self.initial_balance;
        let mut max_dd = 0.0_f64;
        let mut running = self.initial_balance;

        for trade in &self.trade_history {
            running += trade.pnl;
            if running > peak {
                peak = running;
            }
            let dd = (peak - running) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        max_dd
    }

    /// Compute the annualised Sharpe ratio from trade returns.
    /// Uses a risk-free rate of 0 and assumes ~252 trading days * ~288
    /// five-minute bars per day ≈ 72576 bars/year.
    pub fn sharpe_ratio(&self) -> f64 {
        if self.trade_history.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = self
            .trade_history
            .iter()
            .map(|t| t.pnl / (t.entry_price * t.quantity))
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance =
            returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // Annualise: assume ~72576 five-minute bars per year
        let annualisation = (72576.0_f64).sqrt();
        (mean / std_dev) * annualisation
    }

    /// Print a human-readable status summary.
    pub fn print_status(&self) {
        println!("\n══════════════════════════════════════════");
        println!("  📊 Account Status");
        println!("══════════════════════════════════════════");
        println!("  Balance:       ${:.2}", self.balance);
        println!("  Total PnL:     ${:.2}", self.total_pnl());
        println!("  Win Rate:      {:.1}%", self.win_rate() * 100.0);
        println!("  Max Drawdown:  {:.2}%", self.max_drawdown() * 100.0);
        println!("  Sharpe Ratio:  {:.4}", self.sharpe_ratio());
        println!("  Total Trades:  {}", self.trade_history.len());

        if let Some(ref pos) = self.position {
            println!("  ── Open Position ──");
            println!("    Side:    {:?}", pos.side);
            println!("    Entry:   ${:.2}", pos.entry_price);
            println!("    Qty:     {:.6}", pos.quantity);
            println!("    Since:   {}", pos.entry_time.format("%Y-%m-%d %H:%M:%S UTC"));
        } else {
            println!("  Position:      None (flat)");
        }
        println!("══════════════════════════════════════════\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_open_close_long() {
        let mut acc = Account::new(10_000.0);
        let now = Utc::now();
        acc.open_position(Side::Long, 100.0, 500.0, now);
        assert!(acc.is_long());
        let pnl = acc.close_position(110.0, now).unwrap();
        // 5 units * $10 gain = $50
        assert!((pnl - 50.0).abs() < 1e-9);
        assert!((acc.balance - 10_050.0).abs() < 1e-9);
    }

    #[test]
    fn test_open_close_short() {
        let mut acc = Account::new(10_000.0);
        let now = Utc::now();
        acc.open_position(Side::Short, 100.0, 500.0, now);
        assert!(acc.is_short());
        let pnl = acc.close_position(90.0, now).unwrap();
        assert!((pnl - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_win_rate() {
        let mut acc = Account::new(10_000.0);
        let now = Utc::now();
        // Win
        acc.open_position(Side::Long, 100.0, 500.0, now);
        acc.close_position(110.0, now);
        // Loss
        acc.open_position(Side::Long, 100.0, 500.0, now);
        acc.close_position(90.0, now);
        assert!((acc.win_rate() - 0.5).abs() < f64::EPSILON);
    }
}
