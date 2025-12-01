use axum::{Json, extract::State, http::StatusCode};
use common::dependency::DependencyRequest;

use crate::ServerState;

pub async fn fetch_dependency_information(
    State(state): State<ServerState>,
    Json(information): Json<DependencyRequest>,
) -> Result<Json<()>, StatusCode> {
    
    Ok(Json(()))
}

pub async fn fetch_dependency_source(
    State(state): State<ServerState>,
    Json(information): Json<DependencyRequest>,
) -> Result<Vec<u8>, StatusCode> {

    Ok(Vec::new())
}