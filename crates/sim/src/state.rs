use serde::{Deserialize, Serialize};

use crate::rng::Pcg32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HeatBand {
    Quiet = 0,
    Flagged = 1,
    Watched = 2,
    Imminent = 3,
}

impl HeatBand {
    pub fn label(self) -> &'static str {
        match self {
            HeatBand::Quiet => "quiet",
            HeatBand::Flagged => "flagged",
            HeatBand::Watched => "watched",
            HeatBand::Imminent => "imminent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogKind {
    Info,
    Money,
    Warn,
    Danger,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub at: f64,
    pub kind: LogKind,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: u32,
    pub level: u32,
    pub created_at: f64,
    pub heat: f64,
    pub paused: bool,
    pub banned: bool,
    /// Sim time of the last tick where this account generated heat.
    pub last_heat_at: f64,
    pub band: HeatBand,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub rng: Pcg32,
    /// Absolute simulation time in seconds (never wall clock).
    pub now: f64,
    pub run_started_at: f64,
    /// Wall-clock unix seconds of the last save. Only used to compute offline time.
    pub saved_at_unix: f64,
    /// Seconds credited on the last load (0 if loaded in the same session).
    #[serde(default)]
    pub last_offline_secs: f64,

    pub gold: f64,
    pub cash: f64,
    pub run_cash: f64,
    pub lifetime_cash: f64,

    pub accounts: Vec<Account>,
    pub next_account_id: u32,
    pub slots: u32,
    pub slots_bought: u32,

    pub aggression: f64,
    pub script_level: u32,
    pub proxy_level: u32,
    pub humanizer_level: u32,

    pub exposure: f64,
    pub market_price: f64,

    pub tier: u32,
    pub contacts: u32,
    pub runs_completed: u32,
    pub banned_total: u32,

    /// Set when the run is over (post-sweep). Only prestige is legal.
    pub over: bool,
    /// Exposure hit the ceiling; the sweep waits for the player.
    #[serde(default)]
    pub sweep_pending: bool,

    pub log: Vec<LogEntry>,
}

impl State {
    pub fn new(seed: u64, price: f64, base_slots: u32, start_cash: f64) -> Self {
        let mut s = Self {
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
        };
        s.add_account(0.0);
        s
    }

    pub fn add_account(&mut self, at: f64) -> u32 {
        let id = self.next_account_id;
        self.next_account_id += 1;
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

    pub fn active_accounts(&self) -> usize {
        self.accounts
            .iter()
            .filter(|a| !a.paused && !a.banned)
            .count()
    }

    pub fn run_secs(&self) -> f64 {
        self.now - self.run_started_at
    }

    pub fn push_log(&mut self, kind: LogKind, text: impl Into<String>) {
        self.log.push(LogEntry {
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
