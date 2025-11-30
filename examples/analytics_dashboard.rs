//! Analytics Dashboard Example
//! 
//! This example demonstrates how to use the MatchForge analytics system
//! to create a comprehensive dashboard with real-time metrics and insights.

use matchforge::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🎮 MatchForge Analytics Dashboard Example");
    println!("==========================================");
    
    // Create persistence layer
    let persistence = Arc::new(InMemoryAdapter::new());
    
    // Initialize analytics components
    let analytics = Arc::new(AnalyticsMetrics::new(AnalyticsConfig::default()));
    let report_generator = Arc::new(ReportGenerator::new(analytics.clone()));
    let insight_engine = Arc::new(InsightEngine::new(analytics.clone()));
    let dashboard_data = Arc::new(DashboardData::new(
        analytics.clone(),
        report_generator.clone(),
        insight_engine.clone(),
    ));
    
    // Create core matchmaking components
    let queue_manager = Arc::new(QueueManager::new(persistence.clone()));
    let party_manager = Arc::new(PartyManager::new(
        persistence.clone(),
        Arc::new(AverageStrategy),
    ));
    let lobby_manager = Arc::new(LobbyManager::new(persistence.clone()));
    
    // Register queues
    let queues = vec![
        QueueConfig {
            name: "casual_1v1".to_string(),
            format: MatchFormat::one_v_one(),
            constraints: MatchConstraints::permissive(),
        },
        QueueConfig {
            name: "ranked_1v1".to_string(),
            format: MatchFormat::one_v_one(),
            constraints: MatchConstraints::strict(),
        },
        QueueConfig {
            name: "competitive_5v5".to_string(),
            format: MatchFormat::team_v_team(5),
            constraints: MatchConstraints {
                max_rating_difference: 200,
                max_wait_time: Duration::from_secs(300),
                role_requirements: vec![
                    RoleRequirement { role: "tank".to_string(), required: true },
                    RoleRequirement { role: "healer".to_string(), required: true },
                ],
            },
        },
    ];
    
    for queue_config in queues {
        queue_manager.register_queue(queue_config).await?;
        println!("✅ Registered queue");
    }
    
    // Simulate player activity
    println!("\n📊 Simulating player activity...");
    simulate_player_activity(
        queue_manager.clone(),
        party_manager.clone(),
        analytics.clone(),
    ).await?;
    
    // Generate insights
    println!("\n🔍 Generating insights...");
    let insights = insight_engine.generate_insights().await?;
    
    println!("Generated {} insights:", insights.len());
    for (i, insight) in insights.iter().take(5).enumerate() {
        println!("  {}. {} (Severity: {:?}, Confidence: {:.1}%)",
            i + 1,
            insight.title,
            insight.severity,
            insight.confidence * 100.0
        );
        println!("     {}", insight.description);
        if !insight.recommendations.is_empty() {
            println!("     💡 Recommendation: {}", insight.recommendations[0].title);
        }
        println!();
    }
    
    // Generate reports
    println!("📈 Generating reports...");
    let performance_report = report_generator.generate_report(
        ReportType::Performance,
        None,
        ReportFormat::Json,
    ).await?;
    
    println!("✅ Performance report generated: {}", performance_report.title);
    println!("   Data points: {}", performance_report.metadata.data_points);
    println!("   Recommendations: {}", performance_report.recommendations.len());
    
    // Generate dashboard
    println!("\n🎛️  Generating dashboard...");
    let dashboard = dashboard_data.generate_dashboard(None).await?;
    
    println!("✅ Dashboard generated: {}", dashboard.title);
    println!("   Widgets: {}", dashboard.widgets.len());
    println!("   Time range: {} to {}",
        dashboard.time_range.start.format("%Y-%m-%d %H:%M"),
        dashboard.time_range.end.format("%Y-%m-%d %H:%M")
    );
    
    // Display dashboard widgets
    println!("\n📊 Dashboard Widgets:");
    for (i, widget) in dashboard.widgets.iter().enumerate() {
        println!("  {}. {} ({:?})",
            i + 1,
            widget.title,
            widget.widget_type
        );
        
        match &widget.data {
            WidgetData::KPI(kpi) => {
                println!("     Value: {:.1} {} (Trend: {:?})",
                    kpi.value,
                    kpi.unit,
                    kpi.trend
                );
            }
            WidgetData::Chart(chart) => {
                println!("     Chart type: {:?}, Datasets: {}",
                    chart.chart_type,
                    chart.datasets.len()
                );
            }
            WidgetData::Alert(alerts) => {
                println!("     Active alerts: {}", alerts.alerts.len());
            }
            WidgetData::Insight(insights) => {
                println!("     Recent insights: {}", insights.insights.len());
            }
            _ => {
                println!("     Data available");
            }
        }
    }
    
    // Get metrics snapshot
    println!("\n📊 Current Metrics Snapshot:");
    let snapshot = analytics.get_metrics_snapshot().await;
    println!("   Total players: {}", snapshot.total_players);
    println!("   Active players: {}", snapshot.active_players);
    println!("   Total matches: {}", snapshot.total_matches);
    println!("   Average wait time: {:.2} seconds", snapshot.average_wait_time.as_secs_f64());
    println!("   Match quality score: {:.2}", snapshot.match_quality_score);
    println!("   Memory usage: {} MB", snapshot.memory_usage_mb);
    println!("   CPU usage: {:.1}%", snapshot.cpu_usage_percent);
    
    // Generate hourly aggregation
    println!("\n⏰ Hourly Aggregation:");
    let hourly_metrics = analytics.generate_hourly_aggregation().await;
    println!("   Active players: {}", hourly_metrics.active_players);
    println!("   Matches completed: {}", hourly_metrics.matches_completed);
    println!("   Average wait time: {:.2} seconds", hourly_metrics.average_wait_time);
    println!("   New players: {}", hourly_metrics.new_players);
    
    // Predictive analytics
    println!("\n🔮 Predictive Analytics:");
    let predicted_wait_time = analytics.predict_queue_wait_time("casual_1v1", 1500.0).await;
    println!("   Predicted wait time for 1500 rating: {:.1} seconds",
        predicted_wait_time.as_secs_f64()
    );
    
    // Retention analytics
    println!("\n👥 Retention Analytics:");
    let retention_analytics = analytics.get_retention_analytics().await;
    println!("   Day 1 retention: {:.1}%", retention_analytics.day_1_retention * 100.0);
    println!("   Day 7 retention: {:.1}%", retention_analytics.day_7_retention * 100.0);
    println!("   Day 30 retention: {:.1}%", retention_analytics.day_30_retention * 100.0);
    println!("   Average session duration: {:.1} minutes",
        retention_analytics.average_session_duration.as_secs_f64() / 60.0
    );
    println!("   Churn rate: {:.1}%", retention_analytics.churn_rate * 100.0);
    
    // Export dashboard data
    println!("\n💾 Exporting dashboard data...");
    let dashboard_json = serde_json::to_string_pretty(&dashboard)?;
    
    // Save to file
    tokio::fs::write("dashboard_export.json", dashboard_json).await?;
    println!("✅ Dashboard exported to dashboard_export.json");
    
    // Export insights
    let insights_json = serde_json::to_string_pretty(&insights)?;
    tokio::fs::write("insights_export.json", insights_json).await?;
    println!("✅ Insights exported to insights_export.json");
    
    println!("\n🎉 Analytics dashboard example completed successfully!");
    println!("📁 Check the exported files for detailed analytics data.");
    
    Ok(())
}

/// Simulate player activity for demonstration
async fn simulate_player_activity(
    queue_manager: Arc<QueueManager>,
    party_manager: Arc<PartyManager>,
    analytics: Arc<AnalyticsMetrics>,
) -> Result<()> {
    let mut player_count = 0;
    
    // Simulate players joining queues
    for i in 0..50 {
        let player_id = Uuid::new_v4();
        let rating = Rating::new(1200.0 + i as f64 * 20.0, 300.0, 0.06);
        
        // Record new player
        analytics.record_player_activity(player_id, PlayerActivityType::NewPlayer).await;
        player_count += 1;
        
        // Join random queue
        let queue_names = ["casual_1v1", "ranked_1v1", "competitive_5v5"];
        let queue_name = queue_names[i % 3];
        
        let metadata = EntryMetadata {
            preferred_roles: vec![
                if i % 3 == 0 { "tank".to_string() }
                else if i % 3 == 1 { "healer".to_string() }
                else { "dps".to_string() }
            ],
            avoided_players: vec![],
            custom_attributes: HashMap::new(),
        };
        
        let entry = queue_manager.join_queue_solo(
            queue_name.to_string(),
            player_id,
            rating,
            metadata,
        ).await?;
        
        // Record queue activity
        analytics.record_queue_activity(
            queue_name.to_string(),
            QueueActivity::PlayerJoined,
        ).await;
        
        // Simulate some wait times and matches
        if i % 5 == 0 {
            // Simulate match found
            let wait_time = Duration::from_secs(10 + (i % 30) as u64);
            analytics.record_queue_activity(
                queue_name.to_string(),
                QueueActivity::MatchFound(wait_time),
            ).await;
            
            // Record match completion
            let match_data = MatchCompletionData {
                match_id: Uuid::new_v4(),
                average_rating: rating.rating,
                quality_score: 0.7 + (i % 3) as f64 * 0.1,
                duration: Duration::from_secs(300 + (i % 600) as u64),
                rating_changes: vec![],
            };
            analytics.record_match_completed(match_data).await;
        } else if i % 7 == 0 {
            // Simulate player leaving queue
            let wait_time = Duration::from_secs(30 + (i % 120) as u64);
            analytics.record_queue_activity(
                queue_name.to_string(),
                QueueActivity::PlayerLeft(wait_time),
            ).await;
        }
        
        // Create some parties
        if i % 8 == 0 && i > 0 {
            let party = party_manager.create_party(player_id, 3).await?;
            analytics.record_party_activity(3, PartyActivity::Created).await;
            
            // Add some members
            for j in 0..2 {
                let member_id = Uuid::new_v4();
                party_manager.add_member(party.id, member_id).await?;
            }
            
            analytics.record_party_activity(3, PartyActivity::MatchFound(true)).await;
        }
        
        // Simulate performance metrics
        analytics.record_performance(PerformanceMetric::ApiResponseTime(
            Duration::from_millis(50 + (i % 200) as u64)
        )).await;
        
        analytics.record_performance(PerformanceMetric::DatabaseQueryTime(
            Duration::from_millis(10 + (i % 50) as u64)
        )).await;
        
        if i % 10 == 0 {
            analytics.record_performance(PerformanceMetric::MemoryUsage(
                512 + (i % 1024) * 1024 * 1024 // MB to bytes
            )).await;
            
            analytics.record_performance(PerformanceMetric::CpuUsage(
                20.0 + (i % 60) as f64
            )).await;
        }
        
        // Small delay to simulate real-time activity
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Simulate some player sessions
    for i in 0..20 {
        let session_duration = Duration::from_secs(300 + (i % 1800) as u64); // 5-35 minutes
        // In a real implementation, this would track actual session data
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    
    println!("✅ Simulated {} players with various activities", player_count);
    Ok(())
}
