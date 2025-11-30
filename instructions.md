1. High-Level Architecture Overview
Design Philosophy
MatchForge follows a functional-core, imperative-shell architecture with trait-based abstractions for maximum flexibility. The system is designed around these principles:

Pluggable components: MMR algorithms, persistence backends, and matchmaking strategies are all trait-based
Zero-copy where possible: Use references and smart pointers to minimize allocations
Async-first: Built on Tokio for scalable concurrent matchmaking
Type-safe: Leverage Rust's type system to prevent invalid states
Testable: Pure functions for business logic, side effects isolated to adapters

Core Architecture Layers
┌─────────────────────────────────────────────────────────────┐
│                     Game Server Layer                        │
│              (Your game logic integrates here)               │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                  MatchForge Public API                       │
│  (MatchmakingService, Queue Manager, Lobby Manager)         │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                   Core Business Logic                        │
│     ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│     │   MMR    │  │  Queue   │  │  Lobby   │  │  Party   │ │
│     │  System  │  │  Logic   │  │  Logic   │  │  System  │ │
│     └──────────┘  └──────────┘  └──────────┘  └──────────┘ │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              Persistence Abstraction Layer                   │
│                  (Trait-based adapters)                      │
└─────────────────────┬───────────────────────────────────────┘
                      │
         ┌────────────┼────────────┐
         ▼            ▼            ▼
    ┌────────┐  ┌─────────┐  ┌──────────┐
    │In-Memory│  │  Redis  │  │ Postgres │
    │ Adapter │  │ Adapter │  │ Adapter  │
    └────────┘  └─────────┘  └──────────┘
Data Flow

Player joins queue → Queue entry created → Persisted
Matchmaking tick → Runner queries queue → Applies matching algorithm
Match found → Lobby created → Players notified → Queue entries removed
Match complete → MMR updated → Results persisted


2. Complete Crate Structure
matchforge/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs                 # Public API surface
│   ├── prelude.rs             # Convenient re-exports
│   ├── error.rs               # Error types
│   │
│   ├── mmr/
│   │   ├── mod.rs
│   │   ├── rating.rs          # MMR/Rating types
│   │   ├── algorithm.rs       # Trait + implementations (Elo, Glicko2)
│   │   ├── decay.rs           # MMR decay logic
│   │   └── season.rs          # Seasonal reset logic
│   │
│   ├── queue/
│   │   ├── mod.rs
│   │   ├── entry.rs           # QueueEntry struct
│   │   ├── manager.rs         # Queue management
│   │   ├── matcher.rs         # Matching algorithm
│   │   └── constraints.rs     # Match constraints (skill delta, latency)
│   │
│   ├── lobby/
│   │   ├── mod.rs
│   │   ├── lobby.rs           # Lobby struct + lifecycle
│   │   ├── team.rs            # Team assignment logic
│   │   └── state.rs           # Lobby state machine
│   │
│   ├── party/
│   │   ├── mod.rs
│   │   ├── party.rs           # Party struct
│   │   ├── manager.rs         # Party operations
│   │   └── mmr_strategy.rs    # Party MMR aggregation
│   │
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── traits.rs          # Storage traits
│   │   ├── memory.rs          # In-memory implementation
│   │   ├── redis.rs           # Redis adapter (skeleton)
│   │   └── postgres.rs        # Postgres adapter (skeleton)
│   │
│   └── runner/
│       ├── mod.rs
│       ├── tick.rs            # Tick-based runner
│       └── config.rs          # Runner configuration
│
└── examples/
    ├── basic.rs               # Simple 1v1 matchmaking
    ├── party_matchmaking.rs   # Party-based queues
    └── custom_mmr.rs          # Custom MMR algorithm

3. Complete Rust Implementation
3.1 Cargo.toml
toml[package]
name = "matchforge"
version = "0.1.0"
edition = "2021"
authors = ["Quefep"]
license = "MIT OR Apache-2.0"
description = "A plug-and-play matchmaking SDK for multiplayer games"
repository = "https://github.com/yourusername/matchforge"
keywords = ["matchmaking", "game", "multiplayer", "mmr", "elo"]
categories = ["game-development", "network-programming"]

[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"

[dev-dependencies]
tokio-test = "0.4"

[features]
default = []
redis = ["dep:redis"]
postgres = ["dep:sqlx"]

[profile.release]
lto = true
codegen-units = 1
3.2 src/error.rs
rustuse thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MatchForgeError {
    #[error("Player not found: {0}")]
    PlayerNotFound(Uuid),

    #[error("Party not found: {0}")]
    PartyNotFound(Uuid),

    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    #[error("Lobby not found: {0}")]
    LobbyNotFound(Uuid),

    #[error("Player already in queue: {0}")]
    AlreadyInQueue(Uuid),

    #[error("Player not in queue: {0}")]
    NotInQueue(Uuid),

    #[error("Party is full (max size: {0})")]
    PartyFull(usize),

    #[error("Invalid party operation: {0}")]
    InvalidPartyOperation(String),

    #[error("Match constraints not satisfied: {0}")]
    ConstraintsNotSatisfied(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, MatchForgeError>;
3.3 src/mmr/rating.rs
rustuse serde::{Deserialize, Serialize};

/// Represents a player's skill rating
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rating {
    /// Current skill rating (e.g., 1500)
    pub rating: f64,
    /// Rating deviation/uncertainty (used in Glicko systems)
    pub deviation: f64,
    /// Volatility (rate of rating change, Glicko2 only)
    pub volatility: f64,
}

impl Rating {
    pub fn new(rating: f64, deviation: f64, volatility: f64) -> Self {
        Self {
            rating,
            deviation,
            volatility,
        }
    }

    /// Create a default beginner rating
    pub fn default_beginner() -> Self {
        Self {
            rating: 1500.0,
            deviation: 350.0,
            volatility: 0.06,
        }
    }

    /// Get a conservative estimate of skill (rating - 2*deviation)
    pub fn conservative_estimate(&self) -> f64 {
        self.rating - 2.0 * self.deviation
    }
}

impl Default for Rating {
    fn default() -> Self {
        Self::default_beginner()
    }
}

/// Match outcome from a player's perspective
#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Outcome {
    pub fn score(&self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Loss => 0.0,
            Outcome::Draw => 0.5,
        }
    }
}
3.4 src/mmr/algorithm.rs
rustuse super::rating::{Outcome, Rating};
use async_trait::async_trait;

/// Trait for MMR calculation algorithms
#[async_trait]
pub trait MmrAlgorithm: Send + Sync {
    /// Calculate new rating after a match
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating;

    /// Get the name of this algorithm
    fn name(&self) -> &str;
}

/// Simple Elo rating system
pub struct EloAlgorithm {
    k_factor: f64,
}

impl EloAlgorithm {
    pub fn new(k_factor: f64) -> Self {
        Self { k_factor }
    }

    pub fn default() -> Self {
        Self { k_factor: 32.0 }
    }

    fn expected_score(&self, rating_a: f64, rating_b: f64) -> f64 {
        1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) / 400.0))
    }
}

#[async_trait]
impl MmrAlgorithm for EloAlgorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        let expected = self.expected_score(player_rating.rating, opponent_rating.rating);
        let actual = outcome.score();
        let new_rating = player_rating.rating + self.k_factor * (actual - expected);

        Rating {
            rating: new_rating,
            deviation: player_rating.deviation * 0.99, // Slight confidence increase
            volatility: player_rating.volatility,
        }
    }

    fn name(&self) -> &str {
        "Elo"
    }
}

/// Glicko2 rating system (simplified implementation)
pub struct Glicko2Algorithm {
    tau: f64, // System volatility constant
}

impl Glicko2Algorithm {
    pub fn new(tau: f64) -> Self {
        Self { tau }
    }

    pub fn default() -> Self {
        Self { tau: 0.5 }
    }

    fn g(&self, deviation: f64) -> f64 {
        let q = (3.0 * deviation.powi(2)) / std::f64::consts::PI.powi(2);
        1.0 / (1.0 + q).sqrt()
    }

    fn expected_score(&self, rating: f64, opponent_rating: f64, opponent_deviation: f64) -> f64 {
        let g_value = self.g(opponent_deviation);
        1.0 / (1.0 + (-g_value * (rating - opponent_rating) / 400.0).exp())
    }
}

#[async_trait]
impl MmrAlgorithm for Glicko2Algorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        let g_value = self.g(opponent_rating.deviation);
        let expected = self.expected_score(
            player_rating.rating,
            opponent_rating.rating,
            opponent_rating.deviation,
        );
        let actual = outcome.score();

        // Simplified Glicko2 update (full algorithm is more complex)
        let d_squared = 1.0 / (g_value.powi(2) * expected * (1.0 - expected));
        let variance = 1.0 / d_squared;

        let delta = variance * g_value * (actual - expected);
        let new_rating = player_rating.rating + delta;

        // Update deviation (simplified)
        let new_deviation = (player_rating.deviation.powi(2) + variance).sqrt();

        Rating {
            rating: new_rating,
            deviation: new_deviation.min(350.0), // Cap deviation
            volatility: player_rating.volatility,
        }
    }

    fn name(&self) -> &str {
        "Glicko2"
    }
}
3.5 src/mmr/decay.rs
rustuse super::rating::Rating;
use chrono::{DateTime, Duration, Utc};

/// MMR decay strategy
pub trait DecayStrategy: Send + Sync {
    /// Apply decay to a rating based on inactivity
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating;
}

/// Linear decay: reduce rating by a fixed amount per time period
pub struct LinearDecay {
    pub decay_per_day: f64,
    pub max_decay: f64,
}

impl LinearDecay {
    pub fn new(decay_per_day: f64, max_decay: f64) -> Self {
        Self {
            decay_per_day,
            max_decay,
        }
    }

    pub fn default() -> Self {
        Self {
            decay_per_day: 1.0,
            max_decay: 100.0,
        }
    }
}

impl DecayStrategy for LinearDecay {
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating {
        let now = Utc::now();
        let days_inactive = (now - last_match_time).num_days() as f64;

        if days_inactive <= 0.0 {
            return rating;
        }

        let decay_amount = (self.decay_per_day * days_inactive).min(self.max_decay);

        Rating {
            rating: (rating.rating - decay_amount).max(0.0),
            deviation: (rating.deviation + days_inactive * 0.5).min(350.0), // Increase uncertainty
            volatility: rating.volatility,
        }
    }
}

/// No decay strategy
pub struct NoDecay;

impl DecayStrategy for NoDecay {
    fn apply_decay(&self, rating: Rating, _last_match_time: DateTime<Utc>) -> Rating {
        rating
    }
}
3.6 src/mmr/season.rs
rustuse super::rating::Rating;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a competitive season
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl Season {
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now < self.end_time
    }
}

/// Strategy for resetting ratings at season boundaries
pub trait SeasonResetStrategy: Send + Sync {
    /// Calculate the new rating at the start of a season
    fn reset_rating(&self, current_rating: Rating) -> Rating;
}

/// Soft reset: move rating toward the mean
pub struct SoftReset {
    pub target_rating: f64,
    pub reset_percentage: f64, // 0.0 to 1.0
}

impl SoftReset {
    pub fn new(target_rating: f64, reset_percentage: f64) -> Self {
        Self {
            target_rating,
            reset_percentage,
        }
    }

    pub fn default() -> Self {
        Self {
            target_rating: 1500.0,
            reset_percentage: 0.5,
        }
    }
}

impl SeasonResetStrategy for SoftReset {
    fn reset_rating(&self, current_rating: Rating) -> Rating {
        let diff = self.target_rating - current_rating.rating;
        let new_rating = current_rating.rating + diff * self.reset_percentage;

        Rating {
            rating: new_rating,
            deviation: 200.0, // Moderate uncertainty for new season
            volatility: current_rating.volatility,
        }
    }
}

/// Hard reset: everyone starts at the same rating
pub struct HardReset {
    pub reset_rating: f64,
}

impl HardReset {
    pub fn new(reset_rating: f64) -> Self {
        Self { reset_rating }
    }
}

impl SeasonResetStrategy for HardReset {
    fn reset_rating(&self, _current_rating: Rating) -> Rating {
        Rating {
            rating: self.reset_rating,
            deviation: 350.0, // High uncertainty
            volatility: 0.06,
        }
    }
}
3.7 src/mmr/mod.rs
rustpub mod algorithm;
pub mod decay;
pub mod rating;
pub mod season;

pub use algorithm::{EloAlgorithm, Glicko2Algorithm, MmrAlgorithm};
pub use decay::{DecayStrategy, LinearDecay, NoDecay};
pub use rating::{Outcome, Rating};
pub use season::{HardReset, Season, SeasonResetStrategy, SoftReset};
3.8 src/queue/entry.rs
rustuse crate::mmr::Rating;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A player or party's entry in a matchmaking queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    pub id: Uuid,
    pub queue_name: String,
    pub player_ids: Vec<Uuid>,
    pub party_id: Option<Uuid>,
    pub average_rating: Rating,
    pub joined_at: DateTime<Utc>,
    pub metadata: EntryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMetadata {
    /// Optional role preferences (e.g., "tank", "healer", "dps")
    pub roles: Vec<String>,
    /// Region/latency bucket
    pub region: Option<String>,
    /// Custom data for game-specific needs
    pub custom: std::collections::HashMap<String, String>,
}

impl QueueEntry {
    pub fn new_solo(
        queue_name: String,
        player_id: Uuid,
        rating: Rating,
        metadata: EntryMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue_name,
            player_ids: vec![player_id],
            party_id: None,
            average_rating: rating,
            joined_at: Utc::now(),
            metadata,
        }
    }

    pub fn new_party(
        queue_name: String,
        party_id: Uuid,
        player_ids: Vec<Uuid>,
        average_rating: Rating,
        metadata: EntryMetadata,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue_name,
            player_ids,
            party_id: Some(party_id),
            average_rating,
            joined_at: Utc::now(),
            metadata,
        }
    }

    /// Time spent in queue
    pub fn wait_time(&self) -> chrono::Duration {
        Utc::now() - self.joined_at
    }

    /// Is this a solo player?
    pub fn is_solo(&self) -> bool {
        self.party_id.is_none() && self.player_ids.len() == 1
    }

    /// Number of players in this entry
    pub fn player_count(&self) -> usize {
        self.player_ids.len()
    }
}

impl Default for EntryMetadata {
    fn default() -> Self {
        Self {
            roles: Vec::new(),
            region: None,
            custom: std::collections::HashMap::new(),
        }
    }
}
3.9 src/queue/constraints.rs
rustuse super::entry::QueueEntry;

/// Constraints for matching players together
#[derive(Debug, Clone)]
pub struct MatchConstraints {
    /// Maximum MMR difference between players
    pub max_rating_delta: f64,
    /// Must players be in the same region?
    pub same_region_required: bool,
    /// Role requirements (e.g., need 1 tank, 1 healer, 3 dps)
    pub role_requirements: Vec<RoleRequirement>,
    /// Maximum wait time before relaxing constraints
    pub max_wait_time_seconds: i64,
    /// How much to expand search range per second waited
    pub expansion_rate: f64,
}

#[derive(Debug, Clone)]
pub struct RoleRequirement {
    pub role: String,
    pub count: usize,
}

impl MatchConstraints {
    pub fn permissive() -> Self {
        Self {
            max_rating_delta: 500.0,
            same_region_required: false,
            role_requirements: Vec::new(),
            max_wait_time_seconds: 60,
            expansion_rate: 10.0,
        }
    }

    pub fn strict() -> Self {
        Self {
            max_rating_delta: 100.0,
            same_region_required: true,
            role_requirements: Vec::new(),
            max_wait_time_seconds: 300,
            expansion_rate: 5.0,
        }
    }

    /// Calculate effective rating delta based on wait time
    pub fn effective_rating_delta(&self, entry: &QueueEntry) -> f64 {
        let wait_seconds = entry.wait_time().num_seconds();
        let expansion = (wait_seconds as f64) * self.expansion_rate;
        self.max_rating_delta + expansion
    }

    /// Check if two entries can be matched together
    pub fn can_match(&self, entry_a: &QueueEntry, entry_b: &QueueEntry) -> bool {
        // Check rating constraint with expansion
        let max_delta = self.effective_rating_delta(entry_a).max(self.effective_rating_delta(entry_b));
        let rating_diff = (entry_a.average_rating.rating - entry_b.average_rating.rating).abs();

        if rating_diff > max_delta {
            return false;
        }

        // Check region constraint
        if self.same_region_required {
            match (&entry_a.metadata.region, &entry_b.metadata.region) {
                (Some(r1), Some(r2)) if r1 == r2 => {},
                (None, None) => {},
                _ => return false,
            }
        }

        true
    }
}

impl Default for MatchConstraints {
    fn default() -> Self {
        Self::permissive()
    }
}
3.10 src/queue/matcher.rs
rustuse super::{constraints::MatchConstraints, entry::QueueEntry};
use uuid::Uuid;

/// Configuration for a match format
#[derive(Debug, Clone)]
pub struct MatchFormat {
    pub name: String,
    pub team_sizes: Vec<usize>, // e.g., [1, 1] for 1v1, [5, 5] for 5v5
    pub total_players: usize,
}

impl MatchFormat {
    pub fn one_v_one() -> Self {
        Self {
            name: "1v1".to_string(),
            team_sizes: vec![1, 1],
            total_players: 2,
        }
    }

    pub fn two_v_two() -> Self {
        Self {
            name: "2v2".to_string(),
            team_sizes: vec![2, 2],
            total_players: 4,
        }
    }

    pub fn five_v_five() -> Self {
        Self {
            name: "5v5".to_string(),
            team_sizes: vec![5, 5],
            total_players: 10,
        }
    }

    pub fn free_for_all(player_count: usize) -> Self {
        Self {
            name: format!("{}-player-ffa", player_count),
            team_sizes: vec![1; player_count],
            total_players: player_count,
        }
    }
}

/// Result of a successful match
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub match_id: Uuid,
    pub entries: Vec<QueueEntry>,
    pub team_assignments: Vec<usize>, // Index in entries -> team number
}

/// Simple greedy matchmaking algorithm
pub struct GreedyMatcher {
    pub format: MatchFormat,
    pub constraints: MatchConstraints,
}

impl GreedyMatcher {
    pub fn new(format: MatchFormat, constraints: MatchConstraints) -> Self {
        Self { format, constraints }
    }

    /// Attempt to find a match from the given queue entries
    pub fn find_match(&self, entries: &[QueueEntry]) -> Option<MatchResult> {
        if entries.len() < self.format.total_players {
            return None;
        }

        // Calculate total players needed
        let total_needed = self.format.total_players;

        // Try to form a match by greedily selecting compatible entries
        let mut selected: Vec<QueueEntry> = Vec::new();
        let mut player_count = 0;

        // Sort by wait time (prioritize longest waiting)
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort_by_key(|e| e.joined_at);

        for entry in sorted_entries {
            if player_count >= total_needed {
                break;
            }

            // Check if this entry is compatible with already selected entries
            let compatible = selected.is_empty() || selected.iter().all(|s| self.constraints.can_match(s, &entry));

            if compatible && player_count + entry.player_count() <= total_needed {
                player_count += entry.player_count();
                selected.push(entry);
            }
        }

        if player_count == total_needed {
            // Assign teams
            let team_assignments = self.assign_teams(&selected);
            Some(MatchResult {
                match_id: Uuid::new_v4(),
                entries: selected,
                team_assignments,
            })
        } else {
            None
        }
    }

    /// Assign entries to teams
    fn assign_teams(&self, entries: &[QueueEntry]) -> Vec<usize> {
        let mut assignments = Vec::new();
        let mut current_team = 0;
        let mut team_fill: Vec<usize> = vec![0; self.format.team_sizes.len()];

        for entry in entries {
            // Find a team that needs more players
            while team_fill[current_team] >= self.format.team_sizes[current_team] {
                current_team += 1;
                if current_team >= self.format.team_sizes.len() {
                    break;
                }
            }

            assignments.push(current_team);
            team_fill[current_team] += entry.player_count();
        }

        assignments
    }
}
3.11 src/queue/manager.rs
rustuse super::{
    constraints::MatchConstraints,
    entry::{EntryMetadata, QueueEntry},
    matcher::{GreedyMatcher, MatchFormat, MatchResult},
};
use crate::{error::*, mmr::Rating, persistence::PersistenceAdapter};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for a queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub name: String,
    pub format: MatchFormat,
    pub constraints: MatchConstraints,
}

/// Manages multiple queues and their entries
pub struct QueueManager {
    queues: Arc<RwLock<HashMap<String, Vec<QueueEntry>>>>,
    configs: Arc<RwLock<HashMap<String, QueueConfig>>>,
    persistence: Arc<dyn PersistenceAdapter>,
}

impl QueueManager {
    pub fn new(persistence: Arc<dyn PersistenceAdapter>) -> Self {
        Self {
            queues: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            persistence,
        }
    }

    /// Register a new queue
    pub async fn register_queue(&self, config: QueueConfig) -> Result<()> {
        let mut configs = self.configs.write().await;
        let mut queues = self.queues.write().await;

        configs.insert(config.name.clone(), config.clone());
        queues.insert(config.name.clone(), Vec::new());

        Ok(())
    }

    /// Add a solo player to a queue
    pub async fn join_queue_solo(
        &self,
        queue_name: String,
        player_id: Uuid,
        rating: Rating,
        metadata: EntryMetadata,
    ) -> Result<QueueEntry> {
        let entry = QueueEntry::new_solo(queue_name.clone(), player_id, rating, metadata);

        self.add_entry(entry.clone()).await?;
        self.persistence.save_queue_entry(&entry).await?;

        Ok(entry)
    }

    /// Add a party to a queue
    pub async fn join_queue_party(
        &self,
        queue_name: String,
        party_id: Uuid,
        player_ids: Vec<Uuid>,
        average_rating: Rating,
        metadata: EntryMetadata,
    ) -> Result<QueueEntry> {
        let entry = QueueEntry::new_party(queue_name.clone(), party_id, player_ids, average_rating, metadata);

        self.add_entry(entry.clone()).await?;
        self.persistence.save_queue_entry(&entry).await?;

        Ok(entry)
    }

    async fn add_entry(&self, entry: QueueEntry) -> Result<()> {
        let mut queues = self.queues.write().await;
        let queue = queues
            .get_mut(&entry.queue_name)
            .ok_or_else(|| MatchForgeError::QueueNotFounQContinued(entry.queue_name.clone()))?;
    // Check if player already in queue
    for existing in queue.iter() {
        for player_id in &entry.player_ids {
            if existing.player_ids.contains(player_id) {
                return Err(MatchForgeError::AlreadyInQueue(*player_id));
            }
        }
    }

    queue.push(entry);
    Ok(())
}

/// Remove a player from a queue
pub async fn leave_queue(&self, queue_name: &str, player_id: Uuid) -> Result<()> {
    let mut queues = self.queues.write().await;
    let queue = queues
        .get_mut(queue_name)
        .ok_or_else(|| MatchForgeError::QueueNotFound(queue_name.to_string()))?;

    let original_len = queue.len();
    queue.retain(|entry| !entry.player_ids.contains(&player_id));

    if queue.len() == original_len {
        return Err(MatchForgeError::NotInQueue(player_id));
    }

    self.persistence.delete_queue_entry(player_id).await?;

    Ok(())
}

/// Attempt to find matches in a queue
pub async fn find_matches(&self, queue_name: &str) -> Result<Vec<MatchResult>> {
    let configs = self.configs.read().await;
    let config = configs
        .get(queue_name)
        .ok_or_else(|| MatchForgeError::QueueNotFound(queue_name.to_string()))?;

    let queues = self.queues.read().await;
    let entries = queues
        .get(queue_name)
        .ok_or_else(|| MatchForgeError::QueueNotFound(queue_name.to_string()))?;

    let matcher = GreedyMatcher::new(config.format.clone(), config.constraints.clone());

    let mut matches = Vec::new();
    let mut remaining_entries = entries.clone();

    // Keep finding matches until we can't anymore
    while let Some(match_result) = matcher.find_match(&remaining_entries) {
        // Remove matched entries
        let matched_player_ids: Vec<Uuid> = match_result
            .entries
            .iter()
            .flat_map(|e| e.player_ids.clone())
            .collect();

        remaining_entries.retain(|e| {
            !e.player_ids.iter().any(|id| matched_player_ids.contains(id))
        });

        matches.push(match_result);
    }

    Ok(matches)
}

/// Remove matched entries from queue
pub async fn remove_matched_entries(&self, queue_name: &str, entries: &[QueueEntry]) -> Result<()> {
    let mut queues = self.queues.write().await;
    let queue = queues
        .get_mut(queue_name)
        .ok_or_else(|| MatchForgeError::QueueNotFound(queue_name.to_string()))?;

    let entry_ids: Vec<Uuid> = entries.iter().map(|e| e.id).collect();
    queue.retain(|e| !entry_ids.contains(&e.id));

    // Clean up persistence
    for entry in entries {
        for player_id in &entry.player_ids {
            let _ = self.persistence.delete_queue_entry(*player_id).await;
        }
    }

    Ok(())
}

/// Get current queue status
pub async fn get_queue_size(&self, queue_name: &str) -> Result<usize> {
    let queues = self.queues.read().await;
    Ok(queues.get(queue_name).map(|q| q.len()).unwrap_or(0))
}
}

### 3.12 src/queue/mod.rs
```rust
pub mod constraints;
pub mod entry;
pub mod manager;
pub mod matcher;

pub use constraints::{MatchConstraints, RoleRequirement};
pub use entry::{EntryMetadata, QueueEntry};
pub use manager::{QueueConfig, QueueManager};
pub use matcher::{GreedyMatcher, MatchFormat, MatchResult};
```

### 3.13 src/lobby/state.rs
```rust
use serde::{Deserialize, Serialize};

/// Lobby lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyState {
    /// Players are being added to the lobby
    Forming,
    /// All players present, waiting for ready confirmations
    WaitingForReady,
    /// All players ready, lobby can be dispatched to game server
    Ready,
    /// Match has been dispatched to game server
    Dispatched,
    /// Lobby closed (match completed or cancelled)
    Closed,
}

impl LobbyState {
    pub fn can_transition_to(&self, new_state: LobbyState) -> bool {
        use LobbyState::*;
        matches!(
            (self, new_state),
            (Forming, WaitingForReady)
                | (WaitingForReady, Ready)
                | (Ready, Dispatched)
                | (Dispatched, Closed)
                | (_, Closed) // Can always close
        )
    }
}
```

### 3.14 src/lobby/team.rs
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a team in a lobby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub team_id: usize,
    pub player_ids: Vec<Uuid>,
}

impl Team {
    pub fn new(team_id: usize) -> Self {
        Self {
            team_id,
            player_ids: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) {
        self.player_ids.push(player_id);
    }

    pub fn size(&self) -> usize {
        self.player_ids.len()
    }
}

/// Strategy for assigning players to teams
pub trait TeamAssignmentStrategy: Send + Sync {
    /// Assign players to teams based on some criteria
    fn assign_teams(&self, player_ids: Vec<Uuid>, team_sizes: &[usize]) -> Vec<Team>;
}

/// Simple sequential assignment
pub struct SequentialAssignment;

impl TeamAssignmentStrategy for SequentialAssignment {
    fn assign_teams(&self, player_ids: Vec<Uuid>, team_sizes: &[usize]) -> Vec<Team> {
        let mut teams: Vec<Team> = team_sizes
            .iter()
            .enumerate()
            .map(|(i, _)| Team::new(i))
            .collect();

        let mut player_index = 0;
        for (team_index, &size) in team_sizes.iter().enumerate() {
            for _ in 0..size {
                if player_index < player_ids.len() {
                    teams[team_index].add_player(player_ids[player_index]);
                    player_index += 1;
                }
            }
        }

        teams
    }
}
```

### 3.15 src/lobby/lobby.rs
```rust
use super::{
    state::LobbyState,
    team::{SequentialAssignment, Team, TeamAssignmentStrategy},
};
use crate::{error::*, queue::MatchResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

/// A lobby represents a matched set of players ready to play together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: Uuid,
    pub match_id: Uuid,
    pub state: LobbyState,
    pub teams: Vec<Team>,
    pub player_ids: Vec<Uuid>,
    pub ready_players: HashSet<Uuid>,
    pub created_at: DateTime<Utc>,
    pub metadata: LobbyMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LobbyMetadata {
    pub queue_name: String,
    pub game_mode: Option<String>,
    pub map: Option<String>,
    pub server_id: Option<String>,
    pub custom: std::collections::HashMap<String, String>,
}

impl Lobby {
    pub fn from_match_result(
        match_result: MatchResult,
        team_sizes: Vec<usize>,
        metadata: LobbyMetadata,
    ) -> Self {
        let player_ids: Vec<Uuid> = match_result
            .entries
            .iter()
            .flat_map(|e| e.player_ids.clone())
            .collect();

        let strategy = SequentialAssignment;
        let teams = strategy.assign_teams(player_ids.clone(), &team_sizes);

        Self {
            id: Uuid::new_v4(),
            match_id: match_result.match_id,
            state: LobbyState::Forming,
            teams,
            player_ids,
            ready_players: HashSet::new(),
            created_at: Utc::now(),
            metadata,
        }
    }

    pub fn from_match_result_custom(
        match_result: MatchResult,
        team_sizes: Vec<usize>,
        metadata: LobbyMetadata,
        strategy: Arc<dyn TeamAssignmentStrategy>,
    ) -> Self {
        let player_ids: Vec<Uuid> = match_result
            .entries
            .iter()
            .flat_map(|e| e.player_ids.clone())
            .collect();

        let teams = strategy.assign_teams(player_ids.clone(), &team_sizes);

        Self {
            id: Uuid::new_v4(),
            match_id: match_result.match_id,
            state: LobbyState::Forming,
            teams,
            player_ids,
            ready_players: HashSet::new(),
            created_at: Utc::now(),
            metadata,
        }
    }

    /// Transition to a new state
    pub fn transition_to(&mut self, new_state: LobbyState) -> Result<()> {
        if !self.state.can_transition_to(new_state) {
            return Err(MatchForgeError::OperationFailed(format!(
                "Cannot transition from {:?} to {:?}",
                self.state, new_state
            )));
        }
        self.state = new_state;
        Ok(())
    }

    /// Mark a player as ready
    pub fn mark_player_ready(&mut self, player_id: Uuid) -> Result<()> {
        if !self.player_ids.contains(&player_id) {
            return Err(MatchForgeError::PlayerNotFound(player_id));
        }

        self.ready_players.insert(player_id);

        // Auto-transition if all players ready
        if self.ready_players.len() == self.player_ids.len()
            && self.state == LobbyState::WaitingForReady
        {
            self.transition_to(LobbyState::Ready)?;
        }

        Ok(())
    }

    /// Check if all players are ready
    pub fn all_players_ready(&self) -> bool {
        self.ready_players.len() == self.player_ids.len()
    }

    /// Get team for a specific player
    pub fn get_player_team(&self, player_id: Uuid) -> Option<usize> {
        self.teams
            .iter()
            .find(|t| t.player_ids.contains(&player_id))
            .map(|t| t.team_id)
    }
}
```

### 3.16 src/lobby/mod.rs
```rust
pub mod lobby;
pub mod state;
pub mod team;

pub use lobby::{Lobby, LobbyMetadata};
pub use state::LobbyState;
pub use team::{SequentialAssignment, Team, TeamAssignmentStrategy};
```

### 3.17 src/party/party.rs
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A party of players queuing together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: Uuid,
    pub leader_id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub max_size: usize,
    pub created_at: DateTime<Utc>,
}

impl Party {
    pub fn new(leader_id: Uuid, max_size: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            leader_id,
            member_ids: vec![leader_id],
            max_size,
            created_at: Utc::now(),
        }
    }

    pub fn size(&self) -> usize {
        self.member_ids.len()
    }

    pub fn is_full(&self) -> bool {
        self.size() >= self.max_size
    }

    pub fn has_member(&self, player_id: Uuid) -> bool {
        self.member_ids.contains(&player_id)
    }

    pub fn is_leader(&self, player_id: Uuid) -> bool {
        self.leader_id == player_id
    }
}
```

### 3.18 src/party/mmr_strategy.rs
```rust
use crate::mmr::Rating;
use uuid::Uuid;

/// Strategy for calculating party MMR from individual ratings
pub trait PartyMmrStrategy: Send + Sync {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating;
}

/// Use the average MMR
pub struct AverageStrategy;

impl PartyMmrStrategy for AverageStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        if ratings.is_empty() {
            return Rating::default();
        }

        let sum: f64 = ratings.iter().map(|(_, r)| r.rating).sum();
        let avg_rating = sum / ratings.len() as f64;

        let avg_deviation: f64 = ratings.iter().map(|(_, r)| r.deviation).sum::<f64>()
            / ratings.len() as f64;

        Rating {
            rating: avg_rating,
            deviation: avg_deviation,
            volatility: 0.06,
        }
    }
}

/// Use the highest MMR (conservative for opponents)
pub struct MaxStrategy;

impl PartyMmrStrategy for MaxStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        ratings
            .iter()
            .max_by(|a, b| a.1.rating.partial_cmp(&b.1.rating).unwrap())
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }
}

/// Weighted average with penalty for skill gaps
pub struct WeightedWithPenaltyStrategy {
    pub gap_penalty: f64,
}

impl PartyMmrStrategy for WeightedWithPenaltyStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        if ratings.is_empty() {
            return Rating::default();
        }

        let ratings_only: Vec<f64> = ratings.iter().map(|(_, r)| r.rating).collect();
        let max_rating = ratings_only.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_rating = ratings_only.iter().copied().fold(f64::INFINITY, f64::min);
        let gap = max_rating - min_rating;

        let avg_rating: f64 = ratings_only.iter().sum::<f64>() / ratings.len() as f64;
        let penalty = gap * self.gap_penalty;

        Rating {
            rating: avg_rating + penalty,
            deviation: 200.0,
            volatility: 0.06,
        }
    }
}
```

### 3.19 src/party/manager.rs
```rust
use super::{mmr_strategy::PartyMmrStrategy, party::Party};
use crate::{error::*, mmr::Rating, persistence::PersistenceAdapter};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct PartyManager {
    parties: Arc<RwLock<HashMap<Uuid, Party>>>,
    player_to_party: Arc<RwLock<HashMap<Uuid, Uuid>>>,
    persistence: Arc<dyn PersistenceAdapter>,
    mmr_strategy: Arc<dyn PartyMmrStrategy>,
}

impl PartyManager {
    pub fn new(
        persistence: Arc<dyn PersistenceAdapter>,
        mmr_strategy: Arc<dyn PartyMmrStrategy>,
    ) -> Self {
        Self {
            parties: Arc::new(RwLock::new(HashMap::new())),
            player_to_party: Arc::new(RwLock::new(HashMap::new())),
            persistence,
            mmr_strategy,
        }
    }

    /// Create a new party
    pub async fn create_party(&self, leader_id: Uuid, max_size: usize) -> Result<Party> {
        let party = Party::new(leader_id, max_size);

        let mut parties = self.parties.write().await;
        let mut player_map = self.player_to_party.write().await;

        parties.insert(party.id, party.clone());
        player_map.insert(leader_id, party.id);

        self.persistence.save_party(&party).await?;

        Ok(party)
    }

    /// Add a member to a party
    pub async fn add_member(&self, party_id: Uuid, player_id: Uuid) -> Result<()> {
        let mut parties = self.parties.write().await;
        let mut player_map = self.player_to_party.write().await;

        let party = parties
            .get_mut(&party_id)
            .ok_or(MatchForgeError::PartyNotFound(party_id))?;

        if party.is_full() {
            return Err(MatchForgeError::PartyFull(party.max_size));
        }

        if party.has_member(player_id) {
            return Err(MatchForgeError::InvalidPartyOperation(
                "Player already in party".to_string(),
            ));
        }

        party.member_ids.push(player_id);
        player_map.insert(player_id, party_id);

        self.persistence.save_party(party).await?;

        Ok(())
    }

    /// Remove a member from a party
    pub async fn remove_member(&self, party_id: Uuid, player_id: Uuid) -> Result<()> {
        let mut parties = self.parties.write().await;
        let mut player_map = self.player_to_party.write().await;

        let party = parties
            .get_mut(&party_id)
            .ok_or(MatchForgeError::PartyNotFound(party_id))?;

        if !party.has_member(player_id) {
            return Err(MatchForgeError::InvalidPartyOperation(
                "Player not in party".to_string(),
            ));
        }

        party.member_ids.retain(|id| *id != player_id);
        player_map.remove(&player_id);

        // Disband if empty or leader left
        if party.member_ids.is_empty() || player_id == party.leader_id {
            parties.remove(&party_id);
            self.persistence.delete_party(party_id).await?;
        } else {
            self.persistence.save_party(party).await?;
        }

        Ok(())
    }

    /// Calculate party MMR
    pub async fn calculate_party_rating(&self, party_id: Uuid) -> Result<Rating> {
        let parties = self.parties.read().await;
        let party = parties
            .get(&party_id)
            .ok_or(MatchForgeError::PartyNotFound(party_id))?;

        // Fetch ratings for all members
        let mut ratings = Vec::new();
        for &player_id in &party.member_ids {
            if let Ok(Some(rating)) = self.persistence.load_player_rating(player_id).await {
                ratings.push((player_id, rating));
            }
        }

        Ok(self.mmr_strategy.calculate_party_rating(&ratings))
    }

    /// Get party for a player
    pub async fn get_player_party(&self, player_id: Uuid) -> Option<Party> {
        let player_map = self.player_to_party.read().await;
        let parties = self.parties.read().await;

        player_map
            .get(&player_id)
            .and_then(|party_id| parties.get(party_id).cloned())
    }
}
```

### 3.20 src/party/mod.rs
```rust
pub mod manager;
pub mod mmr_strategy;
pub mod party;

pub use manager::PartyManager;
pub use mmr_strategy::{AverageStrategy, MaxStrategy, PartyMmrStrategy, WeightedWithPenaltyStrategy};
pub use party::Party;
```

### 3.21 src/persistence/traits.rs
```rust
use crate::{
    error::Result,
    lobby::Lobby,
    mmr::Rating,
    party::Party,
    queue::QueueEntry,
};
use async_trait::async_trait;
use uuid::Uuid;

/// Main persistence abstraction
#[async_trait]
pub trait PersistenceAdapter: Send + Sync {
    // Player ratings
    async fn save_player_rating(&self, player_id: Uuid, rating: Rating) -> Result<()>;
    async fn load_player_rating(&self, player_id: Uuid) -> Result<Option<Rating>>;

    // Queue entries
    async fn save_queue_entry(&self, entry: &QueueEntry) -> Result<()>;
    async fn load_queue_entries(&self, queue_name: &str) -> Result<Vec<QueueEntry>>;
    async fn delete_queue_entry(&self, player_id: Uuid) -> Result<()>;

    // Parties
    async fn save_party(&self, party: &Party) -> Result<()>;
    async fn load_party(&self, party_id: Uuid) -> Result<Option<Party>>;
    async fn delete_party(&self, party_id: Uuid) -> Result<()>;

    // Lobbies
    async fn save_lobby(&self, lobby: &Lobby) -> Result<()>;
    async fn load_lobby(&self, lobby_id: Uuid) -> Result<Option<Lobby>>;
    async fn delete_lobby(&self, lobby_id: Uuid) -> Result<()>;

    // Match history (optional, for statistics)
    async fn save_match_result(&self, lobby: &Lobby) -> Result<()>;
}
```

### 3.22 src/persistence/memory.rs
```rust
use super::traits::PersistenceAdapter;
use crate::{
    error::Result,
    lobby::Lobby,
    mmr::Rating,
    party::Party,
    queue::QueueEntry,
};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory persistence adapter (for development/testing)
pub struct InMemoryAdapter {
    player_ratings: Arc<RwLock<HashMap<Uuid, Rating>>>,
    queue_entries: Arc<RwLock<HashMap<String, Vec<QueueEntry>>>>,
    parties: Arc<RwLock<HashMap<Uuid, Party>>>,
    lobbies: Arc<RwLock<HashMap<Uuid, Lobby>>>,
    match_history: Arc<RwLock<Vec<Lobby>>>,
}

impl InMemoryAdapter {
    pub fn new() -> Self {
        Self {
            player_ratings: Arc::new(RwLock::new(HashMap::new())),
            queue_entries: Arc::new(RwLock::new(HashMap::new())),
            parties: Arc::new(RwLock::new(HashMap::new())),
            lobbies: Arc::new(RwLock::new(HashMap::new())),
            match_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PersistenceAdapter for InMemoryAdapter {
    async fn save_player_rating(&self, player_id: Uuid, rating: Rating) -> Result<()> {
        let mut ratings = self.player_ratings.write().await;
        ratings.insert(player_id, rating);
        Ok(())
    }

    async fn load_player_rating(&self, player_id: Uuid) -> Result<Option<Rating>> {
        let ratings = self.player_ratings.read().await;
        Ok(ratings.get(&player_id).copied())
    }

    async fn save_queue_entry(&self, entry: &QueueEntry) -> Result<()> {
        let mut entries = self.queue_entries.write().await;
        entries
            .entry(entry.queue_name.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
        Ok(())
    }

    async fn load_queue_entries(&self, queue_name: &str) -> Result<Vec<QueueEntry>> {
        let entries = self.queue_entries.read().await;
        Ok(entries.get(queue_name).cloned().unwrap_or_default())
    }

    async fn delete_queue_entry(&self, player_id: Uuid) -> Result<()> {
        let mut entries = self.queue_entries.write().await;
        for queue_entries in entries.values_mut() {
            queue_entries.retain(|e| !e.player_ids.contains(&player_id));
        }
        Ok(())
    }

    async fn save_party(&self, party: &Party) -> Result<()> {
        let mut parties = self.parties.write().await;
        parties.insert(party.id, party.clone());
        Ok(())
    }

    async fn load_party(&self, party_id: Uuid) -> Result<Option<Party>> {
        let parties = self.parties.read().await;
        Ok(parties.get(&party_id).cloned())
    }

    async fn delete_party(&self, party_id: Uuid) -> Result<()> {
        let mut parties = self.parties.write().await;
        parties.remove(&party_id);
        Ok(())
    }

    async fn save_lobby(&self, lobby: &Lobby) -> Result<()> {
        let mut lobbies = self.lobbies.write().await;
        lobbies.insert(lobby.id, lobby.clone());
        Ok(())
    }

    async fn load_lobby(&self, lobby_id: Uuid) -> Result<Option<Lobby>> {
        let lobbies = self.lobbies.read().await;
        Ok(lobbies.get(&lobby_id).cloned())
    }

    async fn delete_lobby(&self, lobby_id: Uuid) -> Result<()> {
        let mut lobbies = self.lobbies.write().await;
        lobbies.remove(&lobby_id);
        Ok(())
    }

    async fn save_match_result(&self, lobby: &Lobby) -> Result<()> {
        let mut history = self.match_history.write().await;
        history.push(lobby.clone());
        Ok(())
    }
}
```

### 3.23 src/persistence/redis.rs
```rust
use super::traits::PersistenceAdapter;
use crate::{error::*, lobby::Lobby, mmr::Rating, party::Party, queue::QueueEntry};
use async_trait::async_trait;
use uuid::Uuid;

/// Redis persistence adapter (skeleton implementation)
/// 
/// To use this, enable the "redis" feature and implement the actual Redis logic.
pub struct RedisAdapter {
    // redis_client: redis::Client,
}

impl RedisAdapter {
    pub fn new(_connection_string: &str) -> Result<Self> {
        // Example skeleton:
        // let client = redis::Client::open(connection_string)
        //     .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        // Ok(Self { redis_client: client })
        
        Err(MatchForgeError::InvalidConfiguration(
            "Redis adapter not implemented yet. Enable 'redis' feature and implement.".to_string()
        ))
    }
}

#[async_trait]
impl PersistenceAdapter for RedisAdapter {
    async fn save_player_rating(&self, _player_id: Uuid, _rating: Rating) -> Result<()> {
        // Implementation: SET player:{uuid}:rating {json_rating}
        unimplemented!("Redis persistence not implemented")
    }

    async fn load_player_rating(&self, _player_id: Uuid) -> Result<Option<Rating>> {
        // Implementation: GET player:{uuid}:rating
        unimplemented!("Redis persistence not implemented")
    }

    async fn save_queue_entry(&self, _entry: &QueueEntry) -> Result<()> {
        // Implementation: ZADD queue:{name} {timestamp} {json_entry}
        unimplemented!("Redis persistence not implemented")
    }

    async fn load_queue_entries(&self, _queue_name: &str) -> Result<Vec<QueueEntry>> {
        // Implementation: ZRANGE queue:{name} 0 -1
        unimplemented!("Redis persistence not implemented")
    }

    async fn delete_queue_entry(&self, _player_id: Uuid) -> Result<()> {
        // Implementation: ZREM queue entries by player_id
        unimplemented!("Redis persistence not implemented")
    }

    async fn save_party(&self, _party: &Party) -> Result<()> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn load_party(&self, _party_id: Uuid) -> Result<Option<Party>> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn delete_party(&self, _party_id: Uuid) -> Result<()> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn save_lobby(&self, _lobby: &Lobby) -> Result<()> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn load_lobby(&self, _lobby_id: Uuid) -> Result<Option<Lobby>> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn delete_lobby(&self, _lobby_id: Uuid) -> Result<()> {
        unimplemented!("Redis persistence not implemented")
    }

    async fn save_match_result(&self, _lobby: &Lobby) -> Result<()> {
        unimplemented!("Redis persistence not implemented")
    }
}
```

### 3.24 src/persistence/postgres.rs
```rust
use super::traits::PersistenceAdapter;
use crate::{error::*, lobby::Lobby, mmr::Rating, party::Party, queue::QueueEntry};
use async_trait::async_trait;
use uuid::Uuid;

/// Postgres persistence adapter (skeleton implementation)
/// 
/// To use this, enable the "postgres" feature and implement with sqlx.
pub struct PostgresAdapter {
    // pool: sqlx::PgPool,
}

impl PostgresAdapter {
    pub async fn new(_connection_string: &str) -> Result<Self> {
        // Example skeleton:
        // let pool = sqlx::PgPool::connect(connection_string).await
        //     .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        // Ok(Self { pool })
        
        Err(MatchForgeError::InvalidConfiguration(
            "Postgres adapter not implemented yet. Enable 'postgres' feature and implement.".to_string()
        ))
    }
}

#[async_trait]
impl PersistenceAdapter for PostgresAdapter {
    async fn save_player_rating(&self, _player_id: Uuid, _rating: Rating) -> Result<()> {
        // Implementation: INSERT/UPDATE INTO player_ratings table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn load_player_rating(&self, _player_id: Uuid) -> Result<Option<Rating>> {
        // Implementation: SELECT FROM player_ratings table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn save_queue_entry(&self, _entry: &QueueEntry) -> Result<()> {
        // Implementation: INSERT INTO queue_entries table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn load_queue_entries(&self, _queue_name: &str) -> Result<Vec<QueueEntry>> {
        // Implementation: SELECT FROM queue_entries WHERE queue_name = ?
        unimplemented!("Postgres persistence not implemented")
    }

    async fn delete_queue_entry(&self, _player_id: Uuid) -> Result<()> {
        // Implementation: DELETE FROM queue_entries WHERE player_ids @> ?
        unimplemented!("Postgres persistence not implemented")
    }

    async fn save_party(&self, _party: &Party) -> Result<()> {
        // Implementation: INSERT/UPDATE INTO parties table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn load_party(&self, _party_id: Uuid) -> Result<Option<Party>> {
        // Implementation: SELECT FROM parties table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn delete_party(&self, _party_id: Uuid) -> Result<()> {
        // Implementation: DELETE FROM parties table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn save_lobby(&self, _lobby: &Lobby) -> Result<()> {
        // Implementation: INSERT/UPDATE INTO lobbies table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn load_lobby(&self, _lobby_id: Uuid) -> Result<Option<Lobby>> {
        // Implementation: SELECT FROM lobbies table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn delete_lobby(&self, _lobby_id: Uuid) -> Result<()> {
        // Implementation: DELETE FROM lobbies table
        unimplemented!("Postgres persistence not implemented")
    }

    async fn save_match_result(&self, _lobby: &Lobby) -> Result<()> {
        // Implementation: INSERT INTO match_history table
        unimplemented!("Postgres persistence not implemented")
    }
}
```

### 3.25 src/persistence/mod.rs
```rust
pub mod memory;
pub mod postgres;
pub mod redis;
pub mod traits;

#[cfg(feature = "redis")]
pub use redis::RedisAdapter;

#[cfg(feature = "postgres")]
pub use postgres::PostgresAdapter;

pub use memory::InMemoryAdapter;
pub use traits::PersistenceAdapter;
```

### 3.26 src/runner/config.rs
```rust
use serde::{Deserialize, Serialize};

/// Configuration for the matchmaking runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// How often to run matchmaking ticks (in milliseconds)
    pub tick_interval_ms: u64,
    /// Maximum number of matches to process per tick
    pub max_matches_per_tick: usize,
    /// Whether to automatically dispatch ready lobbies
    pub auto_dispatch: bool,
    /// Queue-specific configurations
    pub queue_configs: std::collections::HashMap<String, QueueRunnerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueRunnerConfig {
    /// Enable this queue for automatic processing
    pub enabled: bool,
    /// Priority for processing this queue (lower = higher priority)
    pub priority: u8,
    /// Maximum concurrent matches for this queue
    pub max_concurrent_matches: usize,
}

impl RunnerConfig {
    pub fn default() -> Self {
        let mut queue_configs = std::collections::HashMap::new();
        
        // Default configuration for common queues
        queue_configs.insert("ranked_1v1".to_string(), QueueRunnerConfig {
            enabled: true,
            priority: 1,
            max_concurrent_matches: 100,
        });
        
        queue_configs.insert("casual_5v5".to_string(), QueueRunnerConfig {
            enabled: true,
            priority: 2,
            max_concurrent_matches: 50,
        });

        Self {
            tick_interval_ms: 1000, // 1 second
            max_matches_per_tick: 1000,
            auto_dispatch: true,
            queue_configs,
        }
    }

    pub fn fast() -> Self {
        let mut config = Self::default();
        config.tick_interval_ms = 500; // 0.5 seconds
        config
    }

    pub fn slow() -> Self {
        let mut config = Self::default();
        config.tick_interval_ms = 5000; // 5 seconds
        config
    }
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self::default()
    }
}
```

### 3.27 src/runner/tick.rs
```rust
use super::config::RunnerConfig;
use crate::{
    error::*,
    lobby::{Lobby, LobbyMetadata, LobbyState},
    mmr::Rating,
    persistence::PersistenceAdapter,
    queue::{QueueManager, QueueConfig},
};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use uuid::Uuid;

/// The main matchmaking runner that processes queues periodically
pub struct MatchmakingRunner {
    config: RunnerConfig,
    queue_manager: Arc<QueueManager>,
    persistence: Arc<dyn PersistenceAdapter>,
    running: std::sync::atomic::AtomicBool,
}

impl MatchmakingRunner {
    pub fn new(
        config: RunnerConfig,
        queue_manager: Arc<QueueManager>,
        persistence: Arc<dyn PersistenceAdapter>,
    ) -> Self {
        Self {
            config,
            queue_manager,
            persistence,
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Start the matchmaking runner
    pub async fn start(&self) -> Result<()> {
        if self.running.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return Err(MatchForgeError::OperationFailed(
                "Runner is already running".to_string(),
            ));
        }

        let mut interval = interval(Duration::from_millis(self.config.tick_interval_ms));
        
        loop {
            interval.tick().await;
            
            if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            if let Err(e) = self.process_tick().await {
                eprintln!("Matchmaking tick error: {}", e);
            }
        }

        Ok(())
    }

    /// Stop the matchmaking runner
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Process a single matchmaking tick
    async fn process_tick(&self) -> Result<()> {
        let mut total_matches = 0;

        // Process queues in priority order
        let mut queue_names: Vec<String> = self.config.queue_configs
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.clone())
            .collect();

        queue_names.sort_by(|a, b| {
            let priority_a = self.config.queue_configs.get(a).map(|c| c.priority).unwrap_or(255);
            let priority_b = self.config.queue_configs.get(b).map(|c| c.priority).unwrap_or(255);
            priority_a.cmp(&priority_b)
        });

        for queue_name in queue_names {
            if total_matches >= self.config.max_matches_per_tick {
                break;
            }

            let queue_config = self.config.queue_configs.get(&queue_name);
            let max_for_queue = queue_config.map(|c| c.max_concurrent_matches).unwrap_or(100);
            let remaining = self.config.max_matches_per_tick - total_matches;
            let to_process = remaining.min(max_for_queue);

            match self.process_queue(&queue_name, to_process).await {
                Ok(matches_found) => {
                    total_matches += matches_found;
                    if matches_found > 0 {
                        println!("Found {} matches in queue '{}'", matches_found, queue_name);
                    }
                }
                Err(e) => {
                    eprintln!("Error processing queue '{}': {}", queue_name, e);
                }
            }
        }

        Ok(())
    }

    /// Process a single queue
    async fn process_queue(&self, queue_name: &str, max_matches: usize) -> Result<usize> {
        let matches = self.queue_manager.find_matches(queue_name).await?;
        
        let mut processed = 0;
        for match_result in matches.into_iter().take(max_matches) {
            // Create lobby from match result
            let metadata = LobbyMetadata {
                queue_name: queue_name.to_string(),
                game_mode: Some(queue_name.to_string()),
                ..Default::default()
            };

            let mut lobby = Lobby::from_match_result(match_result.clone(), vec![1, 1], metadata);
            
            // Save lobby
            self.persistence.save_lobby(&lobby).await?;
            
            // Remove matched entries from queue
            self.queue_manager.remove_matched_entries(queue_name, &match_result.entries).await?;
            
            // Auto-dispatch if enabled
            if self.config.auto_dispatch {
                lobby.transition_to(LobbyState::Dispatched)?;
                self.persistence.save_lobby(&lobby).await?;
            }

            processed += 1;
        }

        Ok(processed)
    }

    /// Check if runner is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Lobby manager for handling lobby lifecycle
pub struct LobbyManager {
    persistence: Arc<dyn PersistenceAdapter>,
}

impl LobbyManager {
    pub fn new(persistence: Arc<dyn PersistenceAdapter>) -> Self {
        Self { persistence }
    }

    /// Get a lobby by ID
    pub async fn get_lobby(&self, lobby_id: Uuid) -> Result<Option<Lobby>> {
        self.persistence.load_lobby(lobby_id).await
    }

    /// Mark player as ready in lobby
    pub async fn mark_player_ready(&self, lobby_id: Uuid, player_id: Uuid) -> Result<()> {
        let mut lobby = self.persistence.load_lobby(lobby_id).await?
            .ok_or(MatchForgeError::LobbyNotFound(lobby_id))?;

        lobby.mark_player_ready(player_id)?;
        self.persistence.save_lobby(&lobby).await?;

        Ok(())
    }

    /// Dispatch lobby to game server
    pub async fn dispatch_lobby(&self, lobby_id: Uuid, server_id: String) -> Result<()> {
        let mut lobby = self.persistence.load_lobby(lobby_id).await?
            .ok_or(MatchForgeError::LobbyNotFound(lobby_id))?;

        lobby.metadata.server_id = Some(server_id);
        lobby.transition_to(LobbyState::Dispatched)?;
        
        self.persistence.save_lobby(&lobby).await?;

        Ok(())
    }

    /// Close lobby (match completed or cancelled)
    pub async fn close_lobby(&self, lobby_id: Uuid) -> Result<()> {
        let mut lobby = self.persistence.load_lobby(lobby_id).await?
            .ok_or(MatchForgeError::LobbyNotFound(lobby_id))?;

        lobby.transition_to(LobbyState::Closed)?;
        
        // Save match result to history
        self.persistence.save_match_result(&lobby).await?;
        
        // Clean up lobby
        self.persistence.delete_lobby(lobby_id).await?;

        Ok(())
    }

    /// Update player ratings after match completion
    pub async fn update_ratings(
        &self,
        lobby_id: Uuid,
        outcomes: &[(Uuid, crate::mmr::Outcome)],
        mmr_algorithm: Arc<dyn crate::mmr::MmrAlgorithm>,
    ) -> Result<()> {
        let lobby = self.persistence.load_lobby(lobby_id).await?
            .ok_or(MatchForgeError::LobbyNotFound(lobby_id))?;

        // Group players by teams
        let mut team_ratings: std::collections::HashMap<usize, Vec<(Uuid, Rating)>> = std::collections::HashMap::new();
        
        for (player_id, _) in outcomes {
            if let Some(team_id) = lobby.get_player_team(*player_id) {
                if let Ok(Some(rating)) = self.persistence.load_player_rating(*player_id).await {
                    team_ratings.entry(team_id).or_insert_with(Vec::new).push((*player_id, rating));
                }
            }
        }

        // Update ratings based on team vs team outcomes
        for (team_a_id, team_a_players) in &team_ratings {
            for (team_b_id, team_b_players) in &team_ratings {
                if team_a_id >= team_b_id {
                    continue; // Skip duplicate matchups and same team
                }

                // Determine team outcomes
                let team_a_outcome = self.determine_team_outcome(outcomes, team_a_players);
                let team_b_outcome = self.determine_team_outcome(outcomes, team_b_players);

                // Update ratings for all players in both teams
                for (player_a, rating_a) in team_a_players {
                    for (player_b, rating_b) in team_b_players {
                        let new_rating_a = mmr_algorithm.calculate_new_rating(*rating_a, *rating_b, team_a_outcome);
                        let new_rating_b = mmr_algorithm.calculate_new_rating(*rating_b, *rating_a, team_b_outcome);

                        self.persistence.save_player_rating(*player_a, new_rating_a).await?;
                        self.persistence.save_player_rating(*player_b, new_rating_b).await?;
                    }
                }
            }
        }

        Ok(())
    }

    fn determine_team_outcome(&self, outcomes: &[(Uuid, crate::mmr::Outcome)], team_players: &[(Uuid, Rating)]) -> crate::mmr::Outcome {
        // For simplicity, use the first player's outcome as team outcome
        // In a real implementation, you'd aggregate team performance
        for (player_id, outcome) in outcomes {
            if team_players.iter().any(|(id, _)| id == player_id) {
                return *outcome;
            }
        }
        crate::mmr::Outcome::Loss // Default fallback
    }
}
```

### 3.28 src/runner/mod.rs
```rust
pub mod config;
pub mod tick;

pub use config::{QueueRunnerConfig, RunnerConfig};
pub use tick::{LobbyManager, MatchmakingRunner};
```

### 3.29 src/lib.rs
```rust
//! MatchForge SDK
//! 
//! A comprehensive matchmaking SDK for multiplayer games with MMR systems,
//! queue management, lobby handling, party support, and pluggable persistence.
//! 
//! # Quick Start
//! 
//! ```rust
//! use matchforge::*;
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

pub mod error;
pub mod lobby;
pub mod mmr;
pub mod party;
pub mod persistence;
pub mod queue;
pub mod runner;

// Re-export commonly used types
pub use error::{MatchForgeError, Result};
pub use lobby::{Lobby, LobbyManager, LobbyMetadata, LobbyState};
pub use mmr::{
    AverageStrategy, DecayStrategy, EloAlgorithm, Glicko2Algorithm, LinearDecay, MaxStrategy,
    MmrAlgorithm, NoDecay, Outcome, Rating, Season, SeasonResetStrategy, SoftReset, HardReset,
    WeightedWithPenaltyStrategy,
};
pub use party::{Party, PartyManager, PartyMmrStrategy};
pub use persistence::{InMemoryAdapter, PersistenceAdapter};
pub use queue::{
    EntryMetadata, GreedyMatcher, MatchConstraints, MatchFormat, MatchResult, QueueConfig,
    QueueEntry, QueueManager,
};
pub use runner::{LobbyManager as RunnerLobbyManager, MatchmakingRunner, RunnerConfig};

/// Prelude module for convenient imports
pub mod prelude;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
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
```

### 3.30 src/prelude.rs
```rust
//! Prelude module with commonly used types and traits
//! 
//! Import this module to get all the essential types for using MatchForge:
//! 
//! ```rust
//! use matchforge::prelude::*;
//! ```

pub use crate::{
    error::{MatchForgeError, Result},
    lobby::{Lobby, LobbyManager, LobbyMetadata, LobbyState},
    mmr::{
        AverageStrategy, DecayStrategy, EloAlgorithm, Glicko2Algorithm, LinearDecay, MaxStrategy,
        MmrAlgorithm, NoDecay, Outcome, Rating, Season, SeasonResetStrategy, SoftReset, HardReset,
        WeightedWithPenaltyStrategy,
    },
    party::{Party, PartyManager, PartyMmrStrategy},
    persistence::{InMemoryAdapter, PersistenceAdapter},
    queue::{
        EntryMetadata, GreedyMatcher, MatchConstraints, MatchFormat, MatchResult, QueueConfig,
        QueueEntry, QueueManager,
    },
    runner::{LobbyManager as RunnerLobbyManager, MatchmakingRunner, RunnerConfig},
};

// Re-export common external dependencies
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
pub use uuid::Uuid;
```

## 4. Examples

### 4.1 examples/basic.rs
```rust
//! Basic 1v1 matchmaking example
//! 
//! This example shows how to set up a simple matchmaking system for 1v1 matches
//! using the in-memory persistence adapter.

use matchforge::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));
    
    // Configure 1v1 ranked queue
    let queue_config = QueueConfig {
        name: "ranked_1v1".to_string(),
        format: MatchFormat::one_v_one(),
        constraints: MatchConstraints {
            max_rating_delta: 200.0,
            same_region_required: false,
            role_requirements: vec![],
            max_wait_time_seconds: 300,
            expansion_rate: 5.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create MMR algorithm
    let mmr_algorithm = Arc::new(EloAlgorithm::default());
    
    // Add some players to the queue
    let players = vec![
        (Uuid::new_v4(), Rating::new(1500.0, 350.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1600.0, 300.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1400.0, 320.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1550.0, 280.0, 0.06)),
    ];
    
    println!("Adding players to queue...");
    for (player_id, rating) in &players {
        let entry = queue_manager.join_queue_solo(
            "ranked_1v1".to_string(),
            *player_id,
            *rating,
            EntryMetadata::default(),
        ).await?;
        
        println!("Player {} joined queue (rating: {:.0})", player_id, rating.rating);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("ranked_1v1").await?;
    
    println!("Found {} matches", matches.len());
    
    for (i, match_result) in matches.iter().enumerate() {
        println!("\nMatch {}:", i + 1);
        println!("  Match ID: {}", match_result.match_id);
        
        for (j, entry) in match_result.entries.iter().enumerate() {
            println!("  Team {}: Player {} (rating: {:.0})", 
                j + 1, entry.player_ids[0], entry.average_rating.rating);
        }
        
        // Create lobby
        let lobby = Lobby::from_match_result(
            match_result.clone(),
            vec![1, 1],
            LobbyMetadata {
                queue_name: "ranked_1v1".to_string(),
                game_mode: Some("ranked".to_string()),
                ..Default::default()
            },
        );
        
        lobby_manager.persistence.save_lobby(&lobby).await?;
        println!("  Lobby created: {}", lobby.id);
    }
    
    // Simulate match completion and update ratings
    if let Some(first_match) = matches.first() {
        println!("\nUpdating ratings after match completion...");
        
        // Assume first player won, second lost
        let outcomes = vec![
            (first_match.entries[0].player_ids[0], Outcome::Win),
            (first_match.entries[1].player_ids[0], Outcome::Loss),
        ];
        
        lobby_manager.update_ratings(first_match.match_id, &outcomes, mmr_algorithm).await?;
        
        // Show updated ratings
        for (player_id, _) in &outcomes {
            if let Ok(Some(new_rating)) = persistence.load_player_rating(*player_id).await {
                println!("Player {} new rating: {:.0}", player_id, new_rating.rating);
            }
        }
    }
    
    println!("\nBasic matchmaking example completed successfully!");
    Ok(())
}
```

### 4.2 examples/party_matchmaking.rs
```rust
//! Party-based matchmaking example
//! 
//! This example demonstrates how to handle parties (groups of players)
//! in the matchmaking system.

use matchforge::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));
    
    // Configure 5v5 queue
    let queue_config = QueueConfig {
        name: "team_5v5".to_string(),
        format: MatchFormat::five_v_five(),
        constraints: MatchConstraints {
            max_rating_delta: 300.0,
            same_region_required: true,
            role_requirements: vec![
                RoleRequirement { role: "tank".to_string(), count: 1 },
                RoleRequirement { role: "healer".to_string(), count: 1 },
                RoleRequirement { role: "dps".to_string(), count: 3 },
            ],
            max_wait_time_seconds: 600,
            expansion_rate: 10.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create some parties
    println!("Creating parties...");
    
    // Party 1: 3 players with similar ratings
    let party1_leader = Uuid::new_v4();
    let party1 = party_manager.create_party(party1_leader, 5).await?;
    println!("Created party {} with leader {}", party1.id, party1_leader);
    
    let party1_members = vec![
        (Uuid::new_v4(), Rating::new(1500.0, 300.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1550.0, 280.0, 0.06)),
    ];
    
    for (member_id, rating) in &party1_members {
        persistence.save_player_rating(*member_id, *rating).await?;
        party_manager.add_member(party1.id, *member_id).await?;
        println!("Added member {} to party {}", member_id, party1.id);
    }
    
    // Party 2: 2 players
    let party2_leader = Uuid::new_v4();
    let party2 = party_manager.create_party(party2_leader, 5).await?;
    println!("Created party {} with leader {}", party2.id, party2_leader);
    
    let party2_member = (Uuid::new_v4(), Rating::new(1480.0, 320.0, 0.06));
    persistence.save_player_rating(party2_member.0, party2_member.1).await?;
    party_manager.add_member(party2.id, party2_member.0).await?;
    println!("Added member {} to party {}", party2_member.0, party2.id);
    
    // Add parties to queue
    println!("\nAdding parties to queue...");
    
    // Calculate party ratings and add to queue
    let party1_rating = party_manager.calculate_party_rating(party1.id).await?;
    let party1_metadata = EntryMetadata {
        roles: vec!["tank".to_string(), "healer".to_string(), "dps".to_string()],
        region: Some("us-east".to_string()),
        ..Default::default()
    };
    
    let party1_entry = queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party1.id,
        party1.member_ids.clone(),
        party1_rating,
        party1_metadata,
    ).await?;
    
    println!("Party {} joined queue (avg rating: {:.0})", party1.id, party1_rating.rating);
    
    let party2_rating = party_manager.calculate_party_rating(party2.id).await?;
    let party2_metadata = EntryMetadata {
        roles: vec!["dps".to_string(), "dps".to_string()],
        region: Some("us-east".to_string()),
        ..Default::default()
    };
    
    let party2_entry = queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party2.id,
        party2.member_ids.clone(),
        party2_rating,
        party2_metadata,
    ).await?;
    
    println!("Party {} joined queue (avg rating: {:.0})", party2.id, party2_rating.rating);
    
    // Add some solo players to fill the match
    let solo_players = vec![
        (Uuid::new_v4(), Rating::new(1520.0, 290.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1490.0, 310.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1510.0, 295.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1470.0, 330.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1530.0, 270.0, 0.06), "dps"),
    ];
    
    println!("\nAdding solo players to queue...");
    for (player_id, rating, role) in &solo_players {
        persistence.save_player_rating(*player_id, *rating).await?;
        
        let metadata = EntryMetadata {
            roles: vec![role.to_string()],
            region: Some("us-east".to_string()),
            ..Default::default()
        };
        
        queue_manager.join_queue_solo(
            "team_5v5".to_string(),
            *player_id,
            *rating,
            metadata,
        ).await?;
        
        println!("Solo player {} joined queue (rating: {:.0}, role: {})", player_id, rating.rating, role);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("team_5v5").await?;
    
    println!("Found {} matches", matches.len());
    
    for (i, match_result) in matches.iter().enumerate() {
        println!("\nMatch {}:", i + 1);
        println!("  Match ID: {}", match_result.match_id);
        
        // Show team composition
        let mut team_players: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
        
        for (entry_idx, entry) in match_result.entries.iter().enumerate() {
            let team_id = match_result.team_assignments[entry_idx];
            let team_players = team_players.entry(team_id).or_insert_with(Vec::new);
            
            if entry.is_solo() {
                team_players.push(format!("Solo {}", entry.player_ids[0]));
            } else {
                team_players.push(format!("Party {}", entry.party_id.unwrap()));
            }
        }
        
        for (team_id, players) in team_players {
            println!("  Team {}: {}", team_id + 1, players.join(", "));
        }
    }
    
    println!("\nParty matchmaking example completed successfully!");
    Ok(())
}
```

### 4.3 examples/custom_mmr.rs
```rust
//! Custom MMR algorithm example
//! 
//! This example shows how to implement a custom MMR algorithm
//! and use it with the matchmaking system.

use matchforge::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Custom MMR algorithm that uses a more complex calculation
pub struct CustomMmrAlgorithm {
    k_factor_base: f64,
    k_factor_new_player: f64,
    new_player_threshold: i32, // Number of matches before considered "experienced"
}

impl CustomMmrAlgorithm {
    pub fn new() -> Self {
        Self {
            k_factor_base: 24.0,
            k_factor_new_player: 40.0,
            new_player_threshold: 30,
        }
    }

    fn calculate_k_factor(&self, player_experience: i32) -> f64 {
        if player_experience < self.new_player_threshold {
            self.k_factor_new_player
        } else {
            self.k_factor_base
        }
    }

    fn expected_score(&self, rating_a: f64, rating_b: f64) -> f64 {
        1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) / 400.0))
    }

    // In a real implementation, you'd track player experience/match count
    fn get_player_experience(&self, _player_id: Uuid) -> i32 {
        // For demo purposes, return a random value
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        _player_id.hash(&mut hasher);
        (hasher.finish() % 100) as i32
    }
}

#[async_trait]
impl MmrAlgorithm for CustomMmrAlgorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        // Get player experience (in real implementation, this would come from persistence)
        let player_experience = self.get_player_experience(Uuid::new_v4()); // Demo only
        let k_factor = self.calculate_k_factor(player_experience);

        let expected = self.expected_score(player_rating.rating, opponent_rating.rating);
        let actual = outcome.score();
        let rating_change = k_factor * (actual - expected);

        // Apply volatility adjustment based on performance
        let performance_factor = if (actual - expected).abs() > 0.3 {
            1.1 // Unexpected result, increase volatility
        } else {
            0.95 // Expected result, decrease volatility
        };

        let new_rating = player_rating.rating + rating_change;
        let new_deviation = (player_rating.deviation * 0.95).max(50.0); // Decrease uncertainty
        let new_volatility = (player_rating.volatility * performance_factor).min(0.2);

        Rating {
            rating: new_rating,
            deviation: new_deviation,
            volatility: new_volatility,
        }
    }

    fn name(&self) -> &str {
        "CustomEnhancedElo"
    }
}

/// Custom decay strategy that considers both time and performance
pub struct AdaptiveDecay {
    base_decay_per_day: f64,
    performance_modifier: f64,
}

impl AdaptiveDecay {
    pub fn new() -> Self {
        Self {
            base_decay_per_day: 2.0,
            performance_modifier: 0.5,
        }
    }
}

impl DecayStrategy for AdaptiveDecay {
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating {
        let now = Utc::now();
        let days_inactive = (now - last_match_time).num_days() as f64;

        if days_inactive <= 0.0 {
            return rating;
        }

        // Base decay
        let base_decay = self.base_decay_per_day * days_inactive;
        
        // Reduce decay for high-performing players (low volatility)
        let performance_bonus = if rating.volatility < 0.05 {
            self.performance_modifier * days_inactive
        } else {
            0.0
        };

        let total_decay = (base_decay - performance_bonus).max(0.0);

        Rating {
            rating: (rating.rating - total_decay).max(0.0),
            deviation: (rating.deviation + days_inactive * 0.8).min(350.0),
            volatility: (rating.volatility * 1.1).min(0.2),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components with custom algorithms
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    
    // Use custom MMR algorithm
    let custom_algorithm = Arc::new(CustomMmrAlgorithm::new());
    println!("Using custom MMR algorithm: {}", custom_algorithm.name());
    
    // Use custom decay strategy
    let decay_strategy = AdaptiveDecay::new();
    println!("Using adaptive decay strategy");
    
    // Configure queue with custom constraints
    let queue_config = QueueConfig {
        name: "custom_ranked".to_string(),
        format: MatchFormat::two_v_two(),
        constraints: MatchConstraints {
            max_rating_delta: 150.0,
            same_region_required: false,
            role_requirements: vec![],
            max_wait_time_seconds: 180,
            expansion_rate: 3.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create players with varying experience levels
    let players = vec![
        (Uuid::new_v4(), Rating::new(1800.0, 200.0, 0.04)), // Experienced player
        (Uuid::new_v4(), Rating::new(1200.0, 350.0, 0.08)), // New player
        (Uuid::new_v4(), Rating::new(1600.0, 250.0, 0.05)), // Moderate experience
        (Uuid::new_v4(), Rating::new(1400.0, 300.0, 0.06)), // Moderate experience
    ];
    
    println!("\nAdding players with different experience levels...");
    for (player_id, rating) in &players {
        let entry = queue_manager.join_queue_solo(
            "custom_ranked".to_string(),
            *player_id,
            *rating,
            EntryMetadata::default(),
        ).await?;
        
        println!("Player {} - Rating: {:.0}, Deviation: {:.0}, Volatility: {:.3}", 
            player_id, rating.rating, rating.deviation, rating.volatility);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("custom_ranked").await?;
    
    println!("Found {} matches", matches.len());
    
    if let Some(match_result) = matches.first() {
        println!("\nMatch details:");
        for (i, entry) in match_result.entries.iter().enumerate() {
            println!("  Player {}: {:.0} (deviation: {:.0})", 
                i + 1, entry.average_rating.rating, entry.average_rating.deviation);
        }
        
        // Simulate rating updates with custom algorithm
        println!("\nUpdating ratings with custom algorithm...");
        
        let outcomes = vec![
            (match_result.entries[0].player_ids[0], Outcome::Win),
            (match_result.entries[1].player_ids[0], Outcome::Win),
            (match_result.entries[2].player_ids[0], Outcome::Loss),
            (match_result.entries[3].player_ids[0], Outcome::Loss),
        ];
        
        // Store original ratings for comparison
        let mut original_ratings = std::collections::HashMap::new();
        for (player_id, _) in &outcomes {
            if let Ok(Some(rating)) = persistence.load_player_rating(*player_id).await {
                original_ratings.insert(*player_id, rating);
            }
        }
        
        // Update ratings (simplified - in real implementation you'd use LobbyManager)
        for (player_a_idx, (player_a_id, outcome_a)) in outcomes.iter().enumerate() {
            for (player_b_idx, (player_b_id, outcome_b)) in outcomes.iter().enumerate() {
                if player_a_idx >= player_b_idx {
                    continue;
                }
                
                let rating_a = original_ratings[player_a_id];
                let rating_b = original_ratings[player_b_id];
                
                // Determine outcome for this matchup
                let matchup_outcome = if outcome_a.score() > outcome_b.score() {
                    Outcome::Win
                } else {
                    Outcome::Loss
                };
                
                let new_rating_a = custom_algorithm.calculate_new_rating(rating_a, rating_b, matchup_outcome);
                let new_rating_b = custom_algorithm.calculate_new_rating(rating_b, rating_a, 
                    if matchup_outcome == Outcome::Win { Outcome::Loss } else { Outcome::Win });
                
                persistence.save_player_rating(*player_a_id, new_rating_a).await?;
                persistence.save_player_rating(*player_b_id, new_rating_b).await?;
            }
        }
        
        // Show rating changes
        println!("\nRating changes:");
        for (player_id, original_rating) in &original_ratings {
            if let Ok(Some(new_rating)) = persistence.load_player_rating(*player_id).await {
                let change = new_rating.rating - original_rating.rating;
                println!("  Player {}: {:.0} → {:.0} ({:+.0})", 
                    player_id, original_rating.rating, new_rating.rating, change);
            }
        }
        
        // Demonstrate decay
        println!("\nApplying adaptive decay to inactive players...");
        let old_time = Utc::now() - chrono::Duration::days(10);
        let inactive_rating = Rating::new(1500.0, 200.0, 0.04);
        let decayed_rating = decay_strategy.apply_decay(inactive_rating, old_time);
        
        println!("  Before decay: {:.0} (deviation: {:.0}, volatility: {:.3})", 
            inactive_rating.rating, inactive_rating.deviation, inactive_rating.volatility);
        println!("  After 10 days: {:.0} (deviation: {:.0}, volatility: {:.3})", 
            decayed_rating.rating, decayed_rating.deviation, decayed_rating.volatility);
    }
    
    println!("\nCustom MMR example completed successfully!");
    Ok(())
}
```

## 5. Testing Strategy

### 5.1 Unit Tests
Each module should include comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_queue_operations() {
        // Test queue join/leave functionality
    }
    
    #[tokio::test]
    async fn test_mmr_calculations() {
        // Test MMR algorithm implementations
    }
    
    #[tokio::test]
    async fn test_lobby_lifecycle() {
        // Test lobby state transitions
    }
}
```

### 5.2 Integration Tests
Integration tests should verify end-to-end functionality:

```rust
// tests/integration_tests.rs
use matchforge::prelude::*;

#[tokio::test]
async fn full_matchmaking_flow() {
    // Test complete flow from queue join to match completion
}

#[tokio::test]
async fn party_matchmaking_integration() {
    // Test party-based matchmaking
}
```

### 5.3 Performance Tests
Load testing for high-concurrency scenarios:

```rust
// tests/performance_tests.rs
use matchforge::prelude::*;

#[tokio::test]
async fn high_volume_matchmaking() {
    // Test with 1000+ concurrent players
}
```

## 6. Documentation Guidelines

### 6.1 API Documentation
- All public functions must have rustdoc comments
- Include examples for complex operations
- Document error conditions and edge cases

### 6.2 Architecture Documentation
- Maintain this instructions.md file
- Include diagrams for data flow
- Document configuration options

### 6.3 Example Code
- Keep examples up-to-date with API changes
- Include realistic use cases
- Demonstrate best practices

## 7. Deployment Considerations

### 7.1 Production Checklist
- [ ] Use Redis or Postgres persistence (not InMemoryAdapter)
- [ ] Configure appropriate tick intervals
- [ ] Set up monitoring and metrics
- [ ] Implement proper error handling and retries
- [ ] Add rate limiting for API endpoints
- [ ] Configure logging levels appropriately

### 7.2 Scaling Strategies
- Horizontal scaling: Multiple runner instances
- Database sharding for large player bases
- Regional deployment for latency optimization
- Caching strategies for frequently accessed data

### 7.3 Monitoring Metrics
Key metrics to monitor:
- Queue sizes and wait times
- Match success rates
- MMR distribution
- Lobby lifecycle duration
- Error rates and types

## 8. Contributing Guidelines

### 8.1 Code Style
- Use `rustfmt` for formatting
- Follow Rust naming conventions
- Keep functions focused and small
- Use meaningful variable names

### 8.2 Testing Requirements
- All new features must include tests
- Maintain >90% test coverage
- Include both unit and integration tests
- Performance tests for critical paths

### 8.3 Pull Request Process
- Update documentation for API changes
- Include examples for new features
- Ensure all tests pass
- Add changelog entry for breaking changes

## 9. License and Legal

This project is dual-licensed under MIT OR Apache-2.0. Contributors agree to license their contributions under the same terms.

## 10. Support and Community

- GitHub Issues: Bug reports and feature requests
- Documentation: Comprehensive guides and API reference
- Examples: Real-world usage patterns
- Community Discord: For discussions and questions

---

**MatchForge SDK** - Eliminate matchmaking boilerplate and focus on building great multiplayer games.
        //