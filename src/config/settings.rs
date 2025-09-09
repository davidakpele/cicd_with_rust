use serde::Deserialize;
use dotenv::dotenv;
use std::env;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub git_client_id: String,
    pub git_client_secret: String,
}


impl Settings {
    pub fn new() -> Self {
        dotenv().ok();

        Settings {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            git_client_id: env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
            git_client_secret: env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set"),
        }
    }
}