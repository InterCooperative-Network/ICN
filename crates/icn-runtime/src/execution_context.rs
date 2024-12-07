pub struct ExecutionContext {
    pub caller_did: String,
    pub cooperative_id: String,
    pub timestamp: u64,
    pub block_number: u64,
    pub reputation_score: i64,
    pub permissions: Vec<String>,
}
