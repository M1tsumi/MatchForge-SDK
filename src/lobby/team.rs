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
