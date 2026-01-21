//! Authentication module providing various authentication mechanisms.
//!
//! This module is designed for extensibility:
//! - `password`: Secure password hashing using Argon2
//! - `jwt`: JWT token generation and validation
//! - `provider`: Trait-based authentication providers for future OAuth/AD support
//! - `google`: Google OAuth 2.0 authentication
//! - `ldap`: LDAP/Active Directory authentication

mod password;
mod jwt;
mod provider;
mod google;
mod ldap;

pub use password::PasswordHasher;
pub use jwt::{JwtService, Claims};
pub use provider::{AuthProvider, AuthResult, LocalAuthProvider};
pub use google::GoogleAuthProvider;
pub use ldap::LdapAuthProvider;
