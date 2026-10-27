use serde::Serialize;

use crate::balance::Balance;
use crate::engine::contacts_preview;
use crate::numeric::{count_as_u32, f64_to_u32, u32_to_i32, usize_to_i32};
use crate::state::{Account, LogKind, State};
use crate::text::tag;

/// A flat, render-ready view of the simulation.
#[derive(Clone, Debug, Serialize)]
pub struct ViewModel {
    /// Current in-game day.
    pub day: u32,
    /// Current in-game time as `HH:MM`.
    pub clock: String,
    /// Hours elapsed in the current run.
    pub run_hrs: f64,
    /// Seconds credited by the last load.
    pub offline_secs: f64,
    /// Staged gold.
    pub gold: f64,
    /// Current gold production per hour.
    pub gold_per_hr: f64,
    /// Cash on hand.
    pub cash: f64,
    /// What selling all staged gold would pay right now.
    pub sell_proceeds: f64,
    /// Accounts in the operation.
    pub accounts: Vec<AccountView>,
    /// Number of active accounts.
    pub active: u32,
    /// Total rack slots.
    pub capacity: u32,
    /// Operation-wide exposure.
    pub exposure: f64,
    /// Exposure ceiling for the current tier.
    pub ceiling: f64,
    /// Hours until the sweep, or `None` when no sweep is scheduled.
    pub sweep_eta_hours: Option<f64>,
    /// Current market price in dollars per gram.
    pub market_price: f64,
    /// Global script aggression.
    pub aggression: f64,
    /// Script upgrade level.
    pub script_level: u32,
    /// Proxy upgrade level.
    pub proxy_level: u32,
    /// Humanizer upgrade level.
    pub humanizer_level: u32,
    /// Current purchase costs.
    pub costs: Costs,
    /// Prestige tier.
    pub tier: u32,
    /// Permanent Contacts total.
    pub contacts: u32,
    /// Contacts a prestige would award right now.
    pub pending_contacts: u32,
    /// Completed runs.
    pub runs_completed: u32,
    /// Accounts banned across all runs.
    pub banned_total: u32,
    /// Whether the run is over.
    pub over: bool,
    /// Whether a sweep is waiting for the player.
    pub sweep_pending: bool,
    /// False during the manual phase.
    pub started: bool,
    /// Gold gained per manual click.
    pub click_gold: f64,
    /// Staged gold needed to afford the first bot.
    pub first_bot_gold: f64,
    /// Recent operation log, oldest first.
    pub log: Vec<LogView>,
}

/// A single account row in the view.
#[derive(Clone, Debug, Serialize)]
pub struct AccountView {
    /// Stable account id.
    pub id: u32,
    /// Display tag such as `#A07`.
    pub tag: String,
    /// Current level.
    pub level: u32,
    /// Heat from 0 to 100.
    pub heat: f64,
    /// Warning band label.
    pub band: String,
    /// Whether the account is paused.
    pub paused: bool,
    /// Account age in hours.
    pub age_hrs: f64,
    /// Current yield per hour.
    pub yield_per_hr: f64,
}

/// Purchase costs shown in the command bar.
#[derive(Clone, Debug, Serialize)]
pub struct Costs {
    /// Next account cost.
    pub account: f64,
    /// Next rack slot cost.
    pub slot: f64,
    /// Next script level cost.
    pub script: f64,
    /// Next proxy tier cost.
    pub proxy: f64,
    /// Next humanizer tier cost.
    pub humanizer: f64,
}

/// A single log row in the view.
#[derive(Clone, Debug, Serialize)]
pub struct LogView {
    /// Monotonic log id.
    pub seq: u64,
    /// In-game time as `HH:MM`.
    pub clock: String,
    /// Entry severity.
    pub kind: LogKind,
    /// Player-facing text.
    pub text: String,
}

impl ViewModel {
    pub(crate) fn build(state: &State, balance: &Balance) -> Self {
        let ceiling = balance.exposure_ceiling(state.tier);

        Self {
            day: day_of(state.now),
            clock: clock_of(state.now),
            run_hrs: state.run_secs() / 3600.0,
            offline_secs: state.last_offline_secs,
            gold: state.gold,
            gold_per_hr: gold_per_hour(state, balance),
            cash: state.cash,
            sell_proceeds: sell_proceeds(state, balance),
            accounts: account_views(state, balance),
            active: count_as_u32(state.active_accounts()),
            capacity: state.slots,
            exposure: state.exposure,
            ceiling,
            sweep_eta_hours: sweep_eta_hours(state, balance, ceiling),
            market_price: state.market_price,
            aggression: state.aggression,
            script_level: state.script_level,
            proxy_level: state.proxy_level,
            humanizer_level: state.humanizer_level,
            costs: costs(state, balance),
            tier: state.tier,
            contacts: state.contacts,
            pending_contacts: contacts_preview(state, balance),
            runs_completed: state.runs_completed,
            banned_total: state.banned_total,
            over: state.over,
            sweep_pending: state.sweep_pending,
            started: state.started,
            click_gold: balance.start.manual_click,
            first_bot_gold: first_bot_gold(state, balance),
            log: log_views(state),
        }
    }
}

fn account_views(state: &State, balance: &Balance) -> Vec<AccountView> {
    state
        .accounts
        .iter()
        .map(|account| AccountView {
            id: account.id,
            tag: tag(account.id),
            level: account.level,
            heat: account.heat,
            band: account.band.label().to_string(),
            paused: account.paused,
            age_hrs: (state.now - account.created_at) / 3600.0,
            yield_per_hr: if account.paused {
                0.0
            } else {
                account_yield(account, state, balance)
            },
        })
        .collect()
}

fn account_yield(account: &Account, state: &State, balance: &Balance) -> f64 {
    let script_boost =
        1.0 + balance.upgrades.script_yield_per_level * f64::from(state.script_level);
    let contact_multiplier = 1.0 + balance.prestige.mult_per_contact * f64::from(state.contacts);
    let aggression_yield = state.aggression.powf(balance.heat.aggression_yield_exp);

    balance.accounts.base_yield
        * balance
            .accounts
            .level_growth
            .powi(u32_to_i32(account.level) - 1)
        * script_boost
        * contact_multiplier
        * aggression_yield
}

fn gold_per_hour(state: &State, balance: &Balance) -> f64 {
    state
        .accounts
        .iter()
        .filter(|account| !account.paused && !account.banned)
        .map(|account| account_yield(account, state, balance))
        .sum()
}

fn sell_proceeds(state: &State, balance: &Balance) -> f64 {
    let hit = (state.gold / balance.market.shard_turnover).min(1.0) * balance.market.dump_hit;
    state.gold * state.market_price * (1.0 - hit)
}

fn sweep_eta_hours(state: &State, balance: &Balance, ceiling: f64) -> Option<f64> {
    let exposure_rate = balance.exposure.per_account_hr
        * f64::from(count_as_u32(state.active_accounts()))
        * state.aggression.powf(balance.exposure.aggression_exp);

    if state.over || state.sweep_pending || exposure_rate <= 0.0 {
        None
    } else {
        Some((ceiling - state.exposure).max(0.0) / exposure_rate)
    }
}

fn costs(state: &State, balance: &Balance) -> Costs {
    Costs {
        account: balance.accounts.cost
            * balance
                .accounts
                .cost_growth
                .powi(usize_to_i32(state.accounts.len())),
        slot: balance.slots.base_cost
            * balance
                .slots
                .cost_growth
                .powi(u32_to_i32(state.slots_bought)),
        script: balance.upgrades.script_base_cost
            * balance
                .upgrades
                .script_cost_growth
                .powi(u32_to_i32(state.script_level)),
        proxy: balance.upgrades.proxy_base_cost
            * balance
                .upgrades
                .proxy_cost_growth
                .powi(u32_to_i32(state.proxy_level)),
        humanizer: balance.upgrades.humanizer_base_cost
            * balance
                .upgrades
                .humanizer_cost_growth
                .powi(u32_to_i32(state.humanizer_level)),
    }
}

fn first_bot_gold(state: &State, balance: &Balance) -> f64 {
    let account_cost = balance.accounts.cost
        * balance
            .accounts
            .cost_growth
            .powi(usize_to_i32(state.accounts.len()));
    account_cost / state.market_price.max(0.0001)
}

fn log_views(state: &State) -> Vec<LogView> {
    state
        .log
        .iter()
        .rev()
        .take(60)
        .rev()
        .map(|entry| LogView {
            seq: entry.seq,
            clock: clock_of(entry.at),
            kind: entry.kind,
            text: entry.text.clone(),
        })
        .collect()
}

fn day_of(seconds: f64) -> u32 {
    1 + f64_to_u32(seconds / 86400.0)
}

pub(crate) fn clock_of(seconds: f64) -> String {
    let in_day = seconds.rem_euclid(86400.0);
    let hours = f64_to_u32(in_day / 3600.0);
    let minutes = f64_to_u32((in_day % 3600.0) / 60.0);
    format!("{hours:02}:{minutes:02}")
}
