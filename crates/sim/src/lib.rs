//! Idle Bot Empire — simulation core.
//!
//! Zero I/O, zero clock, zero randomness outside the seeded PRNG held in `State`.
//! The host (browser, test, CLI) owns time and calls `Game::tick(dt)`.

pub mod balance;
pub mod engine;
pub mod harness;
pub mod rng;
pub mod state;
pub mod view;

pub use balance::Balance;
pub use engine::Game;
pub use state::{Account, HeatBand, LogEntry, LogKind, State};
pub use view::{AccountView, ViewModel};

/// Version of the save format. Bump when `State` changes shape.
pub const SAVE_VERSION: u32 = 1;
