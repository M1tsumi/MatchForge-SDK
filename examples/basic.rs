//! Basic 1v1 matchmaking example
//! 
//! This example shows how to set up a simple matchmaking system for 1v1 matches
//! using the in-memory persistence adapter.

use matchforge::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));
    
    // Configure 1v1 ranked queue
    let queue_config = QueueConfig {
        name: "ranked_1v1".to_string(),
        format: MatchFormat::one_v_one(),
        constraints: MatchConstraints {
            max_rating_delta: 200.0,
            same_region_required: false,
            role_requirements: vec![],
            max_wait_time_seconds: 300,
            expansion_rate: 5.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create MMR algorithm
    let mmr_algorithm = Arc::new(EloAlgorithm::default());
    
    // Add some players to the queue
    let players = vec![
        (Uuid::new_v4(), Rating::new(1500.0, 350.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1600.0, 300.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1400.0, 320.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1550.0, 280.0, 0.06)),
    ];
    
    println!("Adding players to queue...");
    for (player_id, rating) in &players {
        let entry = queue_manager.join_queue_solo(
            "ranked_1v1".to_string(),
            *player_id,
            *rating,
            EntryMetadata::default(),
        ).await?;
        
        println!("Player {} joined queue (rating: {:.0})", player_id, rating.rating);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("ranked_1v1").await?;
    
    println!("Found {} matches", matches.len());
    
    for (i, match_result) in matches.iter().enumerate() {
        println!("\nMatch {}:", i + 1);
        println!("  Match ID: {}", match_result.match_id);
        
        for (j, entry) in match_result.entries.iter().enumerate() {
            println!("  Team {}: Player {} (rating: {:.0})", 
                j + 1, entry.player_ids[0], entry.average_rating.rating);
        }
        
        // Create lobby
        let lobby = Lobby::from_match_result(
            match_result.clone(),
            vec![1, 1],
            LobbyMetadata {
                queue_name: "ranked_1v1".to_string(),
                game_mode: Some("ranked".to_string()),
                ..Default::default()
            },
        );
        
        lobby_manager.persistence.save_lobby(&lobby).await?;
        println!("  Lobby created: {}", lobby.id);
    }
    
    // Simulate match completion and update ratings
    if let Some(first_match) = matches.first() {
        println!("\nUpdating ratings after match completion...");
        
        // Assume first player won, second lost
        let outcomes = vec![
            (first_match.entries[0].player_ids[0], Outcome::Win),
            (first_match.entries[1].player_ids[0], Outcome::Loss),
        ];
        
        // Create a lobby manager for rating updates
        let rating_manager = LobbyManager::new(persistence.clone());
        
        // Save the lobby first before updating ratings
        let lobby = Lobby::from_match_result(
            first_match.clone(),
            vec![1, 1],
            LobbyMetadata {
                queue_name: "ranked_1v1".to_string(),
                game_mode: Some("ranked".to_string()),
                ..Default::default()
            },
        );
        persistence.save_lobby(&lobby).await?;
        
        rating_manager.update_ratings(lobby.id, &outcomes, mmr_algorithm).await?;
        
        // Show updated ratings
        for (player_id, _) in &outcomes {
            if let Ok(Some(new_rating)) = persistence.load_player_rating(*player_id).await {
                println!("Player {} new rating: {:.0}", player_id, new_rating.rating);
            }
        }
    }
    
    println!("\nBasic matchmaking example completed successfully!");
    Ok(())
}
