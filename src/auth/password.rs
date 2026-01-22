// Warum Argon2 -> Roschger kennt sich aus und gut erprobt
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};
use crate::error::AppError;

pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    pub fn new() -> Self {

        let params = Params::new(
            64 * 1024, // 64 MiB
            3,         // 3 iterations
            4,         // 4 parallel threads
            None,      // output 32 bit
        )
        .expect("Invalid parameters");

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }

    pub fn hash(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    pub fn verify(&self, password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::InternalError(format!("Invalid hash format: {}", e)))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AppError::InternalError(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic Hashing Tests ====================

    #[test]
    fn test_hash_and_verify_success() {
        let hasher = PasswordHasher::new();
        let password = "test_password_123!";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_hash_and_verify_wrong_password() {
        let hasher = PasswordHasher::new();
        let password = "correct_password";

        let hash = hasher.hash(password).unwrap();
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_same_password_different_hashes() {
        let hasher = PasswordHasher::new();
        let password = "same_password";

        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();

        // Different salts = different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_password() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("").unwrap();

        assert!(hasher.verify("", &hash).unwrap());
        assert!(!hasher.verify("not_empty", &hash).unwrap());
    }

    #[test]
    fn test_very_long_password() {
        let hasher = PasswordHasher::new();
        let password = "a".repeat(1000);

        let hash = hasher.hash(&password).unwrap();
        assert!(hasher.verify(&password, &hash).unwrap());
    }

    #[test]
    fn test_unicode_password() {
        let hasher = PasswordHasher::new();
        let password = "пароль密码🔐";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_password_with_special_characters() {
        let hasher = PasswordHasher::new();
        let password = "P@$$w0rd!#%^&*()[]{}|;':\",./<>?";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
    }

    #[test]
    fn test_password_with_whitespace() {
        let hasher = PasswordHasher::new();
        let password = "password with spaces and\ttabs\nnewlines";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
    }

    // ==================== Hash Format Tests ====================

    #[test]
    fn test_hash_format_is_argon2id() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("test").unwrap();

        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_contains_version() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("test").unwrap();

        assert!(hash.contains("$v=19$")); // Version 0x13 = 19
    }

    #[test]
    fn test_hash_contains_params() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("test").unwrap();

        // m=65536 (64*1024), t=3, p=4
        assert!(hash.contains("m=65536"));
        assert!(hash.contains("t=3"));
        assert!(hash.contains("p=4"));
    }

    // ==================== Invalid Hash Tests ====================

    #[test]
    fn test_verify_invalid_hash_format() {
        let hasher = PasswordHasher::new();
        let result = hasher.verify("password", "not_a_valid_hash");

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_empty_hash() {
        let hasher = PasswordHasher::new();
        let result = hasher.verify("password", "");

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_truncated_hash() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("password").unwrap();
        let truncated = &hash[..hash.len() - 10];

        let result = hasher.verify("password", truncated);
        assert!(result.is_err());
    }

    // ==================== Security Tests ====================

    #[test]
    fn test_similar_passwords_different_results() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("password123").unwrap();

        // Similar but different passwords should not verify
        assert!(!hasher.verify("password124", &hash).unwrap());
        assert!(!hasher.verify("password12", &hash).unwrap());
        assert!(!hasher.verify("Password123", &hash).unwrap());
        assert!(!hasher.verify(" password123", &hash).unwrap());
    }

    #[test]
    fn test_case_sensitive_passwords() {
        let hasher = PasswordHasher::new();
        let hash = hasher.hash("Password").unwrap();

        assert!(hasher.verify("Password", &hash).unwrap());
        assert!(!hasher.verify("password", &hash).unwrap());
        assert!(!hasher.verify("PASSWORD", &hash).unwrap());
    }

    // ==================== Default Implementation ====================

    #[test]
    fn test_default_implementation() {
        let hasher1 = PasswordHasher::new();
        let hasher2 = PasswordHasher::default();

        let hash1 = hasher1.hash("test").unwrap();
        let hash2 = hasher2.hash("test").unwrap();

        // Both should be able to verify each other's hashes
        assert!(hasher1.verify("test", &hash2).unwrap());
        assert!(hasher2.verify("test", &hash1).unwrap());
    }

    // ==================== Performance Sanity Check ====================

    #[test]
    fn test_hashing_completes_in_reasonable_time() {
        let hasher = PasswordHasher::new();
        let start = std::time::Instant::now();

        let _ = hasher.hash("performance_test").unwrap();

        let duration = start.elapsed();
        // Should complete within 5 seconds (generous for CI)
        assert!(duration.as_secs() < 5, "Hashing took too long: {:?}", duration);
    }
}
