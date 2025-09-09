use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;



// Registered repository
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)] 
pub struct Repo {
    pub id: Uuid,
    pub name: String,          // Repo name
    pub url: String,           // Git clone URL
    pub branch: String,        // Default branch to build
    pub created_at: DateTime<Utc>,
}