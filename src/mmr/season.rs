use super::rating::Rating;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a competitive season
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl Season {
    pub fn is_active(&self) -> bool {
        let now = Utc::now();
        now >= self.start_time && now < self.end_time
    }
}

/// Strategy for resetting ratings at season boundaries
pub trait SeasonResetStrategy: Send + Sync {
    /// Calculate the new rating at the start of a season
    fn reset_rating(&self, current_rating: Rating) -> Rating;
}

/// Soft reset: move rating toward the mean
pub struct SoftReset {
    pub target_rating: f64,
    pub reset_percentage: f64, // 0.0 to 1.0
}

impl SoftReset {
    pub fn new(target_rating: f64, reset_percentage: f64) -> Self {
        Self {
            target_rating,
            reset_percentage,
        }
    }

    pub fn default() -> Self {
        Self {
            target_rating: 1500.0,
            reset_percentage: 0.5,
        }
    }
}

impl SeasonResetStrategy for SoftReset {
    fn reset_rating(&self, current_rating: Rating) -> Rating {
        let diff = self.target_rating - current_rating.rating;
        let new_rating = current_rating.rating + diff * self.reset_percentage;

        Rating {
            rating: new_rating,
            deviation: 200.0, // Moderate uncertainty for new season
            volatility: current_rating.volatility,
        }
    }
}

/// Hard reset: everyone starts at the same rating
pub struct HardReset {
    pub reset_rating: f64,
}

impl HardReset {
    pub fn new(reset_rating: f64) -> Self {
        Self { reset_rating }
    }
}

impl SeasonResetStrategy for HardReset {
    fn reset_rating(&self, _current_rating: Rating) -> Rating {
        Rating {
            rating: self.reset_rating,
            deviation: 350.0, // High uncertainty
            volatility: 0.06,
        }
    }
}
