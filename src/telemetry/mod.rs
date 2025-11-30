//! Telemetry and monitoring for MatchForge SDK
//! 
//! This module provides comprehensive monitoring, metrics collection,
//! and observability features for the matchmaking system.

pub mod metrics;
pub mod events;
pub mod monitoring;

pub use metrics::{MatchmakingMetrics, MetricsCollector};
pub use events::{Event, EventCollector, EventType};
pub use monitoring::{MonitoringConfig, MonitoringService};
