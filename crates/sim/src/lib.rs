//! Idle Bot Empire — simulation core.
//!
//! Zero I/O, zero clock, zero randomness outside the seeded PRNG held in `State`.
//! The host (browser, test, CLI) owns time and calls `Game::tick`.

mod balance;
mod engine;
pub mod harness;
mod numeric;
mod rng;
mod state;
mod text;
mod view;

pub use balance::Balance;
pub use engine::{contacts_preview, Command, EngineError, Game, LoadError, Upgrade};
pub use state::{Account, HeatBand, LogEntry, LogKind, State};
pub use view::{AccountView, Costs, LogView, ViewModel};

/// Version of the save format. Bump when `State` changes shape.
pub const SAVE_VERSION: u32 = 1;
