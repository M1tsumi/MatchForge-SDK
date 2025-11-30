//! Advanced analytics metrics for MatchForge SDK
//! 
//! Provides comprehensive metrics collection and analysis for matchmaking operations.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Advanced analytics metrics collector
pub struct AnalyticsMetrics {
    // Player metrics
    total_players: AtomicU64,
    active_players: AtomicU64,
    new_players_today: AtomicU64,
    returning_players: AtomicU64,
    
    // Matchmaking metrics
    total_matches: AtomicU64,
    matches_per_hour: AtomicU64,
    average_wait_time: AtomicI64,
    match_quality_score: AtomicI64,
    matchmaking_success_rate: AtomicI64,
    
    // Queue metrics
    queue_sizes: Arc<RwLock<HashMap<String, u64>>>,
    queue_wait_times: Arc<RwLock<HashMap<String, VecDeque<Duration>>>>,
    abandonment_rates: Arc<RwLock<HashMap<String, f64>>>,
    
    // Rating metrics
    rating_distribution: Arc<RwLock<HashMap<String, u64>>>,
    rating_changes: Arc<RwLock<VecDeque<RatingChange>>>,
    rating_accuracy: AtomicI64,
    
    // Party metrics
    party_sizes: Arc<RwLock<HashMap<usize, u64>>>,
    party_success_rates: Arc<RwLock<HashMap<usize, f64>>>,
    solo_vs_party_win_rates: Arc<RwLock<HashMap<String, f64>>>,
    
    // Performance metrics
    api_response_times: Arc<RwLock<VecDeque<Duration>>>,
    database_query_times: Arc<RwLock<VecDeque<Duration>>>,
    memory_usage: AtomicU64,
    cpu_usage: AtomicI64,
    
    // Business metrics
    player_retention: Arc<RwLock<HashMap<String, f64>>>,
    session_durations: Arc<RwLock<VecDeque<Duration>>>,
    churn_rate: AtomicI64,
    revenue_per_player: AtomicI64,
    
    // Time series data
    hourly_metrics: Arc<RwLock<VecDeque<HourlyMetrics>>>,
    daily_metrics: Arc<RwLock<VecDeque<DailyMetrics>>>,
    
    // Configuration
    config: AnalyticsConfig,
}

/// Analytics configuration
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// How long to keep time series data
    pub retention_period: Duration,
    
    /// How often to aggregate metrics
    pub aggregation_interval: Duration,
    
    /// Maximum number of data points to keep
    pub max_data_points: usize,
    
    /// Enable detailed tracking
    pub enable_detailed_tracking: bool,
    
    /// Enable predictive analytics
    pub enable_predictive_analytics: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            retention_period: Duration::from_secs(90 * 24 * 60 * 60),
            aggregation_interval: Duration::from_hours(1),
            max_data_points: 10000,
            enable_detailed_tracking: true,
            enable_predictive_analytics: true,
        }
    }
}

/// Rating change tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingChange {
    pub player_id: Uuid,
    pub old_rating: f64,
    pub new_rating: f64,
    pub change_amount: f64,
    pub match_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub outcome: String,
}

/// Hourly aggregated metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetrics {
    pub timestamp: DateTime<Utc>,
    pub active_players: u64,
    pub matches_completed: u64,
    pub average_wait_time: f64,
    pub average_rating: f64,
    pub queue_abandonments: u64,
    pub new_players: u64,
}

/// Daily aggregated metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetrics {
    pub date: DateTime<Utc>,
    pub total_players: u64,
    pub active_players: u64,
    pub matches_completed: u64,
    pub average_session_duration: Duration,
    pub retention_rate: f64,
    pub churn_rate: f64,
    pub revenue: f64,
}

impl AnalyticsMetrics {
    /// Create new analytics metrics collector
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            total_players: AtomicU64::new(0),
            active_players: AtomicU64::new(0),
            new_players_today: AtomicU64::new(0),
            returning_players: AtomicU64::new(0),
            total_matches: AtomicU64::new(0),
            matches_per_hour: AtomicU64::new(0),
            average_wait_time: AtomicI64::new(0),
            match_quality_score: AtomicI64::new(0),
            matchmaking_success_rate: AtomicI64::new(0),
            queue_sizes: Arc::new(RwLock::new(HashMap::new())),
            queue_wait_times: Arc::new(RwLock::new(HashMap::new())),
            abandonment_rates: Arc::new(RwLock::new(HashMap::new())),
            rating_distribution: Arc::new(RwLock::new(HashMap::new())),
            rating_changes: Arc::new(RwLock::new(VecDeque::new())),
            rating_accuracy: AtomicI64::new(0),
            party_sizes: Arc::new(RwLock::new(HashMap::new())),
            party_success_rates: Arc::new(RwLock::new(HashMap::new())),
            solo_vs_party_win_rates: Arc::new(RwLock::new(HashMap::new())),
            api_response_times: Arc::new(RwLock::new(VecDeque::new())),
            database_query_times: Arc::new(RwLock::new(VecDeque::new())),
            memory_usage: AtomicU64::new(0),
            cpu_usage: AtomicI64::new(0),
            player_retention: Arc::new(RwLock::new(HashMap::new())),
            session_durations: Arc::new(RwLock::new(VecDeque::new())),
            churn_rate: AtomicI64::new(0),
            revenue_per_player: AtomicI64::new(0),
            hourly_metrics: Arc::new(RwLock::new(VecDeque::new())),
            daily_metrics: Arc::new(RwLock::new(VecDeque::new())),
            config,
        }
    }
    
    /// Record player activity
    pub async fn record_player_activity(&self, player_id: Uuid, activity_type: PlayerActivityType) {
        match activity_type {
            PlayerActivityType::Login => {
                self.active_players.fetch_add(1, Ordering::Relaxed);
                self.returning_players.fetch_add(1, Ordering::Relaxed);
            }
            PlayerActivityType::Logout => {
                self.active_players.fetch_sub(1, Ordering::Relaxed);
            }
            PlayerActivityType::NewPlayer => {
                self.total_players.fetch_add(1, Ordering::Relaxed);
                self.new_players_today.fetch_add(1, Ordering::Relaxed);
                self.active_players.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    
    /// Record match completion
    pub async fn record_match_completed(&self, match_data: MatchCompletionData) {
        self.total_matches.fetch_add(1, Ordering::Relaxed);
        self.match_quality_score.store(
            ((self.match_quality_score.load(Ordering::Relaxed) as f64 + match_data.quality_score) / 2.0) as i64,
            Ordering::Relaxed,
        );
        
        // Update rating distribution
        let mut rating_dist = self.rating_distribution.write().await;
        let rating_bucket = self.get_rating_bucket(match_data.average_rating);
        *rating_dist.entry(rating_bucket).or_insert(0) += 1;
        
        // Record rating changes
        if self.config.enable_detailed_tracking {
            let mut rating_changes = self.rating_changes.write().await;
            for change in match_data.rating_changes {
                rating_changes.push_back(change);
                if rating_changes.len() > self.config.max_data_points {
                    rating_changes.pop_front();
                }
            }
        }
    }
    
    /// Record queue activity
    pub async fn record_queue_activity(&self, queue_name: String, activity: QueueActivity) {
        match activity {
            QueueActivity::PlayerJoined => {
                let mut sizes = self.queue_sizes.write().await;
                *sizes.entry(queue_name).or_insert(0) += 1;
            }
            QueueActivity::PlayerLeft(wait_time) => {
                let mut sizes = self.queue_sizes.write().await;
                if let Some(size) = sizes.get_mut(&queue_name) {
                    *size = size.saturating_sub(1);
                }
                
                // Record wait time
                let mut wait_times = self.queue_wait_times.write().await;
                let queue_wait_times = wait_times.entry(queue_name.clone()).or_insert_with(VecDeque::new);
                queue_wait_times.push_back(wait_time);
                if queue_wait_times.len() > 1000 {
                    queue_wait_times.pop_front();
                }
                
                // Update abandonment rate
                self.update_abandonment_rate(&queue_name).await;
            }
            QueueActivity::MatchFound(wait_time) => {
                // Update average wait time
                let current_avg = self.average_wait_time.load(Ordering::Relaxed) as f64;
                let new_avg = (current_avg + wait_time.as_secs_f64()) / 2.0;
                self.average_wait_time.store(new_avg as i64, Ordering::Relaxed);
                
                // Remove players from queue
                let mut sizes = self.queue_sizes.write().await;
                if let Some(size) = sizes.get_mut(&queue_name) {
                    *size = size.saturating_sub(2); // Assuming 2 players per match
                }
            }
        }
    }
    
    /// Record party activity
    pub async fn record_party_activity(&self, party_size: usize, activity: PartyActivity) {
        match activity {
            PartyActivity::Created => {
                let mut sizes = self.party_sizes.write().await;
                *sizes.entry(party_size).or_insert(0) += 1;
            }
            PartyActivity::MatchFound(success) => {
                let mut success_rates = self.party_success_rates.write().await;
                let current_rate = success_rates.get(&party_size).copied().unwrap_or(0.0);
                let new_rate = (current_rate + if success { 1.0 } else { 0.0 }) / 2.0;
                success_rates.insert(party_size, new_rate);
            }
        }
    }
    
    /// Record performance metrics
    pub async fn record_performance(&self, metric: PerformanceMetric) {
        match metric {
            PerformanceMetric::ApiResponseTime(duration) => {
                let mut times = self.api_response_times.write().await;
                times.push_back(duration);
                if times.len() > 1000 {
                    times.pop_front();
                }
            }
            PerformanceMetric::DatabaseQueryTime(duration) => {
                let mut times = self.database_query_times.write().await;
                times.push_back(duration);
                if times.len() > 1000 {
                    times.pop_front();
                }
            }
            PerformanceMetric::MemoryUsage(bytes) => {
                self.memory_usage.store(bytes, Ordering::Relaxed);
            }
            PerformanceMetric::CpuUsage(percent) => {
                self.cpu_usage.store(percent as i64, Ordering::Relaxed);
            }
        }
    }
    
    /// Get comprehensive metrics snapshot
    pub async fn get_metrics_snapshot(&self) -> MetricsSnapshot {
        let queue_sizes = self.queue_sizes.read().await.clone();
        let rating_distribution = self.rating_distribution.read().await.clone();
        let party_sizes = self.party_sizes.read().await.clone();
        let api_times = self.api_response_times.read().await.clone();
        let db_times = self.database_query_times.read().await.clone();
        
        MetricsSnapshot {
            timestamp: Utc::now(),
            total_players: self.total_players.load(Ordering::Relaxed),
            active_players: self.active_players.load(Ordering::Relaxed),
            new_players_today: self.new_players_today.load(Ordering::Relaxed),
            total_matches: self.total_matches.load(Ordering::Relaxed),
            average_wait_time: Duration::from_secs_f64(self.average_wait_time.load(Ordering::Relaxed) as f64),
            match_quality_score: self.match_quality_score.load(Ordering::Relaxed) as f64,
            matchmaking_success_rate: self.matchmaking_success_rate.load(Ordering::Relaxed) as f64,
            queue_sizes,
            rating_distribution,
            party_sizes,
            average_api_response_time: self.calculate_average_duration(&api_times),
            average_db_query_time: self.calculate_average_duration(&db_times),
            memory_usage_mb: self.memory_usage.load(Ordering::Relaxed) / 1024 / 1024,
            cpu_usage_percent: self.cpu_usage.load(Ordering::Relaxed) as f64,
            revenue_per_player: self.revenue_per_player.load(Ordering::Relaxed) as f64,
        }
    }
    
    /// Generate hourly aggregation
    pub async fn generate_hourly_aggregation(&self) -> HourlyMetrics {
        let snapshot = self.get_metrics_snapshot().await;
        
        HourlyMetrics {
            timestamp: snapshot.timestamp,
            active_players: snapshot.active_players,
            matches_completed: snapshot.total_matches,
            average_wait_time: snapshot.average_wait_time.as_secs_f64(),
            average_rating: self.calculate_average_rating().await,
            queue_abandonments: self.calculate_total_abandonments().await,
            new_players: snapshot.new_players_today,
        }
    }
    
    /// Generate daily aggregation
    pub async fn generate_daily_aggregation(&self) -> DailyMetrics {
        let hourly_metrics = self.hourly_metrics.read().await;
        let total_matches: u64 = hourly_metrics.iter().map(|m| m.matches_completed).sum();
        let avg_active_players: f64 = if hourly_metrics.is_empty() {
            0.0
        } else {
            hourly_metrics.iter().map(|m| m.active_players as f64).sum::<f64>() / hourly_metrics.len() as f64
        };
        
        DailyMetrics {
            date: Utc::now(),
            total_players: self.total_players.load(Ordering::Relaxed),
            active_players: avg_active_players as u64,
            matches_completed: total_matches,
            average_session_duration: self.calculate_average_session_duration().await,
            retention_rate: self.calculate_retention_rate().await,
            churn_rate: self.churn_rate.load(Ordering::Relaxed) as f64,
            revenue: self.revenue_per_player.load(Ordering::Relaxed) as f64 * avg_active_players,
        }
    }
    
    /// Predictive analytics
    pub async fn predict_queue_wait_time(&self, queue_name: &str, player_rating: f64) -> Duration {
        if !self.config.enable_predictive_analytics {
            return Duration::from_secs(60); // Default estimate
        }
        
        let wait_times = self.queue_wait_times.read().await;
        if let Some(queue_wait_times) = wait_times.get(queue_name) {
            if queue_wait_times.len() < 10 {
                return Duration::from_secs(60);
            }
            
            // Simple linear regression based on historical data
            let avg_wait = self.calculate_average_duration(queue_wait_times);
            let queue_size = self.queue_sizes.read().await.get(queue_name).copied().unwrap_or(0);
            
            // Adjust based on queue size and rating deviation from average
            let avg_rating = self.calculate_average_rating().await;
            let rating_factor = 1.0 + (player_rating - avg_rating).abs() / 1000.0;
            let size_factor = 1.0 + (queue_size as f64 / 100.0);
            
            let predicted_duration = Duration::from_secs_f64(
                avg_wait.as_secs_f64() * rating_factor * size_factor
            );
            
            predicted_duration
        } else {
            Duration::from_secs(60)
        }
    }
    
    /// Get player retention analytics
    pub async fn get_retention_analytics(&self) -> RetentionAnalytics {
        let retention_data = self.player_retention.read().await.clone();
        
        RetentionAnalytics {
            day_1_retention: retention_data.get("day_1").copied().unwrap_or(0.0),
            day_7_retention: retention_data.get("day_7").copied().unwrap_or(0.0),
            day_30_retention: retention_data.get("day_30").copied().unwrap_or(0.0),
            average_session_duration: self.calculate_average_session_duration().await,
            churn_rate: self.churn_rate.load(Ordering::Relaxed) as f64,
        }
    }
    
    // Helper methods
    fn get_rating_bucket(&self, rating: f64) -> String {
        if rating < 1000.0 { "0-999".to_string() }
        else if rating < 1200.0 { "1000-1199".to_string() }
        else if rating < 1400.0 { "1200-1399".to_string() }
        else if rating < 1600.0 { "1400-1599".to_string() }
        else if rating < 1800.0 { "1600-1799".to_string() }
        else if rating < 2000.0 { "1800-1999".to_string() }
        else { "2000+".to_string() }
    }
    
    async fn update_abandonment_rate(&self, queue_name: &str) {
        let wait_times = self.queue_wait_times.read().await;
        if let Some(queue_wait_times) = wait_times.get(queue_name) {
            if queue_wait_times.len() > 10 {
                let avg_wait = self.calculate_average_duration(queue_wait_times);
                let abandonment_rate = (avg_wait.as_secs_f64() / 300.0).min(1.0); // 5 minutes = 100% abandonment
                
                let mut rates = self.abandonment_rates.write().await;
                rates.insert(queue_name.to_string(), abandonment_rate);
            }
        }
    }
    
    fn calculate_average_duration(&self, durations: &VecDeque<Duration>) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }
        
        let total: Duration = durations.iter().sum();
        total / durations.len() as u32
    }
    
    async fn calculate_average_rating(&self) -> f64 {
        let rating_dist = self.rating_distribution.read().await;
        if rating_dist.is_empty() {
            return 1500.0;
        }
        
        let mut total_rating = 0.0;
        let mut total_players = 0u64;
        
        for (bucket, count) in rating_dist.iter() {
            let avg_rating = match bucket.as_str() {
                "0-999" => 500.0,
                "1000-1199" => 1100.0,
                "1200-1399" => 1300.0,
                "1400-1599" => 1500.0,
                "1600-1799" => 1700.0,
                "1800-1999" => 1900.0,
                "2000+" => 2100.0,
                _ => 1500.0,
            };
            
            total_rating += avg_rating * *count as f64;
            total_players += count;
        }
        
        if total_players > 0 {
            total_rating / total_players as f64
        } else {
            1500.0
        }
    }
    
    async fn calculate_total_abandonments(&self) -> u64 {
        let rates = self.abandonment_rates.read().await;
        rates.values().map(|rate| (*rate * 100.0) as u64).sum()
    }
    
    async fn calculate_average_session_duration(&self) -> Duration {
        let sessions = self.session_durations.read().await;
        self.calculate_average_duration(&sessions)
    }
    
    async fn calculate_retention_rate(&self) -> f64 {
        let retention_data = self.player_retention.read().await;
        retention_data.get("day_7").copied().unwrap_or(0.0)
    }
}

/// Player activity types
#[derive(Debug, Clone)]
pub enum PlayerActivityType {
    Login,
    Logout,
    NewPlayer,
}

/// Match completion data
#[derive(Debug, Clone)]
pub struct MatchCompletionData {
    pub match_id: Uuid,
    pub average_rating: f64,
    pub quality_score: f64,
    pub duration: Duration,
    pub rating_changes: Vec<RatingChange>,
}

/// Queue activity types
#[derive(Debug, Clone)]
pub enum QueueActivity {
    PlayerJoined,
    PlayerLeft(Duration),
    MatchFound(Duration),
}

/// Party activity types
#[derive(Debug, Clone)]
pub enum PartyActivity {
    Created,
    MatchFound(bool), // success
}

/// Performance metrics
#[derive(Debug, Clone)]
pub enum PerformanceMetric {
    ApiResponseTime(Duration),
    DatabaseQueryTime(Duration),
    MemoryUsage(u64),
    CpuUsage(f64),
}

/// Comprehensive metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_players: u64,
    pub active_players: u64,
    pub new_players_today: u64,
    pub total_matches: u64,
    pub average_wait_time: Duration,
    pub match_quality_score: f64,
    pub matchmaking_success_rate: f64,
    pub queue_sizes: HashMap<String, u64>,
    pub rating_distribution: HashMap<String, u64>,
    pub party_sizes: HashMap<usize, u64>,
    pub average_api_response_time: Duration,
    pub average_db_query_time: Duration,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub revenue_per_player: f64,
}

/// Retention analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionAnalytics {
    pub day_1_retention: f64,
    pub day_7_retention: f64,
    pub day_30_retention: f64,
    pub average_session_duration: Duration,
    pub churn_rate: f64,
}

/// Trait for metrics collection
pub trait MetricsCollector: Send + Sync {
    fn record_metric(&self, event: MetricEvent);
    fn get_metrics(&self) -> MetricsSnapshot;
    fn reset_metrics(&self);
}

/// Metric events
#[derive(Debug, Clone)]
pub enum MetricEvent {
    PlayerJoined { player_id: Uuid, queue: String },
    PlayerLeft { player_id: Uuid, queue: String, wait_time: Duration },
    MatchFound { match_id: Uuid, quality: f64 },
    RatingUpdated { player_id: Uuid, old_rating: f64, new_rating: f64 },
    PartyCreated { party_id: Uuid, size: usize },
    Performance { metric: PerformanceMetric },
}

/// Default metrics collector implementation
pub struct DefaultMetricsCollector {
    analytics: Arc<AnalyticsMetrics>,
}

impl DefaultMetricsCollector {
    pub fn new() -> Self {
        Self {
            analytics: Arc::new(AnalyticsMetrics::new(AnalyticsConfig::default())),
        }
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn record_metric(&self, event: MetricEvent) {
        let analytics = self.analytics.clone();
        tokio::spawn(async move {
            match event {
                MetricEvent::PlayerJoined { player_id, queue } => {
                    analytics.record_player_activity(player_id, PlayerActivityType::Login).await;
                    analytics.record_queue_activity(queue, QueueActivity::PlayerJoined).await;
                }
                MetricEvent::PlayerLeft { player_id, queue, wait_time } => {
                    analytics.record_player_activity(player_id, PlayerActivityType::Logout).await;
                    analytics.record_queue_activity(queue, QueueActivity::PlayerLeft(wait_time)).await;
                }
                MetricEvent::MatchFound { match_id, quality } => {
                    // This would need more context for full implementation
                }
                MetricEvent::RatingUpdated { player_id, old_rating, new_rating } => {
                    // Record rating change
                }
                MetricEvent::PartyCreated { party_id, size } => {
                    analytics.record_party_activity(size, PartyActivity::Created).await;
                }
                MetricEvent::Performance { metric } => {
                    analytics.record_performance(metric).await;
                }
            }
        });
    }
    
    fn get_metrics(&self) -> MetricsSnapshot {
        // This would need to be async in a real implementation
        // For now, return a placeholder
        MetricsSnapshot {
            timestamp: Utc::now(),
            total_players: 0,
            active_players: 0,
            new_players_today: 0,
            total_matches: 0,
            average_wait_time: Duration::ZERO,
            match_quality_score: 0.0,
            matchmaking_success_rate: 0.0,
            queue_sizes: HashMap::new(),
            rating_distribution: HashMap::new(),
            party_sizes: HashMap::new(),
            average_api_response_time: Duration::ZERO,
            average_db_query_time: Duration::ZERO,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            revenue_per_player: 0.0,
        }
    }
    
    fn reset_metrics(&self) {
        // Reset all metrics to zero
    }
}
