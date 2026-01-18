//! Repository layer for data persistence.
//!
//! This module provides a trait-based abstraction over data storage,
//! allowing for different backend implementations (SQLite, PostgreSQL, etc.).

mod traits;
mod sqlite;

pub use traits::UserRepository;
pub use sqlite::SqliteUserRepository;
