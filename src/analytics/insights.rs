//! Insights engine for MatchForge SDK analytics
//! 
//! Provides intelligent insights and recommendations based on matchmaking data.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::metrics::{AnalyticsMetrics, MetricsSnapshot};

/// Insight engine for generating actionable insights
pub struct InsightEngine {
    analytics: Arc<AnalyticsMetrics>,
    config: InsightConfig,
    historical_data: Arc<RwLock<VecDeque<MetricsSnapshot>>>,
    ml_models: Arc<RwLock<HashMap<InsightType, MLModel>>>,
}

/// Insight configuration
#[derive(Debug, Clone)]
pub struct InsightConfig {
    /// Minimum confidence level for insights
    pub min_confidence: f64,
    
    /// How much historical data to analyze
    pub historical_window: Duration,
    
    /// Enable predictive insights
    pub enable_predictions: bool,
    
    /// Enable ML-based insights
    pub enable_ml_insights: bool,
    
    /// Insight generation frequency
    pub generation_interval: Duration,
}

/// Types of insights
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InsightType {
    /// Queue performance insights
    QueuePerformance,
    
    /// Player behavior insights
    PlayerBehavior,
    
    /// Rating system insights
    RatingSystem,
    
    /// System performance insights
    SystemPerformance,
    
    /// Business insights
    BusinessMetrics,
    
    /// Security insights
    SecurityAnomalies,
    
    /// Predictive insights
    PredictiveAnalytics,
}

/// Generated insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: Uuid,
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: f64,
    pub data_points: usize,
    pub generated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub recommendations: Vec<Recommendation>,
    pub evidence: Vec<Evidence>,
    pub metadata: InsightMetadata,
}

/// Insight severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Recommendation for addressing insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub impact: Impact,
    pub effort: Effort,
    pub actions: Vec<String>,
    pub expected_outcome: String,
    pub success_probability: f64,
}

/// Priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Impact levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Impact {
    High,
    Medium,
    Low,
}

/// Effort levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effort {
    High,
    Medium,
    Low,
}

/// Evidence supporting an insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub data: EvidenceData,
    pub weight: f64,
}

/// Evidence types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    Metric,
    Trend,
    Anomaly,
    Correlation,
    UserFeedback,
}

/// Evidence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceData {
    Numeric(f64),
    Percentage(f64),
    Count(u64),
    Duration(Duration),
    Text(String),
    TimeSeries(Vec<(DateTime<Utc>, f64)>),
    Distribution(Vec<(String, u64)>),
}

/// Insight metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightMetadata {
    pub generation_time: Duration,
    pub algorithm_version: String,
    pub data_sources: Vec<String>,
    pub confidence_interval: (f64, f64),
    pub related_insights: Vec<Uuid>,
}

/// Machine learning model for insights
#[derive(Debug, Clone)]
struct MLModel {
    model_type: ModelType,
    accuracy: f64,
    last_trained: DateTime<Utc>,
    training_data_size: usize,
}

/// Model types
#[derive(Debug, Clone)]
enum ModelType {
    LinearRegression,
    TimeSeries,
    AnomalyDetection,
    Clustering,
    Classification,
}

impl InsightEngine {
    /// Create new insight engine
    pub fn new(analytics: Arc<AnalyticsMetrics>) -> Self {
        Self {
            analytics,
            config: InsightConfig::default(),
            historical_data: Arc::new(RwLock::new(VecDeque::new())),
            ml_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Generate insights based on current data
    pub async fn generate_insights(&self) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Get current metrics snapshot
        let current_snapshot = self.analytics.get_metrics_snapshot().await;
        
        // Update historical data
        self.update_historical_data(current_snapshot.clone()).await;
        
        // Generate different types of insights
        insights.extend(self.generate_queue_performance_insights(&current_snapshot).await?);
        insights.extend(self.generate_player_behavior_insights(&current_snapshot).await?);
        insights.extend(self.generate_rating_system_insights(&current_snapshot).await?);
        insights.extend(self.generate_system_performance_insights(&current_snapshot).await?);
        insights.extend(self.generate_business_insights(&current_snapshot).await?);
        
        if self.config.enable_predictions {
            insights.extend(self.generate_predictive_insights(&current_snapshot).await?);
        }
        
        if self.config.enable_ml_insights {
            insights.extend(self.generate_ml_insights(&current_snapshot).await?);
        }
        
        // Filter insights by confidence level
        insights.retain(|insight| insight.confidence >= self.config.min_confidence);
        
        // Sort by severity and confidence
        insights.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then(b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        Ok(insights)
    }
    
    /// Generate queue performance insights
    async fn generate_queue_performance_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Check for high wait times
        if snapshot.average_wait_time > std::time::Duration::from_secs(60) {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::QueuePerformance,
                title: "High Average Wait Times Detected".to_string(),
                description: format!("Average wait time is {:.2} seconds, exceeding the 60-second threshold.", snapshot.average_wait_time.as_secs_f64()),
                severity: Severity::High,
                confidence: 0.9,
                data_points: snapshot.total_matches as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(24),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Expand Matchmaking Constraints".to_string(),
                        description: "Consider relaxing matchmaking constraints to find matches faster.".to_string(),
                        priority: Priority::High,
                        impact: Impact::High,
                        effort: Effort::Medium,
                        actions: vec![
                            "Increase maximum rating difference".to_string(),
                            "Reduce role requirements".to_string(),
                            "Enable cross-region matchmaking".to_string(),
                        ],
                        expected_outcome: "Reduce average wait time by 30-50%".to_string(),
                        success_probability: 0.8,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Current average wait time".to_string(),
                        data: EvidenceData::Duration(Duration::from_std(snapshot.average_wait_time).unwrap_or_default()),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(10),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["queue_metrics".to_string()],
                    confidence_interval: (0.85, 0.95),
                    related_insights: vec![],
                },
            });
        }
        
        // Check for queue abandonment
        let total_queue_size: u64 = snapshot.queue_sizes.values().sum();
        if total_queue_size > 1000 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::QueuePerformance,
                title: "Large Queue Size Detected".to_string(),
                description: format!("Total queue size is {} players, indicating potential matchmaking bottlenecks.", total_queue_size),
                severity: Severity::Medium,
                confidence: 0.8,
                data_points: total_queue_size as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(12),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Increase Matchmaking Frequency".to_string(),
                        description: "Run matchmaking more frequently to process queues faster.".to_string(),
                        priority: Priority::Medium,
                        impact: Impact::Medium,
                        effort: Effort::Low,
                        actions: vec![
                            "Reduce matchmaking interval".to_string(),
                            "Increase concurrent matchmaking threads".to_string(),
                        ],
                        expected_outcome: "Reduce queue processing time by 40%".to_string(),
                        success_probability: 0.9,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Total queue size".to_string(),
                        data: EvidenceData::Count(total_queue_size),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(5),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["queue_metrics".to_string()],
                    confidence_interval: (0.75, 0.85),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate player behavior insights
    async fn generate_player_behavior_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Check for low player retention
        let retention = self.analytics.get_retention_analytics().await;
        if retention.day_7_retention < 0.3 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::PlayerBehavior,
                title: "Low Day 7 Retention".to_string(),
                description: format!("Day 7 retention is {:.1}%, below the 30% threshold.", retention.day_7_retention * 100.0),
                severity: Severity::Critical,
                confidence: 0.85,
                data_points: snapshot.total_players as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(48),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Improve Early Player Experience".to_string(),
                        description: "Focus on improving the first week player experience to increase retention.".to_string(),
                        priority: Priority::Critical,
                        impact: Impact::High,
                        effort: Effort::High,
                        actions: vec![
                            "Implement comprehensive tutorial".to_string(),
                            "Add early-game rewards and milestones".to_string(),
                            "Improve matchmaking for new players".to_string(),
                            "Add mentorship system".to_string(),
                        ],
                        expected_outcome: "Increase day 7 retention to 40%".to_string(),
                        success_probability: 0.7,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Day 7 retention rate".to_string(),
                        data: EvidenceData::Percentage(retention.day_7_retention),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(15),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["player_analytics".to_string()],
                    confidence_interval: (0.8, 0.9),
                    related_insights: vec![],
                },
            });
        }
        
        // Check for short session durations
        if retention.average_session_duration < std::time::Duration::from_secs(900) { // 15 minutes
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::PlayerBehavior,
                title: "Short Average Session Duration".to_string(),
                description: format!("Average session duration is {:.1} minutes, indicating low engagement.", retention.average_session_duration.as_secs_f64() / 60.0),
                severity: Severity::Medium,
                confidence: 0.75,
                data_points: snapshot.active_players as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(24),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Increase Session Engagement".to_string(),
                        description: "Implement features to keep players engaged longer.".to_string(),
                        priority: Priority::Medium,
                        impact: Impact::Medium,
                        effort: Effort::Medium,
                        actions: vec![
                            "Add daily challenges".to_string(),
                            "Implement achievement system".to_string(),
                            "Add social features".to_string(),
                            "Improve match variety".to_string(),
                        ],
                        expected_outcome: "Increase average session to 25 minutes".to_string(),
                        success_probability: 0.6,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Average session duration".to_string(),
                        data: EvidenceData::Duration(Duration::from_std(retention.average_session_duration).unwrap_or_default()),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(8),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["player_analytics".to_string()],
                    confidence_interval: (0.7, 0.8),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate rating system insights
    async fn generate_rating_system_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Check for rating inflation
        let avg_rating = self.calculate_average_rating(&snapshot.rating_distribution);
        if avg_rating > 1700.0 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::RatingSystem,
                title: "Rating Inflation Detected".to_string(),
                description: format!("Average rating is {:.0}, indicating potential rating inflation.", avg_rating),
                severity: Severity::Medium,
                confidence: 0.8,
                data_points: snapshot.total_players as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Adjust Rating System".to_string(),
                        description: "Review and adjust rating calculation parameters to reduce inflation.".to_string(),
                        priority: Priority::Medium,
                        impact: Impact::Medium,
                        effort: Effort::Low,
                        actions: vec![
                            "Reduce K-factor for high-rated players".to_string(),
                            "Implement rating decay".to_string(),
                            "Review rating distribution".to_string(),
                        ],
                        expected_outcome: "Stabilize average rating around 1500".to_string(),
                        success_probability: 0.8,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Average player rating".to_string(),
                        data: EvidenceData::Numeric(avg_rating),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(12),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["rating_analytics".to_string()],
                    confidence_interval: (0.75, 0.85),
                    related_insights: vec![],
                },
            });
        }
        
        // Check for rating distribution issues
        let rating_variance = self.calculate_rating_variance(&snapshot.rating_distribution);
        if rating_variance < 10000.0 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::RatingSystem,
                title: "Low Rating Variance".to_string(),
                description: format!("Rating variance is {:.0}, indicating ratings are too concentrated.", rating_variance),
                severity: Severity::Low,
                confidence: 0.7,
                data_points: snapshot.total_players as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::days(3),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Increase Rating Differentiation".to_string(),
                        description: "Adjust rating system to better differentiate player skill levels.".to_string(),
                        priority: Priority::Low,
                        impact: Impact::Low,
                        effort: Effort::Low,
                        actions: vec![
                            "Increase rating volatility".to_string(),
                            "Review matchmaking constraints".to_string(),
                            "Consider skill-based tier system".to_string(),
                        ],
                        expected_outcome: "Increase rating variance to 15000".to_string(),
                        success_probability: 0.6,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Rating variance".to_string(),
                        data: EvidenceData::Numeric(rating_variance),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(6),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["rating_analytics".to_string()],
                    confidence_interval: (0.65, 0.75),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate system performance insights
    async fn generate_system_performance_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Check for high memory usage
        if snapshot.memory_usage_mb > 2000 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::SystemPerformance,
                title: "High Memory Usage".to_string(),
                description: format!("Memory usage is {} MB, exceeding the 2GB threshold.", snapshot.memory_usage_mb),
                severity: Severity::High,
                confidence: 0.9,
                data_points: 1,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(6),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Optimize Memory Usage".to_string(),
                        description: "Reduce memory consumption through optimization and cleanup.".to_string(),
                        priority: Priority::High,
                        impact: Impact::High,
                        effort: Effort::Medium,
                        actions: vec![
                            "Review memory allocation patterns".to_string(),
                            "Implement better garbage collection".to_string(),
                            "Optimize data structures".to_string(),
                            "Add memory monitoring".to_string(),
                        ],
                        expected_outcome: "Reduce memory usage below 1.5GB".to_string(),
                        success_probability: 0.8,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Current memory usage".to_string(),
                        data: EvidenceData::Numeric(snapshot.memory_usage_mb as f64),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(5),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["system_metrics".to_string()],
                    confidence_interval: (0.85, 0.95),
                    related_insights: vec![],
                },
            });
        }
        
        // Check for high CPU usage
        if snapshot.cpu_usage_percent > 80.0 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::SystemPerformance,
                title: "High CPU Usage".to_string(),
                description: format!("CPU usage is {:.1}%, exceeding the 80% threshold.", snapshot.cpu_usage_percent),
                severity: Severity::High,
                confidence: 0.85,
                data_points: 1,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(4),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Optimize CPU Usage".to_string(),
                        description: "Reduce CPU consumption through algorithm optimization.".to_string(),
                        priority: Priority::High,
                        impact: Impact::High,
                        effort: Effort::Medium,
                        actions: vec![
                            "Optimize matchmaking algorithms".to_string(),
                            "Implement better caching".to_string(),
                            "Add CPU monitoring".to_string(),
                            "Consider horizontal scaling".to_string(),
                        ],
                        expected_outcome: "Reduce CPU usage below 70%".to_string(),
                        success_probability: 0.7,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Current CPU usage".to_string(),
                        data: EvidenceData::Percentage(snapshot.cpu_usage_percent / 100.0),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(5),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["system_metrics".to_string()],
                    confidence_interval: (0.8, 0.9),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate business insights
    async fn generate_business_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        let retention = self.analytics.get_retention_analytics().await;
        
        // Check for high churn rate
        if retention.churn_rate > 0.1 {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::BusinessMetrics,
                title: "High Player Churn Rate".to_string(),
                description: format!("Churn rate is {:.1}%, exceeding the 10% threshold.", retention.churn_rate * 100.0),
                severity: Severity::Critical,
                confidence: 0.9,
                data_points: snapshot.total_players as usize,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(24),
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Reduce Player Churn".to_string(),
                        description: "Implement strategies to improve player retention and reduce churn.".to_string(),
                        priority: Priority::Critical,
                        impact: Impact::High,
                        effort: Effort::High,
                        actions: vec![
                            "Improve onboarding experience".to_string(),
                            "Add loyalty rewards".to_string(),
                            "Implement community features".to_string(),
                            "Enhance customer support".to_string(),
                        ],
                        expected_outcome: "Reduce churn rate to 7%".to_string(),
                        success_probability: 0.7,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Metric,
                        description: "Current churn rate".to_string(),
                        data: EvidenceData::Percentage(retention.churn_rate),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(10),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["business_analytics".to_string()],
                    confidence_interval: (0.85, 0.95),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate predictive insights
    async fn generate_predictive_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Predict queue overflow
        let total_queue_size: u64 = snapshot.queue_sizes.values().sum();
        let growth_rate = self.calculate_queue_growth_rate().await;
        
        if total_queue_size > 500 && growth_rate > 0.1 {
            let predicted_overflow_time = Duration::from_std(std::time::Duration::from_secs_f64(
                (1000.0 - total_queue_size as f64) / (total_queue_size as f64 * growth_rate / 3600.0)
            )).unwrap_or_default();
            
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::PredictiveAnalytics,
                title: "Predicted Queue Overflow".to_string(),
                description: format!("Queue size is growing at {:.1}% per hour. Predicted overflow in {:.1} hours.", growth_rate * 100.0, predicted_overflow_time.as_seconds_f64()),
                severity: Severity::High,
                confidence: 0.75,
                data_points: 100,
                generated_at: Utc::now(),
                expires_at: Utc::now() + predicted_overflow_time,
                recommendations: vec![
                    Recommendation {
                        id: Uuid::new_v4(),
                        title: "Prevent Queue Overflow".to_string(),
                        description: "Take proactive measures to prevent queue overflow.".to_string(),
                        priority: Priority::High,
                        impact: Impact::High,
                        effort: Effort::Medium,
                        actions: vec![
                            "Increase matchmaking frequency".to_string(),
                            "Add temporary server capacity".to_string(),
                            "Implement queue priority system".to_string(),
                        ],
                        expected_outcome: "Prevent queue overflow".to_string(),
                        success_probability: 0.8,
                    },
                ],
                evidence: vec![
                    Evidence {
                        evidence_type: EvidenceType::Trend,
                        description: "Queue growth rate".to_string(),
                        data: EvidenceData::Percentage(growth_rate),
                        weight: 1.0,
                    },
                ],
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(20),
                    algorithm_version: "1.0".to_string(),
                    data_sources: vec!["predictive_analytics".to_string()],
                    confidence_interval: (0.7, 0.8),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Generate ML-based insights
    async fn generate_ml_insights(&self, snapshot: &MetricsSnapshot) -> Result<Vec<Insight>, InsightError> {
        let mut insights = Vec::new();
        
        // Use anomaly detection to find unusual patterns
        if let Some(anomaly) = self.detect_anomalies(snapshot).await? {
            insights.push(Insight {
                id: Uuid::new_v4(),
                insight_type: InsightType::SecurityAnomalies,
                title: "Anomalous Activity Detected".to_string(),
                description: format!("Unusual pattern detected: {}", anomaly.description),
                severity: anomaly.severity,
                confidence: anomaly.confidence,
                data_points: anomaly.data_points,
                generated_at: Utc::now(),
                expires_at: Utc::now() + Duration::hours(12),
                recommendations: anomaly.recommendations,
                evidence: anomaly.evidence,
                metadata: InsightMetadata {
                    generation_time: Duration::milliseconds(50),
                    algorithm_version: "ML-1.0".to_string(),
                    data_sources: vec!["ml_analytics".to_string()],
                    confidence_interval: (anomaly.confidence - 0.1, anomaly.confidence + 0.1),
                    related_insights: vec![],
                },
            });
        }
        
        Ok(insights)
    }
    
    /// Update historical data
    async fn update_historical_data(&self, snapshot: MetricsSnapshot) {
        let mut historical = self.historical_data.write().await;
        historical.push_back(snapshot);
        
        // Keep only data within the historical window
        let cutoff = Utc::now() - self.config.historical_window;
        while let Some(front) = historical.front() {
            if front.timestamp < cutoff {
                historical.pop_front();
            } else {
                break;
            }
        }
    }
    
    /// Calculate average rating from distribution
    fn calculate_average_rating(&self, distribution: &HashMap<String, u64>) -> f64 {
        let mut total_rating = 0.0;
        let mut total_players = 0u64;
        
        for (bucket, count) in distribution {
            let avg_rating = match bucket.as_str() {
                "0-999" => 500.0,
                "1000-1199" => 1100.0,
                "1200-1399" => 1300.0,
                "1400-1599" => 1500.0,
                "1600-1799" => 1700.0,
                "1800-1999" => 1900.0,
                "2000+" => 2100.0,
                _ => 1500.0,
            };
            
            total_rating += avg_rating * *count as f64;
            total_players += count;
        }
        
        if total_players > 0 {
            total_rating / total_players as f64
        } else {
            1500.0
        }
    }
    
    /// Calculate rating variance
    fn calculate_rating_variance(&self, distribution: &HashMap<String, u64>) -> f64 {
        let avg_rating = self.calculate_average_rating(distribution);
        let mut variance = 0.0;
        let mut total_players = 0u64;
        
        for (bucket, count) in distribution {
            let bucket_avg = match bucket.as_str() {
                "0-999" => 500.0,
                "1000-1199" => 1100.0,
                "1200-1399" => 1300.0,
                "1400-1599" => 1500.0,
                "1600-1799" => 1700.0,
                "1800-1999" => 1900.0,
                "2000+" => 2100.0,
                _ => 1500.0,
            };
            
            variance += (bucket_avg - avg_rating).powi(2) * *count as f64;
            total_players += count;
        }
        
        if total_players > 0 {
            variance / total_players as f64
        } else {
            0.0
        }
    }
    
    /// Calculate queue growth rate
    async fn calculate_queue_growth_rate(&self) -> f64 {
        let historical = self.historical_data.read().await;
        if historical.len() < 2 {
            return 0.0;
        }
        
        let recent = historical.back().unwrap();
        let previous = historical.get(historical.len() - 2).unwrap();
        
        let recent_total: u64 = recent.queue_sizes.values().sum();
        let previous_total: u64 = previous.queue_sizes.values().sum();
        
        if previous_total == 0 {
            return 0.0;
        }
        
        let time_diff = recent.timestamp - previous.timestamp;
        let growth_rate = (recent_total as f64 - previous_total as f64) / previous_total as f64;
        growth_rate / time_diff.num_hours() as f64
    }
    
    /// Detect anomalies using ML
    async fn detect_anomalies(&self, snapshot: &MetricsSnapshot) -> Result<Option<AnomalyResult>, InsightError> {
        // Placeholder for anomaly detection
        // In a real implementation, this would use trained ML models
        Ok(None)
    }
}

/// Anomaly detection result
#[derive(Debug, Clone)]
struct AnomalyResult {
    description: String,
    severity: Severity,
    confidence: f64,
    data_points: usize,
    recommendations: Vec<Recommendation>,
    evidence: Vec<Evidence>,
}

/// Insight generation errors
#[derive(Debug, Clone)]
pub enum InsightError {
    DataUnavailable,
    ModelNotTrained,
    InsufficientData,
    GenerationFailed(String),
}

impl std::fmt::Display for InsightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InsightError::DataUnavailable => write!(f, "Required data is unavailable"),
            InsightError::ModelNotTrained => write!(f, "ML model is not trained"),
            InsightError::InsufficientData => write!(f, "Insufficient data for insight generation"),
            InsightError::GenerationFailed(msg) => write!(f, "Insight generation failed: {}", msg),
        }
    }
}

impl std::error::Error for InsightError {}

impl Default for InsightConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            historical_window: Duration::days(30),
            enable_predictions: true,
            enable_ml_insights: true,
            generation_interval: Duration::hours(1),
        }
    }
}
