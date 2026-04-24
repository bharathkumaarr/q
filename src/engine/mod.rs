/// Trading engine — orchestrates data → strategy → risk → execution.
pub mod runner;

pub use runner::{run_live, run_backtest};
