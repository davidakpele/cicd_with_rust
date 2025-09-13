use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use utoipa::ToSchema;

use crate::enums::job_status::JobStatus;

// CI/CD Job
#[derive(Debug, Serialize, Deserialize, FromRow, Clone, ToSchema)] 
pub struct Job {
    pub id: Uuid,
    pub repo_id: Uuid,         // Foreign key to Repo
    pub commit_hash: String,   // Commit SHA
    pub status: JobStatus,     // Enum: Pending, Running, Success, Failed
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

