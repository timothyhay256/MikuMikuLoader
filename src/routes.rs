use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use log::info;
use sekai_injector::{Manager, RequestParams};
use tokio::sync::RwLock;

// TODO: Use SSE and channels
pub async fn total_passthrough(State(state): State<Arc<RwLock<Manager>>>) -> impl IntoResponse {
    state.read().await.statistics.request_count.0.to_string()
}

pub async fn total_proxied(State(state): State<Arc<RwLock<Manager>>>) -> impl IntoResponse {
    state.read().await.statistics.request_count.1.to_string()
}

pub async fn requests(State(state): State<Arc<RwLock<Manager>>>) -> Json<Vec<RequestParams>> {
    Json(state.read().await.statistics.requests.clone()) // TODO: This is expensive. and silly (in a bad way). dont do it.
}

pub async fn set_serve_param(
    State(state): State<Arc<RwLock<Manager>>>,
    Path(param): Path<String>,
) -> String {
    match param.as_str() {
        "start" => {
            info!("Server start requested by web");
            state.write().await.config.inject_resources = false
        }
        "stop" => {
            info!("Server stop requested by web");
            state.write().await.config.inject_resources = true
        }
        _ => return "invalid command".to_string(),
    }

    "Success".to_string()
}
