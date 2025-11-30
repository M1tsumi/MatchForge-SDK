//! Rate limiting for MatchForge SDK
//! 
//! Provides comprehensive rate limiting to prevent abuse and ensure fair usage.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u64,
    /// Time window for rate limiting
    pub window: Duration,
    /// Penalty multiplier for violations
    pub penalty_multiplier: f64,
    /// Maximum penalty duration
    pub max_penalty_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            penalty_multiplier: 2.0,
            max_penalty_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Rate limiting result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitResult {
    Allowed,
    Denied {
        reason: String,
        retry_after: Duration,
    },
}

/// Rate limiter implementation
pub struct RateLimiter {
    config: RateLimitConfig,
    counters: Arc<RwLock<HashMap<Uuid, RateCounter>>>,
    penalties: Arc<RwLock<HashMap<Uuid, Penalty>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            counters: Arc::new(RwLock::new(HashMap::new())),
            penalties: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Check if a request is allowed
    pub async fn check_rate_limit(&self, client_id: Uuid) -> RateLimitResult {
        // Check if client is currently penalized
        if let Some(penalty) = self.get_penalty(client_id).await {
            if !penalty.is_expired() {
                return RateLimitResult::Denied {
                    reason: format!("Rate limit penalty: {}", penalty.reason),
                    retry_after: penalty.remaining_duration(),
                };
            }
        }
        
        // Check rate limit
        let mut counters = self.counters.write().await;
        let counter = counters.entry(client_id).or_insert_with(|| RateCounter::new(self.config.window));
        
        if counter.increment() > self.config.max_requests {
            // Apply penalty
            let penalty_duration = self.calculate_penalty_duration(counter.violations);
            self.apply_penalty(client_id, "Too many requests".to_string(), penalty_duration).await;
            
            return RateLimitResult::Denied {
                reason: "Rate limit exceeded".to_string(),
                retry_after: penalty_duration,
            };
        }
        
        RateLimitResult::Allowed
    }
    
    /// Check rate limit for specific operation type
    pub async fn check_operation_limit(&self, client_id: Uuid, operation: &str) -> RateLimitResult {
        let key = self.generate_operation_key(client_id, operation);
        self.check_rate_limit(key).await
    }
    
    /// Get current rate limit status
    pub async fn get_status(&self, client_id: Uuid) -> RateLimitStatus {
        let counters = self.counters.read().await;
        let penalty = self.get_penalty(client_id).await;
        
        let counter = counters.get(&client_id);
        
        RateLimitStatus {
            current_requests: counter.map(|c| c.count).unwrap_or(0),
            max_requests: self.config.max_requests,
            window_seconds: self.config.window.as_secs(),
            violations: counter.map(|c| c.violations).unwrap_or(0),
            penalty: penalty.map(|p| PenaltyStatus {
                reason: p.reason.clone(),
                expires_at: p.expires_at,
                remaining_seconds: p.remaining_duration().as_secs(),
            }),
        }
    }
    
    /// Reset rate limit for a client
    pub async fn reset(&self, client_id: Uuid) {
        let mut counters = self.counters.write().await;
        counters.remove(&client_id);
        
        let mut penalties = self.penalties.write().await;
        penalties.remove(&client_id);
    }
    
    /// Clean up expired entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        
        // Clean up expired counters
        let mut counters = self.counters.write().await;
        counters.retain(|_, counter| !counter.is_expired(now));
        
        // Clean up expired penalties
        let mut penalties = self.penalties.write().await;
        penalties.retain(|_, penalty| !penalty.is_expired());
    }
    
    async fn get_penalty(&self, client_id: Uuid) -> Option<Penalty> {
        let penalties = self.penalties.read().await;
        penalties.get(&client_id).cloned()
    }
    
    async fn apply_penalty(&self, client_id: Uuid, reason: String, duration: Duration) {
        let mut penalties = self.penalties.write().await;
        penalties.insert(client_id, Penalty {
            reason,
            expires_at: Utc::now() + chrono::Duration::from_std(duration).unwrap(),
        });
    }
    
    fn calculate_penalty_duration(&self, violations: u64) -> Duration {
        let base_duration = self.config.window;
        let multiplier = self.config.penalty_multiplier.powi(violations as i32 - 1);
        let penalty_duration = Duration::from_secs(
            (base_duration.as_secs() as f64 * multiplier) as u64
        );
        
        std::cmp::min(penalty_duration, self.config.max_penalty_duration)
    }
    
    fn generate_operation_key(&self, client_id: Uuid, operation: &str) -> Uuid {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        client_id.hash(&mut hasher);
        operation.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Create UUID from hash using builder
        let bytes = hash.to_le_bytes();
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes[..8].copy_from_slice(&bytes);
        
        Uuid::from_bytes(uuid_bytes)
    }
}

/// Rate counter for tracking requests
#[derive(Debug, Clone)]
struct RateCounter {
    count: u64,
    violations: u64,
    window: Duration,
    window_start: Instant,
}

impl RateCounter {
    fn new(window: Duration) -> Self {
        Self {
            count: 0,
            violations: 0,
            window,
            window_start: Instant::now(),
        }
    }
    
    fn increment(&mut self) -> u64 {
        let now = Instant::now();
        
        // Reset if window has expired
        if now.duration_since(self.window_start) > self.window {
            self.count = 0;
            self.window_start = now;
        }
        
        self.count += 1;
        self.count
    }
    
    fn is_expired(&self, now: Instant) -> bool {
        now.duration_since(self.window_start) > self.window * 2
    }
}

/// Penalty for rate limit violations
#[derive(Debug, Clone)]
struct Penalty {
    reason: String,
    expires_at: DateTime<Utc>,
}

impl Penalty {
    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
    
    fn remaining_duration(&self) -> Duration {
        let remaining = self.expires_at - Utc::now();
        if remaining.num_milliseconds() > 0 {
            Duration::from_millis(remaining.num_milliseconds() as u64)
        } else {
            Duration::ZERO
        }
    }
}

/// Rate limit status information
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub current_requests: u64,
    pub max_requests: u64,
    pub window_seconds: u64,
    pub violations: u64,
    pub penalty: Option<PenaltyStatus>,
}

/// Penalty status information
#[derive(Debug, Clone)]
pub struct PenaltyStatus {
    pub reason: String,
    pub expires_at: DateTime<Utc>,
    pub remaining_seconds: u64,
}

/// Multi-tier rate limiter for different operation types
pub struct MultiTierRateLimiter {
    tiers: HashMap<String, RateLimiter>,
}

impl MultiTierRateLimiter {
    /// Create a new multi-tier rate limiter
    pub fn new() -> Self {
        let mut tiers = HashMap::new();
        
        // Default tier
        tiers.insert("default".to_string(), RateLimiter::new(RateLimitConfig::default()));
        
        // Queue operations (more restrictive)
        tiers.insert("queue_join".to_string(), RateLimiter::new(RateLimitConfig {
            max_requests: 10,
            window: Duration::from_secs(60),
            penalty_multiplier: 2.0,
            max_penalty_duration: Duration::from_secs(300),
        }));
        
        // Party operations (moderate restriction)
        tiers.insert("party".to_string(), RateLimiter::new(RateLimitConfig {
            max_requests: 20,
            window: Duration::from_secs(60),
            penalty_multiplier: 1.5,
            max_penalty_duration: Duration::from_secs(180),
        }));
        
        // Rating operations (least restrictive)
        tiers.insert("rating".to_string(), RateLimiter::new(RateLimitConfig {
            max_requests: 50,
            window: Duration::from_secs(60),
            penalty_multiplier: 1.2,
            max_penalty_duration: Duration::from_secs(120),
        }));
        
        Self { tiers }
    }
    
    /// Check rate limit for a specific tier
    pub async fn check_tier(&self, client_id: Uuid, tier: &str) -> RateLimitResult {
        self.tiers
            .get(tier)
            .unwrap_or(self.tiers.get("default").unwrap())
            .check_rate_limit(client_id)
            .await
    }
    
    /// Add a custom tier
    pub fn add_tier(&mut self, name: String, config: RateLimitConfig) {
        self.tiers.insert(name, RateLimiter::new(config));
    }
    
    /// Get status for all tiers
    pub async fn get_all_status(&self, client_id: Uuid) -> HashMap<String, RateLimitStatus> {
        let mut status = HashMap::new();
        
        for (name, limiter) in &self.tiers {
            status.insert(name.clone(), limiter.get_status(client_id).await);
        }
        
        status
    }
    
    /// Reset all tiers for a client
    pub async fn reset_all(&self, client_id: Uuid) {
        for limiter in self.tiers.values() {
            limiter.reset(client_id).await;
        }
    }
    
    /// Clean up all tiers
    pub async fn cleanup_all(&self) {
        for limiter in self.tiers.values() {
            limiter.cleanup().await;
        }
    }
}

/// Distributed rate limiter for multi-instance deployments
pub struct DistributedRateLimiter {
    local_limiter: RateLimiter,
    // In a real implementation, this would use Redis or another distributed store
    // For now, we'll use the local limiter as a placeholder
}

impl DistributedRateLimiter {
    /// Create a new distributed rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            local_limiter: RateLimiter::new(config),
        }
    }
    
    /// Check rate limit across all instances
    pub async fn check_distributed(&self, client_id: Uuid) -> RateLimitResult {
        // In a real implementation, this would coordinate with other instances
        // For now, we'll just use the local limiter
        self.local_limiter.check_rate_limit(client_id).await
    }
    
    /// Synchronize rate limit data across instances
    pub async fn sync(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would sync data with other instances
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_basic_rate_limiting() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(1),
            penalty_multiplier: 2.0,
            max_penalty_duration: Duration::from_secs(5),
        };
        
        let limiter = RateLimiter::new(config);
        let client_id = Uuid::new_v4();
        
        // First 5 requests should be allowed
        for _ in 0..5 {
            assert_eq!(limiter.check_rate_limit(client_id).await, RateLimitResult::Allowed);
        }
        
        // 6th request should be denied
        let result = limiter.check_rate_limit(client_id).await;
        assert!(matches!(result, RateLimitResult::Denied { .. }));
    }
    
    #[tokio::test]
    async fn test_window_reset() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
            penalty_multiplier: 2.0,
            max_penalty_duration: Duration::from_secs(5),
        };
        
        let limiter = RateLimiter::new(config);
        let client_id = Uuid::new_v4();
        
        // Use up the limit
        for _ in 0..2 {
            assert_eq!(limiter.check_rate_limit(client_id).await, RateLimitResult::Allowed);
        }
        
        // Should be denied
        assert!(matches!(limiter.check_rate_limit(client_id).await, RateLimitResult::Denied { .. }));
        
        // Wait for window to reset
        sleep(Duration::from_millis(150)).await;
        
        // Should be allowed again
        assert_eq!(limiter.check_rate_limit(client_id).await, RateLimitResult::Allowed);
    }
    
    #[tokio::test]
    async fn test_multi_tier_rate_limiting() {
        let limiter = MultiTierRateLimiter::new();
        let client_id = Uuid::new_v4();
        
        // Queue tier should be more restrictive
        for _ in 0..10 {
            assert_eq!(limiter.check_tier(client_id, "queue_join").await, RateLimitResult::Allowed);
        }
        
        // 11th request should be denied
        assert!(matches!(limiter.check_tier(client_id, "queue_join").await, RateLimitResult::Denied { .. }));
        
        // But rating tier should still allow requests
        assert_eq!(limiter.check_tier(client_id, "rating").await, RateLimitResult::Allowed);
    }
}
