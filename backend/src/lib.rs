pub mod api;
pub mod networking;
pub mod middleware;

use thiserror::Error;
use std::error::Error;
use std::fmt;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
