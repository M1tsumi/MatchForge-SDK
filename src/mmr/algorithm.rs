use super::rating::{Outcome, Rating};
use async_trait::async_trait;

/// Trait for MMR calculation algorithms
#[async_trait]
pub trait MmrAlgorithm: Send + Sync {
    /// Calculate new rating after a match
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating;

    /// Get the name of this algorithm
    fn name(&self) -> &str;
}

/// Simple Elo rating system
pub struct EloAlgorithm {
    k_factor: f64,
}

impl EloAlgorithm {
    pub fn new(k_factor: f64) -> Self {
        Self { k_factor }
    }

    pub fn default() -> Self {
        Self { k_factor: 32.0 }
    }

    fn expected_score(&self, rating_a: f64, rating_b: f64) -> f64 {
        1.0 / (1.0 + 10_f64.powf((rating_b - rating_a) / 400.0))
    }
}

#[async_trait]
impl MmrAlgorithm for EloAlgorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        let expected = self.expected_score(player_rating.rating, opponent_rating.rating);
        let actual = outcome.score();
        let new_rating = player_rating.rating + self.k_factor * (actual - expected);

        Rating {
            rating: new_rating,
            deviation: player_rating.deviation * 0.99, // Slight confidence increase
            volatility: player_rating.volatility,
        }
    }

    fn name(&self) -> &str {
        "Elo"
    }
}

/// Glicko2 rating system (simplified implementation)
pub struct Glicko2Algorithm {
    tau: f64, // System volatility constant
}

impl Glicko2Algorithm {
    pub fn new(tau: f64) -> Self {
        Self { tau }
    }

    pub fn default() -> Self {
        Self { tau: 0.5 }
    }

    fn g(&self, deviation: f64) -> f64 {
        let q = (3.0 * deviation.powi(2)) / std::f64::consts::PI.powi(2);
        1.0 / (1.0 + q).sqrt()
    }

    fn expected_score(&self, rating: f64, opponent_rating: f64, opponent_deviation: f64) -> f64 {
        let g_value = self.g(opponent_deviation);
        1.0 / (1.0 + (-g_value * (rating - opponent_rating) / 400.0).exp())
    }
}

#[async_trait]
impl MmrAlgorithm for Glicko2Algorithm {
    fn calculate_new_rating(
        &self,
        player_rating: Rating,
        opponent_rating: Rating,
        outcome: Outcome,
    ) -> Rating {
        let g_value = self.g(opponent_rating.deviation);
        let expected = self.expected_score(
            player_rating.rating,
            opponent_rating.rating,
            opponent_rating.deviation,
        );
        let actual = outcome.score();

        // Simplified Glicko2 update (full algorithm is more complex)
        let d_squared = 1.0 / (g_value.powi(2) * expected * (1.0 - expected));
        let variance = 1.0 / d_squared;

        let delta = variance * g_value * (actual - expected);
        let new_rating = player_rating.rating + delta;

        // Update deviation (simplified)
        let new_deviation = (player_rating.deviation.powi(2) + variance).sqrt();

        Rating {
            rating: new_rating,
            deviation: new_deviation.min(350.0), // Cap deviation
            volatility: player_rating.volatility,
        }
    }

    fn name(&self) -> &str {
        "Glicko2"
    }
}
