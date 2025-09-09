use crate::enums::job_status::JobStatus;
use crate::models::job::{Job};
use crate::repository::job_repository::JobRepository;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Create a new job for a repo
pub async fn create_job(
    db: &PgPool,
    repo_id: String,
    commit_hash: &str,
) -> Result<Job> {
    let repo = JobRepository { db: db.clone() };
    let job = repo.create_job(repo_id, commit_hash).await?;
    Ok(job)
}

/// Get job by ID
pub async fn get_job(
    db: &PgPool,
    job_id: Uuid,
) -> Result<Job> {
    let repo = JobRepository { db: db.clone() };
    let job = repo.get_job(job_id).await?;
    Ok(job)
}

/// List all jobs for a repo
pub async fn list_jobs_by_repo(
    db: &PgPool,
    repo_id: Uuid,
) -> Result<Vec<Job>> {
    let repo = JobRepository { db: db.clone() };
    let jobs = repo.list_jobs_by_repo(repo_id).await?;
    Ok(jobs)
}

/// Update job status
pub async fn update_job_status(
    db: &PgPool,
    job_id: Uuid,
    status: JobStatus,
) -> Result<Job> {
    let repo = JobRepository { db: db.clone() };
    let job = repo.update_status(job_id, status).await?;
    Ok(job)
}
