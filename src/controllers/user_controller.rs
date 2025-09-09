use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::path;

use crate::{repository::user_repository::UserRepository, responses::responses::SafeUser, services::user_service::UserService};

// The generic type T must be constrained to have a 'static lifetime,
// meaning it does not contain any borrowed references.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: Serialize + ToSchema<'static>,
{
    pub data: Option<T>,
    pub error: Option<String>,
}

// The IntoResponse implementation also needs to be generic over the same traits
// as the ApiResponse struct to satisfy the compiler's constraints.
impl<T: Serialize + ToSchema<'static> + Send + Sync + 'static> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

// The helper function also needs to be generic over the 'static lifetime for T.
fn api_response<T: Serialize + ToSchema<'static>>(
    status: StatusCode,
    data: Option<T>,
    error: Option<String>,
) -> impl IntoResponse {
    (status, Json(ApiResponse { data, error }))
}

#[utoipa::path(
    get,
    path = "/users/{user_id}",
    params(
        ("user_id" = i32, Path, description = "The ID of the user to retrieve.")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = ApiResponse<SafeUser>),
        (status = 404, description = "User not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "User"
)]
pub async fn get_user_by_id(
    Extension(db): Extension<PgPool>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let repository = UserRepository { db };
    let service = UserService::new(repository);

    match service.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            let public_user: SafeUser = user.into();
            api_response(StatusCode::OK, Some(public_user), None)
        },
        Ok(None) => {
            api_response(StatusCode::NOT_FOUND, None, Some("User not found".to_string()))
        },
        Err(e) => {
            api_response(StatusCode::INTERNAL_SERVER_ERROR, None, Some(e.to_string()))
        },
    }
}

#[utoipa::path(
    put,
    path = "/users/{user_id}",
    request_body = UpdateUserRequest,
    params(
        ("user_id" = i32, Path, description = "The ID of the user to update.")
    ),
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<SafeUser>),
        (status = 400, description = "Invalid request or user not found", body = ApiResponse<String>)
    ),
    tag = "User"
)]
pub async fn update_user(
    Extension(db): Extension<PgPool>,
    Path(user_id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let repository = UserRepository { db };
    let service = UserService::new(repository);
    
    match service.update_user_profile(user_id, payload).await {
        Ok(user) => api_response(StatusCode::OK, Some(user), None),
        Err(e) => api_response(StatusCode::BAD_REQUEST, None, Some(e.to_string())),
    }
}

#[utoipa::path(
    delete,
    path = "/users/{user_id}",
    params(
        ("user_id" = i32, Path, description = "The ID of the user to delete.")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 400, description = "Invalid request or user not found", body = ApiResponse<String>)
    ),
    tag = "User"
)]
pub async fn delete_user(
    Extension(db): Extension<PgPool>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let repository = UserRepository { db };
    let service = UserService::new(repository);
    
    match service.remove_user(user_id).await {
        Ok(_) => api_response(StatusCode::NO_CONTENT, None::<()>, None),
        Err(e) => api_response(StatusCode::BAD_REQUEST, None, Some(e.to_string())),
    }
}
