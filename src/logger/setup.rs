use anyhow::Result;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialise the global tracing subscriber.
///
/// Returns a [`WorkerGuard`] that **must** be held for the lifetime of the
/// program so that buffered log lines are flushed to the file on shutdown.
pub fn init_logging(level: &str, log_file: &str) -> Result<WorkerGuard> {
    let env_filter = EnvFilter::try_new(level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // File appender (non-blocking)
    let file_appender = tracing_appender::rolling::daily(".", log_file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
