pub mod api;
pub mod governance;
pub mod dsl;
pub mod attestation;
pub mod consensus;
pub mod network;
pub mod services;
pub mod storage;

/// Re-export commonly used types and functions
pub use icn_types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}