use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    bootstrap::github_auth_grant::AppState, repository::deployment_repository::DeploymentRepository, services::deployment_service::DeploymentService
};

#[derive(Deserialize)]
pub struct DeployRequest {
    pub user_id: i64,
    pub repo_url: String,
    pub repo_name: String,
    pub branch: String,
}

pub async fn deploy_project(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<DeployRequest>,
) -> impl IntoResponse {
    let repo = DeploymentRepository::new(pool.clone());
    let service = DeploymentService::new(repo);

    match service
        .deploy_project(
            payload.user_id,
            &payload.repo_url,
            &payload.repo_name,
            &payload.branch, 
        )
        .await
    {
        Ok(deployment) => (StatusCode::CREATED, Json(deployment)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
