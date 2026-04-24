use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::Utc;
use serde::Deserialize;

use crate::config::AppConfig;
use crate::execution::{Account, Side};
use crate::risk::RiskManager;

/// Incoming webhook payload (TradingView-style).
#[derive(Debug, Deserialize)]
pub struct WebhookPayload {
    /// "long" or "short"
    pub signal: String,
    /// Current price
    pub price: f64,
}

/// Shared state passed to axum handlers.
#[derive(Clone)]
struct AppState {
    account: Arc<Mutex<Account>>,
    risk: RiskManager,
    #[allow(dead_code)]
    config: AppConfig,
}

/// Start the webhook HTTP server on the configured port.
pub async fn start_webhook(
    config: AppConfig,
    account: Arc<Mutex<Account>>,
) -> Result<()> {
    let port = config.webhook.port;
    let risk = RiskManager::new(&config);

    let state = AppState {
        account,
        risk,
        config,
    };

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!(addr = %addr, "Starting webhook server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn handle_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookPayload>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    tracing::info!(signal = %payload.signal, price = payload.price, "Received webhook");

    let mut account = state.account.lock().await;
    let now = Utc::now();

    // Close any existing position first
    if account.has_position() {
        account.close_position(payload.price, now);
    }

    let alloc = state.risk.position_size(account.balance);

    match payload.signal.to_lowercase().as_str() {
        "long" => {
            account.open_position(Side::Long, payload.price, alloc, now);
            Ok((StatusCode::OK, "Opened LONG position".to_string()))
        }
        "short" => {
            account.open_position(Side::Short, payload.price, alloc, now);
            Ok((StatusCode::OK, "Opened SHORT position".to_string()))
        }
        other => {
            let msg = format!("Unknown signal: {}", other);
            tracing::warn!(%msg);
            Err((StatusCode::BAD_REQUEST, msg))
        }
    }
}
