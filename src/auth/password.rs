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

// Generated tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = PasswordHasher::new();
        let password = "test_password_123!";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let hasher = PasswordHasher::new();
        let hash1 = hasher.hash("password1").unwrap();
        let hash2 = hasher.hash("password1").unwrap();

        assert_ne!(hash1, hash2);
    }
}
