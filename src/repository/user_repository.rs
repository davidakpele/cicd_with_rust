use sqlx::{PgPool, Row, postgres::PgRow};
use crate::models::users::User;
use crate::controllers::user_controller::UpdateUserRequest;
use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;

pub struct UserRepository {
    pub db: PgPool,
}

impl UserRepository {
    pub async fn find_by_email(&self, email: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, 
                email, 
                username, 
                password, 
                is_active, 
                is_admin, 
                is_verified, 
                is_staff,
                last_login,
                email_verified_at,
                created_at,
                updated_at
            FROM users 
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(row) => Ok(map_user(row)),
            None => Err(anyhow!("User not found")),
        }
    }

    pub async fn find_by_id(&self, user_id: i32) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, 
                email, 
                username, 
                password, 
                is_active, 
                is_admin, 
                is_verified, 
                is_staff,
                last_login,
                email_verified_at,
                created_at,
                updated_at
            FROM users 
            WHERE id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(row) => Ok(map_user(row)),
            None => Err(anyhow!("User not found")),
        }
    }

    pub async fn update_user(&self, user_id: i32, payload: UpdateUserRequest) -> Result<User> {
        let current = self.find_by_id(user_id).await?;

        let new_username = payload.username.unwrap_or(current.username);
        let new_email = payload.email.unwrap_or(current.email);

        sqlx::query(
            "UPDATE users SET username = $1, email = $2 WHERE id = $3"
        )
        .bind(new_username)
        .bind(new_email)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        self.find_by_id(user_id).await
    }

    pub async fn delete_user(&self, user_id: i32) -> Result<()> {
        let rows = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(anyhow!("User not found or already deleted"));
        }

        Ok(())
    }


    pub async fn find_by_username(&self, username: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, 
                email, 
                username, 
                password, 
                is_active, 
                is_admin, 
                is_verified, 
                is_staff,
                last_login,
                email_verified_at,
                created_at,
                updated_at
            FROM users 
            WHERE username = $1
            "#
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some(row) => Ok(map_user(row)),
            None => Err(anyhow!("User not found")),
        }
    }


}

fn map_user(row: PgRow) -> User {
    User {
        id: row.get("id"),
        email: row.get("email"),
        username: row.get("username"),
        password: row.get("password"),
        is_active: row.get("is_active"),
        is_admin: row.get("is_admin"),
        is_verified: row.get("is_verified"),
        is_staff: row.get("is_staff"),
        last_login: row.get::<Option<NaiveDateTime>, _>("last_login"),
        email_verified_at: row.get::<Option<NaiveDateTime>, _>("email_verified_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}