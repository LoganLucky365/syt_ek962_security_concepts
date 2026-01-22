//! Authentication Service Library
//!
//! This library provides authentication mechanisms including:
//! - Local password authentication (Argon2)
//! - JWT token generation and validation
//! - Google OAuth 2.0
//! - LDAP/Active Directory authentication

pub mod auth;
pub mod config;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use auth::{JwtService, LocalAuthProvider, GoogleAuthProvider, LdapAuthProvider, Claims, AuthProvider, AuthResult, PasswordHasher};
pub use config::{Config, JwtConfig, GoogleOAuthConfig, LdapConfig, InitialAdminConfig};
pub use error::AppError;
pub use handlers::{AppState, configure_routes};
pub use models::{User, UserRole, AuthProviderType, UserResponse, CreateUser};
pub use repository::{UserRepository, SqliteUserRepository};
