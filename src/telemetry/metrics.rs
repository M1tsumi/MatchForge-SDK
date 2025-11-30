//! Metrics collection for MatchForge SDK
//! 
//! Provides comprehensive metrics tracking for all matchmaking operations.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

/// Comprehensive matchmaking metrics
#[derive(Debug)]
pub struct MatchmakingMetrics {
    // Queue metrics
    pub queue_sizes: HashMap<String, AtomicUsize>,
    pub total_queue_joins: AtomicU64,
    pub total_queue_leaves: AtomicU64,
    
    // Match metrics
    pub matches_found: AtomicU64,
    pub matches_completed: AtomicU64,
    pub average_wait_time: AtomicU64, // in milliseconds
    pub average_match_quality: AtomicU64, // scaled by 1000
    
    // Player metrics
    pub active_players: AtomicUsize,
    pub total_players: AtomicU64,
    
    // Performance metrics
    pub matchmaking_duration: AtomicU64, // in microseconds
    pub persistence_operations: AtomicU64,
    pub persistence_errors: AtomicU64,
    
    // Party metrics
    pub active_parties: AtomicUsize,
    pub total_parties_created: AtomicU64,
    pub average_party_size: AtomicU64, // scaled by 100
    
    // Lobby metrics
    pub active_lobbies: AtomicUsize,
    pub total_lobbies_created: AtomicU64,
    pub average_lobby_duration: AtomicU64, // in seconds
    
    // Timestamps
    pub last_updated: DateTime<Utc>,
}

impl MatchmakingMetrics {
    pub fn new() -> Self {
        Self {
            queue_sizes: HashMap::new(),
            total_queue_joins: AtomicU64::new(0),
            total_queue_leaves: AtomicU64::new(0),
            matches_found: AtomicU64::new(0),
            matches_completed: AtomicU64::new(0),
            average_wait_time: AtomicU64::new(0),
            average_match_quality: AtomicU64::new(0),
            active_players: AtomicUsize::new(0),
            total_players: AtomicU64::new(0),
            matchmaking_duration: AtomicU64::new(0),
            persistence_operations: AtomicU64::new(0),
            persistence_errors: AtomicU64::new(0),
            active_parties: AtomicUsize::new(0),
            total_parties_created: AtomicU64::new(0),
            average_party_size: AtomicU64::new(0),
            active_lobbies: AtomicUsize::new(0),
            total_lobbies_created: AtomicU64::new(0),
            average_lobby_duration: AtomicU64::new(0),
            last_updated: Utc::now(),
        }
    }
    
    /// Record a queue join
    pub fn record_queue_join(&mut self, queue_name: &str) {
        self.total_queue_joins.fetch_add(1, Ordering::Relaxed);
        self.get_or_create_queue_counter(queue_name).fetch_add(1, Ordering::Relaxed);
        self.active_players.fetch_add(1, Ordering::Relaxed);
        self.total_players.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record a queue leave
    pub fn record_queue_leave(&mut self, queue_name: &str) {
        self.total_queue_leaves.fetch_add(1, Ordering::Relaxed);
        if let Some(counter) = self.queue_sizes.get(queue_name) {
            counter.fetch_sub(1, Ordering::Relaxed);
        }
        self.active_players.fetch_sub(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record a match found
    pub fn record_match_found(&mut self, wait_time_ms: u64, quality_score: f64) {
        self.matches_found.fetch_add(1, Ordering::Relaxed);
        
        // Update average wait time (exponential moving average)
        let current_avg = self.average_wait_time.load(Ordering::Relaxed);
        let new_avg = ((current_avg as f64 * 0.9) + (wait_time_ms as f64 * 0.1)) as u64;
        self.average_wait_time.store(new_avg, Ordering::Relaxed);
        
        // Update average match quality (scaled by 1000)
        let quality_scaled = (quality_score * 1000.0) as u64;
        let current_quality = self.average_match_quality.load(Ordering::Relaxed);
        let new_quality = ((current_quality as f64 * 0.9) + (quality_scaled as f64 * 0.1)) as u64;
        self.average_match_quality.store(new_quality, Ordering::Relaxed);
        
        self.update_timestamp();
    }
    
    /// Record a match completed
    pub fn record_match_completed(&mut self) {
        self.matches_completed.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record matchmaking operation duration
    pub fn record_matchmaking_duration(&mut self, duration_us: u64) {
        let current = self.matchmaking_duration.load(Ordering::Relaxed);
        let new_avg = ((current as f64 * 0.9) + (duration_us as f64 * 0.1)) as u64;
        self.matchmaking_duration.store(new_avg, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record persistence operation
    pub fn record_persistence_operation(&mut self) {
        self.persistence_operations.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record persistence error
    pub fn record_persistence_error(&mut self) {
        self.persistence_errors.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record party creation
    pub fn record_party_created(&mut self, size: usize) {
        self.total_parties_created.fetch_add(1, Ordering::Relaxed);
        self.active_parties.fetch_add(1, Ordering::Relaxed);
        
        // Update average party size (scaled by 100)
        let size_scaled = (size * 100) as u64;
        let current_avg = self.average_party_size.load(Ordering::Relaxed);
        let new_avg = ((current_avg as f64 * 0.9) + (size_scaled as f64 * 0.1)) as u64;
        self.average_party_size.store(new_avg, Ordering::Relaxed);
        
        self.update_timestamp();
    }
    
    /// Record party dissolution
    pub fn record_party_dissolved(&mut self) {
        self.active_parties.fetch_sub(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record lobby creation
    pub fn record_lobby_created(&mut self) {
        self.total_lobbies_created.fetch_add(1, Ordering::Relaxed);
        self.active_lobbies.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }
    
    /// Record lobby closure
    pub fn record_lobby_closed(&mut self, duration_seconds: u64) {
        self.active_lobbies.fetch_sub(1, Ordering::Relaxed);
        
        // Update average lobby duration
        let current_avg = self.average_lobby_duration.load(Ordering::Relaxed);
        let new_avg = ((current_avg as f64 * 0.9) + (duration_seconds as f64 * 0.1)) as u64;
        self.average_lobby_duration.store(new_avg, Ordering::Relaxed);
        
        self.update_timestamp();
    }
    
    /// Get queue size for a specific queue
    pub fn get_queue_size(&self, queue_name: &str) -> usize {
        self.queue_sizes
            .get(queue_name)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
    
    /// Get all queue sizes
    pub fn get_all_queue_sizes(&self) -> HashMap<String, usize> {
        self.queue_sizes
            .iter()
            .map(|(name, counter)| (name.clone(), counter.load(Ordering::Relaxed)))
            .collect()
    }
    
    /// Get current metrics snapshot
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            queue_sizes: self.get_all_queue_sizes(),
            total_queue_joins: self.total_queue_joins.load(Ordering::Relaxed),
            total_queue_leaves: self.total_queue_leaves.load(Ordering::Relaxed),
            matches_found: self.matches_found.load(Ordering::Relaxed),
            matches_completed: self.matches_completed.load(Ordering::Relaxed),
            average_wait_time_ms: self.average_wait_time.load(Ordering::Relaxed),
            average_match_quality: self.average_match_quality.load(Ordering::Relaxed) as f64 / 1000.0,
            active_players: self.active_players.load(Ordering::Relaxed),
            total_players: self.total_players.load(Ordering::Relaxed),
            matchmaking_duration_us: self.matchmaking_duration.load(Ordering::Relaxed),
            persistence_operations: self.persistence_operations.load(Ordering::Relaxed),
            persistence_errors: self.persistence_errors.load(Ordering::Relaxed),
            active_parties: self.active_parties.load(Ordering::Relaxed),
            total_parties_created: self.total_parties_created.load(Ordering::Relaxed),
            average_party_size: self.average_party_size.load(Ordering::Relaxed) as f64 / 100.0,
            active_lobbies: self.active_lobbies.load(Ordering::Relaxed),
            total_lobbies_created: self.total_lobbies_created.load(Ordering::Relaxed),
            average_lobby_duration_seconds: self.average_lobby_duration.load(Ordering::Relaxed),
            timestamp: self.last_updated,
        }
    }
    
    fn get_or_create_queue_counter(&mut self, queue_name: &str) -> &AtomicUsize {
        // Note: This would need to be implemented with Arc<Mutex<HashMap>> for thread safety
        // For now, this is a simplified version
        self.queue_sizes
            .entry(queue_name.to_string())
            .or_insert_with(|| AtomicUsize::new(0))
    }
    
    fn update_timestamp(&mut self) {
        // Note: This would need to be implemented with Arc<Mutex<DateTime<Utc>>>
        // For now, this is a simplified version
    }
}

/// Immutable snapshot of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub queue_sizes: HashMap<String, usize>,
    pub total_queue_joins: u64,
    pub total_queue_leaves: u64,
    pub matches_found: u64,
    pub matches_completed: u64,
    pub average_wait_time_ms: u64,
    pub average_match_quality: f64,
    pub active_players: usize,
    pub total_players: u64,
    pub matchmaking_duration_us: u64,
    pub persistence_operations: u64,
    pub persistence_errors: u64,
    pub active_parties: usize,
    pub total_parties_created: u64,
    pub average_party_size: f64,
    pub active_lobbies: usize,
    pub total_lobbies_created: u64,
    pub average_lobby_duration_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

impl MetricsSnapshot {
    /// Calculate success rate (matches found / queue joins)
    pub fn success_rate(&self) -> f64 {
        if self.total_queue_joins == 0 {
            0.0
        } else {
            self.matches_found as f64 / self.total_queue_joins as f64
        }
    }
    
    /// Calculate completion rate (matches completed / matches found)
    pub fn completion_rate(&self) -> f64 {
        if self.matches_found == 0 {
            0.0
        } else {
            self.matches_completed as f64 / self.matches_found as f64
        }
    }
    
    /// Calculate persistence error rate
    pub fn persistence_error_rate(&self) -> f64 {
        if self.persistence_operations == 0 {
            0.0
        } else {
            self.persistence_errors as f64 / self.persistence_operations as f64
        }
    }
    
    /// Get queue health metrics
    pub fn queue_health(&self) -> QueueHealth {
        let total_queue_size: usize = self.queue_sizes.values().sum();
        let avg_wait_time = self.average_wait_time_ms;
        
        QueueHealth {
            total_queued_players: total_queue_size,
            average_wait_time_ms: avg_wait_time,
            queue_count: self.queue_sizes.len(),
            health_score: self.calculate_health_score(),
        }
    }
    
    fn calculate_health_score(&self) -> f64 {
        // Simple health score calculation (0-100)
        let wait_score = if self.average_wait_time_ms < 30000 { 100.0 } else { 100.0 - (self.average_wait_time_ms as f64 / 1000.0) };
        let success_score = self.success_rate() * 100.0;
        let error_score = (1.0 - self.persistence_error_rate()) * 100.0;
        
        (wait_score + success_score + error_score) / 3.0
    }
}

/// Queue health metrics
#[derive(Debug, Clone)]
pub struct QueueHealth {
    pub total_queued_players: usize,
    pub average_wait_time_ms: u64,
    pub queue_count: usize,
    pub health_score: f64,
}

/// Metrics collector interface
pub trait MetricsCollector: Send + Sync {
    /// Record a metric event
    fn record_metric(&self, event: MetricEvent);
    
    /// Get current metrics snapshot
    fn get_metrics(&self) -> MetricsSnapshot;
    
    /// Reset all metrics
    fn reset_metrics(&self);
}

/// Metric events
#[derive(Debug, Clone)]
pub enum MetricEvent {
    QueueJoin { queue_name: String, player_id: Uuid },
    QueueLeave { queue_name: String, player_id: Uuid },
    MatchFound { match_id: Uuid, wait_time_ms: u64, quality_score: f64 },
    MatchCompleted { match_id: Uuid },
    MatchmakingStarted { queue_name: String },
    MatchmakingCompleted { queue_name: String, duration_us: u64 },
    PersistenceOperation { operation_type: String, success: bool },
    PartyCreated { party_id: Uuid, size: usize },
    PartyDissolved { party_id: Uuid },
    LobbyCreated { lobby_id: Uuid },
    LobbyClosed { lobby_id: Uuid, duration_seconds: u64 },
}

/// Default metrics collector implementation
pub struct DefaultMetricsCollector {
    metrics: Arc<Mutex<MatchmakingMetrics>>,
}

impl DefaultMetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(MatchmakingMetrics::new())),
        }
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn record_metric(&self, event: MetricEvent) {
        let mut metrics = self.metrics.lock().unwrap();
        match event {
            MetricEvent::QueueJoin { queue_name, .. } => {
                metrics.record_queue_join(&queue_name);
            }
            MetricEvent::QueueLeave { queue_name, .. } => {
                metrics.record_queue_leave(&queue_name);
            }
            MetricEvent::MatchFound { match_id: _, wait_time_ms, quality_score } => {
                metrics.record_match_found(wait_time_ms, quality_score);
            }
            MetricEvent::MatchCompleted { .. } => {
                metrics.record_match_completed();
            }
            MetricEvent::MatchmakingStarted { .. } => {
                // No specific action needed for matchmaking start
            }
            MetricEvent::MatchmakingCompleted { duration_us, .. } => {
                metrics.record_matchmaking_duration(duration_us);
            }
            MetricEvent::PersistenceOperation { success: true, .. } => {
                metrics.record_persistence_operation();
            }
            MetricEvent::PersistenceOperation { success: false, .. } => {
                metrics.record_persistence_error();
            }
            MetricEvent::PartyCreated { size, .. } => {
                metrics.record_party_created(size);
            }
            MetricEvent::PartyDissolved { .. } => {
                metrics.record_party_dissolved();
            }
            MetricEvent::LobbyCreated { .. } => {
                metrics.record_lobby_created();
            }
            MetricEvent::LobbyClosed { duration_seconds, .. } => {
                metrics.record_lobby_closed(duration_seconds);
            }
        }
    }
    
    fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.lock().unwrap().get_snapshot()
    }
    
    fn reset_metrics(&self) {
        // Note: This would need to be implemented with proper synchronization
        // For now, this is a placeholder
    }
}
