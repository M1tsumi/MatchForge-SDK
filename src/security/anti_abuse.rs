//! Anti-abuse system for MatchForge SDK
//! 
//! Provides comprehensive abuse detection and prevention mechanisms.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::rate_limiter::RateLimiter;

/// Abuse detection and prevention system
pub struct AntiAbuseSystem {
    config: AntiAbuseConfig,
    rate_limiter: RateLimiter,
    player_behavior: Arc<RwLock<HashMap<Uuid, PlayerBehavior>>>,
    abuse_reports: Arc<RwLock<Vec<AbuseReport>>>,
    reputation_scores: Arc<RwLock<HashMap<Uuid, ReputationScore>>>,
}

/// Anti-abuse configuration
#[derive(Debug, Clone)]
pub struct AntiAbuseConfig {
    /// Enable behavior tracking
    pub enable_behavior_tracking: bool,
    
    /// Enable reputation system
    pub enable_reputation_system: bool,
    
    /// Thresholds for abuse detection
    pub thresholds: AbuseThresholds,
    
    /// Actions to take for different abuse levels
    pub actions: AbuseActions,
    
    /// How long to track player behavior
    pub behavior_retention: Duration,
    
    /// How long to keep abuse reports
    pub report_retention: Duration,
}

/// Abuse detection thresholds
#[derive(Debug, Clone)]
pub struct AbuseThresholds {
    /// Maximum queue leaves per hour
    pub max_queue_leaves_per_hour: u32,
    
    /// Maximum party disbands per hour
    pub max_party_disbands_per_hour: u32,
    
    /// Maximum AFK kicks per day
    pub max_afk_kicks_per_day: u32,
    
    /// Minimum reputation score
    pub min_reputation_score: f64,
    
    /// Maximum report rate (reports per hour)
    pub max_reports_per_hour: u32,
    
    /// Suspicious rating manipulation threshold
    pub rating_manipulation_threshold: f64,
}

impl Default for AbuseThresholds {
    fn default() -> Self {
        Self {
            max_queue_leaves_per_hour: 20,
            max_party_disbands_per_hour: 10,
            max_afk_kicks_per_day: 5,
            min_reputation_score: -50.0,
            max_reports_per_hour: 10,
            rating_manipulation_threshold: 0.8,
        }
    }
}

/// Actions to take for abuse
#[derive(Debug, Clone)]
pub struct AbuseActions {
    /// Warning message
    pub warning_message: String,
    
    /// Temporary ban duration
    pub temp_ban_duration: Duration,
    
    /// Permanent ban threshold
    pub perm_ban_threshold: u32,
    
    /// Reputation penalty for violations
    pub reputation_penalty: f64,
    
    /// Reputation reward for good behavior
    pub reputation_reward: f64,
}

impl Default for AbuseActions {
    fn default() -> Self {
        Self {
            warning_message: "Your behavior has been flagged. Continued violations may result in penalties.".to_string(),
            temp_ban_duration: Duration::from_hours(24),
            perm_ban_threshold: 5,
            reputation_penalty: 10.0,
            reputation_reward: 1.0,
        }
    }
}

impl Default for AntiAbuseConfig {
    fn default() -> Self {
        Self {
            enable_behavior_tracking: true,
            enable_reputation_system: true,
            thresholds: AbuseThresholds::default(),
            actions: AbuseActions::default(),
            behavior_retention: Duration::from_secs(30 * 24 * 60 * 60),
            report_retention: Duration::from_secs(90 * 24 * 60 * 60),
        }
    }
}

/// Player behavior tracking
#[derive(Debug, Clone)]
struct PlayerBehavior {
    player_id: Uuid,
    queue_leaves: Vec<DateTime<Utc>>,
    party_disbands: Vec<DateTime<Utc>>,
    afk_kicks: Vec<DateTime<Utc>>,
    reports_received: Vec<DateTime<Utc>>,
    reports_made: Vec<DateTime<Utc>>,
    matches_abandoned: Vec<DateTime<Utc>>,
    suspicious_activities: Vec<SuspiciousActivity>,
    last_activity: DateTime<Utc>,
}

/// Suspicious activity tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousActivity {
    pub activity_type: SuspiciousActivityType,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, String>,
    pub confidence: f64,
}

/// Types of suspicious activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuspiciousActivityType {
    RatingManipulation,
    QueueDodging,
    AccountSharing,
    BotBehavior,
    ExploitUsage,
    Harassment,
    Cheating,
}

/// Abuse report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbuseReport {
    pub id: Uuid,
    pub reporter_id: Uuid,
    pub reported_player_id: Uuid,
    pub report_type: AbuseReportType,
    pub reason: String,
    pub evidence: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub status: ReportStatus,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
}

/// Types of abuse reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbuseReportType {
    Harassment,
    Cheating,
    Exploiting,
    AccountSharing,
    BotUsage,
    InappropriateName,
    IntentionalFeeding,
    QueueManipulation,
    Other(String),
}

/// Report status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportStatus {
    Pending,
    UnderReview,
    Resolved,
    Dismissed,
}

/// Abuse detection result
#[derive(Debug, Clone)]
pub struct AbuseDetection {
    pub player_id: Uuid,
    pub detected_activities: Vec<SuspiciousActivity>,
    pub abuse_level: AbuseLevel,
    pub recommended_action: Option<AbuseAction>,
    pub confidence: f64,
}

/// Abuse severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbuseLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Actions to take for abuse
#[derive(Debug, Clone)]
pub enum AbuseAction {
    Warning(String),
    TemporaryBan(Duration),
    ReputationPenalty(f64),
    PermanentBan,
    Monitor,
    NoAction,
}

impl AntiAbuseSystem {
    /// Create a new anti-abuse system
    pub fn new(config: AntiAbuseConfig) -> Self {
        Self {
            rate_limiter: RateLimiter::new(super::rate_limiter::RateLimitConfig::default()),
            config,
            player_behavior: Arc::new(RwLock::new(HashMap::new())),
            abuse_reports: Arc::new(RwLock::new(Vec::new())),
            reputation_scores: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Track player activity
    pub async fn track_activity(&self, player_id: Uuid, activity: PlayerActivity) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enable_behavior_tracking {
            return Ok(());
        }
        
        let mut behavior = self.player_behavior.write().await;
        let player_behavior = behavior.entry(player_id).or_insert_with(|| PlayerBehavior {
            player_id,
            queue_leaves: Vec::new(),
            party_disbands: Vec::new(),
            afk_kicks: Vec::new(),
            reports_received: Vec::new(),
            reports_made: Vec::new(),
            matches_abandoned: Vec::new(),
            suspicious_activities: Vec::new(),
            last_activity: Utc::now(),
        });
        
        let now = Utc::now();
        player_behavior.last_activity = now;
        
        match activity {
            PlayerActivity::QueueLeave => {
                player_behavior.queue_leaves.push(now);
                self.update_reputation(player_id, -self.config.actions.reputation_penalty).await;
            }
            PlayerActivity::PartyDisband => {
                player_behavior.party_disbands.push(now);
                self.update_reputation(player_id, -self.config.actions.reputation_penalty).await;
            }
            PlayerActivity::AfkKick => {
                player_behavior.afk_kicks.push(now);
                self.update_reputation(player_id, -self.config.actions.reputation_penalty * 2.0).await;
            }
            PlayerActivity::MatchCompleted => {
                self.update_reputation(player_id, self.config.actions.reputation_reward).await;
            }
            PlayerActivity::GoodSportsmanship => {
                self.update_reputation(player_id, self.config.actions.reputation_reward * 2.0).await;
            }
        }
        
        Ok(())
    }
    
    /// Detect abuse based on player behavior
    pub async fn detect_abuse(&self, player_id: Uuid) -> AbuseDetection {
        let behavior = self.player_behavior.read().await;
        let reputation = self.get_reputation_score(player_id).await;
        
        let mut detected_activities = Vec::new();
        let mut abuse_level = AbuseLevel::None;
        let mut max_confidence: f32 = 0.0;
        
        if let Some(player_behavior) = behavior.get(&player_id) {
            let now = Utc::now();
            
            // Check for queue dodging
            let recent_leaves: Vec<_> = player_behavior.queue_leaves.iter()
                .filter(|&&time| now - time < chrono::Duration::hours(1))
                .collect();
            
            if recent_leaves.len() as u32 > self.config.thresholds.max_queue_leaves_per_hour {
                detected_activities.push(SuspiciousActivity {
                    activity_type: SuspiciousActivityType::QueueDodging,
                    timestamp: now,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("leaves_per_hour".to_string(), recent_leaves.len().to_string());
                        details
                    },
                    confidence: 0.8,
                });
                abuse_level = AbuseLevel::Medium;
                max_confidence = max_confidence.max(0.8);
            }
            
            // Check for party disband abuse
            let recent_disbands: Vec<_> = player_behavior.party_disbands.iter()
                .filter(|&&time| now - time < chrono::Duration::hours(1))
                .collect();
            
            if recent_disbands.len() as u32 > self.config.thresholds.max_party_disbands_per_hour {
                detected_activities.push(SuspiciousActivity {
                    activity_type: SuspiciousActivityType::AccountSharing,
                    timestamp: now,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("disbands_per_hour".to_string(), recent_disbands.len().to_string());
                        details
                    },
                    confidence: 0.6,
                });
                abuse_level = AbuseLevel::Medium;
                max_confidence = max_confidence.max(0.6);
            }
            
            // Check for AFK behavior
            let recent_afk: Vec<_> = player_behavior.afk_kicks.iter()
                .filter(|&&time| now - time < chrono::Duration::days(1))
                .collect();
            
            if recent_afk.len() as u32 > self.config.thresholds.max_afk_kicks_per_day {
                detected_activities.push(SuspiciousActivity {
                    activity_type: SuspiciousActivityType::BotBehavior,
                    timestamp: now,
                    details: {
                        let mut details = HashMap::new();
                        details.insert("afk_kicks_per_day".to_string(), recent_afk.len().to_string());
                        details
                    },
                    confidence: 0.7,
                });
                abuse_level = AbuseLevel::High;
                max_confidence = max_confidence.max(0.7);
            }
            
            // Check reputation score
            if let Some(rep) = reputation {
                if rep.score < self.config.thresholds.min_reputation_score {
                    detected_activities.push(SuspiciousActivity {
                        activity_type: SuspiciousActivityType::Harassment,
                        timestamp: now,
                        details: {
                            let mut details = HashMap::new();
                            details.insert("reputation_score".to_string(), rep.score.to_string());
                            details.insert("violations".to_string(), rep.violation_count.to_string());
                            details
                        },
                        confidence: 0.9,
                    });
                    abuse_level = AbuseLevel::High;
                    max_confidence = max_confidence.max(0.9);
                }
            }
        }
        
        let recommended_action = self.determine_action(&abuse_level, max_confidence.into());
        
        AbuseDetection {
            player_id,
            detected_activities,
            abuse_level,
            recommended_action,
            confidence: max_confidence.into(),
        }
    }
    
    /// Submit an abuse report
    pub async fn submit_report(&self, report: AbuseReport) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut reports = self.abuse_reports.write().await;
        reports.push(report);
        
        // Check if this creates a pattern of abuse
        self.check_report_patterns().await?;
        
        Ok(())
    }
    
    /// Get abuse reports for a player
    pub async fn get_reports_for_player(&self, player_id: Uuid) -> Vec<AbuseReport> {
        let reports = self.abuse_reports.read().await;
        reports.iter()
            .filter(|r| r.reported_player_id == player_id)
            .cloned()
            .collect()
    }
    
    /// Get player reputation score
    pub async fn get_reputation_score(&self, player_id: Uuid) -> Option<ReputationScore> {
        let scores = self.reputation_scores.read().await;
        scores.get(&player_id).cloned()
    }
    
    /// Apply abuse action
    pub async fn apply_action(&self, player_id: Uuid, action: AbuseAction) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match action {
            AbuseAction::ReputationPenalty(penalty) => {
                self.update_reputation(player_id, -penalty).await;
            }
            AbuseAction::TemporaryBan(duration) => {
                // In a real implementation, this would integrate with a ban system
                eprintln!("Player {} temporarily banned for {:?}", player_id, duration);
            }
            AbuseAction::PermanentBan => {
                // In a real implementation, this would integrate with a ban system
                eprintln!("Player {} permanently banned", player_id);
            }
            AbuseAction::Warning(message) => {
                // In a real implementation, this would send a warning to the player
                eprintln!("Warning sent to player {}: {}", player_id, message);
            }
            AbuseAction::Monitor => {
                // Add to monitoring list
                eprintln!("Player {} added to monitoring list", player_id);
            }
            AbuseAction::NoAction => {}
        }
        
        Ok(())
    }
    
    /// Clean up old data
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cutoff = Utc::now() - chrono::Duration::from_std(self.config.behavior_retention).unwrap();
        
        // Clean up old behavior data
        let mut behavior = self.player_behavior.write().await;
        behavior.retain(|_, player_behavior| player_behavior.last_activity > cutoff);
        
        // Clean up old reports
        let report_cutoff = Utc::now() - chrono::Duration::from_std(self.config.report_retention).unwrap();
        let mut reports = self.abuse_reports.write().await;
        reports.retain(|report| report.timestamp > report_cutoff);
        
        Ok(())
    }
    
    async fn update_reputation(&self, player_id: Uuid, delta: f64) {
        if !self.config.enable_reputation_system {
            return;
        }
        
        let mut scores = self.reputation_scores.write().await;
        let score = scores.entry(player_id).or_insert_with(|| ReputationScore {
            score: 0.0,
            last_updated: Utc::now(),
            violation_count: 0,
            positive_actions: 0,
        });
        
        score.score += delta;
        score.last_updated = Utc::now();
        
        if delta < 0.0 {
            score.violation_count += 1;
        } else {
            score.positive_actions += 1;
        }
        
        // Clamp score to reasonable bounds
        score.score = score.score.clamp(-100.0, 100.0);
    }
    
    fn determine_action(&self, abuse_level: &AbuseLevel, confidence: f64) -> Option<AbuseAction> {
        match abuse_level {
            AbuseLevel::None => None,
            AbuseLevel::Low => Some(AbuseAction::Monitor),
            AbuseLevel::Medium => {
                if confidence > 0.7 {
                    Some(AbuseAction::Warning(self.config.actions.warning_message.clone()))
                } else {
                    Some(AbuseAction::Monitor)
                }
            }
            AbuseLevel::High => {
                if confidence > 0.8 {
                    Some(AbuseAction::TemporaryBan(self.config.actions.temp_ban_duration))
                } else {
                    Some(AbuseAction::ReputationPenalty(self.config.actions.reputation_penalty))
                }
            }
            AbuseLevel::Critical => Some(AbuseAction::PermanentBan),
        }
    }
    
    async fn check_report_patterns(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let reports = self.abuse_reports.read().await;
        let now = Utc::now();
        
        // Group reports by reported player
        let mut player_reports: HashMap<Uuid, Vec<&AbuseReport>> = HashMap::new();
        for report in reports.iter() {
            player_reports.entry(report.reported_player_id)
                .or_insert_with(Vec::new)
                .push(report);
        }
        
        // Check for patterns
        for (player_id, player_report_list) in player_reports {
            let recent_reports: Vec<_> = player_report_list.iter()
                .filter(|r| now - r.timestamp < chrono::Duration::hours(1))
                .collect();
            
            if recent_reports.len() as u32 > self.config.thresholds.max_reports_per_hour {
                // This player is being reported frequently - investigate
                eprintln!("Player {} has {} recent reports - investigation recommended", 
                    player_id, recent_reports.len());
            }
        }
        
        Ok(())
    }
}

/// Player activity types
#[derive(Debug, Clone)]
pub enum PlayerActivity {
    QueueLeave,
    PartyDisband,
    AfkKick,
    MatchCompleted,
    GoodSportsmanship,
}

/// Reputation score information
#[derive(Debug, Clone)]
pub struct ReputationScore {
    pub score: f64,
    pub last_updated: DateTime<Utc>,
    pub violation_count: u32,
    pub positive_actions: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_abuse_detection() {
        let config = AntiAbuseConfig::default();
        let system = AntiAbuseSystem::new(config);
        let player_id = Uuid::new_v4();
        
        // Simulate abusive behavior
        for _ in 0..25 {
            system.track_activity(player_id, PlayerActivity::QueueLeave).await.unwrap();
        }
        
        let detection = system.detect_abuse(player_id).await;
        assert!(detection.abuse_level >= AbuseLevel::Medium);
        assert!(!detection.detected_activities.is_empty());
    }
    
    #[tokio::test]
    async fn test_reputation_system() {
        let config = AntiAbuseConfig::default();
        let system = AntiAbuseSystem::new(config);
        let player_id = Uuid::new_v4();
        
        // Start with good behavior
        for _ in 0..10 {
            system.track_activity(player_id, PlayerActivity::MatchCompleted).await.unwrap();
        }
        
        let reputation = system.get_reputation_score(player_id).await;
        assert!(reputation.is_some());
        assert!(reputation.unwrap().score > 0.0);
        
        // Add some bad behavior
        for _ in 0..5 {
            system.track_activity(player_id, PlayerActivity::QueueLeave).await.unwrap();
        }
        
        let reputation = system.get_reputation_score(player_id).await;
        assert!(reputation.is_some());
        // Score should be lower now
        assert!(reputation.unwrap().score < 10.0);
    }
}
