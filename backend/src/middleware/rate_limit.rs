use crate::reputation::ReputationManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower::limit::RateLimit;
use warp::Filter;
use std::collections::HashMap;
use log::{info, error};
use tokio::sync::Mutex;
use icn_types::IcnError;

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

// Rate limit settings by reputation tier
struct RateLimitTier {
    requests_per_minute: u32,
    burst_capacity: u32,
}

pub struct ReputationAwareRateLimiter {
    rate_limits: HashMap<String, RateLimitTier>, // endpoint -> limit settings
    user_limits: Arc<Mutex<HashMap<String, HashMap<String, (Instant, u32)>>>>, // user_id -> endpoint -> (last_reset, count)
    reputation_manager: Arc<ReputationManager>,
}

impl ReputationAwareRateLimiter {
    pub fn new(reputation_manager: Arc<ReputationManager>) -> Self {
        let mut rate_limits = HashMap::new();
        
        // Define default rate limits for different endpoints
        rate_limits.insert("api/v1/governance/proposals".to_string(), RateLimitTier { 
            requests_per_minute: 10, 
            burst_capacity: 15 
        });
        rate_limits.insert("api/v1/federation".to_string(), RateLimitTier { 
            requests_per_minute: 5, 
            burst_capacity: 10 
        });
        
        Self {
            rate_limits,
            user_limits: Arc::new(Mutex::new(HashMap::new())),
            reputation_manager,
        }
    }
    
    pub async fn is_rate_limited(&self, user_id: &str, endpoint: &str) -> bool {
        // Get user reputation
        let reputation = self.reputation_manager.get_reputation(user_id, "api").await;
        
        // Get base rate limits for this endpoint
        let base_tier = self.rate_limits.get(endpoint).unwrap_or(&RateLimitTier {
            requests_per_minute: 5,
            burst_capacity: 10,
        });
        
        // Calculate adjusted limits based on reputation
        // Higher reputation = higher limits, up to 5x the base limit
        let reputation_multiplier = (1.0 + (reputation as f64 / 100.0)).min(5.0);
        let adjusted_rpm = (base_tier.requests_per_minute as f64 * reputation_multiplier) as u32;
        let adjusted_burst = (base_tier.burst_capacity as f64 * reputation_multiplier) as u32;
        
        // Check current usage
        let mut user_limits = self.user_limits.lock().await;
        let endpoint_limits = user_limits
            .entry(user_id.to_string())
            .or_insert_with(HashMap::new);
            
        let now = Instant::now();
        let (last_reset, count) = endpoint_limits
            .entry(endpoint.to_string())
            .or_insert_with(|| (now, 0));
            
        // Reset counter if a minute has passed
        if now.duration_since(*last_reset) > Duration::from_secs(60) {
            *last_reset = now;
            *count = 0;
        }
        
        // Check if user has exceeded their limit
        if *count >= adjusted_burst {
            return true;
        }
        
        // Increment count
        *count += 1;
        false
    }
}

// Create a warp filter that applies reputation-aware rate limiting
pub fn with_reputation_rate_limit(
    reputation_manager: Arc<ReputationManager>,
    path: &str,
) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    let rate_limiter = Arc::new(ReputationAwareRateLimiter::new(reputation_manager));
    let path_str = path.to_string();
    
    warp::any()
        .and(warp::header::<String>("authorization"))
        .and_then(move |auth_header: String| {
            let user_id = extract_user_id_from_auth(&auth_header);
            let rate_limiter = rate_limiter.clone();
            let endpoint = path_str.clone();
            
            async move {
                if rate_limiter.is_rate_limited(&user_id, &endpoint).await {
                    Err(warp::reject::custom(
                        IcnError::RateLimitError(format!("Rate limit exceeded for {}", endpoint))
                    ))
                } else {
                    Ok(())
                }
            }
        })
}

fn extract_user_id_from_auth(auth_header: &str) -> String {
    // Extract user ID from JWT or other auth mechanism
    // This is a placeholder implementation
    if auth_header.starts_with("Bearer ") {
        auth_header[7..].to_string()
    } else {
        "anonymous".to_string()
    }
}
