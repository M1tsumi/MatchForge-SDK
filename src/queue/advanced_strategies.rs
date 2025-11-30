//! Advanced matchmaking strategies
//! 
//! This module provides sophisticated matchmaking algorithms for different
//! tournament formats and competitive scenarios.

use super::{constraints::MatchConstraints, entry::QueueEntry, matcher::{MatchFormat, MatchResult}};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::Utc;
use rand::prelude::SliceRandom;

/// Swiss-style matchmaking strategy
/// 
/// Swiss pairing matches players with similar scores while avoiding repeat matches.
/// Commonly used in chess tournaments and competitive gaming.
pub struct SwissMatcher {
    max_score_difference: f64,
    avoid_rematches: bool,
}

impl SwissMatcher {
    pub fn new(max_score_difference: f64, avoid_rematches: bool) -> Self {
        Self {
            max_score_difference,
            avoid_rematches,
        }
    }
    
    /// Find swiss-style pairings
    pub fn find_pairings(
        &self,
        entries: &[QueueEntry],
        player_scores: &HashMap<Uuid, f64>,
        previous_matchups: &HashMap<Uuid, Vec<Uuid>>,
    ) -> Vec<MatchResult> {
        let mut matches = Vec::new();
        let mut used_players = std::collections::HashSet::new();
        
        // Sort entries by score (descending)
        let mut sorted_entries: Vec<_> = entries.iter().collect();
        sorted_entries.sort_by(|a, b| {
            let score_a = a.player_ids.iter()
                .map(|id| player_scores.get(id).unwrap_or(&0.0))
                .sum::<f64>() / a.player_ids.len() as f64;
            let score_b = b.player_ids.iter()
                .map(|id| player_scores.get(id).unwrap_or(&0.0))
                .sum::<f64>() / b.player_ids.len() as f64;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        for entry in &sorted_entries {
            if used_players.contains(&entry.id) {
                continue;
            }
            
            // Find the best opponent
            if let Some(opponent) = self.find_best_opponent(
                entry,
                &sorted_entries,
                &used_players,
                player_scores,
                previous_matchups,
            ) {
                used_players.insert(entry.id);
                used_players.insert(opponent.id);
                
                matches.push(MatchResult {
                    match_id: Uuid::new_v4(),
                    entries: vec![(*entry).clone(), opponent],
                    team_assignments: vec![0, 1], // Team assignments for 1v1
                });
            }
        }
        
        matches
    }
    
    fn find_best_opponent(
        &self,
        entry: &QueueEntry,
        candidates: &[&QueueEntry],
        used_players: &std::collections::HashSet<Uuid>,
        player_scores: &HashMap<Uuid, f64>,
        previous_matchups: &HashMap<Uuid, Vec<Uuid>>,
    ) -> Option<QueueEntry> {
        let entry_score = entry.player_ids.iter()
            .map(|id| player_scores.get(id).unwrap_or(&0.0))
            .sum::<f64>() / entry.player_ids.len() as f64;
        
        let mut best_opponent = None;
        let mut best_score = f64::INFINITY;
        
        for candidate in candidates {
            if used_players.contains(&candidate.id) || candidate.id == entry.id {
                continue;
            }
            
            let candidate_score = candidate.player_ids.iter()
                .map(|id| player_scores.get(id).unwrap_or(&0.0))
                .sum::<f64>() / candidate.player_ids.len() as f64;
            
            // Check score difference
            let score_diff = (entry_score - candidate_score).abs();
            if score_diff > self.max_score_difference {
                continue;
            }
            
            // Check for previous matchups if enabled
            if self.avoid_rematches {
                let has_remapped = entry.player_ids.iter().any(|id| {
                    previous_matchups.get(id)
                        .map(|opponents| {
                            opponents.iter().any(|opp| candidate.player_ids.contains(opp))
                        })
                        .unwrap_or(false)
                });
                
                if has_remapped {
                    continue;
                }
            }
            
            // Calculate quality score (lower is better)
            let quality_score = score_diff + 
                (entry.average_rating.rating - candidate.average_rating.rating).abs() * 0.01;
            
            if quality_score < best_score {
                best_score = quality_score;
                best_opponent = Some(candidate);
            }
        }
        
        best_opponent.cloned().cloned()
    }
    
    fn calculate_match_quality(
        &self,
        entry1: &QueueEntry,
        entry2: &QueueEntry,
        player_scores: &HashMap<Uuid, f64>,
    ) -> f64 {
        let score1 = entry1.player_ids.iter()
            .map(|id| player_scores.get(id).unwrap_or(&0.0))
            .sum::<f64>() / entry1.player_ids.len() as f64;
        let score2 = entry2.player_ids.iter()
            .map(|id| player_scores.get(id).unwrap_or(&0.0))
            .sum::<f64>() / entry2.player_ids.len() as f64;
        
        let score_diff = (score1 - score2).abs();
        let rating_diff = (entry1.average_rating.rating - entry2.average_rating.rating).abs();
        
        // Quality score: lower is better (0 = perfect match)
        score_diff + rating_diff * 0.01
    }
}

/// Tournament bracket matcher
/// 
/// Handles single and double elimination tournament brackets.
pub struct TournamentMatcher {
    bracket_type: TournamentType,
    seeding_strategy: SeedingStrategy,
}

#[derive(Debug, Clone)]
pub enum TournamentType {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
}

#[derive(Debug, Clone)]
pub enum SeedingStrategy {
    Random,
    ByRating,
    ByScore,
    Manual(Vec<Uuid>),
}

impl TournamentMatcher {
    pub fn new(bracket_type: TournamentType, seeding_strategy: SeedingStrategy) -> Self {
        Self {
            bracket_type,
            seeding_strategy,
        }
    }
    
    /// Generate initial tournament bracket
    pub fn generate_bracket(
        &self,
        entries: Vec<QueueEntry>,
        format: MatchFormat,
    ) -> TournamentBracket {
        let seeded_entries = self.apply_seeding(entries);
        let matches = self.generate_initial_round(seeded_entries, format);
        
        TournamentBracket {
            bracket_type: self.bracket_type.clone(),
            current_round: 1,
            matches,
            completed_matches: Vec::new(),
            eliminated_players: std::collections::HashSet::new(),
        }
    }
    
    fn apply_seeding(&self, entries: Vec<QueueEntry>) -> Vec<QueueEntry> {
        match &self.seeding_strategy {
            SeedingStrategy::Random => {
                let mut seeded = entries;
                seeded.shuffle(&mut rand::thread_rng());
                seeded
            }
            SeedingStrategy::ByRating => {
                let mut seeded = entries;
                seeded.sort_by(|a, b| b.average_rating.rating.partial_cmp(&a.average_rating.rating).unwrap());
                seeded
            }
            SeedingStrategy::ByScore => {
                // This would require external score information
                let mut seeded = entries;
                seeded.sort_by(|a, b| b.average_rating.rating.partial_cmp(&a.average_rating.rating).unwrap());
                seeded
            }
            SeedingStrategy::Manual(order) => {
                let mut seeded = entries;
                seeded.sort_by(|a, b| {
                    let index_a = order.iter().position(|id| a.player_ids.contains(id)).unwrap_or(usize::MAX);
                    let index_b = order.iter().position(|id| b.player_ids.contains(id)).unwrap_or(usize::MAX);
                    index_a.cmp(&index_b)
                });
                seeded
            }
        }
    }
    
    fn generate_initial_round(&self, entries: Vec<QueueEntry>, format: MatchFormat) -> Vec<TournamentMatch> {
        let mut matches = Vec::new();
        let players_per_match = format.players_per_match();
        
        // Create matches by grouping seeded entries
        for chunk in entries.chunks(players_per_match) {
            if chunk.len() == players_per_match {
                matches.push(TournamentMatch {
                    match_id: Uuid::new_v4(),
                    round: 1,
                    bracket_position: matches.len(),
                    entries: chunk.to_vec(),
                    winner: None,
                    format: format.clone(),
                });
            }
        }
        
        // Handle byes if necessary
        if matches.len() < 2 && !entries.is_empty() {
            // Give bye to highest seed
            let bye_entry = &entries[0];
            matches.push(TournamentMatch {
                match_id: Uuid::new_v4(),
                round: 1,
                bracket_position: 0,
                entries: vec![bye_entry.clone()],
                winner: Some(bye_entry.player_ids[0]),
                format: format.clone(),
            });
        }
        
        matches
    }
    
    /// Generate next round matches from winners
    pub fn generate_next_round(&self, bracket: &TournamentBracket, format: MatchFormat) -> Vec<TournamentMatch> {
        let mut next_matches = Vec::new();
        let players_per_match = format.players_per_match();
        
        // Collect winners from completed matches
        let mut winners = Vec::new();
        for match_result in &bracket.completed_matches {
            if let Some(winner) = match_result.winner {
                // Find the entry for the winner
                for entry in &match_result.entries {
                    if entry.player_ids.contains(&winner) {
                        winners.push(entry.clone());
                        break;
                    }
                }
            }
        }
        
        // Create next round matches
        for chunk in winners.chunks(players_per_match) {
            if chunk.len() == players_per_match {
                next_matches.push(TournamentMatch {
                    match_id: Uuid::new_v4(),
                    round: bracket.current_round + 1,
                    bracket_position: next_matches.len(),
                    entries: chunk.to_vec(),
                    winner: None,
                    format: format.clone(),
                });
            }
        }
        
        next_matches
    }
}

/// Tournament bracket structure
#[derive(Debug, Clone)]
pub struct TournamentBracket {
    pub bracket_type: TournamentType,
    pub current_round: u32,
    pub matches: Vec<TournamentMatch>,
    pub completed_matches: Vec<TournamentMatch>,
    pub eliminated_players: std::collections::HashSet<Uuid>,
}

/// Individual tournament match
#[derive(Debug, Clone)]
pub struct TournamentMatch {
    pub match_id: Uuid,
    pub round: u32,
    pub bracket_position: usize,
    pub entries: Vec<QueueEntry>,
    pub winner: Option<Uuid>,
    pub format: MatchFormat,
}

impl TournamentMatch {
    /// Record match result
    pub fn set_winner(&mut self, winner_id: Uuid) {
        self.winner = Some(winner_id);
    }
    
    /// Get all players in this match
    pub fn all_players(&self) -> Vec<Uuid> {
        self.entries.iter().flat_map(|e| e.player_ids.clone()).collect()
    }
    
    /// Check if match is complete
    pub fn is_complete(&self) -> bool {
        self.winner.is_some()
    }
}

/// Skill-based matchmaking with dynamic constraints
/// 
/// This matcher adjusts constraints based on queue size and wait times.
pub struct AdaptiveMatcher {
    base_constraints: MatchConstraints,
    max_wait_time: chrono::Duration,
    expansion_factor: f64,
}

impl AdaptiveMatcher {
    pub fn new(
        base_constraints: MatchConstraints,
        max_wait_time: chrono::Duration,
        expansion_factor: f64,
    ) -> Self {
        Self {
            base_constraints,
            max_wait_time,
            expansion_factor,
        }
    }
    
    /// Find matches with adaptive constraints
    pub fn find_matches(&self, entries: &[QueueEntry], current_time: chrono::DateTime<chrono::Utc>) -> Vec<MatchResult> {
        let mut matches = Vec::new();
        let mut used_entries = std::collections::HashSet::new();
        
        for (i, entry) in entries.iter().enumerate() {
            if used_entries.contains(&entry.id) {
                continue;
            }
            
            let wait_time = current_time - entry.joined_at;
            let constraints = self.adjust_constraints(&wait_time);
            
            // Find compatible entries
            let compatible: Vec<_> = entries[i + 1..]
                .iter()
                .filter(|e| !used_entries.contains(&e.id))
                .filter(|e| self.are_compatible(entry, e, &constraints))
                .collect();
            
            if let Some(best_match) = self.find_best_match(entry, &compatible) {
                used_entries.insert(entry.id);
                used_entries.insert(best_match.id);
                
                matches.push(MatchResult {
                    match_id: Uuid::new_v4(),
                    entries: vec![entry.clone(), best_match.clone()],
                    team_assignments: vec![0, 1], // Team assignments for 1v1
                });
            }
        }
        
        matches
    }
    
    fn adjust_constraints(&self, wait_time: &chrono::Duration) -> MatchConstraints {
        let wait_ratio = (wait_time.num_milliseconds() as f64) / (self.max_wait_time.num_milliseconds() as f64);
        let expansion = 1.0 + (wait_ratio * self.expansion_factor);
        
        MatchConstraints {
            max_rating_delta: self.base_constraints.max_rating_delta * expansion,
            same_region_required: self.base_constraints.same_region_required,
            role_requirements: self.base_constraints.role_requirements.clone(),
            max_wait_time_seconds: self.base_constraints.max_wait_time_seconds,
            expansion_rate: self.base_constraints.expansion_rate,
        }
    }
    
    fn constraints_satisfied(&self, entry: &QueueEntry, constraints: &MatchConstraints) -> bool {
        // Check rating difference
        let rating_diff = (entry.average_rating.rating - constraints.max_rating_delta).abs();
        if rating_diff > constraints.max_rating_delta {
            return false;
        }
        
        // Check wait time
        let wait_time = Utc::now().signed_duration_since(entry.joined_at).num_seconds();
        if wait_time > constraints.max_wait_time_seconds {
            return false;
        }
        
        // Check role requirements
        if !constraints.role_requirements.is_empty() {
            // Simplified role checking - would need more sophisticated logic
            return true;
        }
        
        true
    }
    
    fn are_compatible(&self, entry1: &QueueEntry, entry2: &QueueEntry, constraints: &MatchConstraints) -> bool {
        let rating_diff = (entry1.average_rating.rating - entry2.average_rating.rating).abs();
        if rating_diff > constraints.max_rating_delta {
            return false;
        }
        
        // Check role requirements
        if !constraints.role_requirements.is_empty() {
            // Simplified role checking - would need more sophisticated logic
            return true;
        }
        
        true
    }
    
    fn find_best_match<'a>(&self, entry: &QueueEntry, candidates: &[&'a QueueEntry]) -> Option<&'a QueueEntry> {
        candidates
            .iter()
            .min_by(|a, b| {
                let score_a = self.calculate_quality_score(entry, a);
                let score_b = self.calculate_quality_score(entry, b);
                score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
    
    fn calculate_quality_score(&self, entry1: &QueueEntry, entry2: &QueueEntry) -> f64 {
        let rating_diff = (entry1.average_rating.rating - entry2.average_rating.rating).abs();
        let wait_diff = (entry1.joined_at.timestamp() - entry2.joined_at.timestamp()).abs() as f64;
        
        rating_diff + wait_diff * 0.001
    }
}

/// Fair team balancer for uneven party sizes
/// 
/// This matcher tries to create balanced teams when parties of different sizes are involved.
pub struct FairTeamBalancer {
    balance_strategy: BalanceStrategy,
}

#[derive(Debug, Clone)]
pub enum BalanceStrategy {
    ByRating,
    ByPartySize,
    Hybrid,
}

impl FairTeamBalancer {
    pub fn new(balance_strategy: BalanceStrategy) -> Self {
        Self { balance_strategy }
    }
    
    /// Create balanced teams from mixed party sizes
    pub fn create_balanced_teams(&self, entries: &[QueueEntry], team_sizes: &[usize]) -> Vec<Vec<QueueEntry>> {
        match self.balance_strategy {
            BalanceStrategy::ByRating => self.balance_by_rating(entries, team_sizes),
            BalanceStrategy::ByPartySize => self.balance_by_party_size(entries, team_sizes),
            BalanceStrategy::Hybrid => self.balance_hybrid(entries, team_sizes),
        }
    }
    
    fn balance_by_rating(&self, entries: &[QueueEntry], team_sizes: &[usize]) -> Vec<Vec<QueueEntry>> {
        let mut teams = vec![Vec::new(); team_sizes.len()];
        let mut sorted_entries: Vec<_> = entries.iter().collect();
        sorted_entries.sort_by(|a, b| b.average_rating.rating.partial_cmp(&a.average_rating.rating).unwrap());
        
        // Distribute players using snake draft
        let mut direction = 1;
        let mut current_team = 0;
        
        for entry in sorted_entries {
            teams[current_team].push(entry.clone());
            
            current_team = if direction > 0 {
                if current_team + 1 < teams.len() {
                    current_team + 1
                } else {
                    direction = -1;
                    current_team - 1
                }
            } else {
                if current_team > 0 {
                    current_team - 1
                } else {
                    direction = 1;
                    current_team + 1
                }
            };
        }
        
        teams
    }
    
    fn balance_by_party_size(&self, entries: &[QueueEntry], team_sizes: &[usize]) -> Vec<Vec<QueueEntry>> {
        let mut teams = vec![Vec::new(); team_sizes.len()];
        
        // Sort by party size (largest first)
        let mut sorted_entries: Vec<_> = entries.iter().collect();
        sorted_entries.sort_by(|a, b| b.player_ids.len().cmp(&a.player_ids.len()));
        
        // Assign to teams with most available slots
        for entry in sorted_entries {
            let team_index = teams.iter()
                .enumerate()
                .min_by_key(|(i, team)| {
                    let used_slots = team.iter().map(|e: &QueueEntry| e.player_ids.len()).sum::<usize>();
                    let available = team_sizes[*i] - used_slots;
                    if available >= entry.player_ids.len() {
                        available
                    } else {
                        usize::MAX
                    }
                })
                .map(|(i, _)| i)
                .unwrap_or(0);
            
            teams[team_index].push(entry.clone());
        }
        
        teams
    }
    
    fn balance_hybrid(&self, entries: &[QueueEntry], team_sizes: &[usize]) -> Vec<Vec<QueueEntry>> {
        // First balance by party size, then adjust by rating
        let mut teams = self.balance_by_party_size(entries, team_sizes);
        
        // Calculate team ratings
        let team_ratings: Vec<f64> = teams.iter()
            .map(|team| {
                team.iter().map(|e| e.average_rating.rating).sum::<f64>() / team.len() as f64
            })
            .collect();
        
        // Simple rating balancing - could be more sophisticated
        let avg_rating = team_ratings.iter().sum::<f64>() / team_ratings.len() as f64;
        
        for (i, _team) in teams.iter_mut().enumerate() {
            let current_rating = team_ratings[i];
            if (current_rating - avg_rating).abs() > 100.0 {
                // Team is significantly unbalanced - would need rebalancing logic
                // This is a simplified version
            }
        }
        
        teams
    }
}
