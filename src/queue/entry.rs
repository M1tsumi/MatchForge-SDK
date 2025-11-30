use crate::mmr::Rating;
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
