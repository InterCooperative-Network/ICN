#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    pub governance_participation: u32,
    pub resource_contributions: u32,
    pub technical_support: u32,
    pub dispute_resolutions: u32,
    pub last_decay: DateTime<Utc>,
}

impl ReputationScore {
    pub fn apply_decay(&mut self) {
        let now = Utc::now();
        let days_since_decay = (now - self.last_decay).num_days();
        if days_since_decay > 0 {
            let decay_factor = 0.98f64.powi(days_since_decay as i32);
            self.governance_participation = (self.governance_participation as f64 * decay_factor).round() as u32;
            self.resource_contributions = (self.resource_contributions as f64 * decay_factor).round() as u32;
            self.technical_support = (self.technical_support as f64 * decay_factor).round() as u32;
            self.dispute_resolutions = (self.dispute_resolutions as f64 * decay_factor).round() as u32;
            self.last_decay = now;
        }
    }

    pub fn get_aggregate_score(&self) -> u32 {
        self.governance_participation +
        self.resource_contributions +
        self.technical_support +
        self.dispute_resolutions
    }
}
