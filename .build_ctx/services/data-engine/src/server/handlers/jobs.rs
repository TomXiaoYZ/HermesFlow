use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::server::routes::AppState;

#[derive(Debug, Deserialize)]
pub struct BackfillRequest {
    pub symbol: String,
    pub from: String,
    pub to: Option<String>,
}

#[derive(Serialize)]
pub struct JobResponse {
    pub status: String,
    pub message: String,
}

pub async fn trigger_backfill_job(
    State(state): State<AppState>,
    Json(payload): Json<BackfillRequest>,
) -> impl IntoResponse {
    if let Some(tm) = &state.task_manager {
        match tm.trigger_backfill(payload.symbol.clone(), payload.from.clone(), payload.to.clone()).await {
            Ok(_) => (
                StatusCode::ACCEPTED,
                Json(JobResponse {
                    status: "accepted".to_string(),
                    message: format!("Backfill triggered for {}", payload.symbol),
                }),
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse {
                    status: "error".to_string(),
                    message: e.to_string(),
                }),
            ),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(JobResponse {
                status: "error".to_string(),
                message: "Task Manager not initialized".to_string(),
            }),
        )
    }
}
