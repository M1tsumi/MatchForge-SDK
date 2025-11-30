use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A party of players queuing together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Party {
    pub id: Uuid,
    pub leader_id: Uuid,
    pub member_ids: Vec<Uuid>,
    pub max_size: usize,
    pub created_at: DateTime<Utc>,
}

impl Party {
    pub fn new(leader_id: Uuid, max_size: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            leader_id,
            member_ids: vec![leader_id],
            max_size,
            created_at: Utc::now(),
        }
    }

    pub fn size(&self) -> usize {
        self.member_ids.len()
    }

    pub fn is_full(&self) -> bool {
        self.size() >= self.max_size
    }

    pub fn has_member(&self, player_id: Uuid) -> bool {
        self.member_ids.contains(&player_id)
    }

    pub fn is_leader(&self, player_id: Uuid) -> bool {
        self.leader_id == player_id
    }
}
