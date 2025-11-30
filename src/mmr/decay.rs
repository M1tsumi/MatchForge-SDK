use super::rating::Rating;
use chrono::{DateTime, Utc};

/// MMR decay strategy
pub trait DecayStrategy: Send + Sync {
    /// Apply decay to a rating based on inactivity
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating;
}

/// Linear decay: reduce rating by a fixed amount per time period
pub struct LinearDecay {
    pub decay_per_day: f64,
    pub max_decay: f64,
}

impl LinearDecay {
    pub fn new(decay_per_day: f64, max_decay: f64) -> Self {
        Self {
            decay_per_day,
            max_decay,
        }
    }

    pub fn default() -> Self {
        Self {
            decay_per_day: 1.0,
            max_decay: 100.0,
        }
    }
}

impl DecayStrategy for LinearDecay {
    fn apply_decay(&self, rating: Rating, last_match_time: DateTime<Utc>) -> Rating {
        let now = Utc::now();
        let days_inactive = (now - last_match_time).num_days() as f64;

        if days_inactive <= 0.0 {
            return rating;
        }

        let decay_amount = (self.decay_per_day * days_inactive).min(self.max_decay);

        Rating {
            rating: (rating.rating - decay_amount).max(0.0),
            deviation: (rating.deviation + days_inactive * 0.5).min(350.0), // Increase uncertainty
            volatility: rating.volatility,
        }
    }
}

/// No decay strategy
pub struct NoDecay;

impl DecayStrategy for NoDecay {
    fn apply_decay(&self, rating: Rating, _last_match_time: DateTime<Utc>) -> Rating {
        rating
    }
}
