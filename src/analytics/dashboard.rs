//! Dashboard data provider for MatchForge SDK analytics
//! 
//! Provides data structures and utilities for analytics dashboards.

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{metrics::AnalyticsMetrics, reports::ReportGenerator};
use super::insights::{InsightEngine, Severity as InsightSeverity};

/// Dashboard data provider
pub struct DashboardData {
    analytics: Arc<AnalyticsMetrics>,
    report_generator: Arc<ReportGenerator>,
    insight_engine: Arc<InsightEngine>,
    config: DashboardConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Default time range for dashboard data
    pub default_time_range: Duration,
    
    /// Data refresh interval
    pub refresh_interval: Duration,
    
    /// Maximum data points for charts
    pub max_chart_points: usize,
    
    /// Enable real-time updates
    pub enable_real_time: bool,
    
    /// Dashboard theme
    pub theme: DashboardTheme,
}

/// Dashboard theme configuration
#[derive(Debug, Clone)]
pub struct DashboardTheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
    pub chart_colors: Vec<String>,
}

/// Complete dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub generated_at: DateTime<Utc>,
    pub time_range: TimeRange,
    pub widgets: Vec<Widget>,
    pub layout: DashboardLayout,
    pub metadata: DashboardMetadata,
}

/// Time range for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub preset: Option<TimePreset>,
}

/// Time range presets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePreset {
    LastHour,
    Last24Hours,
    Last7Days,
    Last30Days,
    Last90Days,
    Custom,
}

/// Dashboard widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub data: WidgetData,
    pub config: WidgetConfig,
    pub refresh_interval: Option<Duration>,
}

/// Widget types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetType {
    /// Key performance indicators
    KPI,
    
    /// Line chart
    LineChart,
    
    /// Bar chart
    BarChart,
    
    /// Pie chart
    PieChart,
    
    /// Gauge/meter
    Gauge,
    
    /// Table
    Table,
    
    /// Heatmap
    Heatmap,
    
    /// Alert/notification
    Alert,
    
    /// Insight card
    Insight,
    
    /// Custom widget
    Custom(String),
}

/// Widget position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

/// Widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetData {
    KPI(KPIData),
    Chart(ChartData),
    Gauge(GaugeData),
    Table(TableData),
    Heatmap(HeatmapData),
    Alert(AlertData),
    Insight(InsightData),
    Custom(CustomData),
}

/// KPI widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPIData {
    pub value: f64,
    pub label: String,
    pub unit: String,
    pub trend: Trend,
    pub trend_value: f64,
    pub target: Option<f64>,
    pub status: KPIStatus,
}

/// KPI status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KPIStatus {
    Good,
    Warning,
    Critical,
    Unknown,
}

/// Chart widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: ChartType,
    pub datasets: Vec<ChartDataset>,
    pub labels: Vec<String>,
    pub options: ChartOptions,
}

/// Chart types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Doughnut,
    Radar,
    Polar,
}

/// Chart dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub fill: bool,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub responsive: bool,
    pub maintain_aspect_ratio: bool,
    pub legend: ChartLegend,
    pub scales: Option<ChartScales>,
    pub plugins: ChartPlugins,
}

/// Chart legend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartLegend {
    pub display: bool,
    pub position: LegendPosition,
}

/// Legend position
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegendPosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Chart scales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartScales {
    pub x_axis: Option<AxisScale>,
    pub y_axis: Option<AxisScale>,
}

/// Axis scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisScale {
    pub display: bool,
    pub title: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// Chart plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPlugins {
    pub tooltip: TooltipConfig,
    pub title: Option<ChartTitle>,
}

/// Tooltip configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooltipConfig {
    pub enabled: bool,
    pub mode: TooltipMode,
}

/// Tooltip mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TooltipMode {
    Index,
    Nearest,
    Point,
    X,
    Y,
}

/// Chart title
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTitle {
    pub display: bool,
    pub text: String,
    pub position: TitlePosition,
}

/// Title position
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitlePosition {
    Top,
    Bottom,
    Left,
    Right,
}

/// Gauge widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeData {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub label: String,
    pub unit: String,
    pub thresholds: Vec<GaugeThreshold>,
    pub colors: Vec<GaugeColor>,
}

/// Gauge threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeThreshold {
    pub value: f64,
    pub label: String,
    pub color: String,
}

/// Gauge color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeColor {
    pub from: f64,
    pub to: f64,
    pub color: String,
}

/// Table widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<TableHeader>,
    pub rows: Vec<TableRow>,
    pub pagination: Option<TablePagination>,
    pub sorting: Option<TableSorting>,
}

/// Table header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeader {
    pub key: String,
    pub label: String,
    pub sortable: bool,
    pub width: Option<String>,
}

/// Table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: String,
    pub cells: Vec<TableCell>,
}

/// Table cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableCell {
    Text(String),
    Number(f64),
    Percentage(f64),
    Duration(Duration),
    DateTime(DateTime<Utc>),
    Boolean(bool),
    Status(Status),
}

/// Status indicator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Online,
    Offline,
    Warning,
    Error,
    Unknown,
}

/// Table pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePagination {
    pub page: u32,
    pub page_size: u32,
    pub total: u32,
}

/// Table sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSorting {
    pub column: String,
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Heatmap widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapData {
    pub grid: Vec<HeatmapCell>,
    pub x_labels: Vec<String>,
    pub y_labels: Vec<String>,
    pub color_scale: ColorScale,
}

/// Heatmap cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapCell {
    pub x: usize,
    pub y: usize,
    pub value: f64,
    pub color: String,
}

/// Color scale for heatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScale {
    pub min_color: String,
    pub max_color: String,
    pub steps: Vec<ColorStep>,
}

/// Color step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStep {
    pub value: f64,
    pub color: String,
}

/// Alert widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertData {
    pub alerts: Vec<Alert>,
    pub max_display: usize,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub actions: Vec<AlertAction>,
}

/// Alert level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAction {
    pub label: String,
    pub action: String,
    pub style: ActionStyle,
}

/// Action style
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Danger,
    Success,
}

/// Insight widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightData {
    pub insights: Vec<InsightWidget>,
    pub max_display: usize,
}

/// Insight widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightWidget {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}

/// Custom widget data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomData {
    pub widget_type: String,
    pub data: serde_json::Value,
}

/// Widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub refresh_interval: Option<Duration>,
    pub auto_refresh: bool,
    pub theme: Option<String>,
    pub custom_options: HashMap<String, serde_json::Value>,
}

/// Dashboard layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub layout_type: LayoutType,
    pub columns: u32,
    pub gap: u32,
    pub breakpoints: Vec<Breakpoint>,
}

/// Layout types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutType {
    Grid,
    Flex,
    Masonry,
    Custom,
}

/// Breakpoint for responsive layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub name: String,
    pub width: u32,
    pub columns: u32,
}

/// Dashboard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetadata {
    pub version: String,
    pub generated_by: String,
    pub data_sources: Vec<String>,
    pub last_updated: DateTime<Utc>,
    pub refresh_token: Option<String>,
}

/// Trend indicator
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Up,
    Down,
    Stable,
    Unknown,
}

/// Severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl DashboardData {
    /// Create new dashboard data provider
    pub fn new(
        analytics: Arc<AnalyticsMetrics>,
        report_generator: Arc<ReportGenerator>,
        insight_engine: Arc<InsightEngine>,
    ) -> Self {
        Self {
            analytics,
            report_generator,
            insight_engine,
            config: DashboardConfig::default(),
        }
    }
    
    /// Generate complete dashboard
    pub async fn generate_dashboard(&self, time_range: Option<TimeRange>) -> Result<Dashboard, DashboardError> {
        let time_range = time_range.unwrap_or_else(|| TimeRange {
            start: Utc::now() - self.config.default_time_range,
            end: Utc::now(),
            preset: Some(TimePreset::Last24Hours),
        });
        
        let widgets = self.generate_widgets(&time_range).await?;
        
        Ok(Dashboard {
            id: Uuid::new_v4(),
            title: "MatchForge Analytics Dashboard".to_string(),
            description: "Real-time analytics and insights for MatchForge SDK".to_string(),
            generated_at: Utc::now(),
            time_range,
            widgets,
            layout: DashboardLayout::default(),
            metadata: DashboardMetadata {
                version: "1.0".to_string(),
                generated_by: "MatchForge SDK".to_string(),
                data_sources: vec![
                    "analytics_metrics".to_string(),
                    "insight_engine".to_string(),
                    "report_generator".to_string(),
                ],
                last_updated: Utc::now(),
                refresh_token: None,
            },
        })
    }
    
    /// Generate dashboard widgets
    async fn generate_widgets(&self, time_range: &TimeRange) -> Result<Vec<Widget>, DashboardError> {
        let mut widgets = Vec::new();
        
        // KPI widgets
        widgets.push(self.generate_active_players_kpi().await?);
        widgets.push(self.generate_total_matches_kpi().await?);
        widgets.push(self.generate_average_wait_time_kpi().await?);
        widgets.push(self.generate_match_quality_kpi().await?);
        
        // Chart widgets
        widgets.push(self.generate_matches_over_time_chart().await?);
        widgets.push(self.generate_queue_sizes_chart().await?);
        widgets.push(self.generate_rating_distribution_chart().await?);
        
        // Table widget
        widgets.push(self.generate_queue_status_table().await?);
        
        // Alert widget
        widgets.push(self.generate_alerts_widget().await?);
        
        // Insight widget
        widgets.push(self.generate_insights_widget().await?);
        
        Ok(widgets)
    }
    
    /// Generate active players KPI widget
    async fn generate_active_players_kpi(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::KPI,
            title: "Active Players".to_string(),
            position: WidgetPosition { x: 0, y: 0 },
            size: WidgetSize { width: 3, height: 2 },
            data: WidgetData::KPI(KPIData {
                value: snapshot.active_players as f64,
                label: "Active Players".to_string(),
                unit: "players".to_string(),
                trend: Trend::Up,
                trend_value: 5.2,
                target: Some(1000.0),
                status: KPIStatus::Good,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(30)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(30)),
        })
    }
    
    /// Generate total matches KPI widget
    async fn generate_total_matches_kpi(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::KPI,
            title: "Total Matches".to_string(),
            position: WidgetPosition { x: 3, y: 0 },
            size: WidgetSize { width: 3, height: 2 },
            data: WidgetData::KPI(KPIData {
                value: snapshot.total_matches as f64,
                label: "Total Matches".to_string(),
                unit: "matches".to_string(),
                trend: Trend::Up,
                trend_value: 12.8,
                target: None,
                status: KPIStatus::Good,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(60)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(60)),
        })
    }
    
    /// Generate average wait time KPI widget
    async fn generate_average_wait_time_kpi(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        let status = if snapshot.average_wait_time > std::time::Duration::from_secs(60) {
            KPIStatus::Critical
        } else if snapshot.average_wait_time > std::time::Duration::from_secs(30) {
            KPIStatus::Warning
        } else {
            KPIStatus::Good
        };
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::KPI,
            title: "Average Wait Time".to_string(),
            position: WidgetPosition { x: 6, y: 0 },
            size: WidgetSize { width: 3, height: 2 },
            data: WidgetData::KPI(KPIData {
                value: snapshot.average_wait_time.as_secs_f64(),
                label: "Average Wait Time".to_string(),
                unit: "seconds".to_string(),
                trend: Trend::Down,
                trend_value: -8.5,
                target: Some(30.0),
                status,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(30)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(30)),
        })
    }
    
    /// Generate match quality KPI widget
    async fn generate_match_quality_kpi(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        let status = if snapshot.match_quality_score > 0.8 {
            KPIStatus::Good
        } else if snapshot.match_quality_score > 0.6 {
            KPIStatus::Warning
        } else {
            KPIStatus::Critical
        };
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::KPI,
            title: "Match Quality".to_string(),
            position: WidgetPosition { x: 9, y: 0 },
            size: WidgetSize { width: 3, height: 2 },
            data: WidgetData::KPI(KPIData {
                value: snapshot.match_quality_score,
                label: "Match Quality".to_string(),
                unit: "%".to_string(),
                trend: Trend::Stable,
                trend_value: 0.5,
                target: Some(0.85),
                status,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(60)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(60)),
        })
    }
    
    /// Generate matches over time chart widget
    async fn generate_matches_over_time_chart(&self) -> Result<Widget, DashboardError> {
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::LineChart,
            title: "Matches Over Time".to_string(),
            position: WidgetPosition { x: 0, y: 2 },
            size: WidgetSize { width: 6, height: 4 },
            data: WidgetData::Chart(ChartData {
                chart_type: ChartType::Line,
                datasets: vec![
                    ChartDataset {
                        label: "Matches per Hour".to_string(),
                        data: vec![45.0, 52.0, 48.0, 58.0, 62.0, 55.0, 68.0],
                        background_color: Some("rgba(54, 162, 235, 0.2)".to_string()),
                        border_color: Some("rgba(54, 162, 235, 1)".to_string()),
                        fill: true,
                    },
                ],
                labels: vec![
                    "00:00".to_string(), "04:00".to_string(), "08:00".to_string(),
                    "12:00".to_string(), "16:00".to_string(), "20:00".to_string(), "24:00".to_string(),
                ],
                options: ChartOptions {
                    responsive: true,
                    maintain_aspect_ratio: false,
                    legend: ChartLegend {
                        display: true,
                        position: LegendPosition::Top,
                    },
                    scales: Some(ChartScales {
                        x_axis: Some(AxisScale {
                            display: true,
                            title: Some("Time".to_string()),
                            min: None,
                            max: None,
                        }),
                        y_axis: Some(AxisScale {
                            display: true,
                            title: Some("Matches".to_string()),
                            min: Some(0.0),
                            max: None,
                        }),
                    }),
                    plugins: ChartPlugins {
                        tooltip: TooltipConfig {
                            enabled: true,
                            mode: TooltipMode::Index,
                        },
                        title: None,
                    },
                },
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::minutes(5)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::minutes(5)),
        })
    }
    
    /// Generate queue sizes chart widget
    async fn generate_queue_sizes_chart(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        let mut datasets = Vec::new();
        let mut labels = Vec::new();
        let mut data = Vec::new();
        
        for (queue_name, size) in &snapshot.queue_sizes {
            labels.push(queue_name.clone());
            data.push(*size as f64);
        }
        
        datasets.push(ChartDataset {
            label: "Queue Size".to_string(),
            data,
            background_color: Some("rgba(255, 99, 132, 0.2)".to_string()),
            border_color: Some("rgba(255, 99, 132, 1)".to_string()),
            fill: false,
        });
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::BarChart,
            title: "Queue Sizes".to_string(),
            position: WidgetPosition { x: 6, y: 2 },
            size: WidgetSize { width: 6, height: 4 },
            data: WidgetData::Chart(ChartData {
                chart_type: ChartType::Bar,
                datasets,
                labels,
                options: ChartOptions {
                    responsive: true,
                    maintain_aspect_ratio: false,
                    legend: ChartLegend {
                        display: false,
                        position: LegendPosition::Top,
                    },
                    scales: Some(ChartScales {
                        x_axis: Some(AxisScale {
                            display: true,
                            title: Some("Queue".to_string()),
                            min: None,
                            max: None,
                        }),
                        y_axis: Some(AxisScale {
                            display: true,
                            title: Some("Players".to_string()),
                            min: Some(0.0),
                            max: None,
                        }),
                    }),
                    plugins: ChartPlugins {
                        tooltip: TooltipConfig {
                            enabled: true,
                            mode: TooltipMode::Index,
                        },
                        title: None,
                    },
                },
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(30)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(30)),
        })
    }
    
    /// Generate rating distribution chart widget
    async fn generate_rating_distribution_chart(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        let mut datasets = Vec::new();
        let mut labels = Vec::new();
        let mut data = Vec::new();
        
        for (bucket, count) in &snapshot.rating_distribution {
            labels.push(bucket.clone());
            data.push(*count as f64);
        }
        
        datasets.push(ChartDataset {
            label: "Player Count".to_string(),
            data,
            background_color: Some("rgba(255, 99, 132, 0.2)".to_string()),
            border_color: Some("rgba(255, 99, 132, 1)".to_string()),
            fill: false,
        });
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::PieChart,
            title: "Rating Distribution".to_string(),
            position: WidgetPosition { x: 0, y: 6 },
            size: WidgetSize { width: 6, height: 4 },
            data: WidgetData::Chart(ChartData {
                chart_type: ChartType::Pie,
                datasets,
                labels,
                options: ChartOptions {
                    responsive: true,
                    maintain_aspect_ratio: false,
                    legend: ChartLegend {
                        display: true,
                        position: LegendPosition::Right,
                    },
                    scales: None,
                    plugins: ChartPlugins {
                        tooltip: TooltipConfig {
                            enabled: true,
                            mode: TooltipMode::Point,
                        },
                        title: None,
                    },
                },
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::minutes(10)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::minutes(10)),
        })
    }
    
    /// Generate queue status table widget
    async fn generate_queue_status_table(&self) -> Result<Widget, DashboardError> {
        let snapshot = self.analytics.get_metrics_snapshot().await;
        
        let mut headers = vec![
            TableHeader {
                key: "queue".to_string(),
                label: "Queue".to_string(),
                sortable: true,
                width: None,
            },
            TableHeader {
                key: "size".to_string(),
                label: "Size".to_string(),
                sortable: true,
                width: Some("80px".to_string()),
            },
            TableHeader {
                key: "avg_wait".to_string(),
                label: "Avg Wait".to_string(),
                sortable: true,
                width: Some("100px".to_string()),
            },
            TableHeader {
                key: "status".to_string(),
                label: "Status".to_string(),
                sortable: false,
                width: Some("80px".to_string()),
            },
        ];
        
        let mut rows = Vec::new();
        for (queue_name, size) in &snapshot.queue_sizes {
            let status = if *size > 100 {
                Status::Warning
            } else if *size > 200 {
                Status::Error
            } else {
                Status::Online
            };
            
            rows.push(TableRow {
                id: queue_name.clone(),
                cells: vec![
                    TableCell::Text(queue_name.clone()),
                    TableCell::Number(*size as f64),
                    TableCell::Duration(Duration::seconds(30)), // Placeholder
                    TableCell::Status(status),
                ],
            });
        }
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::Table,
            title: "Queue Status".to_string(),
            position: WidgetPosition { x: 6, y: 6 },
            size: WidgetSize { width: 6, height: 4 },
            data: WidgetData::Table(TableData {
                headers,
                rows,
                pagination: Some(TablePagination {
                    page: 1,
                    page_size: 10,
                    total: snapshot.queue_sizes.len() as u32,
                }),
                sorting: Some(TableSorting {
                    column: "size".to_string(),
                    direction: SortDirection::Desc,
                }),
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::seconds(30)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::seconds(30)),
        })
    }
    
    /// Generate alerts widget
    async fn generate_alerts_widget(&self) -> Result<Widget, DashboardError> {
        let insights = self.insight_engine.generate_insights().await.unwrap_or_default();
        
        let alerts: Vec<Alert> = insights
            .into_iter()
            .filter(|insight| insight.severity >= InsightSeverity::Medium)
            .take(5)
            .map(|insight| Alert {
                id: insight.id,
                level: match insight.severity {
                    InsightSeverity::Critical => AlertLevel::Critical,
                    InsightSeverity::High => AlertLevel::Error,
                    InsightSeverity::Medium => AlertLevel::Warning,
                    InsightSeverity::Low => AlertLevel::Info,
                    InsightSeverity::Info => AlertLevel::Info,
                },
                title: insight.title,
                message: insight.description,
                timestamp: insight.generated_at,
                actions: vec![
                    AlertAction {
                        label: "View Details".to_string(),
                        action: "view".to_string(),
                        style: ActionStyle::Primary,
                    },
                    AlertAction {
                        label: "Dismiss".to_string(),
                        action: "dismiss".to_string(),
                        style: ActionStyle::Secondary,
                    },
                ],
            })
            .collect();
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::Alert,
            title: "Recent Alerts".to_string(),
            position: WidgetPosition { x: 0, y: 10 },
            size: WidgetSize { width: 12, height: 3 },
            data: WidgetData::Alert(AlertData {
                alerts,
                max_display: 5,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::minutes(5)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::minutes(5)),
        })
    }
    
    /// Generate insights widget
    async fn generate_insights_widget(&self) -> Result<Widget, DashboardError> {
        let insights = self.insight_engine.generate_insights().await.unwrap_or_default();
        
        let insight_widgets: Vec<InsightWidget> = insights
            .into_iter()
            .take(3)
            .map(|insight| InsightWidget {
                id: insight.id,
                title: insight.title,
                description: insight.description,
                severity: match insight.severity {
                    InsightSeverity::Critical => Severity::Critical,
                    InsightSeverity::High => Severity::High,
                    InsightSeverity::Medium => Severity::Medium,
                    InsightSeverity::Low => Severity::Low,
                    InsightSeverity::Info => Severity::Info,
                },
                confidence: insight.confidence,
                recommendations: insight.recommendations
                    .into_iter()
                    .take(2)
                    .map(|rec| rec.title)
                    .collect(),
            })
            .collect();
        
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: WidgetType::Insight,
            title: "Recent Insights".to_string(),
            position: WidgetPosition { x: 0, y: 13 },
            size: WidgetSize { width: 12, height: 4 },
            data: WidgetData::Insight(InsightData {
                insights: insight_widgets,
                max_display: 3,
            }),
            config: WidgetConfig {
                refresh_interval: Some(Duration::minutes(15)),
                auto_refresh: true,
                theme: None,
                custom_options: HashMap::new(),
            },
            refresh_interval: Some(Duration::minutes(15)),
        })
    }
}

/// Dashboard generation errors
#[derive(Debug, Clone)]
pub enum DashboardError {
    DataUnavailable,
    WidgetGenerationFailed(String),
    InvalidTimeRange,
    SerializationError,
}

impl std::fmt::Display for DashboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DashboardError::DataUnavailable => write!(f, "Required data is unavailable"),
            DashboardError::WidgetGenerationFailed(msg) => write!(f, "Widget generation failed: {}", msg),
            DashboardError::InvalidTimeRange => write!(f, "Invalid time range"),
            DashboardError::SerializationError => write!(f, "Data serialization error"),
        }
    }
}

impl std::error::Error for DashboardError {}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            default_time_range: Duration::hours(24),
            refresh_interval: Duration::minutes(5),
            max_chart_points: 100,
            enable_real_time: true,
            theme: DashboardTheme::default(),
        }
    }
}

impl Default for DashboardTheme {
    fn default() -> Self {
        Self {
            primary_color: "#007bff".to_string(),
            secondary_color: "#6c757d".to_string(),
            accent_color: "#28a745".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#343a40".to_string(),
            chart_colors: vec![
                "#007bff".to_string(),
                "#28a745".to_string(),
                "#ffc107".to_string(),
                "#dc3545".to_string(),
                "#6f42c1".to_string(),
                "#e83e8c".to_string(),
                "#17a2b8".to_string(),
                "#6c757d".to_string(),
            ],
        }
    }
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::Grid,
            columns: 12,
            gap: 16,
            breakpoints: vec![
                Breakpoint {
                    name: "mobile".to_string(),
                    width: 768,
                    columns: 6,
                },
                Breakpoint {
                    name: "tablet".to_string(),
                    width: 1024,
                    columns: 8,
                },
                Breakpoint {
                    name: "desktop".to_string(),
                    width: 1200,
                    columns: 12,
                },
            ],
        }
    }
}
