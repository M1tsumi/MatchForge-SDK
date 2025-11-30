//! Monitoring service for MatchForge SDK
//! 
//! Provides comprehensive monitoring, health checks, and alerting capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{EventCollector, MetricsCollector};
use super::metrics::MetricsSnapshot;
use crate::error::Result;

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// How often to collect metrics
    pub metrics_interval: Duration,
    
    /// How long to retain metrics data
    pub metrics_retention: Duration,
    
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    
    /// Health check configuration
    pub health_checks: HealthCheckConfig,
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Maximum average wait time (ms)
    pub max_average_wait_time: u64,
    
    /// Minimum success rate (0-1)
    pub min_success_rate: f64,
    
    /// Maximum error rate (0-1)
    pub max_error_rate: f64,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Minimum health score (0-100)
    pub min_health_score: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_average_wait_time: 30000, // 30 seconds
            min_success_rate: 0.8,        // 80%
            max_error_rate: 0.05,         // 5%
            max_queue_size: 1000,
            min_health_score: 70.0,       // 70/100
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// How often to run health checks
    pub interval: Duration,
    
    /// Timeout for health checks
    pub timeout: Duration,
    
    /// Components to check
    pub components: Vec<HealthComponent>,
}

/// Components that can be health checked
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthComponent {
    Persistence,
    QueueManager,
    Matchmaker,
    LobbyManager,
    PartyManager,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            components: vec![
                HealthComponent::Persistence,
                HealthComponent::QueueManager,
                HealthComponent::Matchmaker,
            ],
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_interval: Duration::from_secs(10),
            metrics_retention: Duration::from_hours(24),
            alert_thresholds: AlertThresholds::default(),
            health_checks: HealthCheckConfig::default(),
        }
    }
}

/// Monitoring service
pub struct MonitoringService {
    config: MonitoringConfig,
    metrics_collector: Arc<dyn MetricsCollector>,
    event_collector: Arc<dyn EventCollector>,
    alerts: Arc<RwLock<Vec<Alert>>>,
    health_status: Arc<RwLock<HashMap<HealthComponent, HealthStatus>>>,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(
        config: MonitoringConfig,
        metrics_collector: Arc<dyn MetricsCollector>,
        event_collector: Arc<dyn EventCollector>,
    ) -> Self {
        Self {
            config,
            metrics_collector,
            event_collector,
            alerts: Arc::new(RwLock::new(Vec::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the monitoring service
    pub async fn start(&self) -> Result<()> {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(service.config.metrics_interval);
            loop {
                interval.tick().await;
                if let Err(e) = service.run_monitoring_cycle().await {
                    eprintln!("Monitoring cycle error: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Run a single monitoring cycle
    async fn run_monitoring_cycle(&self) -> Result<()> {
        // Collect metrics
        let metrics = self.metrics_collector.get_metrics();
        
        // Check for alerts
        self.check_alerts(&metrics).await?;
        
        // Run health checks
        self.run_health_checks().await?;
        
        // Clean up old data
        self.cleanup_old_data().await?;
        
        Ok(())
    }
    
    /// Check for alert conditions
    async fn check_alerts(&self, metrics: &MetricsSnapshot) -> Result<()> {
        let mut new_alerts = Vec::new();
        
        // Check wait time
        if metrics.average_wait_time_ms > self.config.alert_thresholds.max_average_wait_time {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Warning,
                title: "High Average Wait Time".to_string(),
                message: format!("Average wait time is {}ms (threshold: {}ms)", 
                    metrics.average_wait_time_ms, 
                    self.config.alert_thresholds.max_average_wait_time),
                timestamp: Utc::now(),
                component: "Matchmaker".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("average_wait_time".to_string(), metrics.average_wait_time_ms.to_string());
                    map.insert("threshold".to_string(), self.config.alert_thresholds.max_average_wait_time.to_string());
                    map
                },
            });
        }
        
        // Check success rate - placeholder
        let success_rate = if metrics.matches_completed > 0 {
            metrics.matches_completed as f64 / metrics.matches_found as f64
        } else {
            1.0
        };
        if success_rate < self.config.alert_thresholds.min_success_rate {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Warning,
                title: "Low Success Rate".to_string(),
                message: format!("Success rate is {:.2}% (threshold: {:.2}%)", 
                    success_rate * 100.0, 
                    self.config.alert_thresholds.min_success_rate * 100.0),
                timestamp: Utc::now(),
                component: "Matchmaker".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("success_rate".to_string(), format!("{:.4}", success_rate));
                    map.insert("threshold".to_string(), format!("{:.4}", self.config.alert_thresholds.min_success_rate));
                    map
                },
            });
        }
        
        // Check error rate
        let error_rate = if metrics.persistence_operations > 0 {
            metrics.persistence_errors as f64 / metrics.persistence_operations as f64
        } else {
            0.0
        };
        if error_rate > self.config.alert_thresholds.max_error_rate {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Error,
                title: "High Error Rate".to_string(),
                message: format!("Error rate is {:.2}% (threshold: {:.2}%)", 
                    error_rate * 100.0, 
                    self.config.alert_thresholds.max_error_rate * 100.0),
                timestamp: Utc::now(),
                component: "Persistence".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("error_rate".to_string(), format!("{:.4}", error_rate));
                    map.insert("threshold".to_string(), format!("{:.4}", self.config.alert_thresholds.max_error_rate));
                    map
                },
            });
        }
        
        // Check queue sizes
        for (queue_name, &size) in &metrics.queue_sizes {
            if size > self.config.alert_thresholds.max_queue_size as usize {
                new_alerts.push(Alert {
                    id: Uuid::new_v4(),
                    level: AlertLevel::Warning,
                    title: "Large Queue Size".to_string(),
                    message: format!("Queue '{}' has {} players (threshold: {})", 
                        queue_name, size, self.config.alert_thresholds.max_queue_size),
                    timestamp: Utc::now(),
                    component: format!("Queue: {}", queue_name),
                    metadata: {
                        let mut map = HashMap::new();
                        map.insert("queue_name".to_string(), queue_name.clone());
                        map.insert("size".to_string(), size.to_string());
                        map.insert("threshold".to_string(), self.config.alert_thresholds.max_queue_size.to_string());
                        map
                    },
                });
            }
        }
        
        // Check health score - placeholder
        let health_score = 100.0; // TODO: Implement health score calculation
        if health_score < self.config.alert_thresholds.min_health_score {
            new_alerts.push(Alert {
                id: Uuid::new_v4(),
                level: AlertLevel::Warning,
                title: "Low Health Score".to_string(),
                message: format!("Health score is {:.1} (threshold: {:.1})", 
                    health_score, 
                    self.config.alert_thresholds.min_health_score),
                timestamp: Utc::now(),
                component: "System".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("health_score".to_string(), format!("{:.1}", health_score));
                    map.insert("threshold".to_string(), format!("{:.1}", self.config.alert_thresholds.min_health_score));
                    map
                },
            });
        }
        
        // Add new alerts
        if !new_alerts.is_empty() {
            let mut alerts = self.alerts.write().await;
            alerts.extend(new_alerts);
            
            // Keep only recent alerts (last 100)
            if alerts.len() > 100 {
                alerts.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                alerts.truncate(100);
            }
        }
        
        Ok(())
    }
    
    /// Run health checks
    async fn run_health_checks(&self) -> Result<()> {
        let mut health_status = self.health_status.write().await;
        
        for component in &self.config.health_checks.components {
            let status = self.check_component_health(component).await;
            health_status.insert(component.clone(), status);
        }
        
        Ok(())
    }
    
    /// Check health of a specific component
    async fn check_component_health(&self, component: &HealthComponent) -> HealthStatus {
        let start = std::time::Instant::now();
        
        let result = match component {
            HealthComponent::Persistence => {
                // Check persistence layer health
                self.check_persistence_health().await
            }
            HealthComponent::QueueManager => {
                // Check queue manager health
                self.check_queue_manager_health().await
            }
            HealthComponent::Matchmaker => {
                // Check matchmaker health
                self.check_matchmaker_health().await
            }
            HealthComponent::LobbyManager => {
                // Check lobby manager health
                self.check_lobby_manager_health().await
            }
            HealthComponent::PartyManager => {
                // Check party manager health
                self.check_party_manager_health().await
            }
        };
        
        HealthStatus {
            component: format!("{:?}", component),
            status: result,
            response_time_ms: start.elapsed().as_millis() as u64,
            last_checked: Utc::now(),
            details: HashMap::new(),
        }
    }
    
    async fn check_persistence_health(&self) -> ComponentStatus {
        // Simplified health check - in a real implementation, this would
        // test actual database connectivity and performance
        ComponentStatus::Healthy
    }
    
    async fn check_queue_manager_health(&self) -> ComponentStatus {
        let metrics = self.metrics_collector.get_metrics();
        
        if metrics.active_players > 10000 {
            ComponentStatus::Degraded("High player load".to_string())
        } else {
            ComponentStatus::Healthy
        }
    }
    
    async fn check_matchmaker_health(&self) -> ComponentStatus {
        let metrics = self.metrics_collector.get_metrics();
        
        if metrics.average_wait_time_ms > 60000 {
            ComponentStatus::Degraded("High wait times".to_string())
        } else if metrics.matches_found > 0 && (metrics.matches_completed as f64 / metrics.matches_found as f64) < 0.5 {
            ComponentStatus::Unhealthy("Low success rate".to_string())
        } else {
            ComponentStatus::Healthy
        }
    }
    
    async fn check_lobby_manager_health(&self) -> ComponentStatus {
        ComponentStatus::Healthy
    }
    
    async fn check_party_manager_health(&self) -> ComponentStatus {
        ComponentStatus::Healthy
    }
    
    /// Clean up old data
    async fn cleanup_old_data(&self) -> Result<()> {
        let cutoff = Utc::now() - self.config.metrics_retention;
        self.event_collector.clear_old_events(cutoff);
        Ok(())
    }
    
    /// Get current alerts
    pub async fn get_alerts(&self) -> Vec<Alert> {
        self.alerts.read().await.clone()
    }
    
    /// Get health status
    pub async fn get_health_status(&self) -> HashMap<HealthComponent, HealthStatus> {
        self.health_status.read().await.clone()
    }
    
    /// Get system health summary
    pub async fn get_system_health(&self) -> SystemHealth {
        let health_status = self.get_health_status().await;
        let alerts = self.get_alerts().await;
        
        let overall_status = if health_status.values().any(|s| matches!(s.status, ComponentStatus::Unhealthy(_))) {
            SystemStatus::Unhealthy
        } else if health_status.values().any(|s| matches!(s.status, ComponentStatus::Degraded(_))) {
            SystemStatus::Degraded
        } else {
            SystemStatus::Healthy
        };
        
        SystemHealth {
            status: overall_status,
            components: health_status,
            alerts: alerts.iter().filter(|a| a.level >= AlertLevel::Warning).cloned().collect(),
            last_updated: Utc::now(),
        }
    }
}

impl Clone for MonitoringService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics_collector: self.metrics_collector.clone(),
            event_collector: self.event_collector.clone(),
            alerts: self.alerts.clone(),
            health_status: self.health_status.clone(),
        }
    }
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub metadata: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Health status of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component: String,
    pub status: ComponentStatus,
    pub response_time_ms: u64,
    pub last_checked: DateTime<Utc>,
    pub details: HashMap<String, String>,
}

/// Component health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentStatus {
    Healthy,
    Degraded(String), // with reason
    Unhealthy(String), // with reason
}

/// System health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: SystemStatus,
    pub components: HashMap<HealthComponent, HealthStatus>,
    pub alerts: Vec<Alert>,
    pub last_updated: DateTime<Utc>,
}

/// Overall system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Monitoring dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringDashboard {
    pub system_health: SystemHealth,
    pub metrics: MetricsSnapshot,
    pub recent_events: Vec<super::Event>,
    pub performance_trends: PerformanceTrends,
}

/// Performance trend data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub wait_time_trend: Vec<(DateTime<Utc>, u64)>,
    pub success_rate_trend: Vec<(DateTime<Utc>, f64)>,
    pub queue_size_trend: Vec<(DateTime<Utc>, HashMap<String, usize>)>,
    pub error_rate_trend: Vec<(DateTime<Utc>, f64)>,
}
