use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::User;
use super::traits::UserRepository;

pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // DB Initialisierung
    pub async fn initialize(&self) -> Result<(), AppError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT,
                role TEXT NOT NULL DEFAULT 'user',
                auth_provider TEXT NOT NULL DEFAULT 'local',
                external_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                UNIQUE(auth_provider, external_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_users_external ON users(auth_provider, external_id)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email, password_hash, role, auth_provider,
                   external_id, created_at, updated_at, is_active
            FROM users
            WHERE id = ? AND is_active = 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let email_lower = email.to_lowercase();

        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email, password_hash, role, auth_provider,
                   external_id, created_at, updated_at, is_active
            FROM users
            WHERE LOWER(email) = ? AND is_active = 1
            "#,
        )
        .bind(email_lower)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_external_id(
        &self,
        provider: &str,
        external_id: &str,
    ) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, name, email, password_hash, role, auth_provider,
                   external_id, created_at, updated_at, is_active
            FROM users
            WHERE auth_provider = ? AND external_id = ? AND is_active = 1
            "#,
        )
        .bind(provider)
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, name, email, password_hash, role, auth_provider,
                              external_id, created_at, updated_at, is_active)
            VALUES (?, ?, LOWER(?), ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role.to_string())
        .bind(&user.auth_provider.to_string())
        .bind(&user.external_id)
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .bind(user.is_active)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("UNIQUE constraint failed") {
                    return AppError::Conflict("Email already registered".to_string());
                }
            }
            AppError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET name = ?, email = LOWER(?), password_hash = ?, role = ?,
                auth_provider = ?, external_id = ?, updated_at = ?, is_active = ?
            WHERE id = ?
            "#,
        )
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.role.to_string())
        .bind(&user.auth_provider.to_string())
        .bind(&user.external_id)
        .bind(user.updated_at.to_rfc3339())
        .bind(user.is_active)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query("UPDATE users SET is_active = 0 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    async fn is_empty(&self) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0 == 0)
    }

    async fn count(&self) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}
