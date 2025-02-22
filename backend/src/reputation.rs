use dashmap::DashMap;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use ark_groth16::{Proof, ProvingKey, VerifyingKey}; // For ZKP
use rand::rngs::OsRng;

pub struct ReputationCache {
    cache: DashMap<String, i32>,
    max_size: usize,
}

impl ReputationCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
        }
    }

    fn get(&self, did: &str) -> Option<i32> {
        self.cache.get(did).map(|v| *v)
    }

    fn set(&self, did: &str, score: i32) {
        if self.cache.len() >= self.max_size {
            // Implement a simple eviction policy (e.g., remove a random entry)
            if let Some(key) = self.cache.iter().next().map(|entry| entry.key().clone()) {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(did.to_string(), score);
    }
}

pub struct ReputationManager {
    cache: ReputationCache,
    contributions: HashMap<String, Vec<Contribution>>,
    decay_rate: f64,
}

impl ReputationManager {
    pub fn new(max_cache_size: usize, decay_rate: f64) -> Self {
        ReputationManager {
            cache: ReputationCache::new(max_cache_size),
            contributions: HashMap::new(),
            decay_rate,
        }
    }

    pub async fn get_reputation(&self, did: &str, category: &str) -> Result<i64, String> {
        if let Some(score) = self.cache.get(did) {
            return Ok(score as i64);
        }

        // Logic to retrieve reputation for a given DID and category
        let score = 100; // Placeholder value

        self.cache.set(did, score);
        Ok(score)
    }

    pub async fn is_eligible(&self, did: &str, min_reputation: i64, category: &str) -> Result<bool, String> {
        let reputation = self.get_reputation(did, category).await?;
        Ok(reputation >= min_reputation)
    }

    pub async fn dynamic_adjustment(&self, did: &str, contribution: i64) -> Result<(), String> {
        // Logic to adjust reputation based on contributions
        // Placeholder logic
        Ok(())
    }

    pub async fn apply_decay(&mut self, did: &str) -> Result<(), String> {
        if let Some(contributions) = self.contributions.get_mut(did) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as f64;

            // Calculate inactivity period
            let last_contribution = contributions.iter()
                .map(|c| c.timestamp)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            
            let inactivity_days = (now - last_contribution) / (24.0 * 60.0 * 60.0);
            
            // Apply stronger decay for longer inactivity
            let decay_rate = match inactivity_days as u32 {
                0..=7 => self.decay_rate,
                8..=30 => self.decay_rate * 2.0,
                _ => self.decay_rate * 4.0,
            };

            // Apply decay to all contributions
            for contribution in contributions.iter_mut() {
                let age = now - contribution.timestamp;
                contribution.score = (contribution.score as f64 * (-decay_rate * age).exp()) as i64;
            }

            // Remove expired contributions
            contributions.retain(|c| c.score > 0);
        }
        Ok(())
    }

    pub async fn reputation_based_access(&self, did: &str, min_reputation: i64) -> Result<bool, String> {
        self.is_eligible(did, min_reputation, "access").await
    }

    pub async fn add_contribution(&mut self, did: &str, contribution: Contribution) {
        self.contributions.entry(did.to_string()).or_insert_with(Vec::new).push(contribution);
    }

    pub async fn verify_cooperation_proof(
        &self,
        did: &str, 
        proof: ContributionProof,
        verifying_key: &VerifyingKey<ark_bls12_381::Bls12_381>
    ) -> Result<bool, String> {
        // Verify the ZKP
        let valid = ark_groth16::verify_proof(
            verifying_key,
            &proof.proof,
            &proof.public_inputs
        ).map_err(|e| e.to_string())?;

        if !valid {
            return Ok(false);
        }

        // Verify contribution timestamp is recent enough
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - proof.timestamp > 30 * 24 * 60 * 60 { // 30 days
            return Ok(false);
        }

        // Record verified contribution
        let contribution = Contribution {
            score: self.calculate_contribution_score(&proof.contribution_type),
            timestamp: proof.timestamp as f64,
        };
        self.add_contribution(did, contribution).await;

        Ok(true)
    }

    fn calculate_contribution_score(&self, contribution_type: &ContributionType) -> i64 {
        match contribution_type {
            ContributionType::ResourceSharing { amount, .. } => {
                // Score based on resource amount shared
                (*amount as f64 * 0.5) as i64
            }
            ContributionType::GovernanceVote { .. } => {
                // Fixed score for governance participation
                10
            }
            ContributionType::FederatedAction { .. } => {
                // Fixed score for federated actions
                15
            }
            ContributionType::TechnicalContribution { .. } => {
                // Higher score for technical contributions
                25
            }
        }
    }

    pub async fn get_contribution_history(&self, did: &str) -> Result<Vec<Contribution>, String> {
        Ok(self.contributions.get(did)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn generate_anonymous_proof(
        &self,
        did: &str,
        proving_key: &ProvingKey<ark_bls12_381::Bls12_381>,
        threshold: i64
    ) -> Result<ContributionProof, String> {
        let reputation = self.get_reputation(did, "global").await?;
        
        // Generate ZKP that reputation > threshold without revealing actual score
        let mut rng = OsRng;
        let proof = ark_groth16::create_random_proof(
            proving_key,
            &[ark_ff::Fp256::from(reputation as u64)],
            &mut rng
        ).map_err(|e| e.to_string())?;

        Ok(ContributionProof {
            contribution_type: ContributionType::GovernanceVote {
                proposal_id: "anon".to_string()
            },
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            proof,
            public_inputs: vec![ark_ff::Fp256::from(threshold as u64)],
        })
    }
}

#[derive(Debug, Clone)]
pub struct ContributionProof {
    pub contribution_type: ContributionType,
    pub timestamp: u64,
    pub proof: Proof<ark_bls12_381::Bls12_381>,
    pub public_inputs: Vec<ark_ff::Fp256<ark_bls12_381::FrParameters>>,
}

#[derive(Debug, Clone)]
pub enum ContributionType {
    ResourceSharing { resource_id: String, amount: u64 },
    GovernanceVote { proposal_id: String },
    FederatedAction { federation_id: String, action_id: String },
    TechnicalContribution { commit_hash: String },
}

pub struct Contribution {
    pub score: i64,
    pub timestamp: f64,
}

impl Contribution {
    pub fn new(score: i64) -> Self {
        Self {
            score,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64,
        }
    }
}
