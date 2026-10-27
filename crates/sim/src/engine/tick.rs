//! Time advancement. Live play and offline catch-up share this path.

use crate::numeric::{count_as_f64, count_as_u32, u32_to_i32};
use crate::state::{LogKind, State};
use crate::text::tag;

use super::support::{heat_band_for, heat_band_message, level_for_age, log_kind_for_band};
use super::Game;

const MAX_TICK_SECONDS: f64 = 1.0;

impl Game {
    /// Advances the simulation by `delta_seconds`.
    ///
    /// The host owns the clock. Every call is clamped to the offline cap and
    /// processed in fixed, at-most-one-second steps, so live play and offline
    /// catch-up use the same code path.
    pub fn tick(&mut self, delta_seconds: f64) {
        let mut remaining = delta_seconds.clamp(0.0, self.balance.offline.cap_secs);
        while remaining > 1e-9 {
            let step_seconds = remaining.min(MAX_TICK_SECONDS);
            self.step(step_seconds);
            remaining -= step_seconds;
            if self.state.over {
                break;
            }
        }
    }

    fn step(&mut self, delta_seconds: f64) {
        let modifiers = StepModifiers::new(&self.state, &self.balance, delta_seconds);
        let mut events = Vec::new();
        let mut terminated = Vec::new();

        let gained_gold = self.accrue_accounts(&modifiers, &mut events, &mut terminated);
        self.remove_terminated(&terminated, &mut events);
        self.respawn_personal_account(modifiers.now, &mut events);

        self.state.now = modifiers.now;
        self.state.gold += gained_gold;
        self.state.exposure += self.balance.exposure.per_account_hr
            * count_as_f64(modifiers.active_accounts)
            * self
                .state
                .aggression
                .powf(self.balance.exposure.aggression_exp)
            * modifiers.hours;

        self.update_market(modifiers.hours);

        for (kind, message) in events {
            self.state.push_log(kind, message);
        }

        self.check_exposure_ceiling();
    }

    fn accrue_accounts(
        &mut self,
        modifiers: &StepModifiers,
        events: &mut Vec<(LogKind, String)>,
        terminated: &mut Vec<u32>,
    ) -> f64 {
        let mut gained_gold = 0.0;

        for account in self.state.accounts.iter_mut() {
            if account.banned {
                continue;
            }

            let age_seconds = modifiers.now - account.created_at;
            let level = level_for_age(age_seconds, &self.balance);
            if level > account.level {
                account.level = level;
            }

            if account.paused {
                if modifiers.now - account.last_heat_at > self.balance.heat.decay_delay
                    && account.heat > 0.0
                {
                    account.heat =
                        (account.heat - self.balance.heat.decay * modifiers.hours).max(0.0);
                    let band = heat_band_for(account.heat, &self.balance);
                    if band != account.band {
                        events.push((
                            LogKind::Info,
                            format!(
                                "{} cooled to {} (heat {:.0}).",
                                tag(account.id),
                                band.label(),
                                account.heat
                            ),
                        ));
                        account.band = band;
                    }
                }
                continue;
            }

            let age_factor = (1.0 + age_seconds / 86400.0 / 120.0).min(2.5);
            let stealth = (modifiers.stealth_multiplier * age_factor).max(0.05);
            let heat_gain =
                self.balance.heat.base * modifiers.crowding * modifiers.heat_aggression / stealth;
            account.heat = (account.heat + heat_gain * modifiers.hours).min(100.0);
            account.last_heat_at = modifiers.now;

            if account.heat >= 100.0 {
                terminated.push(account.id);
            }

            let yield_per_hour = self.balance.accounts.base_yield
                * self
                    .balance
                    .accounts
                    .level_growth
                    .powi(u32_to_i32(account.level) - 1)
                * modifiers.script_boost
                * modifiers.contact_multiplier
                * modifiers.yield_aggression;
            gained_gold += yield_per_hour * modifiers.hours;

            let band = heat_band_for(account.heat, &self.balance);
            if band != account.band {
                events.push((
                    log_kind_for_band(band),
                    heat_band_message(account.id, band, account.heat),
                ));
                account.band = band;
            }
        }

        gained_gold
    }

    fn remove_terminated(&mut self, terminated: &[u32], events: &mut Vec<(LogKind, String)>) {
        if terminated.is_empty() {
            return;
        }

        for account_id in terminated {
            events.push((
                LogKind::Danger,
                format!(
                    "{} hit revocation. Terminated mid-session, no warning left to give.",
                    tag(*account_id)
                ),
            ));
        }

        self.state
            .accounts
            .retain(|account| !terminated.contains(&account.id));
        self.state.banned_total = self
            .state
            .banned_total
            .saturating_add(count_as_u32(terminated.len()));
    }

    fn respawn_personal_account(&mut self, now: f64, events: &mut Vec<(LogKind, String)>) {
        // Never a dead end: you always have one personal account to rebuild from.
        if self.state.accounts.is_empty() && self.state.started && !self.state.over {
            let account_id = self.state.add_account(now);
            events.push((
                LogKind::System,
                format!(
                    "No accounts left. You log back in on {} — your own account, the one you never risk.",
                    tag(account_id)
                ),
            ));
        }
    }

    fn update_market(&mut self, hours: f64) {
        let shock = self.balance.market.drift * self.state.rng.normal() * hours.sqrt();
        self.state.market_price = (self.state.market_price * shock.exp())
            .clamp(self.balance.market.min_price, self.balance.market.max_price);
    }

    fn check_exposure_ceiling(&mut self) {
        let ceiling = self.balance.exposure_ceiling(self.state.tier);
        if self.state.exposure >= ceiling && !self.state.over {
            self.state.exposure = ceiling;
            if !self.state.sweep_pending {
                self.state.sweep_pending = true;
                self.state.push_log(
                    LogKind::Danger,
                    "Sweep incoming. Sell what you can, then face it. Leaving now costs you the staging pile.",
                );
            }
        }
    }
}

struct StepModifiers {
    now: f64,
    hours: f64,
    active_accounts: usize,
    crowding: f64,
    heat_aggression: f64,
    stealth_multiplier: f64,
    script_boost: f64,
    contact_multiplier: f64,
    yield_aggression: f64,
}

impl StepModifiers {
    fn new(state: &State, balance: &crate::balance::Balance, delta_seconds: f64) -> Self {
        let aggression = state.aggression;
        let stealth_multiplier = (balance.upgrades.proxy_base_quality
            + balance.upgrades.proxy_quality_per_level * f64::from(state.proxy_level))
            * (balance.upgrades.humanizer_base_quality
                + balance.upgrades.humanizer_quality_per_level * f64::from(state.humanizer_level));

        Self {
            now: state.now + delta_seconds,
            hours: delta_seconds / 3600.0,
            active_accounts: state.active_accounts(),
            crowding: 1.0 + balance.heat.crowding * count_as_f64(state.active_accounts()),
            heat_aggression: aggression.powf(balance.heat.aggression_heat_exp),
            stealth_multiplier,
            script_boost: 1.0
                + balance.upgrades.script_yield_per_level * f64::from(state.script_level),
            contact_multiplier: 1.0 + balance.prestige.mult_per_contact * f64::from(state.contacts),
            yield_aggression: aggression.powf(balance.heat.aggression_yield_exp),
        }
    }
}
