use serde::{Deserialize, Serialize};

use crate::rng::Pcg32;

/// A warning tier derived from an account's heat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HeatBand {
    /// Heat is below the first warning threshold.
    Quiet = 0,
    /// A GM has opened a ticket.
    Flagged = 1,
    /// The account is being watched.
    Watched = 2,
    /// Action is expected shortly.
    Imminent = 3,
}

impl HeatBand {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Flagged => "flagged",
            Self::Watched => "watched",
            Self::Imminent => "imminent",
        }
    }
}

/// The severity of a log entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogKind {
    /// Neutral information.
    Info,
    /// A cash or gold transaction.
    Money,
    /// A warning that needs attention.
    Warn,
    /// A run-ending or account-ending event.
    Danger,
    /// A system or milestone message.
    System,
}

/// A single line in the operation log.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    /// Monotonic id used by the UI to key rows and preserve scroll position.
    #[serde(default)]
    pub seq: u64,
    /// Simulation time when the entry was written.
    pub at: f64,
    /// Entry severity.
    pub kind: LogKind,
    /// Player-facing text.
    pub text: String,
}

/// One bot account in the operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    /// Stable account id used by commands and display tags.
    pub id: u32,
    /// Current level, derived from age.
    pub level: u32,
    /// Simulation time when the account was acquired.
    pub created_at: f64,
    /// Per-account heat from 0 to 100.
    pub heat: f64,
    /// Whether the account is pulled from the field.
    pub paused: bool,
    /// Whether the account has been removed from play.
    pub banned: bool,
    /// Simulation time of the last tick that generated heat.
    pub last_heat_at: f64,
    /// Current warning tier.
    pub band: HeatBand,
}

/// The complete, serializable simulation state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    /// Save-format version.
    pub version: u32,
    /// Seeded PRNG. Its state is part of the save so reloads replay identically.
    pub(crate) rng: Pcg32,
    /// Absolute simulation time in seconds (never wall clock).
    pub now: f64,
    /// Simulation time when the current run started.
    pub(crate) run_started_at: f64,
    /// Wall-clock unix seconds of the last save; used only for offline time.
    pub(crate) saved_at_unix: f64,
    /// Seconds credited on the last load (0 for a same-session load).
    pub(crate) last_offline_secs: f64,

    /// Staged gold, not yet sold.
    pub gold: f64,
    /// Cash on hand, the value that survives a sweep.
    pub cash: f64,
    /// Cash earned during the current run.
    pub run_cash: f64,
    /// Cash earned across all runs.
    pub lifetime_cash: f64,

    /// Accounts currently owned.
    pub accounts: Vec<Account>,
    /// Next account id to assign.
    pub(crate) next_account_id: u32,
    /// Total rack slots.
    pub slots: u32,
    /// Slots purchased this run.
    pub(crate) slots_bought: u32,

    /// Global script aggression from 0.2 to 2.0.
    pub aggression: f64,
    /// Script upgrade level.
    pub script_level: u32,
    /// Proxy upgrade level.
    pub proxy_level: u32,
    /// Humanizer upgrade level.
    pub humanizer_level: u32,

    /// Operation-wide exposure toward the sweep.
    pub exposure: f64,
    /// Current gold price in dollars per gram.
    pub market_price: f64,

    /// Prestige tier.
    pub tier: u32,
    /// Permanent Contacts total.
    pub contacts: u32,
    /// Completed runs.
    pub(crate) runs_completed: u32,
    /// Accounts banned across all runs.
    pub banned_total: u32,

    /// Whether the run is over and only prestige is legal.
    pub over: bool,
    /// Whether exposure hit the ceiling and a sweep is waiting for the player.
    pub sweep_pending: bool,

    /// Recent operation log.
    pub log: Vec<LogEntry>,
    /// Next monotonic log sequence id.
    pub(crate) next_log_seq: u64,
    /// False until the first bot is bought: the manual phase.
    pub started: bool,
}

impl State {
    pub(crate) fn new(seed: u64, price: f64, base_slots: u32, start_cash: f64) -> Self {
        Self {
            version: crate::SAVE_VERSION,
            rng: Pcg32::new(seed),
            now: 0.0,
            run_started_at: 0.0,
            saved_at_unix: 0.0,
            last_offline_secs: 0.0,
            gold: 0.0,
            cash: start_cash,
            run_cash: 0.0,
            lifetime_cash: 0.0,
            accounts: Vec::new(),
            next_account_id: 1,
            slots: base_slots,
            slots_bought: 0,
            aggression: 1.0,
            script_level: 0,
            proxy_level: 0,
            humanizer_level: 0,
            exposure: 0.0,
            market_price: price,
            tier: 0,
            contacts: 0,
            runs_completed: 0,
            banned_total: 0,
            over: false,
            sweep_pending: false,
            log: Vec::new(),
            next_log_seq: 1,
            started: false,
        }
    }

    pub(crate) fn add_account(&mut self, at: f64) -> u32 {
        let id = self.next_account_id;
        self.next_account_id = self.next_account_id.saturating_add(1);
        self.accounts.push(Account {
            id,
            level: 1,
            created_at: at,
            heat: 0.0,
            paused: false,
            banned: false,
            last_heat_at: at,
            band: HeatBand::Quiet,
        });
        id
    }

    pub(crate) fn active_accounts(&self) -> usize {
        self.accounts
            .iter()
            .filter(|account| !account.paused && !account.banned)
            .count()
    }

    pub(crate) fn run_secs(&self) -> f64 {
        self.now - self.run_started_at
    }

    pub(crate) fn push_log(&mut self, kind: LogKind, text: impl Into<String>) {
        let seq = self.next_log_seq;
        self.next_log_seq = self.next_log_seq.saturating_add(1);
        self.log.push(LogEntry {
            seq,
            at: self.now,
            kind,
            text: text.into(),
        });
        if self.log.len() > 240 {
            let drain = self.log.len() - 240;
            self.log.drain(0..drain);
        }
    }
}
