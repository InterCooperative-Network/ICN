#[derive(Debug, Error)]
pub enum ReputationError {
    // ...existing code...
    #[error("Suspicious behavior detected")]
    SuspiciousBehavior,
    #[error("Sybil attack pattern detected")]
    SybilPattern,
}
