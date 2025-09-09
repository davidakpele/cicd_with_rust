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
    models::repo::Repo,
    services::repo_service,
};

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateRepoRequest {
    pub name: String,
    pub url: String,
    pub branch: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum RepoApiResponse {
    Success(Repo),
    Error(ErrorResponse),
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub target: String,
}

#[utoipa::path(
    post,
    path = "/repos",
    request_body = CreateRepoRequest,
    responses(
        (status = 201, description = "Repository created successfully", body = Repo),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Repos"
)]
pub async fn create_repo(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<CreateRepoRequest>,
) -> impl IntoResponse {
    match repo_service::create_repo(&db, &payload.name, &payload.url, &payload.branch).await {
        Ok(repo) => (
            StatusCode::CREATED,
            Json(RepoApiResponse::Success(repo)),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(RepoApiResponse::Error(ErrorResponse {
                error: e.to_string(),
                target: "repo_creation".to_string(),
            })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/repos/{id}",
    responses(
        (status = 200, description = "Repository found", body = Repo),
        (status = 404, description = "Repository not found", body = ErrorResponse),
    ),
    params(
        ("id" = Uuid, Path, description = "Repo ID")
    ),
    tag = "Repos"
)]
pub async fn get_repo(
    Extension(db): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo_service::get_repo(&db, id).await {
        Ok(repo) => (StatusCode::OK, Json(RepoApiResponse::Success(repo))).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(RepoApiResponse::Error(ErrorResponse {
                error: "Repository not found".to_string(),
                target: "repo".to_string(),
            })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/repos",
    responses(
        (status = 200, description = "List of repositories", body = [Repo]),
        (status = 500, description = "Server error", body = ErrorResponse),
    ),
    tag = "Repos"
)]
pub async fn list_repos(
    Extension(db): Extension<PgPool>,
) -> impl IntoResponse {
    match repo_service::list_repos(&db).await {
        Ok(repos) => (StatusCode::OK, Json(repos)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                target: "repo_list".to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/repos/{id}",
    responses(
        (status = 200, description = "Repository deleted"),
        (status = 404, description = "Repository not found", body = ErrorResponse),
    ),
    params(
        ("id" = Uuid, Path, description = "Repo ID")
    ),
    tag = "Repos"
)]
pub async fn delete_repo(
    Extension(db): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match repo_service::delete_repo(&db, id).await {
        Ok(rows) if rows > 0 => StatusCode::OK.into_response(),
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Repository not found".to_string(),
                target: "repo".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                target: "repo_delete".to_string(),
            }),
        )
            .into_response(),
    }
}
