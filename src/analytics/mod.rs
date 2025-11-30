//! Analytics module for MatchForge SDK
//! 
//! Provides comprehensive analytics and reporting capabilities for matchmaking data.

pub mod metrics;
pub mod reports;
pub mod insights;
pub mod dashboard;

pub use metrics::{AnalyticsMetrics, MetricsCollector};
pub use reports::{ReportGenerator, ReportType, ReportFormat};
pub use insights::{InsightEngine, InsightType, Recommendation};
pub use dashboard::{DashboardData, DashboardConfig};
