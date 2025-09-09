use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::str::FromStr;

use crate::{enums::job_status::JobStatus, models::job::Job};

pub struct JobRepository {
    pub db: PgPool,
}

impl JobRepository {
    /// Create a new job for a repo
    pub async fn create_job(&self, repo_id: String, commit_hash: &str) -> Result<Job> {
        let mut tx = self.db.begin().await?;
        let job_id = Uuid::new_v4();

        let row = sqlx::query(
            r#"
            INSERT INTO jobs (id, repo_id, commit_hash, status, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, repo_id, commit_hash, status, created_at, started_at, finished_at
            "#
        )
        .bind(job_id)
        .bind(repo_id)
        .bind(commit_hash)
        .bind(JobStatus::Pending.to_string()) 
        .bind(Utc::now())
        .fetch_one(&mut *tx)
        .await?;

        let job = map_job(row)?;
        tx.commit().await?;
        Ok(job)
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: Uuid) -> Result<Job> {
        let row = sqlx::query(
            r#"
            SELECT id, repo_id, commit_hash, status, created_at, started_at, finished_at
            FROM jobs
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .fetch_one(&self.db)
        .await?;

        map_job(row)
    }

    /// Update job status (e.g. pending -> running -> success/failed)
    pub async fn update_status(&self, job_id: Uuid, status: JobStatus) -> Result<Job> {
        let row = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2, started_at = CASE WHEN $2 = 'running' THEN NOW() ELSE started_at END,
                finished_at = CASE WHEN $2 IN ('success', 'failed') THEN NOW() ELSE finished_at END
            WHERE id = $1
            RETURNING id, repo_id, commit_hash, status, created_at, started_at, finished_at
            "#
        )
        .bind(job_id)
        .bind(status.to_string())
        .fetch_one(&self.db)
        .await?;

        map_job(row)
    }

    /// List all jobs for a repo
    pub async fn list_jobs_by_repo(&self, repo_id: Uuid) -> Result<Vec<Job>> {
        let rows = sqlx::query(
            r#"
            SELECT id, repo_id, commit_hash, status, created_at, started_at, finished_at
            FROM jobs
            WHERE repo_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(repo_id)
        .fetch_all(&self.db)
        .await?;

        let jobs = rows.into_iter().map(|r| map_job(r).unwrap()).collect();
        Ok(jobs)
    }
}

fn map_job(row: PgRow) -> Result<Job> {
    Ok(Job {
        id: row.get("id"),
        repo_id: row.get("repo_id"),
        commit_hash: row.get("commit_hash"),
        status: JobStatus::from_str(&row.get::<String, _>("status"))?,
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        started_at: row.get("started_at"),
        finished_at: row.get("finished_at"),
    })
}
