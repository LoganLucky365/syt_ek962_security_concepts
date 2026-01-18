use async_trait::async_trait;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::{User, UserRole};
use crate::repository::UserRepository;
use super::password::PasswordHasher;

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub user: User,
}

#[derive(Debug)]
pub struct LocalCredentials {
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &'static str;
    
    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult, AppError>;
}

pub struct LocalAuthProvider {
    repository: Arc<dyn UserRepository>,
    password_hasher: PasswordHasher,
}

impl LocalAuthProvider {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self {
            repository,
            password_hasher: PasswordHasher::new(),
        }
    }

    pub async fn register(
        &self,
        name: &str,
        email: &str,
        password: &str,
        role: UserRole,
    ) -> Result<User, AppError> {
        let password_hash = self.password_hasher.hash(password)?;

        let user = User::new_local(
            name.to_string(),
            email.to_lowercase(),
            password_hash,
            role,
        );

        self.repository.create(&user).await?;

        tracing::info!(
            user_id = %user.id,
            email = %email,
            role = %role,
            "New user registered"
        );

        Ok(user)
    }

    pub fn password_hasher(&self) -> &PasswordHasher {
        &self.password_hasher
    }
}

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    fn name(&self) -> &'static str {
        "local"
    }

    async fn authenticate(&self, email: &str, password: &str) -> Result<AuthResult, AppError> {
        let user = self
            .repository
            .find_by_email(email)
            .await?
            .ok_or_else(|| {
                tracing::warn!(email = %email, "Login attempt for non-existent user");
                AppError::Unauthorized("Invalid credentials".to_string())
            })?;

        let password_hash = user.password_hash.as_ref().ok_or_else(|| {
            tracing::warn!(
                user_id = %user.id,
                "Login attempt with password for non-local user"
            );
            AppError::Unauthorized("Invalid credentials".to_string())
        })?;

        let valid = self.password_hasher.verify(password, password_hash)?;

        if !valid {
            tracing::warn!(user_id = %user.id, "Failed login attempt - invalid password");
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        tracing::info!(user_id = %user.id, "User authenticated successfully");

        Ok(AuthResult { user })
    }
}

// Placeholder Google
#[allow(dead_code)]
mod google_stub {
}

// Placeholder ad
#[allow(dead_code)]
mod ad_stub {
}
