use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use std::fmt;
use std::str::FromStr;
use anyhow::Error;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Success => "success",
            JobStatus::Failed => "failed",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for JobStatus {
    type Err = Error; // 👈 use anyhow::Error

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(JobStatus::Pending),
            "running" => Ok(JobStatus::Running),
            "success" => Ok(JobStatus::Success),
            "failed" => Ok(JobStatus::Failed),
            _ => Err(Error::msg(format!("Invalid job status: {}", s))),
        }
    }
}
