use super::{
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
            .ok_or_else(|| MatchForgeError::QueueNotFound(entry.queue_name.clone()))?;
        
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
