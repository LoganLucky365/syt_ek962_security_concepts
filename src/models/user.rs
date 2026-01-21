use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Admin => write!(f, "admin"),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(UserRole::User),
            "admin" => Ok(UserRole::Admin),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
pub enum AuthProviderType {
    Local,
    Google,
    ActiveDirectory,
    Oidc,
}

impl std::fmt::Display for AuthProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthProviderType::Local => write!(f, "local"),
            AuthProviderType::Google => write!(f, "google"),
            AuthProviderType::ActiveDirectory => write!(f, "activedirectory"),
            AuthProviderType::Oidc => write!(f, "oidc"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub role: UserRole,
    pub auth_provider: AuthProviderType,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl User {
    pub fn new_local(name: String, email: String, password_hash: String, role: UserRole) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            password_hash: Some(password_hash),
            role,
            auth_provider: AuthProviderType::Local,
            external_id: None,
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    pub fn new_external(
        name: String,
        email: String,
        provider: AuthProviderType,
        external_id: String,
        role: UserRole,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            password_hash: None,
            role,
            auth_provider: provider,
            external_id: Some(external_id),
            created_at: now,
            updated_at: now,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 1, max = 100, message = "name between 1 to 100 char"))]
    pub name: String,

    #[validate(email(message = "Invalid email"))]
    pub email: String,

    #[validate(length(min = 12, message = "Password at least 12 char"))]
    pub password: String,

    pub role: Option<UserRole>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            role: user.role,
            created_at: user.created_at,
        }
    }
}
