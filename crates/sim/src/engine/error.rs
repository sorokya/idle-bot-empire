//! Typed errors at the engine boundary.

use std::fmt;

use crate::text::money;

/// A player-facing failure from [`crate::Game::dispatch`].
#[derive(Debug, PartialEq)]
pub enum EngineError {
    /// The current run is over; prestige before acting again.
    RunOver,
    /// No free rack slots remain.
    NoFreeSlots,
    /// A purchase costs more cash than the player has.
    InsufficientCash {
        /// Human-readable item name.
        item: &'static str,
        /// Cost in cash.
        cost: f64,
        /// Cash currently on hand.
        cash: f64,
    },
    /// There is no staged gold to sell.
    NothingToSell,
    /// No sweep is queued yet.
    NoSweepScheduled,
    /// The requested account id does not exist.
    AccountNotFound(u32),
    /// The upgrade name is not recognized.
    UnknownUpgrade(String),
    /// The command verb is not recognized.
    UnknownCommand(String),
    /// A command argument is malformed.
    InvalidArgument(String),
    /// Prestiging now would award zero contacts.
    PrestigeNotWorth,
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RunOver => formatter.write_str("Run is over. Start a new identity."),
            Self::NoFreeSlots => formatter.write_str("No free slots. Buy a slot first."),
            Self::InsufficientCash { item, cost, cash } => {
                write!(
                    formatter,
                    "{item} costs ${} — you have ${}.",
                    money(*cost),
                    money(*cash)
                )
            }
            Self::NothingToSell => formatter.write_str("Nothing in staging to sell."),
            Self::NoSweepScheduled => formatter.write_str("No sweep scheduled yet."),
            Self::AccountNotFound(id) => write!(formatter, "No account {id}"),
            Self::UnknownUpgrade(upgrade) => write!(formatter, "Unknown upgrade '{upgrade}'"),
            Self::UnknownCommand(command) => write!(formatter, "Unknown command '{command}'"),
            Self::InvalidArgument(message) => formatter.write_str(message),
            Self::PrestigeNotWorth => {
                formatter.write_str("Not worth burning yet. Grow the operation first.")
            }
        }
    }
}

impl std::error::Error for EngineError {}

/// A failure while loading a save.
#[derive(Debug)]
pub enum LoadError {
    /// The save is not valid JSON or does not match the state schema.
    Json(serde_json::Error),
    /// The save was written by an unsupported state version.
    UnsupportedVersion {
        /// Version found in the save.
        found: u32,
        /// Version supported by this build.
        supported: u32,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "{error}"),
            Self::UnsupportedVersion { found, supported } => {
                write!(
                    formatter,
                    "Save version {found} is not supported ({supported})"
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}
