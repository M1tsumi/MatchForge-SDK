//! Performance benchmarks for MatchForge SDK
//! 
//! These benchmarks measure the performance of various matchmaking operations
//! under different load scenarios.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use matchforge::prelude::*;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Benchmark basic matchmaking operations
fn bench_basic_matchmaking(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("basic_matchmaking_100_players", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let start = std::time::Instant::now();
                
                for _ in 0..iters {
                    let persistence = Arc::new(InMemoryAdapter::new());
                    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                    
                    // Register queue
                    queue_manager.register_queue(QueueConfig {
                        name: "test_queue".to_string(),
                        format: MatchFormat::one_v_one(),
                        constraints: MatchConstraints::permissive(),
                    }).await.unwrap();
                    
                    // Add 100 players
                    for i in 0..100 {
                        let player_id = Uuid::new_v4();
                        let rating = Rating::new(1500.0 + i as f64 * 10.0, 300.0, 0.06);
                        queue_manager.join_queue_solo(
                            "test_queue".to_string(),
                            player_id,
                            rating,
                            EntryMetadata::default(),
                        ).await.unwrap();
                    }
                    
                    // Find matches
                    let matches = queue_manager.find_matches("test_queue").await.unwrap();
                    black_box(matches);
                }
                
                start.elapsed()
            })
        })
    });
}

/// Benchmark queue operations at different scales
fn bench_queue_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("queue_scaling");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("join_queue", size), size, |b, &size| {
            b.iter_custom(|_| {
                rt.block_on(async {
                    let persistence = Arc::new(InMemoryAdapter::new());
                    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                    
                    queue_manager.register_queue(QueueConfig {
                        name: "scale_test".to_string(),
                        format: MatchFormat::one_v_one(),
                        constraints: MatchConstraints::permissive(),
                    }).await.unwrap();
                    
                    let start = std::time::Instant::now();
                    
                    // Add players
                    for i in 0..size {
                        let player_id = Uuid::new_v4();
                        let rating = Rating::new(1500.0 + i as f64 * 5.0, 300.0, 0.06);
                        queue_manager.join_queue_solo(
                            "scale_test".to_string(),
                            player_id,
                            rating,
                            EntryMetadata::default(),
                        ).await.unwrap();
                    }
                    
                    start.elapsed()
                })
            })
        });
        
        group.bench_with_input(BenchmarkId::new("find_matches", size), size, |b, &size| {
            b.iter_custom(|_| {
                rt.block_on(async {
                    let persistence = Arc::new(InMemoryAdapter::new());
                    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                    
                    queue_manager.register_queue(QueueConfig {
                        name: "scale_test".to_string(),
                        format: MatchFormat::one_v_one(),
                        constraints: MatchConstraints::permissive(),
                    }).await.unwrap();
                    
                    // Pre-populate queue
                    for i in 0..size {
                        let player_id = Uuid::new_v4();
                        let rating = Rating::new(1500.0 + i as f64 * 5.0, 300.0, 0.06);
                        queue_manager.join_queue_solo(
                            "scale_test".to_string(),
                            player_id,
                            rating,
                            EntryMetadata::default(),
                        ).await.unwrap();
                    }
                    
                    let start = std::time::Instant::now();
                    let matches = queue_manager.find_matches("scale_test").await.unwrap();
                    black_box(matches);
                    start.elapsed()
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark MMR calculations
fn bench_mmr_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mmr_calculations");
    
    // Benchmark Elo algorithm
    group.bench_function("elo_update_ratings", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            
            let elo_algorithm = EloAlgorithm::default();
            
            for _ in 0..iters {
                let ratings = vec![
                    Rating::new(1500.0, 300.0, 0.06),
                    Rating::new(1600.0, 280.0, 0.05),
                    Rating::new(1400.0, 320.0, 0.07),
                    Rating::new(1550.0, 290.0, 0.055),
                ];
                
                let outcomes = vec![
                    Outcome::Win,
                    Outcome::Loss,
                    Outcome::Win,
                    Outcome::Loss,
                ];
                
                let updated = elo_algorithm.update_ratings(&ratings, &outcomes);
                black_box(updated);
            }
            
            start.elapsed()
        })
    });
    
    // Benchmark Glicko2 algorithm
    group.bench_function("glicko2_update_ratings", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            
            let glicko2_algorithm = Glicko2Algorithm::default();
            
            for _ in 0..iters {
                let ratings = vec![
                    Rating::new(1500.0, 300.0, 0.06),
                    Rating::new(1600.0, 280.0, 0.05),
                    Rating::new(1400.0, 320.0, 0.07),
                    Rating::new(1550.0, 290.0, 0.055),
                ];
                
                let outcomes = vec![
                    Outcome::Win,
                    Outcome::Loss,
                    Outcome::Win,
                    Outcome::Loss,
                ];
                
                let updated = glicko2_algorithm.update_ratings(&ratings, &outcomes);
                black_box(updated);
            }
            
            start.elapsed()
        })
    });
    
    // Benchmark decay calculations
    group.bench_function("decay_calculations", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            
            let decay_strategy = LinearDecay::new(1.0, 100.0);
            
            for _ in 0..iters {
                let rating = Rating::new(1500.0, 300.0, 0.06);
                let last_match = chrono::Utc::now() - chrono::Duration::days(10);
                let decayed = decay_strategy.apply_decay(rating, last_match);
                black_box(decayed);
            }
            
            start.elapsed()
        })
    });
    
    group.finish();
}

/// Benchmark party operations
fn bench_party_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("party_creation_and_management", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let start = std::time::Instant::now();
                
                for _ in 0..iters {
                    let persistence = Arc::new(InMemoryAdapter::new());
                    let party_manager = Arc::new(PartyManager::new(
                        persistence.clone(),
                        Arc::new(AverageStrategy),
                    ));
                    
                    // Create party
                    let leader_id = Uuid::new_v4();
                    let party = party_manager.create_party(leader_id, 5).await.unwrap();
                    
                    // Add members
                    for i in 0..4 {
                        let member_id = Uuid::new_v4();
                        party_manager.add_member(party.id, member_id).await.unwrap();
                    }
                    
                    // Calculate party rating
                    let rating = party_manager.calculate_party_rating(party.id).await.unwrap();
                    black_box(rating);
                }
                
                start.elapsed()
            })
        })
    });
}

/// Benchmark persistence operations
fn bench_persistence_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("in_memory_persistence", |b| {
        b.iter_custom(|iters| {
            rt.block_on(async {
                let start = std::time::Instant::now();
                
                for _ in 0..iters {
                    let persistence = Arc::new(InMemoryAdapter::new());
                    let player_id = Uuid::new_v4();
                    let rating = Rating::new(1500.0, 300.0, 0.06);
                    
                    // Save rating
                    persistence.save_player_rating(player_id, rating).await.unwrap();
                    
                    // Load rating
                    let loaded = persistence.load_player_rating(player_id).await.unwrap();
                    black_box(loaded);
                }
                
                start.elapsed()
            })
        })
    });
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("concurrent_queue_joins", |b| {
        b.iter_custom(|_| {
            rt.block_on(async {
                let persistence = Arc::new(InMemoryAdapter::new());
                let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                
                queue_manager.register_queue(QueueConfig {
                    name: "concurrent_test".to_string(),
                    format: MatchFormat::one_v_one(),
                    constraints: MatchConstraints::permissive(),
                }).await.unwrap();
                
                let start = std::time::Instant::now();
                
                // Spawn 100 concurrent queue joins
                let mut handles = Vec::new();
                for _ in 0..100 {
                    let queue_manager = queue_manager.clone();
                    let handle = tokio::spawn(async move {
                        let player_id = Uuid::new_v4();
                        let rating = Rating::new(1500.0, 300.0, 0.06);
                        queue_manager.join_queue_solo(
                            "concurrent_test".to_string(),
                            player_id,
                            rating,
                            EntryMetadata::default(),
                        ).await
                    });
                    handles.push(handle);
                }
                
                // Wait for all to complete
                futures::future::join_all(handles).await;
                
                start.elapsed()
            })
        })
    });
}

/// Benchmark matchmaking runner performance
fn bench_matchmaking_runner(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("runner_tick_processing", |b| {
        b.iter_custom(|_| {
            rt.block_on(async {
                let persistence = Arc::new(InMemoryAdapter::new());
                let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                
                queue_manager.register_queue(QueueConfig {
                    name: "runner_test".to_string(),
                    format: MatchFormat::one_v_one(),
                    constraints: MatchConstraints::permissive(),
                }).await.unwrap();
                
                // Pre-populate with 200 players
                for i in 0..200 {
                    let player_id = Uuid::new_v4();
                    let rating = Rating::new(1500.0 + i as f64 * 10.0, 300.0, 0.06);
                    queue_manager.join_queue_solo(
                        "runner_test".to_string(),
                        player_id,
                        rating,
                        EntryMetadata::default(),
                    ).await.unwrap();
                }
                
                let runner = MatchmakingRunner::new(
                    RunnerConfig::default(),
                    queue_manager.clone(),
                    persistence.clone(),
                );
                
                let start = std::time::Instant::now();
                
                // Simulate one tick
                let matches = queue_manager.find_matches("runner_test").await.unwrap();
                black_box(matches);
                
                start.elapsed()
            })
        })
    });
}

/// Memory usage benchmark
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("memory_usage_1000_players", |b| {
        b.iter_custom(|_| {
            rt.block_on(async {
                let persistence = Arc::new(InMemoryAdapter::new());
                let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
                
                queue_manager.register_queue(QueueConfig {
                    name: "memory_test".to_string(),
                    format: MatchFormat::one_v_one(),
                    constraints: MatchConstraints::permissive(),
                }).await.unwrap();
                
                let start = std::time::Instant::now();
                
                // Add 1000 players
                for i in 0..1000 {
                    let player_id = Uuid::new_v4();
                    let rating = Rating::new(1500.0 + i as f64 * 5.0, 300.0, 0.06);
                    queue_manager.join_queue_solo(
                        "memory_test".to_string(),
                        player_id,
                        rating,
                        EntryMetadata::default(),
                    ).await.unwrap();
                }
                
                // Find matches
                let matches = queue_manager.find_matches("memory_test").await.unwrap();
                black_box(matches);
                
                start.elapsed()
            })
        })
    });
}

criterion_group!(
    benches,
    bench_basic_matchmaking,
    bench_queue_scaling,
    bench_mmr_calculations,
    bench_party_operations,
    bench_persistence_operations,
    bench_concurrent_operations,
    bench_matchmaking_runner,
    bench_memory_usage
);

criterion_main!(benches);
