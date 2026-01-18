use async_trait::async_trait;
use crate::error::AppError;
use crate::models::User;

// Created for different auth backends (ad, google, etc)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError>;

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;

    async fn find_by_external_id(&self, provider: &str, external_id: &str)
        -> Result<Option<User>, AppError>;

    async fn create(&self, user: &User) -> Result<(), AppError>;

    async fn update(&self, user: &User) -> Result<(), AppError>;

    async fn delete(&self, id: &str) -> Result<(), AppError>;

    async fn is_empty(&self) -> Result<bool, AppError>;

    async fn count(&self) -> Result<i64, AppError>;
}
