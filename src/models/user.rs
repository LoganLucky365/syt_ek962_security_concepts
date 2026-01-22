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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== UserRole Tests ====================

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::User.to_string(), "user");
        assert_eq!(UserRole::Admin.to_string(), "admin");
    }

    #[test]
    fn test_user_role_from_str() {
        assert_eq!("user".parse::<UserRole>().unwrap(), UserRole::User);
        assert_eq!("admin".parse::<UserRole>().unwrap(), UserRole::Admin);
        assert_eq!("User".parse::<UserRole>().unwrap(), UserRole::User);
        assert_eq!("ADMIN".parse::<UserRole>().unwrap(), UserRole::Admin);
    }

    #[test]
    fn test_user_role_from_str_invalid() {
        assert!("invalid".parse::<UserRole>().is_err());
        assert!("".parse::<UserRole>().is_err());
        assert!("superuser".parse::<UserRole>().is_err());
    }

    // ==================== AuthProviderType Tests ====================

    #[test]
    fn test_auth_provider_type_display() {
        assert_eq!(AuthProviderType::Local.to_string(), "local");
        assert_eq!(AuthProviderType::Google.to_string(), "google");
        assert_eq!(AuthProviderType::ActiveDirectory.to_string(), "activedirectory");
        assert_eq!(AuthProviderType::Oidc.to_string(), "oidc");
    }

    // ==================== User Creation Tests ====================

    #[test]
    fn test_new_local_user() {
        let user = User::new_local(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::User,
        );

        assert!(!user.id.is_empty());
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, Some("hashed_password".to_string()));
        assert_eq!(user.role, UserRole::User);
        assert_eq!(user.auth_provider, AuthProviderType::Local);
        assert!(user.external_id.is_none());
        assert!(user.is_active);
    }

    #[test]
    fn test_new_local_admin() {
        let user = User::new_local(
            "Admin User".to_string(),
            "admin@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::Admin,
        );

        assert_eq!(user.role, UserRole::Admin);
        assert!(user.is_admin());
    }

    #[test]
    fn test_new_external_user_google() {
        let user = User::new_external(
            "Google User".to_string(),
            "google@example.com".to_string(),
            AuthProviderType::Google,
            "google-sub-123".to_string(),
            UserRole::User,
        );

        assert!(!user.id.is_empty());
        assert_eq!(user.name, "Google User");
        assert_eq!(user.email, "google@example.com");
        assert!(user.password_hash.is_none());
        assert_eq!(user.role, UserRole::User);
        assert_eq!(user.auth_provider, AuthProviderType::Google);
        assert_eq!(user.external_id, Some("google-sub-123".to_string()));
        assert!(user.is_active);
    }

    #[test]
    fn test_new_external_user_active_directory() {
        let user = User::new_external(
            "AD User".to_string(),
            "ad@company.com".to_string(),
            AuthProviderType::ActiveDirectory,
            "ad-username".to_string(),
            UserRole::Admin,
        );

        assert_eq!(user.auth_provider, AuthProviderType::ActiveDirectory);
        assert_eq!(user.external_id, Some("ad-username".to_string()));
        assert!(user.is_admin());
    }

    // ==================== User ID Tests ====================

    #[test]
    fn test_user_ids_are_unique() {
        let user1 = User::new_local(
            "User 1".to_string(),
            "user1@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        );
        let user2 = User::new_local(
            "User 2".to_string(),
            "user2@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        );

        assert_ne!(user1.id, user2.id);
    }

    #[test]
    fn test_user_id_is_valid_uuid() {
        let user = User::new_local(
            "Test".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        );

        // Should be parseable as UUID
        assert!(uuid::Uuid::parse_str(&user.id).is_ok());
    }

    // ==================== is_admin Tests ====================

    #[test]
    fn test_is_admin_returns_true_for_admin() {
        let user = User::new_local(
            "Admin".to_string(),
            "admin@example.com".to_string(),
            "hash".to_string(),
            UserRole::Admin,
        );

        assert!(user.is_admin());
    }

    #[test]
    fn test_is_admin_returns_false_for_user() {
        let user = User::new_local(
            "User".to_string(),
            "user@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        );

        assert!(!user.is_admin());
    }

    // ==================== Timestamps Tests ====================

    #[test]
    fn test_timestamps_are_set_on_creation() {
        let before = Utc::now();
        let user = User::new_local(
            "Test".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        );
        let after = Utc::now();

        assert!(user.created_at >= before && user.created_at <= after);
        assert!(user.updated_at >= before && user.updated_at <= after);
        assert_eq!(user.created_at, user.updated_at);
    }

    // ==================== UserResponse Tests ====================

    #[test]
    fn test_user_response_from_user() {
        let user = User::new_local(
            "Test User".to_string(),
            "test@example.com".to_string(),
            "secret_hash".to_string(),
            UserRole::User,
        );

        let response: UserResponse = user.clone().into();

        assert_eq!(response.id, user.id);
        assert_eq!(response.name, user.name);
        assert_eq!(response.email, user.email);
        assert_eq!(response.role, user.role);
        assert_eq!(response.created_at, user.created_at);
    }

    #[test]
    fn test_user_response_does_not_include_password() {
        let user = User::new_local(
            "Test".to_string(),
            "test@example.com".to_string(),
            "secret_password_hash".to_string(),
            UserRole::User,
        );

        let response: UserResponse = user.into();
        let json = serde_json::to_string(&response).unwrap();

        assert!(!json.contains("secret_password_hash"));
        assert!(!json.contains("password"));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_user_serialization_excludes_password_hash() {
        let user = User::new_local(
            "Test".to_string(),
            "test@example.com".to_string(),
            "secret_hash".to_string(),
            UserRole::User,
        );

        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.contains("secret_hash"));
    }

    #[test]
    fn test_user_role_serialization() {
        let json = serde_json::to_string(&UserRole::Admin).unwrap();
        assert_eq!(json, "\"Admin\"");

        let json = serde_json::to_string(&UserRole::User).unwrap();
        assert_eq!(json, "\"User\"");
    }

    #[test]
    fn test_user_role_deserialization() {
        let role: UserRole = serde_json::from_str("\"Admin\"").unwrap();
        assert_eq!(role, UserRole::Admin);

        let role: UserRole = serde_json::from_str("\"User\"").unwrap();
        assert_eq!(role, UserRole::User);
    }
}
