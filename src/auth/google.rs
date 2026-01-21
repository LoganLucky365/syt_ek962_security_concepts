use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl,
    basic::{BasicClient, BasicTokenType},
    StandardTokenResponse, EmptyExtraTokenFields,
};
use reqwest::Client as HttpClient;
use serde::Deserialize;
use std::sync::Arc;

use crate::config::GoogleOAuthConfig;
use crate::error::AppError;
use crate::models::{AuthProviderType, User, UserRole};
use crate::repository::UserRepository;
use super::AuthResult;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    pub name: Option<String>,
    pub picture: Option<String>,
}

type GoogleTokenResponse = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

pub struct GoogleAuthProvider {
    client_id: ClientId,
    client_secret: ClientSecret,
    auth_url: AuthUrl,
    token_url: TokenUrl,
    redirect_uri: RedirectUrl,
    http_client: HttpClient,
    repository: Arc<dyn UserRepository>,
}

impl GoogleAuthProvider {
    pub fn new(config: &GoogleOAuthConfig, repository: Arc<dyn UserRepository>) -> Self {
        Self {
            client_id: ClientId::new(config.client_id.clone()),
            client_secret: ClientSecret::new(config.client_secret.clone()),
            auth_url: AuthUrl::new(GOOGLE_AUTH_URL.to_string()).expect("Invalid auth URL"),
            token_url: TokenUrl::new(GOOGLE_TOKEN_URL.to_string()).expect("Invalid token URL"),
            redirect_uri: RedirectUrl::new(config.redirect_uri.clone()).expect("Invalid redirect URI"),
            http_client: HttpClient::new(),
            repository,
        }
    }

    pub fn authorization_url(&self) -> (String, CsrfToken) {
        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone());

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        (auth_url.to_string(), csrf_token)
    }

    pub async fn exchange_code(&self, code: &str) -> Result<GoogleUserInfo, AppError> {
        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_uri.clone());

        let token_result: GoogleTokenResponse = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&self.http_client)
            .await
            .map_err(|e| {
                tracing::error!("token exchange problem: {:?}", e);
                AppError::OAuthError("cant get auth code".to_string())
            })?;

        let access_token = token_result.access_token().secret();

        let user_info = self
            .http_client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Userinfo net fetched: {:?}", e);
                AppError::OAuthError("cant get user info".to_string())
            })?
            .json::<GoogleUserInfo>()
            .await
            .map_err(|e| {
                tracing::error!("userinfo not parsed: {:?}", e);
                AppError::OAuthError("parsing user info not good".to_string())
            })?;

        if !user_info.email_verified {
            return Err(AppError::OAuthError(
                "Email not valid".to_string(),
            ));
        }

        Ok(user_info)
    }

    pub async fn authenticate_or_create(&self, google_user: GoogleUserInfo) -> Result<(AuthResult, bool), AppError> {
        if let Some(user) = self
            .repository
            .find_by_external_id("google", &google_user.sub)
            .await?
        {
            tracing::info!(
                user_id = %user.id,
                google_sub = %google_user.sub,
                "existing user"
            );
            return Ok((AuthResult { user }, false));
        }

        if let Some(existing) = self.repository.find_by_email(&google_user.email).await? {
            tracing::warn!(
                email = %google_user.email,
                existing_provider = ?existing.auth_provider,
                "email already registered"
            );
            return Err(AppError::Conflict(
                "email already registerd".to_string(),
            ));
        }

        let name = google_user
            .name
            .unwrap_or_else(|| google_user.email.split('@').next().unwrap_or("User").to_string());

        let user = User::new_external(
            name,
            google_user.email.to_lowercase(),
            AuthProviderType::Google,
            google_user.sub.clone(),
            UserRole::User,
        );

        self.repository.create(&user).await?;

        tracing::info!(
            user_id = %user.id,
            google_sub = %google_user.sub,
            email = %user.email,
            "google user created"
        );

        Ok((AuthResult { user }, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_user_info_deserialization() {
        let json = r#"{
            "sub": "123456789",
            "email": "test@gmail.com",
            "email_verified": true,
            "name": "Test User",
            "picture": "https://example.com/photo.jpg"
        }"#;

        let info: GoogleUserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.sub, "123456789");
        assert_eq!(info.email, "test@gmail.com");
        assert!(info.email_verified);
        assert_eq!(info.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_google_user_info_minimal() {
        let json = r#"{
            "sub": "123",
            "email": "test@gmail.com"
        }"#;

        let info: GoogleUserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.sub, "123");
        assert!(!info.email_verified);
        assert!(info.name.is_none());
    }
}
