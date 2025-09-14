use sqlx::{PgPool, Row}; // Add Row trait import
use std::process::Command;
use crate::models::deployment::Deployment;
use std::fs;
use anyhow::{Result, Context, anyhow};

pub struct DeploymentRepository {
    pub db: PgPool,
}

impl DeploymentRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create_deployment(
        &self,
        user_id: i64,
        repo_name: &str,
        branch: &str,
    ) -> Result<Deployment> {
        let record = sqlx::query_as::<_, Deployment>(
            r#"
            INSERT INTO deployments (
                user_id, repo_name, branch, status, port, in_use, created_at, updated_at
            )
            VALUES ($1, $2, $3, 'pending', NULL, false, now(), now())
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(repo_name)
        .bind(branch)
        .fetch_one(&self.db)
        .await?;

        Ok(record)
    }

    pub async fn update_status(&self, id: i64, status: &str, logs: Option<&str>) -> Result<()> {
        sqlx::query(
            "UPDATE deployments SET status = $1, logs = $2, updated_at = now() WHERE id = $3"
        )
        .bind(status)
        .bind(logs)
        .bind(id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    // Run `git clone` into a target directory
    pub fn clone_repo(&self, repo_url: &str, branch: &str, target_dir: &str) -> Result<String> {
        let output = Command::new("git")
            .args(&["clone", "--branch", branch, repo_url, target_dir])
            .output()
            .with_context(|| format!("Failed to clone repository from {}", repo_url))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn assign_port(&self, deployment_id: i64) -> Result<i32> {
        let mut tx = self.db.begin().await?;

        // 1. Try to find a released port first (lock candidate row)
        if let Some(row) = sqlx::query(
            r#"
            SELECT port 
            FROM deployments 
            WHERE in_use = false AND port IS NOT NULL
            ORDER BY port ASC 
            LIMIT 1
            FOR UPDATE SKIP LOCKED
            "#
        )
        .fetch_optional(&mut *tx)
        .await?
        {
            let port: i32 = row.get("port");

            sqlx::query(
                r#"
                UPDATE deployments 
                SET port = $1, in_use = true, updated_at = now() 
                WHERE id = $2
                "#
            )
            .bind(port)
            .bind(deployment_id)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            return Ok(port);
        }

        // 2. Otherwise lock the table, then compute the next available port
        let row = sqlx::query(
            r#"
            SELECT port 
            FROM deployments 
            WHERE port IS NOT NULL
            ORDER BY port DESC 
            LIMIT 1
            FOR UPDATE
            "#
        )
        .fetch_optional(&mut *tx)
        .await?;

        let new_port = match row {
            Some(r) => {
                let max_port: Option<i32> = r.get("port");
                max_port.unwrap_or(2999) + 1
            }
            None => 3000,
        };

        if new_port > 9000 {
            tx.rollback().await?;
            anyhow::bail!("No free ports available in range 3000-9000");
        }

        sqlx::query(
            r#"
            UPDATE deployments 
            SET port = $1, in_use = true, updated_at = now() 
            WHERE id = $2
            "#
        )
        .bind(new_port)
        .bind(deployment_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(new_port)
    }

    // When a deployment stops, fails, or is deleted, the port is freed for reuse. 
    pub async fn release_port(&self, deployment_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE deployments 
            SET in_use = false, updated_at = now(), status = 'stopped'
            WHERE id = $1
            "#
        )
        .bind(deployment_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}