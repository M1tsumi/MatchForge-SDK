use serde::{Deserialize, Serialize};

/// Represents a player's skill rating
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rating {
    /// Current skill rating (e.g., 1500)
    pub rating: f64,
    /// Rating deviation/uncertainty (used in Glicko systems)
    pub deviation: f64,
    /// Volatility (rate of rating change, Glicko2 only)
    pub volatility: f64,
}

impl Rating {
    pub fn new(rating: f64, deviation: f64, volatility: f64) -> Self {
        Self {
            rating,
            deviation,
            volatility,
        }
    }

    /// Create a default beginner rating
    pub fn default_beginner() -> Self {
        Self {
            rating: 1500.0,
            deviation: 350.0,
            volatility: 0.06,
        }
    }

    /// Get a conservative estimate of skill (rating - 2*deviation)
    pub fn conservative_estimate(&self) -> f64 {
        self.rating - 2.0 * self.deviation
    }
}

impl Default for Rating {
    fn default() -> Self {
        Self::default_beginner()
    }
}

/// Match outcome from a player's perspective
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Outcome {
    pub fn score(&self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Loss => 0.0,
            Outcome::Draw => 0.5,
        }
    }
}
