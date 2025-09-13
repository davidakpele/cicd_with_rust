use crate::repository::github_account_repository::GithubAccountRepository;
use crate::models::github_account::GitHubAccount;
use anyhow::Result;
use serde_json::json;

pub struct GithubAccountService {
    pub repository: GithubAccountRepository,
}

impl GithubAccountService {
    pub fn new(repository: GithubAccountRepository) -> Self {
        Self { repository }
    }

    pub async fn save_or_update_user(
        &self,
        user_id: i64,
        github_id: i64,
        username: &str,
        email: Option<String>,
        avatar_url: Option<String>,
        github_token: &str,
    ) -> Result<GitHubAccount> {
        self.repository
            .upsert_github_account(user_id, github_id, username, email, avatar_url, github_token)
            .await
    }

    pub async fn save_or_fetch_user(
        &self,
        user_id: i64,
        github_id: i64,
        username: &str,
        email: Option<String>,
        avatar_url: Option<String>,
        github_token: &str,
    ) -> Result<GitHubAccount> {
        if let Some(user) = self.repository.find_by_username(username).await? {
            Ok(user)
        } else {
            self.repository
                .upsert_github_account(user_id, github_id, username, email, avatar_url, github_token)
                .await
        }
    }

    pub async fn get_user_repos(
        &self,
        github_token: &str,
    ) -> Result<Vec<serde_json::Value>, reqwest::Error> {
        self.repository.fetch_user_repos(github_token).await
    }


    pub async fn get_repo_by_id(&self, id: i64) -> Result<serde_json::Value> {
        match self.repository.fetch_repo_by_id(id).await? {
            Some(repo_json) => Ok(repo_json),
            None => Err(anyhow::anyhow!("Repository not found")),
        }
    }


    pub async fn delete_repo(&self, id: i64) -> Result<()> {
        self.repository.delete_repo_by_id(id).await
    }

    pub async fn get_repos_by_username(
        &self,
        username: &str,
    ) -> Result<Vec<serde_json::Value>> {
        self.repository.fetch_repos_by_username(username).await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<serde_json::Value> {
        if let Some(user) = self.repository.find_by_username(username).await? {
            Ok(json!(user))
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }


    pub async fn get_repo_branches(
        &self,
        username: &str,
        repo_name: &str,
    ) -> Result<Vec<serde_json::Value>> {
        if let Some(user) = self.repository.find_by_username(username).await? {
            self.repository
                .fetch_branches_for_repo(&user.github_token, username, repo_name)
                .await
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    pub async fn get_user_account_by_id(
        &self,
        user_id: i64,
    ) -> Result<GitHubAccount, sqlx::Error> {
        self.repository.find_by_user_id(user_id).await
    }


    

}
