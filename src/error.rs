use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MatchForgeError {
    #[error("Player not found: {0}")]
    PlayerNotFound(Uuid),

    #[error("Party not found: {0}")]
    PartyNotFound(Uuid),

    #[error("Queue not found: {0}")]
    QueueNotFound(String),

    #[error("Lobby not found: {0}")]
    LobbyNotFound(Uuid),

    #[error("Player already in queue: {0}")]
    AlreadyInQueue(Uuid),

    #[error("Player not in queue: {0}")]
    NotInQueue(Uuid),

    #[error("Party is full (max size: {0})")]
    PartyFull(usize),

    #[error("Invalid party operation: {0}")]
    InvalidPartyOperation(String),

    #[error("Match constraints not satisfied: {0}")]
    ConstraintsNotSatisfied(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

pub type Result<T> = std::result::Result<T, MatchForgeError>;
