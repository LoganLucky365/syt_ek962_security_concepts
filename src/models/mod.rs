//! Domain models for the authentication service.

mod user;

pub use user::{User, UserRole, CreateUser, AuthProviderType, UserResponse};
