use axum::{
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

use algo_core::model::TIMEFRAMES;

mod backup;
mod broker;
mod market;
mod optionchain;
mod realtime;
mod scrip;
mod stats;

use broker::DhanState;
use market::MarketState;
use realtime::RealtimeState;

/// Combined router state: the Dhan session/feed plus the realtime trading
/// engine. Handlers keep extracting the narrower `State<DhanState>` /
/// `State<MarketState>` / `State<RealtimeState>` via `FromRef`.
#[derive(Clone)]
pub struct AppState {
    pub dhan: DhanState,
    pub rt: RealtimeState,
}

impl FromRef<AppState> for DhanState {
    fn from_ref(state: &AppState) -> DhanState {
        state.dhan.clone()
    }
}

impl FromRef<AppState> for MarketState {
    fn from_ref(state: &AppState) -> MarketState {
        state.dhan.market.clone()
    }
}

impl FromRef<AppState> for RealtimeState {
    fn from_ref(state: &AppState) -> RealtimeState {
        state.rt.clone()
    }
}

#[derive(Deserialize)]
struct CandleReq {
    #[serde(default)]
    security_id: i64,
    #[serde(default)]
    exchange_segment: String,
    #[serde(default)]
    instrument_type: String,
    #[serde(default)]
    timeframe: String,
}

async fn candles_post(
    State(st): State<DhanState>,
    Json(req): Json<CandleReq>,
) -> impl IntoResponse {
    let tf = if req.timeframe.is_empty() {
        "5min"
    } else {
        &req.timeframe
    };
    // Only real Dhan candles are served. Without a live session (or when Dhan
    // returns nothing) the chart gets an empty series instead of synthetic data.
    if req.exchange_segment.is_empty() {
        return Json(json!({
            "ok": false,
            "data": [],
            "error": "not connected to Dhan",
        }));
    }
    match st
        .fetch_candles(
            req.security_id,
            &req.exchange_segment,
            &req.instrument_type,
            tf,
        )
        .await
    {
        Ok(data) if !data.is_empty() => Json(json!({ "ok": true, "data": data })),
        Ok(_) => {
            tracing::warn!(
                "Dhan returned empty candles: sec={} exch={} inst={} tf={}",
                req.security_id,
                req.exchange_segment,
                req.instrument_type,
                tf
            );
            Json(json!({ "ok": false, "data": [], "error": "no candles returned" }))
        }
        Err(e) => {
            tracing::warn!(
                "Dhan candle fetch failed: sec={} exch={} inst={} tf={} err={:?}",
                req.security_id,
                req.exchange_segment,
                req.instrument_type,
                tf,
                e
            );
            Json(json!({ "ok": false, "data": [], "error": e.to_string() }))
        }
    }
}

async fn candles_get(
    State(st): State<DhanState>,
    Query(req): Query<CandleReq>,
) -> impl IntoResponse {
    candles_post(State(st), Json(req)).await
}

async fn indicators() -> impl IntoResponse {
    Json(algo_core::indicators::catalog_json())
}

async fn timeframes() -> impl IntoResponse {
    let list: Vec<_> = TIMEFRAMES
        .iter()
        .map(|t| json!({ "key": t.key, "label": t.label }))
        .collect();
    Json(list)
}

async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let static_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
    let index = ServeFile::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"));

    let market = MarketState::new();
    let dhan = DhanState::new(market.clone());
    dhan.spawn_watchdog();
    scrip::spawn_warm();
    let rt = RealtimeState::new(dhan.clone());
    let paper_rt = RealtimeState::new_paper(dhan.clone());
    let state = AppState { dhan, rt };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/indicators", get(indicators))
        .route("/api/timeframes", get(timeframes))
        .route("/api/candles", get(candles_get).post(candles_post))
        .route("/api/symbols", get(market::symbols))
        .route("/api/watchlists", get(market::symbols))
        .route("/api/commodities", get(market::commodities))
        .route("/api/quotes", post(market::quotes_post))
        .route("/api/expiries", post(optionchain::expiries))
        .route("/api/option_chain", post(optionchain::option_chain))
        .route("/api/option_chain_all", post(optionchain::option_chain_all))
        .route("/api/auto_strikes", post(optionchain::auto_strikes))
        .route("/api/option_security", post(optionchain::option_security))
        .route("/api/oc/subscribe", post(optionchain::oc_subscribe))
        .route("/api/lot_sizes", get(optionchain::lot_sizes))
        .route("/ws", get(market::ws_handler))
        .route("/api/connect", post(broker::connect))
        .route("/api/status", get(broker::status))
        .route("/api/feed/status", get(broker::feed_status))
        .route("/api/feed/reset", post(broker::feed_reset))
        .route("/api/feed/restart", post(broker::feed_restart))
        .route("/api/account", get(realtime::account_overview))
        .route("/api/backup/config", get(backup::config_get).post(backup::config_post))
        .route("/api/backup/path_test", post(backup::path_test_handler))
        .route("/api/backup/snapshot", post(backup::snapshot))
        .route("/api/backup/list", get(backup::list))
        .route("/api/backup/read", get(backup::read))
        .route("/api/backup/import", post(backup::import))
        .merge(realtime::router::<AppState>())
        .with_state(state)
        .merge(realtime::paper_router().with_state(paper_rt))
        .fallback_service(ServeDir::new(static_dir).fallback(index))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ))
        .layer(CorsLayer::permissive());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("new Rust algo app listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    let _ = StatusCode::OK;
}
