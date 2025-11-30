//! Integration tests for MatchForge SDK
//! 
//! These tests verify the complete functionality of the matchmaking system
//! across all components working together.

use matchforge::prelude::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// Test complete matchmaking flow from queue to lobby
#[tokio::test]
async fn test_complete_matchmaking_flow() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));
    let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));

    // Register multiple queues
    let queue_configs = vec![
        QueueConfig {
            name: "ranked_1v1".to_string(),
            format: MatchFormat::one_v_one(),
            constraints: MatchConstraints::strict(),
        },
        QueueConfig {
            name: "casual_5v5".to_string(),
            format: MatchFormat::five_v_five(),
            constraints: MatchConstraints::permissive(),
        },
    ];

    for config in queue_configs {
        queue_manager.register_queue(config).await?;
    }

    // Test 1v1 matchmaking
    let player_ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
    let ratings: Vec<Rating> = (0..4).map(|i| Rating::new(1500.0 + i as f64 * 50.0, 300.0, 0.06)).collect();

    for (player_id, rating) in player_ids.iter().zip(ratings.iter()) {
        queue_manager.join_queue_solo(
            "ranked_1v1".to_string(),
            *player_id,
            *rating,
            EntryMetadata::default(),
        ).await?;
    }

    let matches = queue_manager.find_matches("ranked_1v1").await?;
    assert_eq!(matches.len(), 2, "Should find 2 matches for 4 players");

    // Verify lobby creation
    for match_result in &matches {
        let lobby = Lobby::from_match_result(
            match_result.clone(),
            vec![1, 1],
            LobbyMetadata {
                queue_name: "ranked_1v1".to_string(),
                game_mode: Some("ranked".to_string()),
                ..Default::default()
            },
        );

        assert_eq!(lobby.player_ids.len(), 2);
        assert_eq!(lobby.teams.len(), 2);
        assert_eq!(lobby.state, LobbyState::Forming);

        persistence.save_lobby(&lobby).await?;
    }

    // Test rating updates
    let mmr_algorithm = Arc::new(EloAlgorithm::default());
    let first_match = &matches[0];
    let outcomes = vec![
        (first_match.entries[0].player_ids[0], Outcome::Win),
        (first_match.entries[1].player_ids[0], Outcome::Loss),
    ];

    // Save initial ratings
    for (player_id, rating) in player_ids.iter().zip(ratings.iter()) {
        persistence.save_player_rating(*player_id, *rating).await?;
    }

    // Update ratings
    lobby_manager.update_ratings(first_match.match_id, &outcomes, mmr_algorithm).await?;

    // Verify rating changes
    let winner_rating = persistence.load_player_rating(outcomes[0].0).await?.unwrap();
    let loser_rating = persistence.load_player_rating(outcomes[1].0).await?.unwrap();

    assert!(winner_rating.rating > ratings[0].rating, "Winner should gain rating");
    assert!(loser_rating.rating < ratings[1].rating, "Loser should lose rating");

    Ok(())
}

/// Test party matchmaking functionality
#[tokio::test]
async fn test_party_matchmaking() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));

    // Register queue
    queue_manager.register_queue(QueueConfig {
        name: "team_5v5".to_string(),
        format: MatchFormat::five_v_five(),
        constraints: MatchConstraints::permissive(),
    }).await?;

    // Create parties
    let party1 = party_manager.create_party(Uuid::new_v4(), 5).await?;
    let party2 = party_manager.create_party(Uuid::new_v4(), 5).await?;

    // Add members to parties
    let party1_members: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
    let party2_members: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();

    for member_id in &party1_members {
        persistence.save_player_rating(*member_id, Rating::default_beginner()).await?;
        party_manager.add_member(party1.id, *member_id).await?;
    }

    for member_id in &party2_members {
        persistence.save_player_rating(*member_id, Rating::default_beginner()).await?;
        party_manager.add_member(party2.id, *member_id).await?;
    }

    // Calculate party ratings and add to queue
    let party1_rating = party_manager.calculate_party_rating(party1.id).await?;
    let party2_rating = party_manager.calculate_party_rating(party2.id).await?;

    queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party1.id,
        party1.member_ids.clone(),
        party1_rating,
        EntryMetadata::default(),
    ).await?;

    queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party2.id,
        party2.member_ids.clone(),
        party2_rating,
        EntryMetadata::default(),
    ).await?;

    // Add solo players to fill the match
    let solo_players: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
    for player_id in &solo_players {
        persistence.save_player_rating(*player_id, Rating::default_beginner()).await?;
        queue_manager.join_queue_solo(
            "team_5v5".to_string(),
            *player_id,
            Rating::default_beginner(),
            EntryMetadata::default(),
        ).await?;
    }

    // Find matches
    let matches = queue_manager.find_matches("team_5v5").await?;
    assert!(!matches.is_empty(), "Should find at least one match");

    let match_result = &matches[0];
    assert_eq!(match_result.entries.iter().map(|e| e.player_count()).sum::<usize>(), 10);

    Ok(())
}

/// Test matchmaking runner functionality
#[tokio::test]
async fn test_matchmaking_runner() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));

    // Register queue
    queue_manager.register_queue(QueueConfig {
        name: "test_queue".to_string(),
        format: MatchFormat::two_v_two(),
        constraints: MatchConstraints::permissive(),
    }).await?;

    // Add players to queue
    let players: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
    for player_id in &players {
        queue_manager.join_queue_solo(
            "test_queue".to_string(),
            *player_id,
            Rating::default_beginner(),
            EntryMetadata::default(),
        ).await?;
    }

    // Create runner with fast tick interval
    let runner_config = RunnerConfig {
        tick_interval_ms: 100,
        max_matches_per_tick: 10,
        auto_dispatch: false,
        queue_configs: {
            let mut configs = std::collections::HashMap::new();
            configs.insert("test_queue".to_string(), QueueRunnerConfig {
                enabled: true,
                priority: 1,
                max_concurrent_matches: 10,
            });
            configs
        },
    };

    let runner = MatchmakingRunner::new(runner_config, queue_manager.clone(), persistence.clone());

    // Start runner in background
    let runner_handle = {
        let runner = runner.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(200)).await; // Run for a short time
            runner.stop();
        })
    };

    // Start runner
    let start_handle = tokio::spawn(async move {
        runner.start().await
    });

    // Wait for runner to finish
    let (_, _) = tokio::join!(runner_handle, start_handle);

    // Check that matches were found
    let queue_size = queue_manager.get_queue_size("test_queue").await?;
    assert!(queue_size < 4, "Players should have been matched and removed from queue");

    Ok(())
}

/// Test MMR decay functionality
#[tokio::test]
async fn test_mmr_decay() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let player_id = Uuid::new_v4();
    let original_rating = Rating::new(1600.0, 200.0, 0.04);

    // Save initial rating
    persistence.save_player_rating(player_id, original_rating).await?;

    // Apply decay
    let decay_strategy = LinearDecay::new(1.0, 100.0);
    let old_time = Utc::now() - chrono::Duration::days(10);
    let decayed_rating = decay_strategy.apply_decay(original_rating, old_time);

    // Verify decay was applied
    assert!(decayed_rating.rating < original_rating.rating, "Rating should decay");
    assert!(decayed_rating.deviation > original_rating.deviation, "Deviation should increase");

    Ok(())
}

/// Test season reset functionality
#[tokio::test]
async fn test_season_reset() -> Result<()> {
    let player_id = Uuid::new_v4();
    let original_rating = Rating::new(2000.0, 150.0, 0.03);

    // Test soft reset
    let soft_reset = SoftReset::new(1500.0, 0.5);
    let soft_reset_rating = soft_reset.reset_rating(original_rating);

    assert!(soft_reset_rating.rating < original_rating.rating, "Soft reset should lower rating");
    assert!(soft_reset_rating.rating > 1500.0, "Soft reset should not go all the way to target");

    // Test hard reset
    let hard_reset = HardReset::new(1500.0);
    let hard_reset_rating = hard_reset.reset_rating(original_rating);

    assert_eq!(hard_reset_rating.rating, 1500.0, "Hard reset should set exact rating");
    assert_eq!(hard_reset_rating.deviation, 350.0, "Hard reset should set high deviation");

    Ok(())
}

/// Test lobby lifecycle management
#[tokio::test]
async fn test_lifecycle_management() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));

    // Create a test lobby
    let player_ids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
    let mut lobby = Lobby {
        id: Uuid::new_v4(),
        match_id: Uuid::new_v4(),
        state: LobbyState::Forming,
        teams: vec![
            Team {
                team_id: 0,
                player_ids: player_ids[0..2].to_vec(),
            },
            Team {
                team_id: 1,
                player_ids: player_ids[2..4].to_vec(),
            },
        ],
        player_ids: player_ids.clone(),
        ready_players: std::collections::HashSet::new(),
        created_at: Utc::now(),
        metadata: LobbyMetadata::default(),
    };

    persistence.save_lobby(&lobby).await?;

    // Test state transitions
    lobby.transition_to(LobbyState::WaitingForReady)?;
    persistence.save_lobby(&lobby).await?;

    // Mark players as ready
    for player_id in &player_ids {
        lobby_manager.mark_player_ready(lobby.id, *player_id).await?;
    }

    // Verify lobby is ready
    let updated_lobby = lobby_manager.get_lobby(lobby.id).await?.unwrap();
    assert_eq!(updated_lobby.state, LobbyState::Ready);

    // Test dispatch
    lobby_manager.dispatch_lobby(lobby.id, "server-123".to_string()).await?;
    let dispatched_lobby = lobby_manager.get_lobby(lobby.id).await?.unwrap();
    assert_eq!(dispatched_lobby.state, LobbyState::Dispatched);
    assert_eq!(dispatched_lobby.metadata.server_id, Some("server-123".to_string()));

    // Test close
    lobby_manager.close_lobby(lobby.id).await?;
    let closed_lobby = lobby_manager.get_lobby(lobby.id).await?;
    assert!(closed_lobby.is_none(), "Lobby should be deleted after closing");

    Ok(())
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));

    let player_id = Uuid::new_v4();

    // Test joining non-existent queue
    let result = queue_manager.join_queue_solo(
        "non_existent".to_string(),
        player_id,
        Rating::default_beginner(),
        EntryMetadata::default(),
    ).await;
    assert!(result.is_err(), "Should fail for non-existent queue");

    // Test leaving queue when not in queue
    let result = queue_manager.leave_queue("test_queue", player_id).await;
    assert!(result.is_err(), "Should fail when not in queue");

    // Test duplicate party member
    let party = party_manager.create_party(player_id, 5).await?;
    let result = party_manager.add_member(party.id, player_id).await;
    assert!(result.is_err(), "Should fail to add existing member");

    // Test full party
    let party2 = party_manager.create_party(Uuid::new_v4(), 2).await?;
    let new_member = Uuid::new_v4();
    party_manager.add_member(party2.id, new_member).await?;
    
    let another_member = Uuid::new_v4();
    let result = party_manager.add_member(party2.id, another_member).await;
    assert!(result.is_err(), "Should fail to add member to full party");

    Ok(())
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));

    // Register queue
    queue_manager.register_queue(QueueConfig {
        name: "concurrent_test".to_string(),
        format: MatchFormat::one_v_one(),
        constraints: MatchConstraints::permissive(),
    }).await?;

    // Add many players concurrently
    let mut handles = Vec::new();
    for i in 0..20 {
        let queue_manager = queue_manager.clone();
        let handle = tokio::spawn(async move {
            let player_id = Uuid::new_v4();
            let rating = Rating::new(1500.0 + i as f64 * 10.0, 300.0, 0.06);
            
            queue_manager.join_queue_solo(
                "concurrent_test".to_string(),
                player_id,
                rating,
                EntryMetadata::default(),
            ).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    let results: Vec<Result<QueueEntry>> = futures::future::join_all(handles).await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All operations should succeed
    for result in &results {
        assert!(result.is_ok(), "All queue joins should succeed");
    }

    // Verify queue size
    let queue_size = queue_manager.get_queue_size("concurrent_test").await?;
    assert_eq!(queue_size, 20, "All players should be in queue");

    // Find matches concurrently
    let matches = queue_manager.find_matches("concurrent_test").await?;
    assert_eq!(matches.len(), 10, "Should find 10 matches for 20 players");

    Ok(())
}
