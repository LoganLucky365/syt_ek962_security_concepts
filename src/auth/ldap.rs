// ldap authentication blind -> die credentials die eingegeben werden verwendet um ldap server zu binden
use async_trait::async_trait;
use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use std::sync::Arc;
use std::time::Duration;

use crate::config::LdapConfig;
use crate::error::AppError;
use crate::models::{AuthProviderType, User, UserRole};
use crate::repository::UserRepository;

use super::{AuthProvider, AuthResult};

pub struct LdapAuthProvider {
    config: LdapConfig,
    repository: Arc<dyn UserRepository>,
}

impl LdapAuthProvider {
    pub fn new(config: LdapConfig, repository: Arc<dyn UserRepository>) -> Self {
        Self { config, repository }
    }

    //construct Domain name -> bsp CN=username,OU=Users,DC=domain,DC=com or using userPrincipalName: username@domain.com
    fn build_bind_dn(&self, username: &str) -> String {
        if self.config.use_upn {
            format!("{}@{}", username, self.config.domain)
        } else {
            format!(
                "CN={},{}",
                username, self.config.user_base_dn
            )
        }
    }

    //tries to connect to ldap server
    async fn bind_user(&self, username: &str, password: &str) -> Result<ldap3::Ldap, AppError> {
        let settings = LdapConnSettings::new()
            .set_conn_timeout(Duration::from_secs(self.config.timeout_secs))
            .set_starttls(self.config.use_starttls);

        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.config.url)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "cannt connect ldap server");
                AppError::LdapError(format!("Connection failed: {}", e))
            })?;

        // Spawn con handler
        ldap3::drive!(conn);

        let bind_dn = self.build_bind_dn(username);
        tracing::debug!(bind_dn = %bind_dn, "tried ldap bind");

        let result = ldap.simple_bind(&bind_dn, password).await.map_err(|e| {
            tracing::warn!(error = %e, username = %username, "bind faild");
            AppError::Unauthorized("invalid imputs".to_string())
        })?;

        if result.rc != 0 {
            tracing::warn!(
                username = %username,
                result_code = result.rc,
                "ldap rejected"
            );
            return Err(AppError::Unauthorized("Invalid credentials".to_string()));
        }

        tracing::debug!(username = %username, "ldap bind successful");
        Ok(ldap)
    }

    //search attributes
    async fn fetch_user_info(&self, ldap: &mut ldap3::Ldap, username: &str) -> Result<LdapUserInfo, AppError> {
        let search_filter = format!("({}={})", self.config.username_attribute, username);

        let (entries, _result) = ldap
            .search(
                &self.config.user_base_dn,
                Scope::Subtree,
                &search_filter,
                vec!["cn", "mail", "displayName", "sAMAccountName", "userPrincipalName", "memberOf"],
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "search failed");
                AppError::LdapError(format!("Search failed: {}", e))
            })?
            .success()
            .map_err(|e| {
                tracing::error!(error = %e, "search result error");
                AppError::LdapError(format!("Search result error: {}", e))
            })?;

        let entry = entries.into_iter().next().ok_or_else(|| {
            tracing::warn!(username = %username, "User not found");
            AppError::LdapError("User not found".to_string())
        })?;

        let entry = SearchEntry::construct(entry);

        let display_name = entry
            .attrs
            .get("displayName")
            .and_then(|v| v.first())
            .or_else(|| entry.attrs.get("cn").and_then(|v| v.first()))
            .cloned()
            .unwrap_or_else(|| username.to_string());

        let email = entry
            .attrs
            .get("mail")
            .and_then(|v| v.first())
            .cloned()
            .or_else(|| entry.attrs.get("userPrincipalName").and_then(|v| v.first()).cloned())
            .unwrap_or_else(|| format!("{}@{}", username, self.config.domain));

        let sam_account = entry
            .attrs
            .get("sAMAccountName")
            .and_then(|v| v.first())
            .cloned()
            .unwrap_or_else(|| username.to_string());

        // Check group membership for admin role
        let groups: Vec<String> = entry
            .attrs
            .get("memberOf")
            .cloned()
            .unwrap_or_default();

        let is_admin = self.config.admin_group.as_ref().map_or(false, |admin_group| {
            groups.iter().any(|g| g.to_lowercase().contains(&admin_group.to_lowercase()))
        });

        Ok(LdapUserInfo {
            username: sam_account,
            display_name,
            email,
            is_admin,
        })
    }

    //create user record
    async fn sync_user(&self, info: &LdapUserInfo) -> Result<User, AppError> {
        let provider = "activedirectory";
        let external_id = &info.username;

        // user alredy exist
        if let Some(existing) = self.repository.find_by_external_id(provider, external_id).await? {
            tracing::debug!(
                user_id = %existing.id,
                username = %info.username,
                "Found existing LDAP user"
            );
            return Ok(existing);
        }

        // check existing by mail
        if let Some(existing) = self.repository.find_by_email(&info.email).await? {
            if existing.auth_provider == AuthProviderType::ActiveDirectory {
                return Ok(existing);
            }
            tracing::warn!(
                email = %info.email,
                existing_provider = %existing.auth_provider,
                "Email there different methode"
            );
            return Err(AppError::Conflict(
                "Email there different methode".to_string(),
            ));
        }

        // Create new user
        let role = if info.is_admin {
            UserRole::Admin
        } else {
            UserRole::User
        };

        let user = User::new_external(
            info.display_name.clone(),
            info.email.clone(),
            AuthProviderType::ActiveDirectory,
            info.username.clone(),
            role,
        );

        self.repository.create(&user).await?;

        tracing::info!(
            user_id = %user.id,
            username = %info.username,
            email = %info.email,
            role = %role,
            "Created user"
        );

        Ok(user)
    }
}

struct LdapUserInfo {
    username: String,
    display_name: String,
    email: String,
    is_admin: bool,
}

#[async_trait]
impl AuthProvider for LdapAuthProvider {
    fn name(&self) -> &'static str {
        "ldap"
    }

    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthResult, AppError> {
        tracing::info!(username = %username, server = %self.config.url, "LDAP authentication attempt");

        // bind
        let mut ldap = self.bind_user(username, password).await?;

        // fetch info
        let user_info = self.fetch_user_info(&mut ldap, username).await?;

        // unbind
        let _ = ldap.unbind().await;

        // sync to db
        let user = self.sync_user(&user_info).await?;

        tracing::info!(
            user_id = %user.id,
            username = %user_info.username,
            "auth successful"
        );

        Ok(AuthResult { user })
    }
}
