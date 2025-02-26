use crate::reputation::ReputationManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower::limit::RateLimit;
use warp::Filter;
use std::collections::HashMap;
use log::{info, error};

pub trait RateLimitingOperations {
    fn calculate_limit_multiplier(&self, did: &str) -> f64;
    fn get_rate_limit(&self, did: &str) -> u32;
    fn check_rate_limit(&self, did: &str) -> bool;
}

pub struct AdaptiveRateLimiter {
    reputation_manager: Arc<ReputationManager>,
    base_limit: u32,
    max_limit: u32,
    min_limit: u32,
    cache: HashMap<String, u32>,
}

impl AdaptiveRateLimiter {
    pub fn new(reputation_manager: Arc<ReputationManager>) -> Self {
        Self {
            reputation_manager,
            base_limit: 10,
            max_limit: 50,
            min_limit: 5,
            cache: HashMap::new(),
        }
    }

    async fn calculate_limit_multiplier(&self, did: &str) -> f64 {
        let reputation = self.reputation_manager.get_reputation(did, "api").await;
        let base_multiplier = (reputation as f64 / 100.0).min(1.0).max(0.1);
        
        // Additional multipliers based on federation status
        let federation_multiplier = if self.is_federation_admin(did).await {
            2.0
        } else {
            1.0
        };

        base_multiplier * federation_multiplier
    }

    async fn get_rate_limit(&self, did: &str) -> u32 {
        if let Some(&cached_limit) = self.cache.get(did) {
            return cached_limit;
        }

        let multiplier = self.calculate_limit_multiplier(did).await;
        let limit = (self.base_limit as f64 * multiplier) as u32;
        let clamped_limit = limit.clamp(self.min_limit, self.max_limit);
        self.cache.insert(did.to_string(), clamped_limit);
        clamped_limit
    }

    async fn check_rate_limit(&self, did: &str) -> bool {
        let limit = self.get_rate_limit(did).await;
        // ...existing rate check logic...
        info!("Rate limit for {}: {}", did, limit);
        true
    }
}

impl RateLimitingOperations for AdaptiveRateLimiter {
    fn calculate_limit_multiplier(&self, did: &str) -> f64 {
        // Placeholder implementation
        1.0
    }

    fn get_rate_limit(&self, did: &str) -> u32 {
        // Placeholder implementation
        10
    }

    fn check_rate_limit(&self, did: &str) -> bool {
        // Placeholder implementation
        true
    }
}

pub fn with_rate_limit<F: Filter>(
    filter: F,
    requests_per_second: u64
) -> impl Filter<Extract = F::Extract, Error = warp::Rejection> + Clone {
    let rate_limiter = ServiceBuilder::new()
        .layer(RateLimit::new(
            requests_per_second, 
            Duration::from_secs(1)
        ))
        .service(warp::service(filter));

    warp::service(rate_limiter)
}
