use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)] 
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub password: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub is_verified: bool,
    pub is_staff: bool,
    pub last_login: Option<NaiveDateTime>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}