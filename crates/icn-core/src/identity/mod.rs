// src/identity/mod.rs

pub mod did;
// pub mod key_pair;
// pub mod identity_manager;
// pub mod authentication;
pub mod identity_system;

pub use did::DID;
// pub use key_pair::KeyPair;
// pub use identity_manager::IdentityManager;
// pub use authentication::Authentication;
pub use identity_system::IdentitySystem;
