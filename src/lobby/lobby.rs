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
