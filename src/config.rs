use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt: JwtConfig,
    pub initial_admin_config: String,
    pub google_oauth: Option<GoogleOAuthConfig>,
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_secs: i64,
    pub issuer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InitialAdminConfig {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub password_hash: Option<String>,
}

impl InitialAdminConfig {
    pub fn from_env() -> Option<Self> {
        let name = env::var("INITIAL_ADMIN_NAME").ok()?;
        let email = env::var("INITIAL_ADMIN_EMAIL").ok()?;

        let password_hash = env::var("INITIAL_ADMIN_PASSWORD_HASH").ok();
        let password = env::var("INITIAL_ADMIN_PASSWORD").ok();

        if password_hash.is_none() && password.is_none() {
            return None;
        }

        Some(Self {
            name,
            email,
            password,
            password_hash,
        })
    }
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./auth.db?mode=rwc".to_string()),
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| {
                    tracing::warn!(
                        "JWT_SECRET not set, using random secret. \
                         This will invalidate all tokens on restart!"
                    );
                    let secret: [u8; 32] = rand::random();
                    hex::encode(secret)
                }),
                expiration_secs: env::var("JWT_EXPIRATION_SECS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .expect("JWT_EXPIRATION_SECS must be a valid number"),
                issuer: env::var("JWT_ISSUER")
                    .unwrap_or_else(|_| "auth-service".to_string()),
            },
            initial_admin_config: env::var("INITIAL_ADMIN_CONFIG")
                .unwrap_or_else(|_| "initial_admin.json".to_string()),
            google_oauth: Self::google_oauth_from_env(),
        }
    }

    fn google_oauth_from_env() -> Option<GoogleOAuthConfig> {
        let client_id = env::var("GOOGLE_CLIENT_ID").ok()?;
        let client_secret = env::var("GOOGLE_CLIENT_SECRET").ok()?;
        let redirect_uri = env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:8080/auth/google/callback".to_string());

        Some(GoogleOAuthConfig {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

pub fn validate_jwt_secret(secret: &str) -> Result<(), String> {
    if secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters (256 bits)".to_string());
    }
    Ok(())
}
