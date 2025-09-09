use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use crate::models::users::User;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct SafeUser {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub is_verified: bool,
    pub is_staff: bool,
    pub last_login: Option<NaiveDateTime>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for SafeUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            is_active: user.is_active,
            is_admin: user.is_admin,
            is_verified: user.is_verified,
            is_staff: user.is_staff,
            last_login: user.last_login,
            email_verified_at: user.email_verified_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}