use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    pub encryption_key: String,
    pub environment: Environment,
    pub max_connections: u32,
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost:5432/mindful_code".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse()
            .unwrap_or(3001);

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-in-production".to_string());

        let encryption_key = env::var("ENCRYPTION_KEY")
            .unwrap_or_else(|_| "change-this-32-byte-key-in-production!!".to_string());

        let environment = match env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .as_str()
        {
            "production" => Environment::Production,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        let max_connections = env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);

        let worker_threads = env::var("TOKIO_WORKER_THREADS")
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .unwrap_or(4);

        Ok(Config {
            database_url,
            port,
            jwt_secret,
            encryption_key,
            environment,
            max_connections,
            worker_threads,
        })
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }
}