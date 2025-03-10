use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiMonopolyConfig {
    /// Maximum reputation score allowed
    pub max_reputation: i64,
    
    /// Decay rate for high reputation scores (0.0 - 1.0)
    pub decay_rate: f64,
    
    /// Threshold above which decay starts (percentage of max_reputation)
    pub decay_threshold: f64,
    
    /// Interval between decay calculations (in seconds)
    pub decay_interval: u64,
    
    /// Power factor for quadratic voting (typically 0.5)
    pub voting_power_factor: f64,
    
    /// Maximum voting power allowed
    pub max_voting_power: f64,
}

impl Default for AntiMonopolyConfig {
    fn default() -> Self {
        Self {
            max_reputation: 1_000_000,
            decay_rate: 0.05,
            decay_threshold: 0.8,
            decay_interval: 86400, // 24 hours
            voting_power_factor: 0.5,
            max_voting_power: 100.0,
        }
    }
}

pub struct AntiMonopolyManager {
    config: AntiMonopolyConfig,
    reputation_stats: Arc<RwLock<ReputationStats>>,
}

#[derive(Debug, Default)]
struct ReputationStats {
    total_reputation: i64,
    member_reputations: HashMap<String, i64>,
    last_decay: std::time::SystemTime,
}

impl AntiMonopolyManager {
    pub fn new(config: AntiMonopolyConfig) -> Self {
        Self {
            config,
            reputation_stats: Arc::new(RwLock::new(ReputationStats::default())),
        }
    }
    
    /// Calculate voting power using quadratic formula
    pub fn calculate_voting_power(&self, reputation: i64) -> f64 {
        let normalized = (reputation as f64) / (self.config.max_reputation as f64);
        let power = normalized.powf(self.config.voting_power_factor);
        power.min(self.config.max_voting_power)
    }
    
    /// Apply anti-monopoly decay to a reputation score
    pub async fn apply_decay(&self, member_id: &str, current_score: i64) -> i64 {
        let mut stats = self.reputation_stats.write().await;
        
        // Update member's reputation in stats
        stats.member_reputations.insert(member_id.to_string(), current_score);
        
        // Calculate total reputation
        stats.total_reputation = stats.member_reputations.values().sum();
        
        // Check if decay should be applied
        let threshold = (self.config.max_reputation as f64 * self.config.decay_threshold) as i64;
        if current_score > threshold {
            // Calculate decay amount
            let excess = current_score - threshold;
            let decay_amount = (excess as f64 * self.config.decay_rate) as i64;
            current_score - decay_amount
        } else {
            current_score
        }
    }
    
    /// Calculate reputation dominance for a member
    pub async fn calculate_dominance(&self, member_id: &str) -> f64 {
        let stats = self.reputation_stats.read().await;
        
        if let Some(reputation) = stats.member_reputations.get(member_id) {
            if stats.total_reputation > 0 {
                *reputation as f64 / stats.total_reputation as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Check if a member has too much influence
    pub async fn has_excessive_influence(&self, member_id: &str) -> bool {
        let dominance = self.calculate_dominance(member_id).await;
        dominance > self.config.decay_threshold
    }
    
    /// Get reputation distribution statistics
    pub async fn get_distribution_stats(&self) -> ReputationDistribution {
        let stats = self.reputation_stats.read().await;
        
        let mut distribution = ReputationDistribution {
            total_reputation: stats.total_reputation,
            member_count: stats.member_reputations.len(),
            top_holders: Vec::new(),
            gini_coefficient: 0.0,
        };
        
        // Calculate top holders
        let mut holders: Vec<_> = stats.member_reputations.iter().collect();
        holders.sort_by_key(|(_, &rep)| -rep);
        
        distribution.top_holders = holders.into_iter()
            .take(10)
            .map(|(id, rep)| (id.clone(), *rep))
            .collect();
            
        // Calculate Gini coefficient
        if !stats.member_reputations.is_empty() {
            let n = stats.member_reputations.len() as f64;
            let mean = stats.total_reputation as f64 / n;
            
            let sum_abs_diff = stats.member_reputations.values()
                .flat_map(|&x| stats.member_reputations.values().map(move |&y| (x - y).abs() as f64))
                .sum::<f64>();
                
            distribution.gini_coefficient = sum_abs_diff / (2.0 * n * n * mean);
        }
        
        distribution
    }
    
    /// Perform periodic maintenance
    pub async fn maintain(&self) {
        let mut stats = self.reputation_stats.write().await;
        
        // Check if decay interval has elapsed
        let now = std::time::SystemTime::now();
        if now.duration_since(stats.last_decay)
            .unwrap_or_default()
            .as_secs() >= self.config.decay_interval
        {
            // Apply decay to all members above threshold
            let threshold = (self.config.max_reputation as f64 * self.config.decay_threshold) as i64;
            
            for rep in stats.member_reputations.values_mut() {
                if *rep > threshold {
                    let excess = *rep - threshold;
                    let decay_amount = (excess as f64 * self.config.decay_rate) as i64;
                    *rep -= decay_amount;
                }
            }
            
            // Update total reputation
            stats.total_reputation = stats.member_reputations.values().sum();
            stats.last_decay = now;
        }
    }
}

#[derive(Debug)]
pub struct ReputationDistribution {
    pub total_reputation: i64,
    pub member_count: usize,
    pub top_holders: Vec<(String, i64)>,
    pub gini_coefficient: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_voting_power_calculation() {
        let config = AntiMonopolyConfig::default();
        let manager = AntiMonopolyManager::new(config);
        
        // Test quadratic voting power
        let power = manager.calculate_voting_power(1_000_000);
        assert!(power <= manager.config.max_voting_power);
        
        // Test that voting power increases sub-linearly
        let power1 = manager.calculate_voting_power(100_000);
        let power2 = manager.calculate_voting_power(200_000);
        assert!(power2 < power1 * 2.0);
    }
    
    #[tokio::test]
    async fn test_reputation_decay() {
        let config = AntiMonopolyConfig {
            max_reputation: 1000,
            decay_rate: 0.1,
            decay_threshold: 0.8,
            ..Default::default()
        };
        let manager = AntiMonopolyManager::new(config);
        
        // Test decay above threshold
        let score = manager.apply_decay("test1", 900).await;
        assert!(score < 900);
        
        // Test no decay below threshold
        let score = manager.apply_decay("test2", 700).await;
        assert_eq!(score, 700);
    }
    
    #[tokio::test]
    async fn test_dominance_calculation() {
        let manager = AntiMonopolyManager::new(Default::default());
        
        // Add some test data
        {
            let mut stats = manager.reputation_stats.write().await;
            stats.member_reputations.insert("test1".to_string(), 600);
            stats.member_reputations.insert("test2".to_string(), 400);
            stats.total_reputation = 1000;
        }
        
        let dominance = manager.calculate_dominance("test1").await;
        assert_eq!(dominance, 0.6);
    }
} 