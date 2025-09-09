use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub id: i64,
    pub email: String,
    pub username: String,
}