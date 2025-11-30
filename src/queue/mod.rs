pub mod constraints;
pub mod entry;
pub mod manager;
pub mod matcher;
pub mod advanced_strategies;

pub use constraints::{MatchConstraints, RoleRequirement};
pub use entry::{EntryMetadata, QueueEntry};
pub use manager::{QueueConfig, QueueManager};
pub use matcher::{GreedyMatcher, MatchFormat, MatchResult};
pub use advanced_strategies::{
    AdaptiveMatcher, FairTeamBalancer, SeedingStrategy, SwissMatcher, 
    TournamentBracket, TournamentMatch, TournamentMatcher, TournamentType,
};
