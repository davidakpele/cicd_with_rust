use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use chrono::{DateTime, NaiveDateTime, Utc};


#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)]
pub struct GitHubAccount {
    pub id: i64,
    pub user_id: i64, // just added this to tract user
    pub email: Option<String>,        // GitHub email (may be null)
    pub username: String,             // GitHub login name
    pub avatar_url: Option<String>,   // GitHub avatar
    pub github_profile_url: Option<String>,
    pub github_repo_url: Option<String>,
    pub github_id: i64,               // GitHub numeric id
    pub github_token: String,         // store encrypted access token
    pub is_active: bool,              // whether account is active in your system
    pub is_verified: bool,            // set true after OAuth success
    pub last_login: Option<NaiveDateTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
