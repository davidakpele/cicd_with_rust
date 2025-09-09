use axum::{
    extract::{Query, Extension},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Json, Router,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode,
    ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{repository::github_account_repository::GithubAccountRepository, services::github_account_service::{self, GithubAccountService}};

#[derive(Clone)]
pub struct AppState {
    pub oauth_client: BasicClient,
    pub db: PgPool,
}

// Redirect user to GitHub login page
pub async fn github_login(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = state.oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    Redirect::to(auth_url.as_str())
}

// GitHub redirects back here with code
#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}


    pub async fn github_callback(
    Extension(state): Extension<Arc<AppState>>,
    Query(query): Query<AuthRequest>,
) -> impl IntoResponse {
    let token_res = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await;

    match token_res {
        Ok(token) => {
            let access_token = token.access_token().secret().to_string();

            let client = reqwest::Client::new();
            let user_info: serde_json::Value = client
                .get("https://api.github.com/user")
                .header("Authorization", format!("token {}", access_token))
                .header("User-Agent", "axum-ci-cd")
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();

            let github_id = user_info["id"].as_i64().unwrap();
            let username = user_info["login"].as_str().unwrap().to_string();
            let avatar_url = user_info["avatar_url"].as_str().map(|s| s.to_string());
            let github_profile_url = user_info["url"].as_str().map(|s| s.to_string());
            let github_repo_url = user_info["repos_url"].as_str().map(|s| s.to_string());
            let email = user_info["email"].as_str().map(|s| s.to_string());

            let repo = GithubAccountRepository::new(state.db.clone());
            let service = GithubAccountService::new(repo);

            match service
                .save_or_fetch_user(github_id, &username, email, avatar_url, &access_token)
                .await
            {
                Ok(user) => {
                    // Fetch user repos via service
                    let repos = service.get_user_repos(&access_token).await.unwrap_or_default();

                    let result = serde_json::json!({
                        "user": user,
                        "repos": repos
                    });

                    (StatusCode::OK, Json(result)).into_response()
                }
                Err(e) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }

        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err.to_string() })),
        )
        .into_response(),
    }


}
