use serde::{Deserialize, Serialize};

/// Lobby lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LobbyState {
    /// Players are being added to the lobby
    Forming,
    /// All players present, waiting for ready confirmations
    WaitingForReady,
    /// All players ready, lobby can be dispatched to game server
    Ready,
    /// Match has been dispatched to game server
    Dispatched,
    /// Lobby closed (match completed or cancelled)
    Closed,
}

impl LobbyState {
    pub fn can_transition_to(&self, new_state: LobbyState) -> bool {
        use LobbyState::*;
        matches!(
            (self, new_state),
            (Forming, WaitingForReady)
                | (WaitingForReady, Ready)
                | (Ready, Dispatched)
                | (Dispatched, Closed)
                | (_, Closed) // Can always close
        )
    }
}
