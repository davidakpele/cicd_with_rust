use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;
use utoipa::path;
use uuid::Uuid;

use crate::{
    enums::job_status::JobStatus,
    models::job::Job,
    services::job_service, // <- call service instead of repo
};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateJobRequest {
    pub repo_url: String,
    pub commit_hash: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum JobApiResponse {
    Success(Job),
    Error(ErrorResponse),
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub target: String,
}

#[utoipa::path(
    post,
    path = "/jobs",
    request_body = CreateJobRequest,
    responses(
        (status = 201, description = "Job created successfully", body = Job),
        (status = 400, description = "Invalid payload", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    tag = "Jobs"
)]
pub async fn create_job(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<CreateJobRequest>,
) -> impl IntoResponse {
    match job_service::create_job(&db, payload.repo_url.clone(), &payload.commit_hash).await {
        Ok(job) => (
            StatusCode::CREATED,
            Json(JobApiResponse::Success(job)),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JobApiResponse::Error(ErrorResponse {
                error: e.to_string(),
                target: "job_creation".to_string(),
            })),
        ).into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/jobs/{id}",
    responses(
        (status = 200, description = "Job found", body = Job),
        (status = 404, description = "Job not found", body = ErrorResponse),
    ),
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    tag = "Jobs"
)]
pub async fn get_job(
    Extension(db): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match job_service::get_job(&db, id).await {
        Ok(job) => (StatusCode::OK, Json(JobApiResponse::Success(job))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(JobApiResponse::Error(ErrorResponse {
                error: "Job not found".to_string(),
                target: "job".to_string(),
            })),
        ).into_response(),
    }
}

#[utoipa::path(
    patch,
    path = "/jobs/{id}/status",
    request_body = JobStatus,
    responses(
        (status = 200, description = "Job status updated", body = Job),
        (status = 404, description = "Job not found", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, Path, description = "Job ID")
    ),
    tag = "Jobs"
)]
pub async fn update_job_status(
    Extension(db): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Json(new_status): Json<JobStatus>,
) -> impl IntoResponse {
    match job_service::update_job_status(&db, id, new_status).await {
        Ok(job) => (StatusCode::OK, Json(JobApiResponse::Success(job))).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JobApiResponse::Error(ErrorResponse {
                error: "Failed to update job status".to_string(),
                target: "job_status".to_string(),
            })),    
        ).into_response(),
    }
}
