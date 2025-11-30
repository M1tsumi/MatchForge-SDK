use super::traits::PersistenceAdapter;
use crate::{error::*, lobby::Lobby, mmr::Rating, party::Party, queue::QueueEntry};
use async_trait::async_trait;
use sqlx::{postgres::PgRow, PgPool, Row};
use uuid::Uuid;

/// Postgres persistence adapter
/// 
/// Provides a production-ready persistence layer using PostgreSQL as the backend.
/// Supports all MatchForge operations with proper SQL schema and indexing.
pub struct PostgresAdapter {
    pool: PgPool,
}

impl PostgresAdapter {
    /// Create a new Postgres adapter with the given connection string
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPool::connect(connection_string).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let adapter = Self { pool };
        
        // Initialize database schema
        adapter.init_schema().await?;
        
        Ok(adapter)
    }
    
    /// Initialize the database schema
    async fn init_schema(&self) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS player_ratings (
                player_id UUID PRIMARY KEY,
                rating DOUBLE PRECISION NOT NULL,
                deviation DOUBLE PRECISION NOT NULL,
                volatility DOUBLE PRECISION NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_player_ratings_updated_at ON player_ratings(updated_at);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS queue_entries (
                id UUID PRIMARY KEY,
                queue_name VARCHAR(255) NOT NULL,
                player_ids UUID[] NOT NULL,
                party_id UUID,
                average_rating DOUBLE PRECISION NOT NULL,
                average_deviation DOUBLE PRECISION NOT NULL,
                average_volatility DOUBLE PRECISION NOT NULL,
                joined_at TIMESTAMP WITH TIME ZONE NOT NULL,
                metadata JSONB DEFAULT '{}',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_queue_entries_queue_name ON queue_entries(queue_name);
            CREATE INDEX IF NOT EXISTS idx_queue_entries_joined_at ON queue_entries(joined_at);
            CREATE INDEX IF NOT EXISTS idx_queue_entries_player_ids ON queue_entries USING GIN(player_ids);
            CREATE INDEX IF NOT EXISTS idx_queue_entries_party_id ON queue_entries(party_id);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS parties (
                id UUID PRIMARY KEY,
                leader_id UUID NOT NULL,
                member_ids UUID[] NOT NULL,
                max_size INTEGER NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_parties_leader_id ON parties(leader_id);
            CREATE INDEX IF NOT EXISTS idx_parties_member_ids ON parties USING GIN(member_ids);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lobbies (
                id UUID PRIMARY KEY,
                match_id UUID NOT NULL,
                state VARCHAR(50) NOT NULL,
                player_ids UUID[] NOT NULL,
                teams JSONB NOT NULL,
                ready_players UUID[] DEFAULT '{}',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                metadata JSONB DEFAULT '{}'
            );
            
            CREATE INDEX IF NOT EXISTS idx_lobbies_match_id ON lobbies(match_id);
            CREATE INDEX IF NOT EXISTS idx_lobbies_state ON lobbies(state);
            CREATE INDEX IF NOT EXISTS idx_lobbies_created_at ON lobbies(created_at);
            CREATE INDEX IF NOT EXISTS idx_lobbies_player_ids ON lobbies USING GIN(player_ids);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS match_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                match_id UUID NOT NULL,
                lobby_data JSONB NOT NULL,
                completed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_match_history_match_id ON match_history(match_id);
            CREATE INDEX IF NOT EXISTS idx_match_history_completed_at ON match_history(completed_at);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS player_match_history (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                player_id UUID NOT NULL,
                match_id UUID NOT NULL,
                outcome VARCHAR(20) NOT NULL,
                rating_before DOUBLE PRECISION,
                rating_after DOUBLE PRECISION,
                played_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_player_match_history_player_id ON player_match_history(player_id);
            CREATE INDEX IF NOT EXISTS idx_player_match_history_match_id ON player_match_history(match_id);
            CREATE INDEX IF NOT EXISTS idx_player_match_history_played_at ON player_match_history(played_at);
            "#
        ).execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Helper to convert row to Rating
    fn row_to_rating(row: &PgRow) -> Result<Rating> {
        Ok(Rating {
            rating: row.try_get("rating")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            deviation: row.try_get("deviation")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            volatility: row.try_get("volatility")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
        })
    }
    
    /// Helper to convert row to QueueEntry
    fn row_to_queue_entry(row: &PgRow) -> Result<QueueEntry> {
        let metadata_json: serde_json::Value = row.try_get("metadata")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let metadata: EntryMetadata = serde_json::from_value(metadata_json)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(QueueEntry {
            id: row.try_get("id")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            queue_name: row.try_get("queue_name")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            player_ids: row.try_get("player_ids")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            party_id: row.try_get("party_id")
                .map_err(|_| MatchForgeError::PersistenceError("Failed to parse party_id".to_string()))?,
            average_rating: Rating {
                rating: row.try_get("average_rating")
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
                deviation: row.try_get("average_deviation")
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
                volatility: row.try_get("average_volatility")
                    .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            },
            joined_at: row.try_get("joined_at")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            metadata,
        })
    }
    
    /// Helper to convert row to Party
    fn row_to_party(row: &PgRow) -> Result<Party> {
        Ok(Party {
            id: row.try_get("id")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            leader_id: row.try_get("leader_id")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            member_ids: row.try_get("member_ids")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            max_size: row.try_get("max_size")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            created_at: row.try_get("created_at")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
        })
    }
    
    /// Helper to convert row to Lobby
    fn row_to_lobby(row: &PgRow) -> Result<Lobby> {
        let teams_json: serde_json::Value = row.try_get("teams")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let teams: Vec<Team> = serde_json::from_value(teams_json)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let metadata_json: serde_json::Value = row.try_get("metadata")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let metadata: LobbyMetadata = serde_json::from_value(metadata_json)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let state_str: String = row.try_get("state")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let state = match state_str.as_str() {
            "Forming" => LobbyState::Forming,
            "WaitingForReady" => LobbyState::WaitingForReady,
            "Ready" => LobbyState::Ready,
            "Dispatched" => LobbyState::Dispatched,
            "Closed" => LobbyState::Closed,
            _ => return Err(MatchForgeError::PersistenceError(format!("Invalid lobby state: {}", state_str))),
        };
        
        let ready_players: std::collections::HashSet<Uuid> = row.try_get("ready_players")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(Lobby {
            id: row.try_get("id")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            match_id: row.try_get("match_id")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            state,
            teams,
            player_ids: row.try_get("player_ids")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            ready_players,
            created_at: row.try_get("created_at")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            metadata,
        })
    }
}

#[async_trait]
impl PersistenceAdapter for PostgresAdapter {
    async fn save_player_rating(&self, player_id: Uuid, rating: Rating) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO player_ratings (player_id, rating, deviation, volatility)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (player_id) 
            DO UPDATE SET 
                rating = EXCLUDED.rating,
                deviation = EXCLUDED.deviation,
                volatility = EXCLUDED.volatility,
                updated_at = NOW()
            "#
        )
        .bind(player_id)
        .bind(rating.rating)
        .bind(rating.deviation)
        .bind(rating.volatility)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_player_rating(&self, player_id: Uuid) -> Result<Option<Rating>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let row = sqlx::query(
            "SELECT rating, deviation, volatility FROM player_ratings WHERE player_id = $1"
        )
        .bind(player_id)
        .fetch_optional(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(row.map(|r| self.row_to_rating(&r)).transpose()?)
    }

    async fn save_queue_entry(&self, entry: &QueueEntry) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let metadata_json = serde_json::to_value(&entry.metadata)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO queue_entries (
                id, queue_name, player_ids, party_id, 
                average_rating, average_deviation, average_volatility,
                joined_at, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) 
            DO UPDATE SET 
                queue_name = EXCLUDED.queue_name,
                player_ids = EXCLUDED.player_ids,
                party_id = EXCLUDED.party_id,
                average_rating = EXCLUDED.average_rating,
                average_deviation = EXCLUDED.average_deviation,
                average_volatility = EXCLUDED.average_volatility,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(entry.id)
        .bind(&entry.queue_name)
        .bind(&entry.player_ids)
        .bind(entry.party_id)
        .bind(entry.average_rating.rating)
        .bind(entry.average_rating.deviation)
        .bind(entry.average_rating.volatility)
        .bind(entry.joined_at)
        .bind(metadata_json)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_queue_entries(&self, queue_name: &str) -> Result<Vec<QueueEntry>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let rows = sqlx::query(
            "SELECT * FROM queue_entries WHERE queue_name = $1 ORDER BY joined_at ASC"
        )
        .bind(queue_name)
        .fetch_all(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(self.row_to_queue_entry(&row)?);
        }
        
        Ok(entries)
    }

    async fn delete_queue_entry(&self, player_id: Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query("DELETE FROM queue_entries WHERE $1 = ANY(player_ids)")
        .bind(player_id)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn save_party(&self, party: &Party) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO parties (id, leader_id, member_ids, max_size)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) 
            DO UPDATE SET 
                leader_id = EXCLUDED.leader_id,
                member_ids = EXCLUDED.member_ids,
                max_size = EXCLUDED.max_size
            "#
        )
        .bind(party.id)
        .bind(party.leader_id)
        .bind(&party.member_ids)
        .bind(party.max_size as i32)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_party(&self, party_id: Uuid) -> Result<Option<Party>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let row = sqlx::query("SELECT * FROM parties WHERE id = $1")
        .bind(party_id)
        .fetch_optional(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(row.map(|r| self.row_to_party(&r)).transpose()?)
    }

    async fn delete_party(&self, party_id: Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query("DELETE FROM parties WHERE id = $1")
        .bind(party_id)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn save_lobby(&self, lobby: &Lobby) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let teams_json = serde_json::to_value(&lobby.teams)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let metadata_json = serde_json::to_value(&lobby.metadata)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let ready_players: Vec<Uuid> = lobby.ready_players.iter().cloned().collect();
        let state_str = format!("{:?}", lobby.state);
        
        sqlx::query(
            r#"
            INSERT INTO lobbies (
                id, match_id, state, player_ids, teams, ready_players, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) 
            DO UPDATE SET 
                state = EXCLUDED.state,
                player_ids = EXCLUDED.player_ids,
                teams = EXCLUDED.teams,
                ready_players = EXCLUDED.ready_players,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(lobby.id)
        .bind(lobby.match_id)
        .bind(&state_str)
        .bind(&lobby.player_ids)
        .bind(teams_json)
        .bind(&ready_players)
        .bind(metadata_json)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn load_lobby(&self, lobby_id: Uuid) -> Result<Option<Lobby>> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let row = sqlx::query("SELECT * FROM lobbies WHERE id = $1")
        .bind(lobby_id)
        .fetch_optional(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(row.map(|r| self.row_to_lobby(&r)).transpose()?)
    }

    async fn delete_lobby(&self, lobby_id: Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query("DELETE FROM lobbies WHERE id = $1")
        .bind(lobby_id)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }

    async fn save_match_result(&self, lobby: &Lobby) -> Result<()> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let lobby_data = serde_json::to_value(lobby)
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        sqlx::query(
            "INSERT INTO match_history (match_id, lobby_data) VALUES ($1, $2)"
        )
        .bind(lobby.match_id)
        .bind(lobby_data)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(())
    }
}

/// Additional utility methods for Postgres adapter
impl PostgresAdapter {
    /// Get queue statistics
    pub async fn get_queue_stats(&self, queue_name: &str) -> Result<QueueStats> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as size,
                AVG(EXTRACT(EPOCH FROM (NOW() - joined_at))) as avg_wait_seconds,
                AVG(average_rating) as avg_rating
            FROM queue_entries 
            WHERE queue_name = $1
            "#
        )
        .bind(queue_name)
        .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(QueueStats {
            name: queue_name.to_string(),
            size: row.try_get("size")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?,
            avg_wait_time_seconds: row.try_get::<Option<f64>, _>("avg_wait_seconds")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
                .unwrap_or(0.0) as i64,
            avg_rating: row.try_get::<Option<f64>, _>("avg_rating")
                .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
                .unwrap_or(0.0),
        })
    }
    
    /// Get player statistics
    pub async fn get_player_stats(&self, player_id: Uuid) -> Result<PlayerStats> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Get rating
        let rating = self.load_player_rating(player_id).await?;
        
        // Get match history count
        let matches_played: i64 = sqlx::query(
            "SELECT COUNT(*) FROM player_match_history WHERE player_id = $1"
        )
        .bind(player_id)
        .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Check if player is in queue
        let in_queue: bool = sqlx::query(
            "SELECT EXISTS(SELECT 1 FROM queue_entries WHERE $1 = ANY(player_ids))"
        )
        .bind(player_id)
        .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("exists")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Check if player is in party
        let party_row = sqlx::query("SELECT id FROM parties WHERE $1 = ANY(member_ids)")
        .bind(player_id)
        .fetch_optional(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let party_id = party_row.map(|r| r.try_get("id"))
            .transpose()
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(PlayerStats {
            player_id,
            rating,
            matches_played: matches_played as usize,
            in_queue,
            party_id,
        })
    }
    
    /// Clean up expired data
    pub async fn cleanup_expired_data(&self) -> Result<CleanupStats> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        let mut stats = CleanupStats::default();
        
        // Clean up old queue entries (older than 1 hour)
        let cutoff_time = Utc::now() - chrono::Duration::hours(1);
        
        let result = sqlx::query(
            "DELETE FROM queue_entries WHERE joined_at < $1"
        )
        .bind(cutoff_time)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        stats.cleaned_queue_entries = result.rows_affected() as usize;
        
        // Clean up old lobbies (closed for more than 24 hours)
        let lobby_cutoff = Utc::now() - chrono::Duration::hours(24);
        
        let result = sqlx::query(
            "DELETE FROM lobbies WHERE state = 'Closed' AND created_at < $1"
        )
        .bind(lobby_cutoff)
        .execute(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        stats.cleaned_lobbies = result.rows_affected() as usize;
        
        Ok(stats)
    }
    
    /// Get database performance metrics
    pub async fn get_database_metrics(&self) -> Result<DatabaseMetrics> {
        let mut conn = self.pool.acquire().await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        // Get table sizes
        let player_ratings_count: i64 = sqlx::query("SELECT COUNT(*) FROM player_ratings")
            .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let queue_entries_count: i64 = sqlx::query("SELECT COUNT(*) FROM queue_entries")
            .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let parties_count: i64 = sqlx::query("SELECT COUNT(*) FROM parties")
            .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let lobbies_count: i64 = sqlx::query("SELECT COUNT(*) FROM lobbies")
            .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        let match_history_count: i64 = sqlx::query("SELECT COUNT(*) FROM match_history")
            .fetch_one(&mut conn).await
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?
            .try_get("count")
            .map_err(|e| MatchForgeError::PersistenceError(e.to_string()))?;
        
        Ok(DatabaseMetrics {
            player_ratings_count: player_ratings_count as usize,
            queue_entries_count: queue_entries_count as usize,
            parties_count: parties_count as usize,
            lobbies_count: lobbies_count as usize,
            match_history_count: match_history_count as usize,
        })
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

/// Database performance metrics
#[derive(Debug, Clone)]
pub struct DatabaseMetrics {
    pub player_ratings_count: usize,
    pub queue_entries_count: usize,
    pub parties_count: usize,
    pub lobbies_count: usize,
    pub match_history_count: usize,
}
