//! Prelude module with commonly used types and traits
//! 
//! Import this module to get all the essential types for using MatchForge:
//! 
//! ```rust
//! use matchforge::prelude::*;
//! ```

pub use crate::{
    error::{MatchForgeError, Result},
    lobby::{Lobby, LobbyMetadata, LobbyState},
    mmr::{
        DecayStrategy, EloAlgorithm, Glicko2Algorithm, LinearDecay,
        MmrAlgorithm, NoDecay, Outcome, Rating, Season, SeasonResetStrategy, SoftReset, HardReset,
    },
    party::{AverageStrategy, MaxStrategy, Party, PartyManager, PartyMmrStrategy, WeightedWithPenaltyStrategy},
    persistence::{InMemoryAdapter, PersistenceAdapter},
    queue::{
        EntryMetadata, GreedyMatcher, MatchConstraints, MatchFormat, MatchResult, QueueConfig,
        QueueEntry, QueueManager,
    },
    runner::{LobbyManager, MatchmakingRunner},
    analytics::{
        AnalyticsMetrics, ReportGenerator, InsightEngine, DashboardData,
    },
    telemetry::{
        MatchmakingMetrics, MetricsCollector, Event, EventCollector, MonitoringService,
    },
    security::{
        RateLimiter, AntiAbuseSystem, SecurityManager, SecurityConfig,
        RateLimitConfig, RateLimitResult,
    },
};

// Re-export common external dependencies
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
pub use uuid::Uuid;
