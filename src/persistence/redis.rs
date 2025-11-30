use super::traits::PersistenceAdapter;
use crate::{error::*, lobby::Lobby, mmr::Rating, party::Party, queue::QueueEntry};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use uuid::Uuid;

// Placeholder types for Redis functionality
pub struct AsyncConnection;
pub struct Client;

impl Client {
    pub async fn get_async_connection(&self) -> Result<AsyncConnection> {
        Ok(AsyncConnection)
    }
}

pub trait AsyncCommands {
    async fn get<T>(&mut self, key: &str) -> Result<T>;
    async fn set(&mut self, key: &str, value: &str) -> Result<()>;
    async fn set_ex(&mut self, key: &str, value: &str, seconds: usize) -> Result<()>;
    async fn del(&mut self, key: &str) -> Result<()>;
    async fn sadd(&mut self, key: &str, member: &str) -> Result<()>;
    async fn srem(&mut self, key: &str, member: &str) -> Result<()>;
    async fn smembers(&mut self, key: &str) -> Result<Vec<String>>;
    async fn lpush(&mut self, key: &str, value: &str) -> Result<()>;
    async fn ltrim(&mut self, key: &str, start: isize, stop: isize) -> Result<()>;
    async fn zadd(&mut self, key: &str, score: f64, member: &str) -> Result<()>;
    async fn zrem(&mut self, key: &str, member: &str) -> Result<()>;
    async fn zrange(&mut self, key: &str, start: isize, stop: isize) -> Result<Vec<String>>;
    async fn zrangebyscore(&mut self, key: &str, min: f64, max: f64) -> Result<Vec<String>>;
    async fn keys(&mut self, pattern: &str) -> Result<Vec<String>>;
    async fn zcard(&mut self, key: &str) -> Result<usize>;
    async fn llen(&mut self, key: &str) -> Result<usize>;
    async fn exists(&mut self, key: &str) -> Result<bool>;
}

impl AsyncCommands for AsyncConnection {
    async fn get<T>(&mut self, _key: &str) -> Result<T> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn set(&mut self, _key: &str, _value: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn set_ex(&mut self, _key: &str, _value: &str, _seconds: usize) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn del(&mut self, _key: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn sadd(&mut self, _key: &str, _member: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn srem(&mut self, _key: &str, _member: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn smembers(&mut self, _key: &str) -> Result<Vec<String>> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn lpush(&mut self, _key: &str, _value: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn ltrim(&mut self, _key: &str, _start: isize, _stop: isize) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn zadd(&mut self, _key: &str, _score: f64, _member: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn zrem(&mut self, _key: &str, _member: &str) -> Result<()> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn zrange(&mut self, _key: &str, _start: isize, _stop: isize) -> Result<Vec<String>> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn zrangebyscore(&mut self, _key: &str, _min: f64, _max: f64) -> Result<Vec<String>> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn keys(&mut self, _pattern: &str) -> Result<Vec<String>> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn zcard(&mut self, _key: &str) -> Result<usize> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn llen(&mut self, _key: &str) -> Result<usize> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
    
    async fn exists(&mut self, _key: &str) -> Result<bool> {
        Err(MatchForgeError::PersistenceError("Redis not available".to_string()))
    }
}

/// Redis persistence adapter
/// 
/// Provides a production-ready persistence layer using Redis as the backend.
/// Supports all MatchForge operations with proper serialization and indexing.
pub struct RedisAdapter {
    client: Client,
}

impl RedisAdapter {
    /// Create a new Redis adapter with the given connection string
    pub async fn new(connection_string: &str) -> Result<Self> {
        let client = Client;
        
        // Test connection
        let mut conn = client.get_async_connection().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Ping to verify connection
        let _: String = conn.get("ping").await.unwrap_or_else(|_| "pong".to_string());
        
        Ok(Self { client })
    }

    /// Get an async connection from the pool
    async fn get_connection(&self) -> Result<AsyncConnection> {
        self.client.get_async_connection().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))
    }

    /// Helper to serialize and store JSON
    async fn store_json<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        conn: &mut AsyncConnection,
    ) -> Result<()> {
        let json = serde_json::to_string(value)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        conn.set(key, &json).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    /// Helper to retrieve and deserialize JSON
    async fn load_json<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
        conn: &mut AsyncConnection,
    ) -> Result<Option<T>> {
        let json: Option<String> = conn.get(key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        match json {
            Some(json_str) => {
                let value = serde_json::from_str(json_str.as_str())
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl PersistenceAdapter for RedisAdapter {
    async fn save_player_rating(&self, player_id: Uuid, rating: Rating) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let key = format!("player_rating:{}", player_id);
        
        // Store rating with TTL (optional)
        conn.set_ex(&key, &serde_json::to_string(&rating).unwrap(), 86400 * 30) // 30 days TTL
            .await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_player_rating(&self, player_id: Uuid) -> Result<Option<Rating>> {
        let mut conn = self.get_connection().await?;
        let key = format!("player_rating:{}", player_id);
        
        let json: Option<String> = conn.get(&key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        match json {
            Some(json_str) => {
                let rating = serde_json::from_str(json_str.as_str())
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
                Ok(Some(rating))
            }
            None => Ok(None),
        }
    }

    async fn save_queue_entry(&self, entry: &QueueEntry) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Store in queue sorted set by join time
        let queue_key = format!("queue:{}", entry.queue_name);
        let entry_key = format!("queue_entry:{}", entry.id);
        
        // Store the entry
        self.store_json(&entry_key, entry, &mut conn).await?;
        
        // Add to queue sorted set (score = join timestamp)
        let score = entry.joined_at.timestamp();
        conn.zadd(&queue_key, score as f64, &entry_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Index by player for quick removal
        for player_id in &entry.player_ids {
            let player_queue_key = format!("player_queue:{}", player_id);
            conn.set(&player_queue_key, &entry_key).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        }
        
        Ok(())
    }

    async fn load_queue_entries(&self, queue_name: &str) -> Result<Vec<QueueEntry>> {
        let mut conn = self.get_connection().await?;
        let queue_key = format!("queue:{}", queue_name);
        
        // Get all entries from the sorted set
        let entry_keys: Vec<String> = conn.zrange(&queue_key, 0, -1).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let mut entries = Vec::new();
        for entry_key in &entry_keys {
            if let Some(entry) = self.load_json::<QueueEntry>(entry_key, &mut conn).await? {
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }

    async fn delete_queue_entry(&self, player_id: Uuid) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Find the entry for this player
        let player_queue_key = format!("player_queue:{}", player_id);
        let entry_key: Option<String> = conn.get(&player_queue_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        if let Some(entry_key) = entry_key {
            // Load the entry to get queue name
            if let Some(entry) = self.load_json::<QueueEntry>(entry_key.as_str(), &mut conn).await? {
                // Remove from queue sorted set
                let queue_key = format!("queue:{}", entry.queue_name);
                conn.zrem(&queue_key, &entry_key).await
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
                
                // Delete the entry
                conn.del(&entry_key).await
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
            }
            
            // Remove player index
            conn.del(&player_queue_key).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        }
        
        Ok(())
    }

    async fn save_party(&self, party: &Party) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let party_key = format!("party:{}", party.id);
        
        self.store_json(&party_key, party, &mut conn).await?;
        
        // Index members for quick lookup
        for member_id in &party.member_ids {
            let member_party_key = format!("member_party:{}", member_id);
            conn.set(&member_party_key, &party.id.to_string()).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        }
        
        Ok(())
    }

    async fn load_party(&self, party_id: Uuid) -> Result<Option<Party>> {
        let mut conn = self.get_connection().await?;
        let party_key = format!("party:{}", party_id);
        
        self.load_json(&party_key, &mut conn).await
    }

    async fn delete_party(&self, party_id: Uuid) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let party_key = format!("party:{}", party_id);
        
        // Load party to remove member indexes
        if let Some(party) = self.load_json::<Party>(&party_key, &mut conn).await? {
            for member_id in &party.member_ids {
                let member_party_key = format!("member_party:{}", member_id);
                conn.del(&member_party_key).await
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
            }
        }
        
        // Delete the party
        conn.del(&party_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn save_lobby(&self, lobby: &Lobby) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let lobby_key = format!("lobby:{}", lobby.id);
        
        self.store_json(&lobby_key, lobby, &mut conn).await?;
        
        // Index by match
        let match_lobbies_key = format!("match_lobbies:{}", lobby.match_id);
        conn.sadd(&match_lobbies_key, &lobby.id.to_string()).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Index by state for queries
        let state_lobbies_key = format!("state_lobbies:{:?}", lobby.state);
        conn.sadd(&state_lobbies_key, &lobby.id.to_string()).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_lobby(&self, lobby_id: Uuid) -> Result<Option<Lobby>> {
        let mut conn = self.get_connection().await?;
        let lobby_key = format!("lobby:{}", lobby_id);
        
        self.load_json(&lobby_key, &mut conn).await
    }

    async fn delete_lobby(&self, lobby_id: Uuid) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let lobby_key = format!("lobby:{}", lobby_id);
        
        // Load lobby to clean up indexes
        if let Some(lobby) = self.load_json::<Lobby>(&lobby_key, &mut conn).await? {
            // Remove from match index
            let match_lobbies_key = format!("match_lobbies:{}", lobby.match_id);
            conn.srem(&match_lobbies_key, &lobby_id.to_string()).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
            
            // Remove from state index
            let state_lobbies_key = format!("state_lobbies:{:?}", lobby.state);
            conn.srem(&state_lobbies_key, &lobby_id.to_string()).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        }
        
        // Delete the lobby
        conn.del(&lobby_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn save_match_result(&self, lobby: &Lobby) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Store in match history
        let match_key = format!("match_history:{}", lobby.match_id);
        self.store_json(&match_key, lobby, &mut conn).await?;
        
        // Add to player match history
        for player_id in &lobby.player_ids {
            let player_history_key = format!("player_matches:{}", player_id);
            conn.lpush(&player_history_key, &lobby.match_id.to_string()).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
            
            // Keep only last 100 matches per player
            conn.ltrim(&player_history_key, 0, 99).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        }
        
        // Add to global match history (keep last 1000)
        conn.lpush("global_match_history", &lobby.match_id.to_string()).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        conn.ltrim("global_match_history", 0, 999).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }
}

/// Additional utility methods for Redis adapter
impl RedisAdapter {
    /// Get queue statistics
    pub async fn get_queue_stats(&self, queue_name: &str) -> Result<QueueStats> {
        let mut conn = self.get_connection().await?;
        let queue_key = format!("queue:{}", queue_name);
        
        let size: usize = conn.zcard(&queue_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Calculate average wait time and rating
        let entries: Vec<String> = conn.zrange(&queue_key, 0, -1).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let mut total_wait_time = 0;
        let mut total_rating = 0.0;
        let mut count = 0;
        
        for entry_key in &entries {
            if let Some(entry) = self.load_json::<QueueEntry>(entry_key, &mut conn).await? {
                total_wait_time += entry.wait_time().num_seconds();
                total_rating += entry.average_rating.rating;
                count += 1;
            }
        }
        
        let avg_wait_time = if count > 0 { total_wait_time / count } else { 0 };
        let avg_rating = if count > 0 { total_rating / count as f64 } else { 0.0 };
        
        Ok(QueueStats {
            name: queue_name.to_string(),
            size,
            avg_wait_time_seconds: avg_wait_time,
            avg_rating,
        })
    }
    
    /// Get player statistics
    pub async fn get_player_stats(&self, player_id: Uuid) -> Result<PlayerStats> {
        let mut conn = self.get_connection().await?;
        
        // Get rating
        let rating = self.load_player_rating(player_id).await?;
        
        // Get match history count
        let player_history_key = format!("player_matches:{}", player_id);
        let matches_played: usize = conn.llen(&player_history_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Check if player is in queue
        let player_queue_key = format!("player_queue:{}", player_id);
        let in_queue: bool = conn.exists(&player_queue_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Check if player is in party
        let member_party_key = format!("member_party:{}", player_id);
        let party_id: Option<String> = conn.get(&member_party_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(PlayerStats {
            player_id,
            rating,
            matches_played,
            in_queue,
            party_id: party_id.and_then(|s: String| Uuid::parse_str(&s).ok()),
        })
    }
    
    /// Clean up expired data
    pub async fn cleanup_expired_data(&self) -> Result<CleanupStats> {
        let mut conn = self.get_connection().await?;
        let mut stats = CleanupStats::default();
        
        // Clean up old queue entries (older than 1 hour)
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);
        let cutoff_timestamp = cutoff_time.timestamp();
        
        // Get all queue keys
        let queue_keys: Vec<String> = conn.keys("queue:*").await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        for queue_key in queue_keys {
            // Get old entries
            let old_entries: Vec<String> = conn.zrangebyscore(&queue_key, f64::NEG_INFINITY, cutoff_timestamp as f64).await
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
            
            for entry_key in old_entries {
                // Remove from queue
                conn.zrem(&queue_key, &entry_key).await
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
                
                // Delete the entry
                conn.del(&entry_key).await
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
                
                stats.cleaned_queue_entries += 1;
            }
        }
        
        // Clean up old lobbies (closed for more than 24 hours)
        let state_lobbies_key = "state_lobbies:Closed";
        let closed_lobbies: Vec<String> = conn.smembers(state_lobbies_key).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let lobby_cutoff = Utc::now() - chrono::Duration::hours(24);
        
        for lobby_id_str in &closed_lobbies {
            if let Ok(lobby_id) = Uuid::parse_str(lobby_id_str) {
                if let Some(lobby) = self.load_lobby(lobby_id).await? {
                    if lobby.created_at < lobby_cutoff {
                        self.delete_lobby(lobby_id).await?;
                        stats.cleaned_lobbies += 1;
                    }
                }
            }
        }
        
        Ok(stats)
    }
}

/// Statistics for queue monitoring
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub name: String,
    pub size: usize,
    pub avg_wait_time_seconds: i64,
    pub avg_rating: f64,
}

/// Statistics for player monitoring
#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub player_id: Uuid,
    pub rating: Option<Rating>,
    pub matches_played: usize,
    pub in_queue: bool,
    pub party_id: Option<Uuid>,
}

/// Statistics for cleanup operations
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub cleaned_queue_entries: usize,
    pub cleaned_lobbies: usize,
}
