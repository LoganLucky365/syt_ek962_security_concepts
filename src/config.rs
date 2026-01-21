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
    pub ldap: Option<LdapConfig>,
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone)]
pub struct LdapConfig {
    // ldap url zb ldap://dc-01.tgm.ac.at:389 or ldaps://dc-01.tgm.ac.at:636
    pub url: String,
    pub user_base_dn: String,
    // domain name
    pub domain: String,
    // use user@domain instead of dn
    pub use_upn: bool,
    pub use_starttls: bool,
    pub username_attribute: String,
    pub timeout_secs: u64,
    // ad groupe name for admin role
    pub admin_group: Option<String>,
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
            ldap: Self::ldap_from_env(),
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

    fn ldap_from_env() -> Option<LdapConfig> {
        let url = env::var("LDAP_URL").ok()?;
        let user_base_dn = env::var("LDAP_USER_BASE_DN").ok()?;
        let domain = env::var("LDAP_DOMAIN").ok()?;

        let use_upn = env::var("LDAP_USE_UPN")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true); // Default to UPN for AD

        let use_starttls = env::var("LDAP_USE_STARTTLS")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let username_attribute = env::var("LDAP_USERNAME_ATTRIBUTE")
            .unwrap_or_else(|_| "sAMAccountName".to_string());

        let timeout_secs = env::var("LDAP_TIMEOUT_SECS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .unwrap_or(10);

        let admin_group = env::var("LDAP_ADMIN_GROUP").ok();

        Some(LdapConfig {
            url,
            user_base_dn,
            domain,
            use_upn,
            use_starttls,
            username_attribute,
            timeout_secs,
            admin_group,
        })
    }
}

pub fn validate_jwt_secret(secret: &str) -> Result<(), String> {
    if secret.len() < 32 {
        return Err("JWT_SECRET must be at least 32 characters (256 bits)".to_string());
    }
    Ok(())
}
