use super::entry::QueueEntry;

/// Constraints for matching players together
#[derive(Debug, Clone)]
pub struct MatchConstraints {
    /// Maximum MMR difference between players
    pub max_rating_delta: f64,
    /// Must players be in the same region?
    pub same_region_required: bool,
    /// Role requirements (e.g., need 1 tank, 1 healer, 3 dps)
    pub role_requirements: Vec<RoleRequirement>,
    /// Maximum wait time before relaxing constraints
    pub max_wait_time_seconds: i64,
    /// How much to expand search range per second waited
    pub expansion_rate: f64,
}

#[derive(Debug, Clone)]
pub struct RoleRequirement {
    pub role: String,
    pub count: usize,
}

impl MatchConstraints {
    pub fn permissive() -> Self {
        Self {
            max_rating_delta: 500.0,
            same_region_required: false,
            role_requirements: Vec::new(),
            max_wait_time_seconds: 60,
            expansion_rate: 10.0,
        }
    }

    pub fn strict() -> Self {
        Self {
            max_rating_delta: 100.0,
            same_region_required: true,
            role_requirements: Vec::new(),
            max_wait_time_seconds: 300,
            expansion_rate: 5.0,
        }
    }

    /// Calculate effective rating delta based on wait time
    pub fn effective_rating_delta(&self, entry: &QueueEntry) -> f64 {
        let wait_seconds = entry.wait_time().num_seconds();
        let expansion = (wait_seconds as f64) * self.expansion_rate;
        self.max_rating_delta + expansion
    }

    /// Check if two entries can be matched together
    pub fn can_match(&self, entry_a: &QueueEntry, entry_b: &QueueEntry) -> bool {
        // Check rating constraint with expansion
        let max_delta = self.effective_rating_delta(entry_a).max(self.effective_rating_delta(entry_b));
        let rating_diff = (entry_a.average_rating.rating - entry_b.average_rating.rating).abs();

        if rating_diff > max_delta {
            return false;
        }

        // Check region constraint
        if self.same_region_required {
            match (&entry_a.metadata.region, &entry_b.metadata.region) {
                (Some(r1), Some(r2)) if r1 == r2 => {},
                (None, None) => {},
                _ => return false,
            }
        }

        true
    }
}

impl Default for MatchConstraints {
    fn default() -> Self {
        Self::permissive()
    }
}
