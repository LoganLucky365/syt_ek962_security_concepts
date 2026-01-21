use std::fs;
use std::path::Path;
use std::sync::Arc;

use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use syt_ek962_security_concepts::auth::{JwtService, LocalAuthProvider, GoogleAuthProvider, LdapAuthProvider};
use syt_ek962_security_concepts::config::{Config, InitialAdminConfig};
use syt_ek962_security_concepts::handlers::{configure_routes, AppState};
use syt_ek962_security_concepts::models::UserRole;
use syt_ek962_security_concepts::repository::{SqliteUserRepository, UserRepository};

async fn initialize_admin(
    config_path: &str,
    auth_provider: &LocalAuthProvider,
    repository: &SqliteUserRepository,
) -> Result<(), Box<dyn std::error::Error>> {
    if !repository.is_empty().await? {
        tracing::info!("DB already with user");
        return Ok(());
    }

    let admin_config = if let Some(config) = InitialAdminConfig::from_env() {
        tracing::info!("From Environment");
        config
    } else {
        let path = Path::new(config_path);
        if !path.exists() {
            tracing::warn!(
                "cant create initial admin {}",
                config_path
            );
            return Ok(());
        }

        tracing::info!("Loading json {}", config_path);
        let config_content = fs::read_to_string(path)?;
        serde_json::from_str(&config_content)?
    };

    if let Some(password_hash) = &admin_config.password_hash {
        tracing::info!("pre hashed pw");

        let user = syt_ek962_security_concepts::models::User::new_local(
            admin_config.name.clone(),
            admin_config.email.to_lowercase(),
            password_hash.clone(),
            UserRole::Admin,
        );

        repository.create(&user).await?;
    } else if let Some(password) = &admin_config.password {
        if password.len() < 12 {
            return Err("pw more than 12 char".into());
        }

        auth_provider
            .register(
                &admin_config.name,
                &admin_config.email,
                password,
                UserRole::Admin,
            )
            .await?;

        tracing::warn!(
            "pw is plain text"
        );
    } else {
        return Err("no pw or hash".into());
    }

    tracing::info!(
        email = %admin_config.email,
        "user created"
    );

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    tracing::info!(
        host = %config.host,
        port = %config.port
    );

    if config.jwt.secret.len() < 32 {
        tracing::error!(
            "jwt to short (32)"
        );
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "JWT to short (32)",
        ));
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed db con");

    let repository = SqliteUserRepository::new(pool);
    repository
        .initialize()
        .await
        .expect("db schema problem");

    let repository: Arc<dyn syt_ek962_security_concepts::repository::UserRepository> = Arc::new(repository);

    let sqlite_repo = {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&config.database_url)
            .await
            .expect("admin init db pool wrong");
        SqliteUserRepository::new(pool)
    };

    let auth_provider = LocalAuthProvider::new(Arc::clone(&repository));

    if let Err(e) = initialize_admin(
        &config.initial_admin_config,
        &auth_provider,
        &sqlite_repo,
    )
    .await
    {
        tracing::error!("problem initialize admin {}", e);
    }

    let jwt_service = JwtService::new(config.jwt.clone());

    let google_provider = config.google_oauth.as_ref().map(|oauth_config| {
        tracing::info!("Google OAuth enabled");
        GoogleAuthProvider::new(oauth_config, Arc::clone(&repository))
    });

    if google_provider.is_none() {
        tracing::info!("Google OAuth config not found");
    }

    let ldap_provider = config.ldap.as_ref().map(|ldap_config| {
        tracing::info!(
            url = %ldap_config.url,
            domain = %ldap_config.domain,
            "ldap activated"
        );
        LdapAuthProvider::new(ldap_config.clone(), Arc::clone(&repository))
    });

    if ldap_provider.is_none() {
        tracing::info!("ldap conf not found");
    }

    let app_state = web::Data::new(AppState {
        jwt_service,
        auth_provider,
        google_provider,
        ldap_provider,
        repository,
    });

    let host = config.host.clone();
    let port = config.port;

    // Start server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .configure(configure_routes)
            // Debug endpoint
            .route(
                "/health",
                web::get().to(|| async { actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "healthy"})) }),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
