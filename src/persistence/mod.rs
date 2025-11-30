pub mod memory;
#[cfg(feature = "postgres")]
pub mod postgres;
pub mod redis;
pub mod traits;

#[cfg(feature = "redis")]
pub use redis::{CleanupStats, PlayerStats, QueueStats, RedisAdapter};

#[cfg(feature = "postgres")]
pub use postgres::{CleanupStats as PgCleanupStats, DatabaseMetrics, PlayerStats as PgPlayerStats, PostgresAdapter, QueueStats as PgQueueStats};

pub use memory::InMemoryAdapter;
pub use traits::PersistenceAdapter;
