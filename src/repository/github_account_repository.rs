use sqlx::{PgPool, FromRow, Row}; // Add Row trait import
use serde::Deserialize;
use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use crate::models::github_account::GitHubAccount;

pub struct GithubAccountRepository {
    pub db: PgPool,
}

impl GithubAccountRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn upsert_github_account(
        &self,
        user_id: i64,
        github_id: i64,
        username: &str,
        email: Option<String>,
        avatar_url: Option<String>,
        github_token: &str,
    ) -> Result<GitHubAccount> {
        let now = Utc::now();

        let record = sqlx::query_as::<_, GitHubAccount>(
            r#"
            INSERT INTO github_accounts 
                (user_id, github_id, username, email, avatar_url, github_token, is_active, is_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, true, true, $7, $7)
            ON CONFLICT (github_id)
            DO UPDATE SET
                username = EXCLUDED.username,
                email = EXCLUDED.email,
                avatar_url = EXCLUDED.avatar_url,
                github_token = EXCLUDED.github_token,
                updated_at = EXCLUDED.updated_at
            RETURNING *
            "#
        )
        .bind(user_id)
        .bind(github_id)
        .bind(username)
        .bind(email)
        .bind(avatar_url)
        .bind(github_token)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        Ok(record)
    }

   
    pub async fn find_by_username(&self, username: &str) -> Result<Option<GitHubAccount>> {
        let record = sqlx::query_as::<_, GitHubAccount>(
            "SELECT * FROM github_accounts WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        Ok(record)
    }

    
    // Fetch GitHub user's repos using their token
    pub async fn fetch_user_repos(
        &self,
        github_token: &str,
    ) -> Result<Vec<serde_json::Value>, reqwest::Error> {
        let client = reqwest::Client::new();
        let repos: Vec<serde_json::Value> = client
            .get("https://api.github.com/user/repos")
            .header("Authorization", format!("token {}", github_token))
            .header("User-Agent", "axum-ci-cd")
            .send()
            .await?
            .json()
            .await?;

        Ok(repos)
    }

    pub async fn fetch_repos_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let client = reqwest::Client::new();
        let repos: Vec<serde_json::Value> = client
            .get(format!("https://api.github.com/users/{}/repos", username))
            .header("User-Agent", "axum-ci-cd")
            .send()
            .await?
            .json()
            .await?;
        Ok(repos)
    }

    pub async fn fetch_repo_by_id(&self, id: i64) -> Result<Option<serde_json::Value>> {
        if let Some(row) = sqlx::query(
            "SELECT * FROM github_accounts WHERE user_id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        {
            let repo_json = serde_json::json!({
                "user": {
                    "id": row.get::<i64, _>("id"),
                    "username": row.get::<String, _>("username"),
                    "url": row.get::<Option<String>, _>("github_repo_url"),
                    "profile_url": row.get::<Option<String>, _>("github_profile_url"),
                }
            });
            Ok(Some(repo_json))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_repo_by_id(&self, id: i64) -> Result<()> {
        sqlx::query(
            "DELETE FROM github_accounts WHERE id = $1"
        )
        .bind(id)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn fetch_branches_for_repo(
        &self,
        github_token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let client = reqwest::Client::new();
        let url = format!("https://api.github.com/repos/{}/{}/branches", owner, repo);

        let branches: Vec<serde_json::Value> = client
            .get(&url)
            .header("Authorization", format!("token {}", github_token))
            .header("User-Agent", "axum-ci-cd")
            .send()
            .await?
            .json()
            .await?;

        Ok(branches)
    }

    pub async fn find_by_user_id(&self, user_id: i64) -> Result<GitHubAccount, sqlx::Error> {
        let account = sqlx::query_as::<_, GitHubAccount>(
            r#"
            SELECT *
            FROM github_accounts
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.db)   
        .await?;

        Ok(account)
    }
}