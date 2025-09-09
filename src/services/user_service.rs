use crate::repository::user_repository::UserRepository;
use crate::models::users::User;
use crate::controllers::user_controller::UpdateUserRequest;
use anyhow::Result;

pub struct UserService {
    pub repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User> {
        self.repository.find_by_email(email).await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        self.repository.find_by_username(username).await
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        match self.repository.find_by_id(user_id).await {
            Ok(user) => Ok(Some(user)),
            Err(e) if e.downcast_ref::<sqlx::Error>().map_or(false, |err| matches!(err, sqlx::Error::RowNotFound)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn update_user_profile(&self, user_id: i32, payload: UpdateUserRequest) -> Result<User> {
        self.repository.update_user(user_id, payload).await
    }

    pub async fn remove_user(&self, user_id: i32) -> Result<()> {
        self.repository.delete_user(user_id).await
    }

}