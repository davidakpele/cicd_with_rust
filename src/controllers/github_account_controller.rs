use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{bootstrap::github_auth_grant::AppState, services::github_account_service::GithubAccountService};
use crate::repository::github_account_repository::GithubAccountRepository;

#[utoipa::path(
    get,
    path = "/github/:username/repos",
    responses(
        (status = 200, description = "List of user repos", body = [serde_json::Value]),
        (status = 404, description = "User not found"),
    )
)]
pub async fn list_repos(
    Path(username): Path<String>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let repo = GithubAccountRepository::new(pool.clone());
    let service = GithubAccountService::new(repo);

    match service.get_repos_by_username(&username).await {
        Ok(repos) => (StatusCode::OK, Json(repos)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/github/repos/{id}",
    responses(
        (status = 200, description = "Repository found", body = serde_json::Value),
        (status = 404, description = "Repository not found"),
    )
)]
pub async fn fetch_repo_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let repo = GithubAccountRepository::new(pool.clone());
    let service = GithubAccountService::new(repo);

    match service.get_repo_by_id(id).await {
        Ok(repo) => (StatusCode::OK, Json(repo)).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/github/repos/{id}",
    responses(
        (status = 204, description = "Repository deleted"),
        (status = 404, description = "Repository not found"),
    )
)]
pub async fn delete_repo_by_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let repo = GithubAccountRepository::new(pool.clone());
    let service = GithubAccountService::new(repo);

    match service.delete_repo(id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "/repos/{username}/{repo}/branches",
    params(
        ("username" = String, Path, description = "GitHub username"),
        ("repo" = String, Path, description = "Repository name")
    ),
    responses(
        (status = 200, description = "List of branches", body = [serde_json::Value])
    )
)]
pub async fn list_branches(
    Extension(state): Extension<Arc<AppState>>,
    Path((username, repo_name)): Path<(String, String)>,   
) -> impl IntoResponse {
    let repo_repository = GithubAccountRepository::new(state.db.clone());  
    let service = GithubAccountService::new(repo_repository);

    match service.get_repo_branches(&username, &repo_name).await {  
        Ok(branches) => (StatusCode::OK, Json(branches)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
