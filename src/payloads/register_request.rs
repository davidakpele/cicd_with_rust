use validator::Validate;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct RegisterRequest {
    #[validate(email, required)]
    pub email: Option<String>,

    #[validate(length(min = 6, message = "Password must be at least 6 characters"), required)]
    pub password: Option<String>,

    #[validate(length(min = 3, message = "Username must be at least 3 characters"), required)]
    pub username: Option<String>,
}