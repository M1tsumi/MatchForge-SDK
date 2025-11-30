use serde::{Deserialize, Serialize};

/// Configuration for the matchmaking runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// How often to run matchmaking ticks (in milliseconds)
    pub tick_interval_ms: u64,
    /// Maximum number of matches to process per tick
    pub max_matches_per_tick: usize,
    /// Whether to automatically dispatch ready lobbies
    pub auto_dispatch: bool,
    /// Queue-specific configurations
    pub queue_configs: std::collections::HashMap<String, QueueRunnerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueRunnerConfig {
    /// Enable this queue for automatic processing
    pub enabled: bool,
    /// Priority for processing this queue (lower = higher priority)
    pub priority: u8,
    /// Maximum concurrent matches for this queue
    pub max_concurrent_matches: usize,
}

impl RunnerConfig {
    pub fn default() -> Self {
        let mut queue_configs = std::collections::HashMap::new();
        
        // Default configuration for common queues
        queue_configs.insert("ranked_1v1".to_string(), QueueRunnerConfig {
            enabled: true,
            priority: 1,
            max_concurrent_matches: 100,
        });
        
        queue_configs.insert("casual_5v5".to_string(), QueueRunnerConfig {
            enabled: true,
            priority: 2,
            max_concurrent_matches: 50,
        });

        Self {
            tick_interval_ms: 1000, // 1 second
            max_matches_per_tick: 1000,
            auto_dispatch: true,
            queue_configs,
        }
    }

    pub fn fast() -> Self {
        let mut config = Self::default();
        config.tick_interval_ms = 500; // 0.5 seconds
        config
    }

    pub fn slow() -> Self {
        let mut config = Self::default();
        config.tick_interval_ms = 5000; // 5 seconds
        config
    }
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self::default()
    }
}
