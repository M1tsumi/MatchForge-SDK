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
