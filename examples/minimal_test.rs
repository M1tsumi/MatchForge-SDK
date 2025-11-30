//! Minimal integration test for MatchForge SDK

use matchforge::prelude::*;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting MatchForge SDK Minimal Test");
    
    // Test 1: Analytics system
    test_analytics().await?;
    
    // Test 2: Queue system
    test_queue_system().await?;
    
    println!("✅ Core tests passed! MatchForge SDK is functioning correctly.");
    Ok(())
}

async fn test_analytics() -> Result<()> {
    println!("\n📊 Testing Analytics...");
    
    // Import the specific types we need
    use matchforge::analytics::metrics::{AnalyticsMetrics, PlayerActivityType, MatchCompletionData, RatingChange, AnalyticsConfig};
    
    let config = AnalyticsConfig::default();
    let analytics = Arc::new(AnalyticsMetrics::new(config));
    
    // Record some player activity
    let player_id = uuid::Uuid::new_v4();
    analytics.record_player_activity(player_id, PlayerActivityType::NewPlayer).await;
    
    // Create rating changes for match completion
    let match_id = uuid::Uuid::new_v4();
    let rating_changes = vec![
        RatingChange {
            player_id,
            old_rating: 1500.0,
            new_rating: 1525.0,
            change_amount: 25.0,
            match_id,
            timestamp: chrono::Utc::now(),
            outcome: "win".to_string(),
        }
    ];
    
    // Record a match
    let match_data = MatchCompletionData {
        match_id,
        average_rating: 1500.0,
        quality_score: 0.85,
        duration: Duration::from_secs(1200),
        rating_changes,
    };
    analytics.record_match_completed(match_data).await;
    
    // Get metrics snapshot
    let snapshot = analytics.get_metrics_snapshot().await;
    println!("   ✅ Total players: {}", snapshot.total_players);
    println!("   ✅ Total matches: {}", snapshot.total_matches);
    println!("   ✅ Match quality: {:.2}", snapshot.match_quality_score);
    
    assert!(snapshot.total_players > 0, "Should have recorded player activity");
    assert!(snapshot.total_matches > 0, "Should have recorded match completion");
    
    Ok(())
}

async fn test_queue_system() -> Result<()> {
    println!("\n📋 Testing Queue System...");
    
    // Create components
    let persistence = Arc::new(InMemoryAdapter::new());
    
    // Create queue manager
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    
    // Create queue config
    let queue_config = QueueConfig {
        name: "ranked".to_string(),
        format: MatchFormat::one_v_one(),
        constraints: MatchConstraints::permissive(),
    };
    
    // Register the queue
    queue_manager.register_queue(queue_config).await?;
    
    // Create a rating for players
    let rating = Rating {
        rating: 1500.0,
        deviation: 350.0,
        volatility: 0.06,
    };
    
    // Create metadata
    let metadata = EntryMetadata {
        region: Some("us-east".to_string()),
        roles: vec!["damage".to_string()],
        custom: std::collections::HashMap::new(),
    };
    
    // Add some players to queue
    let player1 = uuid::Uuid::new_v4();
    let player2 = uuid::Uuid::new_v4();
    
    queue_manager.join_queue_solo("ranked".to_string(), player1, rating.clone(), metadata.clone()).await?;
    queue_manager.join_queue_solo("ranked".to_string(), player2, rating, metadata).await?;
    
    // Get queue size
    let queue_size = queue_manager.get_queue_size("ranked").await?;
    println!("   ✅ Queue has {} players", queue_size);
    assert_eq!(queue_size, 2, "Should have 2 players in queue");
    
    Ok(())
}
