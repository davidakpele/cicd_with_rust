use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)] 
pub struct JobLog {
    pub id: Uuid,
    pub job_id: Uuid,          // Foreign key to Job
    pub line: String,          // Log line
    pub created_at: DateTime<Utc>,
}