//! Advanced Analytics Example
//! 
//! This example demonstrates advanced analytics features including
//! custom reports, ML-based insights, and real-time monitoring.

use matchforge::prelude::*;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    println!("🧠 MatchForge Advanced Analytics Example");
    println!("=======================================");
    
    // Create persistence layer
    let persistence = Arc::new(InMemoryAdapter::new());
    
    // Initialize advanced analytics components
    let analytics_config = AnalyticsConfig {
        retention_period: Duration::days(90),
        aggregation_interval: Duration::from_hours(1),
        max_data_points: 10000,
        enable_detailed_tracking: true,
        enable_predictive_analytics: true,
    };
    
    let analytics = Arc::new(AnalyticsMetrics::new(analytics_config));
    let report_generator = Arc::new(ReportGenerator::new(analytics.clone()));
    let insight_engine = Arc::new(InsightEngine::new(analytics.clone()));
    
    // Create monitoring service
    let monitoring_config = MonitoringConfig {
        metrics_interval: Duration::from_secs(10),
        metrics_retention: Duration::from_hours(24),
        alert_thresholds: AlertThresholds {
            max_wait_time_ms: 30000,
            min_success_rate: 0.8,
            max_queue_size: 1000,
            max_memory_usage_mb: 2000,
            max_cpu_usage_percent: 80.0,
        },
        health_checks: HealthCheckConfig {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            retries: 3,
        },
    };
    
    let metrics_collector = Arc::new(DefaultMetricsCollector::new());
    let event_collector = Arc::new(MemoryEventCollector::new(10000));
    let monitoring_service = Arc::new(MonitoringService::new(
        monitoring_config,
        metrics_collector.clone(),
        event_collector.clone(),
    ));
    
    // Start monitoring
    let monitoring_handle = {
        let monitoring_service = monitoring_service.clone();
        tokio::spawn(async move {
            if let Err(e) = monitoring_service.start().await {
                eprintln!("Monitoring service error: {}", e);
            }
        })
    };
    
    // Simulate comprehensive activity
    println!("\n📊 Simulating comprehensive activity...");
    simulate_comprehensive_activity(analytics.clone()).await?;
    
    // Generate comprehensive reports
    println!("\n📈 Generating comprehensive reports...");
    
    let report_types = vec![
        ReportType::Performance,
        ReportType::PlayerAnalytics,
        ReportType::QueueAnalytics,
        ReportType::RatingAnalytics,
        ReportType::PartyAnalytics,
        ReportType::SystemHealth,
        ReportType::BusinessAnalytics,
    ];
    
    let mut reports = Vec::new();
    for report_type in report_types {
        let report = report_generator.generate_report(
            report_type.clone(),
            None,
            ReportFormat::Json,
        ).await?;
        
        reports.push(report.clone());
        println!("✅ Generated {}: {} recommendations",
            report.title,
            report.recommendations.len()
        );
        
        // Display key insights
        if !report.data.summary.key_insights.is_empty() {
            println!("   Key insights:");
            for insight in &report.data.summary.key_insights {
                println!("     - {}", insight);
            }
        }
    }
    
    // Advanced insights analysis
    println!("\n🔍 Advanced insights analysis...");
    let insights = insight_engine.generate_insights().await?;
    
    // Group insights by type
    let mut insights_by_type: HashMap<InsightType, Vec<_>> = HashMap::new();
    for insight in &insights {
        insights_by_type.entry(insight.insight_type.clone())
            .or_insert_with(Vec::new)
            .push(insight);
    }
    
    println!("Insights breakdown:");
    for (insight_type, type_insights) in insights_by_type {
        println!("  {:?}: {} insights", insight_type, type_insights.len());
        
        // Show highest priority insight for each type
        if let Some(highest_priority) = type_insights.iter().max_by(|a, b| {
            a.severity.cmp(&b.severity).then(b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        }) {
            println!("    Priority: {} (Severity: {:?}, Confidence: {:.1}%)",
                highest_priority.title,
                highest_priority.severity,
                highest_priority.confidence * 100.0
            );
        }
    }
    
    // Predictive analytics
    println!("\n🔮 Predictive analytics...");
    
    // Predict queue overflow
    let snapshot = analytics.get_metrics_snapshot().await;
    let total_queue_size: u64 = snapshot.queue_sizes.values().sum();
    
    if total_queue_size > 0 {
        let growth_rate = calculate_growth_rate(&analytics).await;
        let predicted_time_to_overflow = if growth_rate > 0.0 {
            Duration::from_secs_f64((1000.0 - total_queue_size as f64) / (total_queue_size as f64 * growth_rate / 3600.0))
        } else {
            Duration::from_secs(u64::MAX)
        };
        
        println!("Current queue size: {}", total_queue_size);
        println!("Growth rate: {:.2}% per hour", growth_rate * 100.0);
        
        if predicted_time_to_overflow < Duration::from_days(1) {
            println!("⚠️  Predicted queue overflow in: {:.1} hours",
                predicted_time_to_overflow.as_secs_f64()
            );
        } else {
            println!("✅ No queue overflow predicted in next 24 hours");
        }
    }
    
    // Predict player churn
    let retention = analytics.get_retention_analytics().await;
    let churn_risk = calculate_churn_risk(&retention).await;
    
    println!("Churn risk analysis:");
    println!("  Current churn rate: {:.1}%", retention.churn_rate * 100.0);
    println!("  Predicted churn risk: {:.1}%", churn_risk * 100.0);
    
    if churn_risk > retention.churn_rate * 1.2 {
        println!("⚠️  Increasing churn risk detected!");
    }
    
    // Performance optimization recommendations
    println!("\n⚡ Performance optimization recommendations...");
    
    let performance_recommendations = generate_performance_recommendations(&snapshot).await;
    for (i, recommendation) in performance_recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, recommendation.title);
        println!("     {}", recommendation.description);
        println!("     Expected impact: {:?}", recommendation.impact);
        println!("     Effort required: {:?}", recommendation.effort);
        println!();
    }
    
    // Business intelligence
    println!("💼 Business intelligence...");
    
    let business_insights = generate_business_insights(&analytics).await;
    for insight in &business_insights {
        println!("  • {}", insight);
    }
    
    // System health check
    println!("\n🏥 System health check...");
    let health_status = monitoring_service.get_system_health().await?;
    
    println!("Overall health: {:?}", health_status.overall_status);
    println!("Component health:");
    for (component, status) in &health_status.component_status {
        println!("  {:?}: {:?}", component, status.status);
    }
    
    // Alert analysis
    println!("\n🚨 Alert analysis...");
    let alerts = monitoring_service.get_alerts().await?;
    
    println!("Active alerts: {}", alerts.len());
    for alert in alerts.iter().take(5) {
        println!("  [{:?}] {} - {}",
            alert.level,
            alert.title,
            alert.message
        );
    }
    
    // Export comprehensive analytics data
    println!("\n💾 Exporting comprehensive analytics data...");
    
    // Create comprehensive export
    let export_data = AnalyticsExport {
        timestamp: Utc::now(),
        metrics_snapshot: snapshot,
        insights,
        reports,
        health_status,
        retention_analytics: retention,
        business_insights,
        performance_recommendations,
    };
    
    let export_json = serde_json::to_string_pretty(&export_data)?;
    tokio::fs::write("advanced_analytics_export.json", export_json).await?;
    println!("✅ Comprehensive analytics exported to advanced_analytics_export.json");
    
    // Generate summary report
    println!("\n📊 Summary Report:");
    println!("================");
    println!("Total Players: {}", snapshot.total_players);
    println!("Active Players: {}", snapshot.active_players);
    println!("Total Matches: {}", snapshot.total_matches);
    println!("Average Wait Time: {:.2}s", snapshot.average_wait_time.as_secs_f64());
    println!("Match Quality Score: {:.2}", snapshot.match_quality_score);
    println!("System Health: {:?}", health_status.overall_status);
    println!("Active Insights: {}", insights.len());
    println!("Generated Recommendations: {}", reports.iter().map(|r| r.recommendations.len()).sum::<usize>());
    
    // Wait for monitoring to complete
    let _ = monitoring_handle.await;
    
    println!("\n🎉 Advanced analytics example completed successfully!");
    println!("📁 Check advanced_analytics_export.json for comprehensive analytics data.");
    
    Ok(())
}

/// Simulate comprehensive activity for advanced analytics
async fn simulate_comprehensive_activity(analytics: Arc<AnalyticsMetrics>) -> Result<()> {
    println!("Simulating diverse activity patterns...");
    
    // Simulate different player segments
    let player_segments = vec![
        ("new_players", 20, 1200.0, 0.8), // (segment, count, avg_rating, retention_rate)
        ("casual_players", 50, 1400.0, 0.6),
        ("competitive_players", 30, 1600.0, 0.7),
        ("veteran_players", 15, 1800.0, 0.9),
    ];
    
    for (segment, count, avg_rating, retention_rate) in player_segments {
        for i in 0..count {
            let player_id = Uuid::new_v4();
            let rating = Rating::new(
                avg_rating + (i as f64 - count as f64 / 2.0) * 10.0,
                300.0 - (i as f64 * 2.0),
                0.06,
            );
            
            // Record player activity
            analytics.record_player_activity(player_id, PlayerActivityType::NewPlayer).await;
            
            // Simulate different behavior patterns based on segment
            match segment {
                "new_players" => {
                    // New players have shorter sessions and higher abandonment
                    simulate_new_player_behavior(&analytics, player_id, rating).await;
                }
                "casual_players" => {
                    // Casual players play irregularly
                    simulate_casual_player_behavior(&analytics, player_id, rating).await;
                }
                "competitive_players" => {
                    // Competitive players are more consistent
                    simulate_competitive_player_behavior(&analytics, player_id, rating).await;
                }
                "veteran_players" => {
                    // Veteran players have high engagement
                    simulate_veteran_player_behavior(&analytics, player_id, rating).await;
                }
                _ => {}
            }
            
            // Simulate retention outcome
            if rand::random::<f64>() > retention_rate {
                analytics.record_player_activity(player_id, PlayerActivityType::Logout).await;
            }
        }
    }
    
    // Simulate system performance variations
    simulate_system_performance_variations(&analytics).await;
    
    println!("✅ Comprehensive activity simulation completed");
    Ok(())
}

/// Simulate new player behavior
async fn simulate_new_player_behavior(analytics: &AnalyticsMetrics, player_id: Uuid, rating: Rating) {
    // New players often leave queues early
    let wait_time = Duration::from_secs(20 + rand::random::<u64>() % 40);
    analytics.record_queue_activity(
        "casual_1v1".to_string(),
        QueueActivity::PlayerLeft(wait_time),
    ).await;
    
    // Short session duration
    let session_duration = Duration::from_secs(300 + rand::random::<u64>() % 600); // 5-15 minutes
    // In a real implementation, this would track actual session data
}

/// Simulate casual player behavior
async fn simulate_casual_player_behavior(analytics: &AnalyticsMetrics, player_id: Uuid, rating: Rating) {
    // Casual players have varied wait times
    let wait_time = Duration::from_secs(10 + rand::random::<u64>() % 120);
    
    if rand::random::<f64>() > 0.3 {
        analytics.record_queue_activity(
            "casual_1v1".to_string(),
            QueueActivity::MatchFound(wait_time),
        ).await;
        
        // Record match completion
        let match_data = MatchCompletionData {
            match_id: Uuid::new_v4(),
            average_rating: rating.rating,
            quality_score: 0.6 + rand::random::<f64>() * 0.3,
            duration: Duration::from_secs(600 + rand::random::<u64>() % 600),
            rating_changes: vec![],
        };
        analytics.record_match_completed(match_data).await;
    } else {
        analytics.record_queue_activity(
            "casual_1v1".to_string(),
            QueueActivity::PlayerLeft(wait_time),
        ).await;
    }
}

/// Simulate competitive player behavior
async fn simulate_competitive_player_behavior(analytics: &AnalyticsMetrics, player_id: Uuid, rating: Rating) {
    // Competitive players are more patient
    let wait_time = Duration::from_secs(30 + rand::random::<u64>() % 90);
    
    analytics.record_queue_activity(
        "ranked_1v1".to_string(),
        QueueActivity::MatchFound(wait_time),
    ).await;
    
    // Higher quality matches
    let match_data = MatchCompletionData {
        match_id: Uuid::new_v4(),
        average_rating: rating.rating,
        quality_score: 0.8 + rand::random::<f64>() * 0.2,
        duration: Duration::from_secs(900 + rand::random::<u64>() % 900),
        rating_changes: vec![],
    };
    analytics.record_match_completed(match_data).await;
}

/// Simulate veteran player behavior
async fn simulate_veteran_player_behavior(analytics: &AnalyticsMetrics, player_id: Uuid, rating: Rating) {
    // Veterans often play in parties
    analytics.record_party_activity(3, PartyActivity::Created).await;
    
    let wait_time = Duration::from_secs(15 + rand::random::<u64>() % 45);
    analytics.record_queue_activity(
        "competitive_5v5".to_string(),
        QueueActivity::MatchFound(wait_time),
    ).await;
    
    analytics.record_party_activity(3, PartyActivity::MatchFound(true)).await;
    
    // High quality matches
    let match_data = MatchCompletionData {
        match_id: Uuid::new_v4(),
        average_rating: rating.rating,
        quality_score: 0.85 + rand::random::<f64>() * 0.15,
        duration: Duration::from_secs(1200 + rand::random::<u64>() % 600),
        rating_changes: vec![],
    };
    analytics.record_match_completed(match_data).await;
}

/// Simulate system performance variations
async fn simulate_system_performance_variations(analytics: &AnalyticsMetrics) {
    for i in 0..100 {
        // Simulate varying API response times
        let api_time = Duration::from_millis(50 + (i % 200) as u64);
        analytics.record_performance(PerformanceMetric::ApiResponseTime(api_time)).await;
        
        // Simulate varying database query times
        let db_time = Duration::from_millis(5 + (i % 50) as u64);
        analytics.record_performance(PerformanceMetric::DatabaseQueryTime(db_time)).await;
        
        // Simulate memory usage patterns
        let memory_usage = 512 + (i as f64 * 10.0).sin() * 256.0 + 512.0; // Oscillating pattern
        analytics.record_performance(PerformanceMetric::MemoryUsage(
            memory_usage as u64 * 1024 * 1024 // Convert to bytes
        )).await;
        
        // Simulate CPU usage patterns
        let cpu_usage = 40.0 + (i as f64 * 0.1).sin() * 30.0; // Oscillating pattern
        analytics.record_performance(PerformanceMetric::CpuUsage(cpu_usage)).await;
        
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Calculate growth rate from analytics data
async fn calculate_growth_rate(analytics: &AnalyticsMetrics) -> f64 {
    // In a real implementation, this would analyze historical data
    // For now, return a simulated growth rate
    0.05 // 5% growth per hour
}

/// Calculate churn risk
async fn calculate_churn_risk(retention: &RetentionAnalytics) -> f64 {
    // Simple churn risk calculation based on retention metrics
    let base_risk = 1.0 - retention.day_7_retention;
    let session_factor = if retention.average_session_duration < Duration::from_minutes(15) {
        0.2
    } else {
        0.0
    };
    
    (base_risk + session_factor).min(1.0)
}

/// Generate performance recommendations
async fn generate_performance_recommendations(snapshot: &MetricsSnapshot) -> Vec<Recommendation> {
    let mut recommendations = Vec::new();
    
    if snapshot.average_wait_time > Duration::from_secs(30) {
        recommendations.push(Recommendation {
            id: Uuid::new_v4(),
            title: "Optimize Matchmaking Algorithm".to_string(),
            description: "Current wait times exceed 30 seconds. Consider algorithm optimization.".to_string(),
            priority: Priority::High,
            impact: Impact::High,
            effort: Effort::Medium,
            actions: vec![
                "Implement parallel matchmaking".to_string(),
                "Optimize data structures".to_string(),
                "Add caching layer".to_string(),
            ],
            expected_outcome: "Reduce wait times by 40%".to_string(),
            success_probability: 0.8,
        });
    }
    
    if snapshot.memory_usage_mb > 1500 {
        recommendations.push(Recommendation {
            id: Uuid::new_v4(),
            title: "Reduce Memory Footprint".to_string(),
            description: "Memory usage is high. Implement memory optimization strategies.".to_string(),
            priority: Priority::Medium,
            impact: Impact::Medium,
            effort: Effort::Low,
            actions: vec![
                "Optimize data structures".to_string(),
                "Implement object pooling".to_string(),
                "Add memory monitoring".to_string(),
            ],
            expected_outcome: "Reduce memory usage by 25%".to_string(),
            success_probability: 0.9,
        });
    }
    
    if snapshot.cpu_usage_percent > 70.0 {
        recommendations.push(Recommendation {
            id: Uuid::new_v4(),
            title: "Optimize CPU Usage".to_string(),
            description: "CPU usage is elevated. Consider performance optimizations.".to_string(),
            priority: Priority::Medium,
            impact: Impact::Medium,
            effort: Effort::Medium,
            actions: vec![
                "Profile hot paths".to_string(),
                "Optimize algorithms".to_string(),
                "Consider horizontal scaling".to_string(),
            ],
            expected_outcome: "Reduce CPU usage by 20%".to_string(),
            success_probability: 0.7,
        });
    }
    
    recommendations
}

/// Generate business insights
async fn generate_business_insights(analytics: &AnalyticsMetrics) -> Vec<String> {
    let mut insights = Vec::new();
    
    let snapshot = analytics.get_metrics_snapshot().await;
    let retention = analytics.get_retention_analytics().await;
    
    // Revenue insights
    let daily_revenue = snapshot.active_players as f64 * 2.5; // $2.50 per player per day
    let monthly_revenue = daily_revenue * 30.0;
    
    insights.push(format!("Estimated daily revenue: ${:.2}", daily_revenue));
    insights.push(format!("Estimated monthly revenue: ${:.2}", monthly_revenue));
    
    // Player value insights
    let player_ltv = calculate_ltv(&retention, &snapshot);
    insights.push(format!("Average player lifetime value: ${:.2}", player_ltv));
    
    // Engagement insights
    let engagement_rate = snapshot.active_players as f64 / snapshot.total_players as f64;
    insights.push(format!("Player engagement rate: {:.1}%", engagement_rate * 100.0));
    
    // Growth insights
    let daily_new_players = snapshot.new_players_today;
    insights.push(format!("Daily new player acquisition: {}", daily_new_players));
    
    insights
}

/// Calculate customer lifetime value
fn calculate_ltv(retention: &RetentionAnalytics, snapshot: &MetricsSnapshot) -> f64 {
    let avg_revenue_per_player = 2.5; // $2.50 per day
    let avg_lifetime_days = if retention.churn_rate > 0.0 {
        1.0 / retention.churn_rate
    } else {
        365.0 // Default to 1 year if no churn
    };
    
    avg_revenue_per_player * avg_lifetime_days
}

/// Comprehensive analytics export structure
#[derive(Debug, Serialize, Deserialize)]
struct AnalyticsExport {
    timestamp: DateTime<Utc>,
    metrics_snapshot: MetricsSnapshot,
    insights: Vec<Insight>,
    reports: Vec<Report>,
    health_status: SystemHealth,
    retention_analytics: RetentionAnalytics,
    business_insights: Vec<String>,
    performance_recommendations: Vec<Recommendation>,
}

// Re-import types needed for the example
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// Mock types that would be imported from the analytics module
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    title: String,
    recommendations: Vec<Recommendation>,
    data: ReportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportData {
    summary: ReportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportSummary {
    key_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Insight {
    id: Uuid,
    insight_type: InsightType,
    title: String,
    description: String,
    severity: Severity,
    confidence: f64,
    recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Recommendation {
    id: Uuid,
    title: String,
    description: String,
    priority: Priority,
    impact: Impact,
    effort: Effort,
    actions: Vec<String>,
    expected_outcome: String,
    success_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemHealth {
    overall_status: HealthStatus,
    component_status: HashMap<HealthComponent, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComponentHealth {
    status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetentionAnalytics {
    day_1_retention: f64,
    day_7_retention: f64,
    day_30_retention: f64,
    average_session_duration: Duration,
    churn_rate: f64,
}

// Mock enums and types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum InsightType {
    QueuePerformance,
    PlayerBehavior,
    RatingSystem,
    SystemPerformance,
    BusinessMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Impact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum Effort {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum HealthComponent {
    Database,
    Redis,
    Queue,
    Matchmaking,
    Monitoring,
}
