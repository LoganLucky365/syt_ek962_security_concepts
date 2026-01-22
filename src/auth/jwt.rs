use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};

use crate::config::JwtConfig;
use crate::error::AppError;
use crate::models::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    // UserID
    pub sub: String,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
}

impl Claims {
    pub fn new(user_id: &str, email: &str, role: UserRole, config: &JwtConfig) -> Self {
        let now = Utc::now();
        let exp = now + Duration::seconds(config.expiration_secs);

        Self {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            iss: config.issuer.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn get_role(&self) -> Result<UserRole, AppError> {
        self.role
            .parse()
            .map_err(|_| AppError::InternalError("Invalid role in token".to_string()))
    }
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    config: JwtConfig,
}

impl JwtService {
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&config.issuer]);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.leeway = 0; 

        Self {
            encoding_key,
            decoding_key,
            validation,
            config,
        }
    }

    pub fn generate_token(
        &self,
        user_id: &str,
        email: &str,
        role: UserRole,
    ) -> Result<String, AppError> {
        let claims = Claims::new(user_id, email, role, &self.config);

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("Token generation failed: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)?;
        Ok(token_data.claims)
    }
    
    pub fn decode_without_validation(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_at_least_32_chars_long".to_string(),
            expiration_secs: 3600,
            issuer: "test-issuer".to_string(),
        }
    }

    fn short_expiry_config() -> JwtConfig {
        JwtConfig {
            secret: "test_secret_key_at_least_32_chars_long".to_string(),
            expiration_secs: 1,
            issuer: "test-issuer".to_string(),
        }
    }

    // ==================== Token Generation Tests ====================

    #[test]
    fn test_generate_and_validate_token() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn test_generate_token_for_admin() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("admin-456", "admin@example.com", UserRole::Admin)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "admin-456");
        assert_eq!(claims.email, "admin@example.com");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_token_contains_correct_issuer() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.iss, "test-issuer");
    }

    #[test]
    fn test_token_contains_timestamps() {
        let service = JwtService::new(test_config());
        let before = Utc::now().timestamp();

        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let after = Utc::now().timestamp();
        let claims = service.validate_token(&token).unwrap();

        assert!(claims.iat >= before && claims.iat <= after);
        assert_eq!(claims.exp, claims.iat + 3600);
    }

    // ==================== Token Validation Tests ====================

    #[test]
    fn test_invalid_token_format() {
        let service = JwtService::new(test_config());
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_token() {
        let service = JwtService::new(test_config());
        let result = service.validate_token("");
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_token_signature() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let tampered = format!("{}x", token);
        let result = service.validate_token(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_wrong_secret() {
        let service1 = JwtService::new(test_config());
        let token = service1
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let different_config = JwtConfig {
            secret: "different_secret_key_at_least_32_chars".to_string(),
            expiration_secs: 3600,
            issuer: "test-issuer".to_string(),
        };
        let service2 = JwtService::new(different_config);

        let result = service2.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_wrong_issuer_rejected() {
        let service1 = JwtService::new(test_config());
        let token = service1
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let different_issuer_config = JwtConfig {
            secret: "test_secret_key_at_least_32_chars_long".to_string(),
            expiration_secs: 3600,
            issuer: "different-issuer".to_string(),
        };
        let service2 = JwtService::new(different_issuer_config);

        let result = service2.validate_token(&token);
        assert!(result.is_err());
    }

    // ==================== Token Expiration Tests ====================

    #[test]
    fn test_expired_token_rejected() {
        let service = JwtService::new(short_expiry_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        // Wait for token to expire
        std::thread::sleep(std::time::Duration::from_secs(2));

        let result = service.validate_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_is_expired_method() {
        let config = test_config();
        let claims = Claims::new("user-123", "test@example.com", UserRole::User, &config);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_claims_is_expired_for_past_token() {
        let claims = Claims {
            sub: "user-123".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            exp: Utc::now().timestamp() - 100,
            iat: Utc::now().timestamp() - 200,
            iss: "test".to_string(),
        };
        assert!(claims.is_expired());
    }

    // ==================== Claims Tests ====================

    #[test]
    fn test_claims_get_role_user() {
        let config = test_config();
        let claims = Claims::new("user-123", "test@example.com", UserRole::User, &config);
        assert_eq!(claims.get_role().unwrap(), UserRole::User);
    }

    #[test]
    fn test_claims_get_role_admin() {
        let config = test_config();
        let claims = Claims::new("admin-123", "admin@example.com", UserRole::Admin, &config);
        assert_eq!(claims.get_role().unwrap(), UserRole::Admin);
    }

    #[test]
    fn test_claims_get_role_invalid() {
        let claims = Claims {
            sub: "user-123".to_string(),
            email: "test@example.com".to_string(),
            role: "invalid_role".to_string(),
            exp: Utc::now().timestamp() + 3600,
            iat: Utc::now().timestamp(),
            iss: "test".to_string(),
        };
        assert!(claims.get_role().is_err());
    }

    // ==================== Decode Without Validation Tests ====================

    #[test]
    fn test_decode_without_validation_expired_token() {
        let service = JwtService::new(short_expiry_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should fail with normal validation
        assert!(service.validate_token(&token).is_err());

        // Should succeed without validation
        let claims = service.decode_without_validation(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_token_with_special_characters_in_email() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-123", "test+special@example.com", UserRole::User)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.email, "test+special@example.com");
    }

    #[test]
    fn test_token_with_unicode_in_user_id() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-äöü-123", "test@example.com", UserRole::User)
            .unwrap();

        let claims = service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user-äöü-123");
    }

    #[test]
    fn test_multiple_tokens_for_same_user_are_valid() {
        let service = JwtService::new(test_config());
        let token1 = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();
        let token2 = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let claims1 = service.validate_token(&token1).unwrap();
        let claims2 = service.validate_token(&token2).unwrap();
        assert_eq!(claims1.sub, claims2.sub);
    }
}
