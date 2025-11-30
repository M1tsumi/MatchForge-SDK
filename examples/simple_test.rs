//! Simple integration test for MatchForge SDK

use matchforge::prelude::*;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Starting MatchForge SDK Integration Test");
    
    // Test 1: Analytics system
    test_analytics().await?;
    
    // Test 2: Queue system
    test_queue_system().await?;
    
    // Test 3: Party system
    test_party_system().await?;
    
    // Test 4: Security system
    test_security_system().await?;
    
    println!("✅ All tests passed! MatchForge SDK is functioning correctly.");
    Ok(())
}

async fn test_analytics() -> Result<()> {
    println!("\n📊 Testing Analytics...");
    
    // Import the specific types we need
    use matchforge::analytics::metrics::{AnalyticsMetrics, PlayerActivityType, MatchCompletionData, RatingChange};
    
    let config = AnalyticsConfig::default();
    let analytics = Arc::new(AnalyticsMetrics::new(config));
    
    // Record some player activity
    let player_id = uuid::Uuid::new_v4();
    analytics.record_player_activity(player_id, PlayerActivityType::Login).await;
    
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

async fn test_party_system() -> Result<()> {
    println!("\n👥 Testing Party System...");
    
    let persistence = Arc::new(InMemoryAdapter::new());
    
    // Use AverageStrategy which has a simple implementation
    let mmr_strategy = Arc::new(AverageStrategy);
    
    let party_manager = Arc::new(PartyManager::new(persistence, mmr_strategy));
    
    // Create a party
    let leader = uuid::Uuid::new_v4();
    let member1 = uuid::Uuid::new_v4();
    let member2 = uuid::Uuid::new_v4();
    
    let party = party_manager.create_party(leader, 4).await?;
    println!("   ✅ Created party: {}", party.id);
    
    // Add members
    party_manager.add_member(party.id, member1).await?;
    party_manager.add_member(party.id, member2).await?;
    
    // Get the party using the leader ID
    let updated_party = party_manager.get_player_party(leader).await?;
    println!("   ✅ Party size: {}", updated_party.member_ids.len());
    assert_eq!(updated_party.member_ids.len(), 3, "Party should have 3 members");
    
    Ok(())
}

async fn test_security_system() -> Result<()> {
    println!("\n🔒 Testing Security System...");
    
    // Test rate limiting
    let rate_limit_config = RateLimitConfig {
        max_requests: 10,
        window_seconds: 60,
        burst_size: 5,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(rate_limit_config));
    
    let client_ip = "192.168.1.1".to_string();
    
    // Test some requests
    for i in 0..5 {
        let result = rate_limiter.check_rate_limit(&client_ip).await?;
        println!("   ✅ Request {}: {}", i + 1, result.allowed);
        assert!(result.allowed, "First 5 requests should be allowed");
    }
    
    // Test anti-abuse system
    let anti_abuse_config = AntiAbuseConfig {
        enabled: true,
        max_violations_per_hour: 5,
        violation_decay_hours: 24,
    };
    
    let anti_abuse = Arc::new(AntiAbuseSystem::new(anti_abuse_config));
    
    let player_id = uuid::Uuid::new_v4();
    
    // Record some normal activity
    for _ in 0..3 {
        let result = anti_abuse.record_activity(player_id, "normal_activity").await?;
        assert!(!result.is_flagged, "Normal activity should not be flagged");
    }
    
    println!("   ✅ Security systems working correctly");
    
    Ok(())
}
