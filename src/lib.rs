//! MatchForge SDK
//! 
//! A comprehensive matchmaking SDK for multiplayer games with MMR systems,
//! queue management, lobby handling, party support, and pluggable persistence.
//! 
//! # Quick Start
//! 
//! ```rust
//! use matchforge::prelude::*;
//! use std::sync::Arc;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create persistence layer
//!     let persistence = Arc::new(InMemoryAdapter::new());
//!     
//!     // Create managers
//!     let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
//!     let party_manager = Arc::new(PartyManager::new(persistence.clone(), Arc::new(AverageStrategy)));
//!     let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));
//!     
//!     // Configure queues
//!     let queue_config = QueueConfig {
//!         name: "ranked_1v1".to_string(),
//!         format: MatchFormat::one_v_one(),
//!         constraints: MatchConstraints::strict(),
//!     };
//!     queue_manager.register_queue(queue_config).await?;
//!     
//!     // Start matchmaking runner
//!     let runner = MatchmakingRunner::new(
//!         RunnerConfig::default(),
//!         queue_manager.clone(),
//!         persistence.clone(),
//!     );
//!     
//!     // Add player to queue
//!     let player_id = Uuid::new_v4();
//!     let rating = Rating::default_beginner();
//!     let entry = queue_manager.join_queue_solo(
//!         "ranked_1v1".to_string(),
//!         player_id,
//!         rating,
//!         EntryMetadata::default(),
//!     ).await?;
//!     
//!     // Start runner in background
//!     tokio::spawn(async move {
//!         if let Err(e) = runner.start().await {
//!             eprintln!("Runner error: {}", e);
//!         }
//!     });
//!     
//!     Ok(())
//! }
//! ```

pub mod analytics;
pub mod error;
pub mod lobby;
pub mod mmr;
pub mod party;
pub mod persistence;
pub mod queue;
pub mod runner;
pub mod security;
pub mod telemetry;

// Re-export commonly used types
pub use error::{MatchForgeError, Result};
pub use lobby::{Lobby, LobbyMetadata, LobbyState};
pub use mmr::{
    DecayStrategy, EloAlgorithm, Glicko2Algorithm, LinearDecay,
    MmrAlgorithm, NoDecay, Outcome, Rating, Season, SeasonResetStrategy, SoftReset, HardReset,
};
pub use party::{AverageStrategy, MaxStrategy, Party, PartyManager, PartyMmrStrategy, WeightedWithPenaltyStrategy};
pub use persistence::{InMemoryAdapter, PersistenceAdapter};
pub use queue::{
    EntryMetadata, GreedyMatcher, MatchConstraints, MatchFormat, MatchResult, QueueConfig,
    QueueEntry, QueueManager,
};
pub use runner::{LobbyManager, MatchmakingRunner, RunnerConfig};
pub use analytics::{AnalyticsMetrics, ReportGenerator, InsightEngine, DashboardData};
pub use telemetry::{MatchmakingMetrics, MetricsCollector, Event, EventCollector, MonitoringService};
pub use security::{RateLimiter, AntiAbuseSystem, SecurityManager, SecurityConfig};

/// Prelude module for convenient imports
pub mod prelude;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn basic_matchmaking_flow() -> Result<()> {
        let persistence = Arc::new(InMemoryAdapter::new());
        let queue_manager = Arc::new(QueueManager::new(persistence.clone()));

        // Register queue
        let queue_config = QueueConfig {
            name: "test_queue".to_string(),
            format: MatchFormat::one_v_one(),
            constraints: MatchConstraints::permissive(),
        };
        queue_manager.register_queue(queue_config).await?;

        // Add two players
        let player1 = Uuid::new_v4();
        let player2 = Uuid::new_v4();
        let rating = Rating::default_beginner();

        queue_manager.join_queue_solo(
            "test_queue".to_string(),
            player1,
            rating,
            EntryMetadata::default(),
        ).await?;

        queue_manager.join_queue_solo(
            "test_queue".to_string(),
            player2,
            rating,
            EntryMetadata::default(),
        ).await?;

        // Find matches
        let matches = queue_manager.find_matches("test_queue").await?;
        assert_eq!(matches.len(), 1);

        Ok(())
    }
}
