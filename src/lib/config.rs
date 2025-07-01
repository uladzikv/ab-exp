use anyhow::Context;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_port: String,
    pub database_url: String,
    pub auth_token: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Config> {
        dotenv().expect("failed to load .env file");

        let server_port = load_env("SERVER_PORT")?;
        let database_url = load_env("DATABASE_URL")?;
        let auth_token = load_env("AUTH_TOKEN")?;

        Ok(Config {
            server_port,
            database_url,
            auth_token,
        })
    }
}

fn load_env(key: &str) -> anyhow::Result<String> {
    env::var(key).with_context(|| format!("failed to load environment variable {}", key))
}
