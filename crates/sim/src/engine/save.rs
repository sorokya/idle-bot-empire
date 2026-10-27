//! Versioned save/load at the host boundary.

use crate::balance::Balance;
use crate::state::State;

use super::{Game, LoadError};

impl Game {
    /// Serializes the current state to JSON.
    ///
    /// # Panics
    /// Panics if the state cannot be serialized. The state schema is expected to
    /// remain JSON-compatible, so this is a programmer invariant.
    #[allow(clippy::expect_used)]
    pub fn save(&mut self, unix_seconds: f64) -> String {
        self.state.saved_at_unix = unix_seconds;
        serde_json::to_string(&self.state).expect("state serializes")
    }

    /// Loads a save and applies capped offline progress.
    ///
    /// # Errors
    /// Returns [`LoadError`] when the JSON is invalid or the save version is
    /// unsupported.
    pub fn load(json: &str, unix_seconds: f64) -> Result<Self, LoadError> {
        let mut state: State = serde_json::from_str(json).map_err(LoadError::Json)?;
        if state.version != crate::SAVE_VERSION {
            return Err(LoadError::UnsupportedVersion {
                found: state.version,
                supported: crate::SAVE_VERSION,
            });
        }

        let balance = Balance::load();
        let offline_seconds = (unix_seconds - state.saved_at_unix).max(0.0);
        state.last_offline_secs = offline_seconds.min(balance.offline.cap_secs);
        let mut game = Self { state, balance };

        if offline_seconds > 1.0 {
            game.tick(offline_seconds);
        }
        Ok(game)
    }
}
