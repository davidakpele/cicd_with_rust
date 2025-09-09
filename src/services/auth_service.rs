use rand::rngs::OsRng; 
use crate::payloads::login_request::LoginRequest;
use crate::payloads::register_request::RegisterRequest;
use crate::responses::login_responses::LoginResponse;
use crate::responses::responses::SafeUser;
use crate::{
    repository::auth_repository::AuthenticationRepository,
    utils::jwt::generate_token,
};
use anyhow::{Result, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{SaltString};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn register_user(
    db: &PgPool,
    req: RegisterRequest,
) -> Result<SafeUser> {
    let hashed_password = hash_password(&req.password.as_deref().unwrap())?;
    let repo = AuthenticationRepository { db: db.clone() };
    let user = repo.create_user(&req.email.as_deref().unwrap(), &req.username.as_deref().unwrap(), &hashed_password).await?;
    Ok(user)
}

pub async fn login_user(
    db: &PgPool,
    req: LoginRequest,
) -> Result<LoginResponse> {
    let repo = AuthenticationRepository { db: db.clone() };
    let user = repo.login_user(&req.email.as_deref().unwrap()).await?;

    let is_valid = verify_password(&req.password.as_deref().unwrap(), &user.password)?;
    if !is_valid {
        return Err(anyhow!("Invalid credentials"));
    }

    let is_admin = user.is_admin;

    let token = generate_token(
        user.id,
        user.email.clone(),
        is_admin,
    )?;

    Ok(LoginResponse {
        token,
        id: user.id,
        email: user.email.clone(),
        username: user.username.clone(),
    })
}

fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e.to_string()))? 
        .to_string();

    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow!(e.to_string()))?;

    let is_valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(is_valid)
}