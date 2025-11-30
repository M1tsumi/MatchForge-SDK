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
