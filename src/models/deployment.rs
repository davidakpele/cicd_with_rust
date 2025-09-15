use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Deployment {
    pub id: i64,
    pub user_id: i64,          // from github_accounts.id
    pub repo_name: String,
    pub branch: String,
    pub status: String,        // pending, running, success, failed
    pub deployment_id: Option<i64>, // self-reference, can be NULL
    pub container_id: Option<String>,
    pub port: Option<i32>,   
    pub live_url: Option<String>, 
    pub in_use: Option<bool>,  // default true
    pub logs: Option<String>,  // build logs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

}
