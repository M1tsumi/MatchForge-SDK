//! Custom MMR algorithm example
//! 
//! This example shows how to implement a custom MMR algorithm
//! and use it with the matchmaking system.

use matchforge::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

/// Custom MMR algorithm that uses a more complex calculation
pub struct CustomMmrAlgorithm {
    k_factor_base: f64,
    k_factor_new_player: f64,
    new_player_threshold: i32, // Number of matches before considered "experienced"
}

impl CustomMmrAlgorithm {
    pub fn new() -> Self {
        Self {
            k_factor_base: 24.0,
            k_factor_new_player: 40.0,
            new_player_threshold: 30,
        }
    }

    fn calculate_k_factor(&self, player_experience: i32) -> f64 {
        if player_experience < self.new_player_threshold {
            self.k_factor_new_player
        } else {
            self.k_factor_base
        }
    }

    fn expected_score(&self, rating_a: f64, rating_b: f64) -> f64 {
        1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) / 400.0))
    }

    // In a real implementation, you'd track player experience/match count
    fn get_player_experience(&self, _player_id: Uuid) -> i32 {
        // For demo purposes, return a random value
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        _player_id.hash(&mut hasher);
        (hasher.finish() % 100) as i32
    }
}

#[async_trait]
impl MmrAlgorithm for CustomMmrAlgorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        // Get player experience (in real implementation, this would come from persistence)
        let player_experience = self.get_player_experience(Uuid::new_v4()); // Demo only
        let k_factor = self.calculate_k_factor(player_experience);

        let expected = self.expected_score(player_rating.rating, opponent_rating.rating);
        let actual = outcome.score();
        let rating_change = k_factor * (actual - expected);

        // Apply volatility adjustment based on performance
        let performance_factor = if (actual - expected).abs() > 0.3 {
            1.1 // Unexpected result, increase volatility
        } else {
            0.95 // Expected result, decrease volatility
        };

        let new_rating = player_rating.rating + rating_change;
        let new_deviation = (player_rating.deviation * 0.95).max(50.0); // Decrease uncertainty
        let new_volatility = (player_rating.volatility * performance_factor).min(0.2);

        Rating {
            rating: new_rating,
            deviation: new_deviation,
            volatility: new_volatility,
        }
    }

    fn name(&self) -> &str {
        "CustomEnhancedElo"
    }
}

/// Custom decay strategy that considers both time and performance
pub struct AdaptiveDecay {
    base_decay_per_day: f64,
    performance_modifier: f64,
}

impl AdaptiveDecay {
    pub fn new() -> Self {
        Self {
            base_decay_per_day: 2.0,
            performance_modifier: 0.5,
        }
    }
}

impl DecayStrategy for AdaptiveDecay {
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating {
        let now = Utc::now();
        let days_inactive = (now - last_match_time).num_days() as f64;

        if days_inactive <= 0.0 {
            return rating;
        }

        // Base decay
        let base_decay = self.base_decay_per_day * days_inactive;
        
        // Reduce decay for high-performing players (low volatility)
        let performance_bonus = if rating.volatility < 0.05 {
            self.performance_modifier * days_inactive
        } else {
            0.0
        };

        let total_decay = (base_decay - performance_bonus).max(0.0);

        Rating {
            rating: (rating.rating - total_decay).max(0.0),
            deviation: (rating.deviation + days_inactive * 0.8).min(350.0),
            volatility: (rating.volatility * 1.1).min(0.2),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize components with custom algorithms
    let persistence = Arc::new(InMemoryAdapter::new());
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    
    // Use custom MMR algorithm
    let custom_algorithm = Arc::new(CustomMmrAlgorithm::new());
    println!("Using custom MMR algorithm: {}", custom_algorithm.name());
    
    // Use custom decay strategy
    let decay_strategy = AdaptiveDecay::new();
    println!("Using adaptive decay strategy");
    
    // Configure queue with custom constraints
    let queue_config = QueueConfig {
        name: "custom_ranked".to_string(),
        format: MatchFormat::two_v_two(),
        constraints: MatchConstraints {
            max_rating_delta: 150.0,
            same_region_required: false,
            role_requirements: vec![],
            max_wait_time_seconds: 180,
            expansion_rate: 3.0,
        },
    };
    
    queue_manager.register_queue(queue_config).await?;
    
    // Create players with varying experience levels
    let players = vec![
        (Uuid::new_v4(), Rating::new(1800.0, 200.0, 0.04)), // Experienced player
        (Uuid::new_v4(), Rating::new(1200.0, 350.0, 0.08)), // New player
        (Uuid::new_v4(), Rating::new(1600.0, 250.0, 0.05)), // Moderate experience
        (Uuid::new_v4(), Rating::new(1400.0, 300.0, 0.06)), // Moderate experience
    ];
    
    println!("\nAdding players with different experience levels...");
    for (player_id, rating) in &players {
        let entry = queue_manager.join_queue_solo(
            "custom_ranked".to_string(),
            *player_id,
            *rating,
            EntryMetadata::default(),
        ).await?;
        
        println!("Player {} - Rating: {:.0}, Deviation: {:.0}, Volatility: {:.3}", 
            player_id, rating.rating, rating.deviation, rating.volatility);
    }
    
    // Process matchmaking
    println!("\nProcessing matchmaking...");
    let matches = queue_manager.find_matches("custom_ranked").await?;
    
    println!("Found {} matches", matches.len());
    
    if let Some(match_result) = matches.first() {
        println!("\nMatch details:");
        for (i, entry) in match_result.entries.iter().enumerate() {
            println!("  Player {}: {:.0} (deviation: {:.0})", 
                i + 1, entry.average_rating.rating, entry.average_rating.deviation);
        }
        
        // Simulate rating updates with custom algorithm
        println!("\nUpdating ratings with custom algorithm...");
        
        let outcomes = vec![
            (match_result.entries[0].player_ids[0], Outcome::Win),
            (match_result.entries[1].player_ids[0], Outcome::Win),
            (match_result.entries[2].player_ids[0], Outcome::Loss),
            (match_result.entries[3].player_ids[0], Outcome::Loss),
        ];
        
        // Store original ratings for comparison
        let mut original_ratings = std::collections::HashMap::new();
        for (player_id, _) in &outcomes {
            if let Ok(Some(rating)) = persistence.load_player_rating(*player_id).await {
                original_ratings.insert(*player_id, rating);
            }
        }
        
        // Update ratings (simplified - in real implementation you'd use LobbyManager)
        for (player_a_idx, (player_a_id, outcome_a)) in outcomes.iter().enumerate() {
            for (player_b_idx, (player_b_id, outcome_b)) in outcomes.iter().enumerate() {
                if player_a_idx >= player_b_idx {
                    continue;
                }
                
                let rating_a = original_ratings[player_a_id];
                let rating_b = original_ratings[player_b_id];
                
                // Determine outcome for this matchup
                let matchup_outcome = if outcome_a.score() > outcome_b.score() {
                    Outcome::Win
                } else {
                    Outcome::Loss
                };
                
                let new_rating_a = custom_algorithm.calculate_new_rating(rating_a, rating_b, matchup_outcome);
                let new_rating_b = custom_algorithm.calculate_new_rating(rating_b, rating_a, 
                    if matchup_outcome == Outcome::Win { Outcome::Loss } else { Outcome::Win });
                
                persistence.save_player_rating(*player_a_id, new_rating_a).await?;
                persistence.save_player_rating(*player_b_id, new_rating_b).await?;
            }
        }
        
        // Show rating changes
        println!("\nRating changes:");
        for (player_id, original_rating) in &original_ratings {
            if let Ok(Some(new_rating)) = persistence.load_player_rating(*player_id).await {
                let change = new_rating.rating - original_rating.rating;
                println!("  Player {}: {:.0} → {:.0} ({:+.0})", 
                    player_id, original_rating.rating, new_rating.rating, change);
            }
        }
        
        // Demonstrate decay
        println!("\nApplying adaptive decay to inactive players...");
        let old_time = Utc::now() - chrono::Duration::days(10);
        let inactive_rating = Rating::new(1500.0, 200.0, 0.04);
        let decayed_rating = decay_strategy.apply_decay(inactive_rating, old_time);
        
        println!("  Before decay: {:.0} (deviation: {:.0}, volatility: {:.3})", 
            inactive_rating.rating, inactive_rating.deviation, inactive_rating.volatility);
        println!("  After 10 days: {:.0} (deviation: {:.0}, volatility: {:.3})", 
            decayed_rating.rating, decayed_rating.deviation, decayed_rating.volatility);
    }
    
    println!("\nCustom MMR example completed successfully!");
    Ok(())
}
