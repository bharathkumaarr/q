/// Market data ingestion — Binance REST client and CSV loader.
pub mod binance;
pub mod candle;
pub mod csv_loader;

pub use candle::Candle;
pub use binance::fetch_candles;
pub use csv_loader::load_csv;
