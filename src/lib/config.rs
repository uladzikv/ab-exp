use anyhow::Context;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_port: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Config> {
        dotenv().expect("failed to load .env file");

        let server_port = load_env("DATABASE_URL")?;
        let database_url = load_env("SERVER_PORT")?;

        Ok(Config {
            server_port,
            database_url,
        })
    }
}

fn load_env(key: &str) -> anyhow::Result<String> {
    env::var(key).with_context(|| format!("failed to load environment variable {}", key))
}
