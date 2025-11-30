use super::{constraints::MatchConstraints, entry::QueueEntry};
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

    pub fn team_v_team(team_size: usize) -> Self {
        Self {
            name: format!("{}v{}", team_size, team_size),
            team_sizes: vec![team_size, team_size],
            total_players: team_size * 2,
        }
    }

    /// Get the total number of players per match
    pub fn players_per_match(&self) -> usize {
        self.total_players
    }

    /// Get the number of teams
    pub fn team_count(&self) -> usize {
        self.team_sizes.len()
    }

    /// Get the size of a specific team
    pub fn team_size(&self, team_index: usize) -> Option<usize> {
        self.team_sizes.get(team_index).copied()
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
