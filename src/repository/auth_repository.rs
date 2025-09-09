use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;
use crate::{models::users::User, responses::responses::SafeUser};
use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;

pub struct AuthenticationRepository {
    pub db: PgPool,
}

impl AuthenticationRepository {
    pub async fn create_user(&self, email: &str, username: &str, password: &str) -> Result<SafeUser> {
        let mut tx = self.db.begin().await?;
        let uuid = Uuid::new_v4().to_string();

        let row = sqlx::query(
            r#"
            INSERT INTO users (uuid, email, username, password)
            VALUES ($1, $2, $3, $4)
            RETURNING 
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
            "#
        )
        .bind(&uuid)
        .bind(email)
        .bind(username)
        .bind(password)
        .fetch_one(&mut *tx)
        .await?;

        let user = map_user(row);
        tx.commit().await?;
        Ok(SafeUser::from(user))
    }

    pub async fn login_user(&self, email: &str) -> Result<User> {
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
            None => Err(anyhow!("Invalid credentials")),
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