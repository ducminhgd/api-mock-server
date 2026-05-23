use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            jwt_secret: require_env("JWT_SECRET")?,
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse::<u16>()
                .map_err(|_| "PORT must be a valid u16".to_string())?,
        })
    }
}

fn require_env(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("required env var {key} is not set"))
}
