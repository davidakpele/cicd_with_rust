use crate::models::repo::Repo;
use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct RepoRepository {
    pub db: PgPool,
}

impl RepoRepository {
    /// Create a new repository record
    pub async fn create_repo(
        &self,
        name: &str,
        url: &str,
        branch: &str,
    ) -> Result<Repo> {
        let rec = sqlx::query_as::<_, Repo>(
            r#"
            INSERT INTO repos (id, name, url, branch, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(url)
        .bind(branch)
        .fetch_one(&self.db)
        .await?;

        Ok(rec)
    }

    /// Get a repo by ID
    pub async fn get_repo(&self, repo_id: Uuid) -> Result<Repo> {
        let rec = sqlx::query_as::<_, Repo>(
            r#"
            SELECT * FROM repos WHERE id = $1
            "#
        )
        .bind(repo_id)
        .fetch_one(&self.db)
        .await?;

        Ok(rec)
    }

    /// List all repos
    pub async fn list_repos(&self) -> Result<Vec<Repo>> {
        let recs = sqlx::query_as::<_, Repo>(
            r#"
            SELECT * FROM repos ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(recs)
    }

    /// Delete a repo
    pub async fn delete_repo(&self, repo_id: Uuid) -> Result<u64> {
        let rows_affected = sqlx::query(
            r#"
            DELETE FROM repos WHERE id = $1
            "#
        )
        .bind(repo_id)
        .execute(&self.db)
        .await?
        .rows_affected();

        Ok(rows_affected)
    }
}
