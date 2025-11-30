use crate::mmr::Rating;
use uuid::Uuid;

/// Strategy for calculating party MMR from individual ratings
pub trait PartyMmrStrategy: Send + Sync {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating;
}

/// Use the average MMR
pub struct AverageStrategy;

impl PartyMmrStrategy for AverageStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        if ratings.is_empty() {
            return Rating::default();
        }

        let sum: f64 = ratings.iter().map(|(_, r)| r.rating).sum();
        let avg_rating = sum / ratings.len() as f64;

        let avg_deviation: f64 = ratings.iter().map(|(_, r)| r.deviation).sum::<f64>()
            / ratings.len() as f64;

        Rating {
            rating: avg_rating,
            deviation: avg_deviation,
            volatility: 0.06,
        }
    }
}

/// Use the highest MMR (conservative for opponents)
pub struct MaxStrategy;

impl PartyMmrStrategy for MaxStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        ratings
            .iter()
            .max_by(|a, b| a.1.rating.partial_cmp(&b.1.rating).unwrap())
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }
}

/// Weighted average with penalty for skill gaps
pub struct WeightedWithPenaltyStrategy {
    pub gap_penalty: f64,
}

impl PartyMmrStrategy for WeightedWithPenaltyStrategy {
    fn calculate_party_rating(&self, ratings: &[(Uuid, Rating)]) -> Rating {
        if ratings.is_empty() {
            return Rating::default();
        }

        let ratings_only: Vec<f64> = ratings.iter().map(|(_, r)| r.rating).collect();
        let max_rating = ratings_only.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_rating = ratings_only.iter().copied().fold(f64::INFINITY, f64::min);
        let gap = max_rating - min_rating;

        let avg_rating: f64 = ratings_only.iter().sum::<f64>() / ratings.len() as f64;
        let penalty = gap * self.gap_penalty;

        Rating {
            rating: avg_rating + penalty,
            deviation: 200.0,
            volatility: 0.06,
        }
    }
}
