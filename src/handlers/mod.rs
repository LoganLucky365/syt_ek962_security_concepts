//! HTTP request handlers for the authentication service.

mod auth;

pub use auth::{register_user, signin, verify_token, configure_routes, AppState};
