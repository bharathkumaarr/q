/// RSI strategy — indicator calculation and signal generation.
pub mod rsi;
pub mod signal;

pub use rsi::compute_rsi;
pub use signal::{Signal, generate_signal};
