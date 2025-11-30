//! Event system for MatchForge SDK
//! 
//! Provides comprehensive event tracking and logging for all matchmaking operations.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MatchForge events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub data: EventData,
    pub metadata: HashMap<String, String>,
}

/// Types of events that can occur in the matchmaking system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    // Queue events
    PlayerJoinedQueue,
    PlayerLeftQueue,
    QueueSizeChanged,
    
    // Matchmaking events
    MatchmakingStarted,
    MatchmakingCompleted,
    MatchFound,
    MatchQualityCalculated,
    
    // Lobby events
    LobbyCreated,
    LobbyStateChange,
    LobbyDispatched,
    LobbyClosed,
    
    // Party events
    PartyCreated,
    PartyMemberAdded,
    PartyMemberRemoved,
    PartyDissolved,
    
    // Rating events
    RatingUpdated,
    RatingDecayApplied,
    SeasonReset,
    
    // System events
    PersistenceOperation,
    Error,
    Warning,
    Info,
}

/// Event-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventData {
    QueueJoin {
        queue_name: String,
        player_id: Uuid,
        rating: f64,
    },
    QueueLeave {
        queue_name: String,
        player_id: Uuid,
        reason: String,
    },
    QueueSizeChange {
        queue_name: String,
        old_size: usize,
        new_size: usize,
    },
    MatchmakingStart {
        queue_name: String,
        player_count: usize,
    },
    MatchmakingComplete {
        queue_name: String,
        matches_found: usize,
        duration_ms: u64,
    },
    MatchFound {
        match_id: Uuid,
        player_ids: Vec<Uuid>,
        quality_score: f64,
        wait_time_ms: u64,
    },
    LobbyCreated {
        lobby_id: Uuid,
        match_id: Uuid,
        player_count: usize,
    },
    LobbyStateChange {
        lobby_id: Uuid,
        old_state: String,
        new_state: String,
    },
    LobbyDispatched {
        lobby_id: Uuid,
        server_id: String,
    },
    LobbyClosed {
        lobby_id: Uuid,
        duration_seconds: u64,
        reason: String,
    },
    PartyCreated {
        party_id: Uuid,
        leader_id: Uuid,
        max_size: usize,
    },
    PartyMemberAdded {
        party_id: Uuid,
        player_id: Uuid,
    },
    PartyMemberRemoved {
        party_id: Uuid,
        player_id: Uuid,
        reason: String,
    },
    PartyDissolved {
        party_id: Uuid,
        member_count: usize,
    },
    RatingUpdate {
        player_id: Uuid,
        old_rating: f64,
        new_rating: f64,
        algorithm: String,
    },
    RatingDecay {
        player_id: Uuid,
        old_rating: f64,
        new_rating: f64,
        days_inactive: u64,
    },
    SeasonReset {
        player_id: Uuid,
        old_rating: f64,
        new_rating: f64,
        reset_type: String,
    },
    PersistenceOperation {
        operation: String,
        entity_type: String,
        entity_id: Uuid,
        success: bool,
        duration_ms: u64,
    },
    Error {
        error_code: String,
        message: String,
        context: HashMap<String, String>,
    },
    Warning {
        message: String,
        context: HashMap<String, String>,
    },
    Info {
        message: String,
        context: HashMap<String, String>,
    },
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, data: EventData) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            timestamp: Utc::now(),
            data,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
    
    /// Get event severity
    pub fn severity(&self) -> EventSeverity {
        match self.event_type {
            EventType::Error => EventSeverity::Error,
            EventType::Warning => EventSeverity::Warning,
            EventType::Info => EventSeverity::Info,
            _ => EventSeverity::Debug,
        }
    }
    
    /// Check if event is related to a specific player
    pub fn involves_player(&self, player_id: Uuid) -> bool {
        match &self.data {
            EventData::QueueJoin { player_id: pid, .. } => *pid == player_id,
            EventData::QueueLeave { player_id: pid, .. } => *pid == player_id,
            EventData::RatingUpdate { player_id: pid, .. } => *pid == player_id,
            EventData::RatingDecay { player_id: pid, .. } => *pid == player_id,
            EventData::SeasonReset { player_id: pid, .. } => *pid == player_id,
            EventData::PartyMemberAdded { player_id: pid, .. } => *pid == player_id,
            EventData::PartyMemberRemoved { player_id: pid, .. } => *pid == player_id,
            EventData::MatchFound { player_ids, .. } => player_ids.contains(&player_id),
            _ => false,
        }
    }
    
    /// Check if event is related to a specific queue
    pub fn involves_queue(&self, queue_name: &str) -> bool {
        match &self.data {
            EventData::QueueJoin { queue_name: q, .. } => q == queue_name,
            EventData::QueueLeave { queue_name: q, .. } => q == queue_name,
            EventData::QueueSizeChange { queue_name: q, .. } => q == queue_name,
            EventData::MatchmakingStart { queue_name: q, .. } => q == queue_name,
            EventData::MatchmakingComplete { queue_name: q, .. } => q == queue_name,
            _ => false,
        }
    }
}

/// Event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Event collector interface
pub trait EventCollector: Send + Sync {
    /// Record an event
    fn record_event(&self, event: Event);
    
    /// Get events by type
    fn get_events_by_type(&self, event_type: EventType) -> Vec<Event>;
    
    /// Get events by player
    fn get_events_by_player(&self, player_id: Uuid) -> Vec<Event>;
    
    /// Get events by queue
    fn get_events_by_queue(&self, queue_name: &str) -> Vec<Event>;
    
    /// Get events in time range
    fn get_events_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Event>;
    
    /// Get recent events
    fn get_recent_events(&self, limit: usize) -> Vec<Event>;
    
    /// Clear old events
    fn clear_old_events(&self, older_than: DateTime<Utc>);
}

/// In-memory event collector implementation
pub struct MemoryEventCollector {
    events: std::sync::Mutex<Vec<Event>>,
    max_events: usize,
}

impl MemoryEventCollector {
    /// Create a new memory event collector
    pub fn new(max_events: usize) -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::with_capacity(max_events)),
            max_events,
        }
    }
    
    /// Add event to the collector
    fn add_event(&self, event: Event) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
        
        // Remove oldest events if we exceed the limit
        if events.len() > self.max_events {
            let remove_count = events.len() - self.max_events;
            events.drain(0..remove_count);
        }
    }
    
    /// Drain events up to a certain count
    pub fn drain_events(&mut self, count: usize) -> Vec<Event> {
        let mut events = self.events.lock().unwrap();
        let len = events.len();
        if len <= count {
            let result = events.clone();
            events.clear();
            result
        } else {
            let result = events.iter().take(count).cloned().collect();
            *events = events[count..].to_vec();
            result
        }
    }
    
    /// Filter events by predicate
    fn filter_events<F>(&self, predicate: F) -> Vec<Event>
    where
        F: Fn(&Event) -> bool,
    {
        let events = self.events.lock().unwrap();
        events.iter().filter(|e| predicate(e)).cloned().collect()
    }
}

impl EventCollector for MemoryEventCollector {
    fn record_event(&self, event: Event) {
        self.add_event(event);
    }
    
    fn get_events_by_type(&self, event_type: EventType) -> Vec<Event> {
        self.filter_events(|e| std::mem::discriminant(&e.event_type) == std::mem::discriminant(&event_type))
    }
    
    fn get_events_by_player(&self, player_id: Uuid) -> Vec<Event> {
        self.filter_events(|e| e.involves_player(player_id))
    }
    
    fn get_events_by_queue(&self, queue_name: &str) -> Vec<Event> {
        self.filter_events(|e| e.involves_queue(queue_name))
    }
    
    fn get_events_by_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<Event> {
        self.filter_events(|e| e.timestamp >= start && e.timestamp <= end)
    }
    
    fn get_recent_events(&self, limit: usize) -> Vec<Event> {
        let events = self.events.lock().unwrap();
        let start = if events.len() > limit { events.len() - limit } else { 0 };
        events[start..].iter().cloned().collect()
    }
    
    fn clear_old_events(&self, older_than: DateTime<Utc>) {
        let mut events = self.events.lock().unwrap();
        events.retain(|e| e.timestamp >= older_than);
    }
}

/// Event builder for convenient event creation
pub struct EventBuilder {
    event_type: EventType,
    metadata: HashMap<String, String>,
}

impl EventBuilder {
    /// Create a new event builder
    pub fn new(event_type: EventType) -> Self {
        Self {
            event_type,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata
    pub fn metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Build a queue join event
    pub fn queue_join(queue_name: String, player_id: Uuid, rating: f64) -> Event {
        Event::new(
            EventType::PlayerJoinedQueue,
            EventData::QueueJoin {
                queue_name,
                player_id,
                rating,
            },
        )
    }
    
    /// Build a queue leave event
    pub fn queue_leave(queue_name: String, player_id: Uuid, reason: String) -> Event {
        Event::new(
            EventType::PlayerLeftQueue,
            EventData::QueueLeave {
                queue_name,
                player_id,
                reason,
            },
        )
    }
    
    /// Build a match found event
    pub fn match_found(match_id: Uuid, player_ids: Vec<Uuid>, quality_score: f64, wait_time_ms: u64) -> Event {
        Event::new(
            EventType::MatchFound,
            EventData::MatchFound {
                match_id,
                player_ids,
                quality_score,
                wait_time_ms,
            },
        )
    }
    
    /// Build a lobby created event
    pub fn lobby_created(lobby_id: Uuid, match_id: Uuid, player_count: usize) -> Event {
        Event::new(
            EventType::LobbyCreated,
            EventData::LobbyCreated {
                lobby_id,
                match_id,
                player_count,
            },
        )
    }
    
    /// Build a party created event
    pub fn party_created(party_id: Uuid, leader_id: Uuid, max_size: usize) -> Event {
        Event::new(
            EventType::PartyCreated,
            EventData::PartyCreated {
                party_id,
                leader_id,
                max_size,
            },
        )
    }
    
    /// Build a rating update event
    pub fn rating_update(player_id: Uuid, old_rating: f64, new_rating: f64, algorithm: String) -> Event {
        Event::new(
            EventType::RatingUpdated,
            EventData::RatingUpdate {
                player_id,
                old_rating,
                new_rating,
                algorithm,
            },
        )
    }
    
    /// Build an error event
    pub fn error(error_code: String, message: String, context: HashMap<String, String>) -> Event {
        Event::new(
            EventType::Error,
            EventData::Error {
                error_code,
                message,
                context,
            },
        )
    }
    
    /// Build a persistence operation event
    pub fn persistence_operation(operation: String, entity_type: String, entity_id: Uuid, success: bool, duration_ms: u64) -> Event {
        Event::new(
            EventType::PersistenceOperation,
            EventData::PersistenceOperation {
                operation,
                entity_type,
                entity_id,
                success,
                duration_ms,
            },
        )
    }
}

/// Event aggregator for analytics
pub struct EventAggregator {
    collector: Box<dyn EventCollector>,
}

impl EventAggregator {
    /// Create a new event aggregator
    pub fn new(collector: Box<dyn EventCollector>) -> Self {
        Self { collector }
    }
    
    /// Get player activity summary
    pub fn get_player_activity(&self, player_id: Uuid, since: DateTime<Utc>) -> PlayerActivity {
        let events = self.collector.get_events_by_time_range(since, Utc::now());
        let player_events: Vec<_> = events.iter().filter(|e| e.involves_player(player_id)).collect();
        
        let queue_joins = player_events.iter().filter(|e| matches!(e.event_type, EventType::PlayerJoinedQueue)).count();
        let matches_played = player_events.iter().filter(|e| matches!(e.event_type, EventType::MatchFound)).count();
        let rating_updates = player_events.iter().filter(|e| matches!(e.event_type, EventType::RatingUpdated)).count();
        
        PlayerActivity {
            player_id,
            queue_joins,
            matches_played,
            rating_updates,
            last_activity: player_events.last().map(|e| e.timestamp),
        }
    }
    
    /// Get queue performance metrics
    pub fn get_queue_performance(&self, queue_name: &str, since: DateTime<Utc>) -> QueuePerformance {
        let events = self.collector.get_events_by_time_range(since, Utc::now());
        let queue_events: Vec<_> = events.iter().filter(|e| e.involves_queue(queue_name)).collect();
        
        let joins = queue_events.iter().filter(|e| matches!(e.event_type, EventType::PlayerJoinedQueue)).count();
        let leaves = queue_events.iter().filter(|e| matches!(e.event_type, EventType::PlayerLeftQueue)).count();
        let matches_found = queue_events.iter().filter(|e| matches!(e.event_type, EventType::MatchFound)).count();
        
        // Calculate average wait time
        let wait_times: Vec<u64> = queue_events.iter()
            .filter_map(|e| {
                if let EventData::MatchFound { wait_time_ms, .. } = &e.data {
                    Some(*wait_time_ms)
                } else {
                    None
                }
            })
            .collect();
        
        let avg_wait_time = if wait_times.is_empty() {
            0
        } else {
            wait_times.iter().sum::<u64>() / wait_times.len() as u64
        };
        
        QueuePerformance {
            queue_name: queue_name.to_string(),
            joins,
            leaves,
            matches_found,
            avg_wait_time_ms: avg_wait_time,
            success_rate: if joins == 0 { 0.0 } else { matches_found as f64 / joins as f64 },
        }
    }
}

/// Player activity summary
#[derive(Debug, Clone)]
pub struct PlayerActivity {
    pub player_id: Uuid,
    pub queue_joins: usize,
    pub matches_played: usize,
    pub rating_updates: usize,
    pub last_activity: Option<DateTime<Utc>>,
}

/// Queue performance metrics
#[derive(Debug, Clone)]
pub struct QueuePerformance {
    pub queue_name: String,
    pub joins: usize,
    pub leaves: usize,
    pub matches_found: usize,
    pub avg_wait_time_ms: u64,
    pub success_rate: f64,
}
