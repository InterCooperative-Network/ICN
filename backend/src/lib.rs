pub mod api;
pub mod networking;
pub mod middleware;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
