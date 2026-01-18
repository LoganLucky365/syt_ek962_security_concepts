//! Authentication module providing various authentication mechanisms.
//!
//! This module is designed for extensibility:
//! - `password`: Secure password hashing using Argon2
//! - `jwt`: JWT token generation and validation
//! - `provider`: Trait-based authentication providers for future OAuth/AD support

mod password;
mod jwt;
mod provider;

pub use password::PasswordHasher;
pub use jwt::{JwtService, Claims};
pub use provider::{AuthProvider, AuthResult, LocalAuthProvider};
