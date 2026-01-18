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

// Generated tests
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
    fn test_invalid_token() {
        let service = JwtService::new(test_config());
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_token() {
        let service = JwtService::new(test_config());
        let token = service
            .generate_token("user-123", "test@example.com", UserRole::User)
            .unwrap();

        let tampered = format!("{}x", token);
        let result = service.validate_token(&tampered);
        assert!(result.is_err());
    }
}
