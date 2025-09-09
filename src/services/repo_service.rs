use crate::models::repo::Repo;
use crate::repository::repo_repository::RepoRepository;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Create a new repository
pub async fn create_repo(
    db: &PgPool,
    name: &str,
    url: &str,
    branch: &str,
) -> Result<Repo> {
    let repo = RepoRepository { db: db.clone() };
    let new_repo = repo.create_repo(name, url, branch).await?;
    Ok(new_repo)
}

/// Get a repository by ID
pub async fn get_repo(
    db: &PgPool,
    repo_id: Uuid,
) -> Result<Repo> {
    let repo = RepoRepository { db: db.clone() };
    let found = repo.get_repo(repo_id).await?;
    Ok(found)
}

/// List all repositories
pub async fn list_repos(db: &PgPool) -> Result<Vec<Repo>> {
    let repo = RepoRepository { db: db.clone() };
    let repos = repo.list_repos().await?;
    Ok(repos)
}

/// Delete a repository by ID
pub async fn delete_repo(
    db: &PgPool,
    repo_id: Uuid,
) -> Result<u64> {
    let repo = RepoRepository { db: db.clone() };
    let rows_affected = repo.delete_repo(repo_id).await?;
    Ok(rows_affected)
}
