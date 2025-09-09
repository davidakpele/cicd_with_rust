use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Serialize};
use validator::Validate;
use sqlx::PgPool;
use utoipa::path;
use crate::{payloads::{login_request::LoginRequest, register_request::RegisterRequest}, responses::login_responses::LoginResponse, services::auth_service::{self}};


#[derive(Serialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum AuthApiResponse<T> {
    Success(T),
    Error(ErrorResponse),
}


#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub targe: String,
}


#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = User),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "Email already in use", body = ErrorResponse)
    ),
    tag = "Auth"
)]
pub async fn register_user(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate payload first
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Validation error: {}", e),
                targe: "error".to_string(),
            }),
        )
            .into_response();
    }

    // Check if email exists first
    let user_repo = crate::repository::user_repository::UserRepository { db: db.clone() };
    let user_service = crate::services::user_service::UserService::new(user_repo);
    

     match user_service.get_user_by_username(payload.username.as_deref().unwrap()).await {
        Ok(_) => {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Username already taken, Choose another username.".to_string(),
                    targe: "username_error".to_string(),
                }),
            )
            .into_response();
        }
        Err(e) if !e.to_string().contains("not found") => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                    targe: "error".to_string(),
                }),
            )
            .into_response();
        }
        _ => {}
    }

    match user_service.get_user_by_email(payload.email.as_deref().unwrap()).await {
        Ok(_) => {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "Email already been used by another user.".to_string(),
                    targe: "email_error".to_string(),
                }),
            )
            .into_response();
        }
        Err(e) if !e.to_string().contains("not found") => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                    targe: "error".to_string(),
                }),
            )
            .into_response();
        }
        _ => {}
    }

    // Proceed with registration if email is available
    match auth_service::register_user(&db, payload).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e.to_string(),targe: "".to_string() })).into_response(),
    }
}


#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthApiResponse<LoginResponse>), // <-- Correct body type for success
        (status = 400, description = "Validation error", body = AuthApiResponse<ErrorResponse>), // <-- Use AuthApiResponse here
        (status = 401, description = "Invalid credentials", body = AuthApiResponse<ErrorResponse>) // <-- Use AuthApiResponse here
    ),
    tag = "Auth"
)]
pub async fn login_user(
    Extension(db): Extension<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthApiResponse::Error(ErrorResponse {
                error: format!("Validation error: {}", e),
                 targe: "error".to_string(),
            })),
        );
    }

    match auth_service::login_user(&db, payload).await {
        Ok(auth_response) => {
            let login_response = LoginResponse {
                token: auth_response.token,
                id: auth_response.id,
                email: auth_response.email,
                username: auth_response.username,
            };
            (StatusCode::OK, Json(AuthApiResponse::Success(login_response)))
        },
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(AuthApiResponse::Error(ErrorResponse {
                error: e.to_string(),
                targe: "error".to_string(),
            })),
        ),
    }

}