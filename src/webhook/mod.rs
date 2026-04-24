/// Optional webhook listener — receives TradingView-style alerts via HTTP.
pub mod server;

pub use server::start_webhook;
