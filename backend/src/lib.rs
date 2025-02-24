pub mod api;
pub mod services;
pub mod database;
pub mod networking;
pub mod core;
pub mod main;

pub mod federation;
pub mod governance;
pub mod resources;
pub mod reputation;

#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub mod test_config;
#[cfg(test)]
pub mod test_macros;

// Re-export test utilities for integration tests
#[cfg(test)]
pub use test_utils::*;
#[cfg(test)]
pub use test_config::*;
#[cfg(test)]
pub use test_macros::*;
