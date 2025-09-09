use crate::repository::github_account_repository::GithubAccountRepository;
use crate::models::github_account::GitHubAccount;
use anyhow::Result;

pub struct GithubAccountService {
    pub repository: GithubAccountRepository,
}

impl GithubAccountService {
    pub fn new(repository: GithubAccountRepository) -> Self {
        Self { repository }
    }

    pub async fn save_or_update_user(
        &self,
        github_id: i64,
        username: &str,
        email: Option<String>,
        avatar_url: Option<String>,
        github_token: &str,
    ) -> Result<GitHubAccount> {
        self.repository
            .upsert_github_account(github_id, username, email, avatar_url, github_token)
            .await
    }

    pub async fn save_or_fetch_user(
        &self,
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
                .upsert_github_account(github_id, username, email, avatar_url, github_token)
                .await
        }
    }

    pub async fn get_user_repos(&self, github_token: &str) -> Result<Vec<serde_json::Value>> {
        self.repository.fetch_user_repos(github_token).await
    }


    pub async fn get_repo_by_id(&self, id: i64) -> Result<serde_json::Value> {
        self.repository.fetch_repo_by_id(id).await
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


}
