use super::config::RunnerConfig;
use crate::{
    error::*,
    lobby::{Lobby, LobbyMetadata, LobbyState},
    mmr::Rating,
    persistence::PersistenceAdapter,
    queue::QueueManager,
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
    pub persistence: Arc<dyn PersistenceAdapter>,
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
