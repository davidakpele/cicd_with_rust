use serde::Deserialize;
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginRequest {
    /// Email of the user
    #[schema(example = "user@example.com")]
    #[validate(email, required)]
    pub email: Option<String>,

    /// Password of the user
    #[schema(example = "userpassword123")]
    #[validate(length(min = 6, message = "Password must be at least 6 characters"), required)]
    pub password: Option<String>,
}