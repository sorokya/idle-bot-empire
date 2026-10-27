use serde::Serialize;

use crate::balance::Balance;
use crate::engine::contacts_preview;
use crate::state::{Account, LogKind, State};

#[derive(Clone, Debug, Serialize)]
pub struct ViewModel {
    pub day: u32,
    pub clock: String,
    pub run_hrs: f64,
    pub offline_secs: f64,

    pub gold: f64,
    pub gold_per_hr: f64,
    pub cash: f64,

    pub accounts: Vec<AccountView>,
    pub active: u32,
    pub capacity: u32,

    pub exposure: f64,
    pub ceiling: f64,
    pub sweep_eta_hours: Option<f64>,

    pub market_price: f64,
    pub aggression: f64,
    pub script_level: u32,
    pub proxy_level: u32,
    pub humanizer_level: u32,

    pub costs: Costs,

    pub tier: u32,
    pub contacts: u32,
    pub pending_contacts: u32,
    pub runs_completed: u32,
    pub banned_total: u32,

    pub over: bool,
    pub sweep_pending: bool,
    pub log: Vec<LogView>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountView {
    pub id: u32,
    pub tag: String,
    pub level: u32,
    pub heat: f64,
    pub band: String,
    pub paused: bool,
    pub age_hrs: f64,
    pub yield_per_hr: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Costs {
    pub account: f64,
    pub slot: f64,
    pub script: f64,
    pub proxy: f64,
    pub humanizer: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct LogView {
    pub clock: String,
    pub kind: LogKind,
    pub text: String,
}

impl ViewModel {
    pub fn build(s: &State, bal: &Balance) -> Self {
        let script_boost = 1.0 + bal.upgrades.script_yield_per_level * s.script_level as f64;
        let perm = 1.0 + bal.prestige.mult_per_contact * s.contacts as f64;
        let yield_aggr = s.aggression.powf(bal.heat.aggression_yield_exp);

        let per_account = |a: &Account| {
            bal.accounts.base_yield
                * bal.accounts.level_growth.powi(a.level as i32 - 1)
                * script_boost
                * perm
                * yield_aggr
        };

        let gold_per_hr: f64 = s
            .accounts
            .iter()
            .filter(|a| !a.paused && !a.banned)
            .map(per_account)
            .sum();

        let accounts: Vec<AccountView> = s
            .accounts
            .iter()
            .map(|a| AccountView {
                id: a.id,
                tag: crate::engine::tag(a.id),
                level: a.level,
                heat: a.heat,
                band: a.band.label().to_string(),
                paused: a.paused,
                age_hrs: (s.now - a.created_at) / 3600.0,
                yield_per_hr: if a.paused { 0.0 } else { per_account(a) },
            })
            .collect();

        let ceiling = bal.exposure_ceiling(s.tier);
        let exposure_rate = bal.exposure.per_account_hr
            * s.active_accounts() as f64
            * s.aggression.powf(bal.exposure.aggression_exp);
        let sweep_eta_hours = if s.over || s.sweep_pending || exposure_rate <= 0.0 {
            None
        } else {
            Some(((ceiling - s.exposure).max(0.0)) / exposure_rate)
        };

        let mut log: Vec<LogView> = s
            .log
            .iter()
            .rev()
            .take(14)
            .rev()
            .map(|e| LogView {
                clock: clock_of(e.at),
                kind: e.kind,
                text: e.text.clone(),
            })
            .collect();
        log.reverse();
        log.reverse();

        Self {
            day: day_of(s.now),
            clock: clock_of(s.now),
            run_hrs: s.run_secs() / 3600.0,
            offline_secs: s.last_offline_secs,
            gold: s.gold,
            gold_per_hr,
            cash: s.cash,
            accounts,
            active: s.active_accounts() as u32,
            capacity: s.slots,
            exposure: s.exposure,
            ceiling,
            sweep_eta_hours,
            market_price: s.market_price,
            aggression: s.aggression,
            script_level: s.script_level,
            proxy_level: s.proxy_level,
            humanizer_level: s.humanizer_level,
            costs: Costs {
                account: bal.accounts.cost * bal.accounts.cost_growth.powi(s.accounts.len() as i32),
                slot: bal.slots.base_cost * bal.slots.cost_growth.powi(s.slots_bought as i32),
                script: bal.upgrades.script_base_cost
                    * bal.upgrades.script_cost_growth.powi(s.script_level as i32),
                proxy: bal.upgrades.proxy_base_cost
                    * bal.upgrades.proxy_cost_growth.powi(s.proxy_level as i32),
                humanizer: bal.upgrades.humanizer_base_cost
                    * bal
                        .upgrades
                        .humanizer_cost_growth
                        .powi(s.humanizer_level as i32),
            },
            tier: s.tier,
            contacts: s.contacts,
            pending_contacts: contacts_preview(s, bal),
            runs_completed: s.runs_completed,
            banned_total: s.banned_total,
            over: s.over,
            sweep_pending: s.sweep_pending,
            log,
        }
    }
}

fn day_of(secs: f64) -> u32 {
    1 + (secs / 86400.0).floor() as u32
}

pub fn clock_of(secs: f64) -> String {
    let in_day = secs.rem_euclid(86400.0);
    let h = (in_day / 3600.0).floor() as u32;
    let m = ((in_day % 3600.0) / 60.0).floor() as u32;
    format!("{h:02}:{m:02}")
}
