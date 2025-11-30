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
