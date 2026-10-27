//! The game engine: state transitions, actions, and persistence.

mod actions;
mod command;
mod error;
mod save;
mod support;
mod sweep;
mod tick;

pub use command::{Command, Upgrade};
pub use error::{EngineError, LoadError};
pub use sweep::contacts_preview;

use crate::balance::Balance;
use crate::state::{LogKind, State};
use crate::view::ViewModel;

/// The simulation state plus the loaded balance sheet.
pub struct Game {
    /// Mutable simulation state, including the seeded RNG.
    pub state: State,
    /// Balance values loaded from `balance.toml`.
    pub balance: Balance,
}

impl Game {
    /// Creates a new run from `seed`.
    ///
    /// The seed is stored in the state, so the same seed and the same command
    /// sequence produce the same result.
    pub fn new(seed: u64) -> Self {
        let balance = Balance::load();
        let state = State::new(
            seed,
            balance.market.price,
            balance.accounts.base_slots,
            balance.start.cash,
        );
        let mut game = Self { state, balance };
        game.state.push_log(
            LogKind::System,
            "Day one. No bots, no script, no plan. Just you and a mouse.",
        );
        game.state.push_log(
            LogKind::Info,
            "Farm by hand until you can afford a script. Then never do this again.",
        );
        game
    }

    /// Returns the flat, render-ready view of the current state.
    pub fn view(&self) -> ViewModel {
        ViewModel::build(&self.state, &self.balance)
    }
}
