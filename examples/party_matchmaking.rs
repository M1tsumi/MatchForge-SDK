//! Party-based matchmaking example
//! 
//! This example demonstrates how to handle parties (groups of players)
//! in the matchmaking system.

use matchforge::prelude::*;
use matchforge::queue::RoleRequirement;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));
    
    // Configure 5v5 queue
    let queue_config = QueueConfig {
        name: "team_5v5".to_string(),
        format: MatchFormat::five_v_five(),
        constraints: MatchConstraints {
            max_rating_delta: 300.0,
            same_region_required: true,
            role_requirements: vec![
                RoleRequirement { role: "tank".to_string(), count: 1 },
                RoleRequirement { role: "healer".to_string(), count: 1 },
                RoleRequirement { role: "dps".to_string(), count: 3 },
            ],
            max_wait_time_seconds: 600,
            expansion_rate: 10.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create some parties
    println!("Creating parties...");
    
    // Party 1: 3 players with similar ratings
    let party1_leader = Uuid::new_v4();
    let party1 = party_manager.create_party(party1_leader, 5).await?;
    println!("Created party {} with leader {}", party1.id, party1_leader);
    
    let party1_members = vec![
        (Uuid::new_v4(), Rating::new(1500.0, 300.0, 0.06)),
        (Uuid::new_v4(), Rating::new(1550.0, 280.0, 0.06)),
    ];
    
    for (member_id, rating) in &party1_members {
        persistence.save_player_rating(*member_id, *rating).await?;
        party_manager.add_member(party1.id, *member_id).await?;
        println!("Added member {} to party {}", member_id, party1.id);
    }
    
    // Party 2: 2 players
    let party2_leader = Uuid::new_v4();
    let party2 = party_manager.create_party(party2_leader, 5).await?;
    println!("Created party {} with leader {}", party2.id, party2_leader);
    
    let party2_member = (Uuid::new_v4(), Rating::new(1480.0, 320.0, 0.06));
    persistence.save_player_rating(party2_member.0, party2_member.1).await?;
    party_manager.add_member(party2.id, party2_member.0).await?;
    println!("Added member {} to party {}", party2_member.0, party2.id);
    
    // Add parties to queue
    println!("\nAdding parties to queue...");
    
    // Calculate party ratings and add to queue
    let party1_rating = party_manager.calculate_party_rating(party1.id).await?;
    let party1_metadata = EntryMetadata {
        roles: vec!["tank".to_string(), "healer".to_string(), "dps".to_string()],
        region: Some("us-east".to_string()),
        ..Default::default()
    };
    
    let party1_entry = queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party1.id,
        party1.member_ids.clone(),
        party1_rating,
        party1_metadata,
    ).await?;
    
    println!("Party {} joined queue (avg rating: {:.0})", party1.id, party1_rating.rating);
    
    let party2_rating = party_manager.calculate_party_rating(party2.id).await?;
    let party2_metadata = EntryMetadata {
        roles: vec!["dps".to_string(), "dps".to_string()],
        region: Some("us-east".to_string()),
        ..Default::default()
    };
    
    let party2_entry = queue_manager.join_queue_party(
        "team_5v5".to_string(),
        party2.id,
        party2.member_ids.clone(),
        party2_rating,
        party2_metadata,
    ).await?;
    
    println!("Party {} joined queue (avg rating: {:.0})", party2.id, party2_rating.rating);
    
    // Add some solo players to fill the match
    let solo_players = vec![
        (Uuid::new_v4(), Rating::new(1520.0, 290.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1490.0, 310.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1510.0, 295.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1470.0, 330.0, 0.06), "dps"),
        (Uuid::new_v4(), Rating::new(1530.0, 270.0, 0.06), "dps"),
    ];
    
    println!("\nAdding solo players to queue...");
    for (player_id, rating, role) in &solo_players {
        persistence.save_player_rating(*player_id, *rating).await?;
        
        let metadata = EntryMetadata {
            roles: vec![role.to_string()],
            region: Some("us-east".to_string()),
            ..Default::default()
        };
        
        queue_manager.join_queue_solo(
            "team_5v5".to_string(),
            *player_id,
            *rating,
            metadata,
        ).await?;
        
        println!("Solo player {} joined queue (rating: {:.0}, role: {})", player_id, rating.rating, role);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("team_5v5").await?;
    
    println!("Found {} matches", matches.len());
    
    for (i, match_result) in matches.iter().enumerate() {
        println!("\nMatch {}:", i + 1);
        println!("  Match ID: {}", match_result.match_id);
        
        // Show team composition
        let mut team_players: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
        
        for (entry_idx, entry) in match_result.entries.iter().enumerate() {
            let team_id = match_result.team_assignments[entry_idx];
            let team_players = team_players.entry(team_id).or_insert_with(Vec::new);
            
            if entry.is_solo() {
                team_players.push(format!("Solo {}", entry.player_ids[0]));
            } else {
                team_players.push(format!("Party {}", entry.party_id.unwrap()));
            }
        }
        
        for (team_id, players) in team_players {
            println!("  Team {}: {}", team_id + 1, players.join(", "));
        }
    }
    
    println!("\nParty matchmaking example completed successfully!");
    Ok(())
}
