use crate::balance::Balance;
use crate::state::{Account, HeatBand, LogKind, State};
use crate::view::ViewModel;

const MAX_STEP: f64 = 1.0;

pub struct Game {
    pub state: State,
    pub balance: Balance,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let balance = Balance::load();
        let state = State::new(
            seed,
            balance.market.price,
            balance.accounts.base_slots,
            balance.start.cash,
        );
        let mut g = Self { state, balance };
        g.state
            .push_log(LogKind::System, "Operation online. One account, one script, no plan.");
        g.state.push_log(
            LogKind::Info,
            "Sell gold to buy accounts. Accounts make heat. Heat makes problems.",
        );
        g
    }

    // ---------------------------------------------------------------- time

    /// Advance the simulation. The host owns the clock; this is the only way
    /// time passes, live or offline.
    pub fn tick(&mut self, dt_secs: f64) {
        let mut remaining = dt_secs.clamp(0.0, self.balance.offline.cap_secs);
        while remaining > 1e-9 {
            let step = remaining.min(MAX_STEP);
            self.step(step);
            remaining -= step;
            if self.state.over {
                break;
            }
        }
    }

    fn step(&mut self, dt: f64) {
        let hrs = dt / 3600.0;
        let bal = &self.balance;

        let now = self.state.now + dt;
        let n_active = self.state.active_accounts();
        let crowding = 1.0 + bal.heat.crowding * n_active as f64;
        let aggr = self.state.aggression;
        let perm = 1.0 + bal.prestige.mult_per_contact * self.state.contacts as f64;
        let script_boost =
            1.0 + bal.upgrades.script_yield_per_level * self.state.script_level as f64;
        let stealth_mult = (bal.upgrades.proxy_base_quality
            + bal.upgrades.proxy_quality_per_level * self.state.proxy_level as f64)
            * (bal.upgrades.humanizer_base_quality
                + bal.upgrades.humanizer_quality_per_level * self.state.humanizer_level as f64);
        let heat_aggr = aggr.powf(bal.heat.aggression_heat_exp);
        let yield_aggr = aggr.powf(bal.heat.aggression_yield_exp);

        let mut gained_gold = 0.0;
        let mut events: Vec<(LogKind, String)> = Vec::new();
        let mut terminated: Vec<u32> = Vec::new();

        for a in self.state.accounts.iter_mut() {
            if a.banned {
                continue;
            }
            let age = now - a.created_at;
            let lvl = level_for(age, bal);
            if lvl > a.level {
                a.level = lvl;
            }

            if a.paused {
                if now - a.last_heat_at > bal.heat.decay_delay && a.heat > 0.0 {
                    a.heat = (a.heat - bal.heat.decay * hrs).max(0.0);
                    let band = band_for(a.heat, bal);
                    if band != a.band {
                        events.push((
                            LogKind::Info,
                            format!("{} cooled to {} (heat {:.0}).", tag(a.id), band.label(), a.heat),
                        ));
                        a.band = band;
                    }
                }
                continue;
            }

            let age_factor = (1.0 + age / 86400.0 / 120.0).min(2.5);
            let stealth = (stealth_mult * age_factor).max(0.05);
            let heat_gain = bal.heat.base * crowding * heat_aggr / stealth;
            a.heat = (a.heat + heat_gain * hrs).min(100.0);
            a.last_heat_at = now;
            if a.heat >= 100.0 {
                terminated.push(a.id);
            }

            let y = bal.accounts.base_yield
                * bal.accounts.level_growth.powi(a.level as i32 - 1)
                * script_boost
                * perm
                * yield_aggr;
            gained_gold += y * hrs;

            let band = band_for(a.heat, bal);
            if band != a.band {
                events.push((band_kind(band), band_message(a.id, band, a.heat)));
                a.band = band;
            }
        }

        if !terminated.is_empty() {
            for id in &terminated {
                events.push((
                    LogKind::Danger,
                    format!("{} hit revocation. Terminated mid-session, no warning left to give.", tag(*id)),
                ));
            }
            self.state.accounts.retain(|a| !terminated.contains(&a.id));
            self.state.banned_total += terminated.len() as u32;
        }

        // Never a dead end: you always have one personal account to rebuild from.
        // That is the pressure valve that keeps a wipe from bricking a save.
        if self.state.accounts.is_empty() && !self.state.over {
            let id = self.state.add_account(now);
            events.push((
                LogKind::System,
                format!(
                    "No accounts left. You log back in on {} — your own account, the one you never risk.",
                    tag(id)
                ),
            ));
        }

        self.state.now = now;
        self.state.gold += gained_gold;
        self.state.exposure += bal.exposure.per_account_hr
            * n_active as f64
            * aggr.powf(bal.exposure.aggression_exp)
            * hrs;

        // market: slow random walk, bounded.
        let shock = bal.market.drift * self.state.rng.normal() * hrs.sqrt();
        self.state.market_price =
            (self.state.market_price * shock.exp()).clamp(bal.market.min_price, bal.market.max_price);

        for (kind, text) in events {
            self.state.push_log(kind, text);
        }

        let ceiling = bal.exposure_ceiling(self.state.tier);
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

    // -------------------------------------------------------------- sweep

    fn sweep(&mut self) {
        let bal = &self.balance;
        let s = &mut self.state;

        s.push_log(
            LogKind::Danger,
            "PATCH DAY. Shard goes down for maintenance. Some accounts do not come back.",
        );

        let accounts = std::mem::take(&mut s.accounts);
        let mut kept: Vec<Account> = Vec::with_capacity(accounts.len());
        let mut banned = 0u32;

        for a in accounts {
            let p = if a.heat < bal.sweep.min_ban_heat {
                0.0
            } else {
                (a.heat / bal.sweep.ban_divisor).clamp(0.0, 0.95)
            };
            if s.rng.f64() < p {
                banned += 1;
                s.push_log(
                    LogKind::Danger,
                    format!(
                        "{} banned. Level {}, heat {:.0}. Nothing recoverable.",
                        tag(a.id),
                        a.level,
                        a.heat
                    ),
                );
            } else {
                kept.push(a);
            }
        }

        let survivors = kept.len() as u32;
        s.accounts = kept;
        let seized = s.gold;
        s.gold = 0.0;
        s.banned_total += banned;
        s.over = true;
        s.sweep_pending = false;

        s.push_log(
            LogKind::Danger,
            format!(
                "{} of {} accounts banned. {} gold in staging seized.",
                banned,
                banned + survivors,
                num(seized)
            ),
        );

        let gain = contacts_preview(s, bal);
        if survivors == 0 {
            s.push_log(LogKind::System, "Nothing left but the lesson.");
        }
        s.push_log(
            LogKind::System,
            format!("New identity available: +{gain} contacts."),
        );
    }

    // ------------------------------------------------------------ actions

    pub fn account_cost(&self) -> f64 {
        self.balance.accounts.cost
            * self
                .balance
                .accounts
                .cost_growth
                .powi(self.state.accounts.len() as i32)
    }

    pub fn slot_cost(&self) -> f64 {
        self.balance.slots.base_cost
            * self
                .balance
                .slots
                .cost_growth
                .powi(self.state.slots_bought as i32)
    }

    pub fn script_cost(&self) -> f64 {
        self.balance.upgrades.script_base_cost
            * self
                .balance
                .upgrades
                .script_cost_growth
                .powi(self.state.script_level as i32)
    }

    pub fn proxy_cost(&self) -> f64 {
        self.balance.upgrades.proxy_base_cost
            * self
                .balance
                .upgrades
                .proxy_cost_growth
                .powi(self.state.proxy_level as i32)
    }

    pub fn humanizer_cost(&self) -> f64 {
        self.balance.upgrades.humanizer_base_cost
            * self
                .balance
                .upgrades
                .humanizer_cost_growth
                .powi(self.state.humanizer_level as i32)
    }

    pub fn dispatch(&mut self, cmd: &str) -> Result<(), String> {
        let (verb, arg) = match cmd.split_once(':') {
            Some((v, a)) => (v.trim(), Some(a.trim())),
            None => (cmd.trim(), None),
        };
        match verb {
            "buy_account" => self.buy_account(),
            "buy_slot" => self.buy_slot(),
            "sell" => self.sell(),
            "upgrade" => match arg.unwrap_or("") {
                "script" => self.buy_script(),
                "proxy" => self.buy_proxy(),
                "humanizer" => self.buy_humanizer(),
                other => Err(format!("Unknown upgrade '{other}'")),
            },
            "pause" => self.toggle_pause(parse_id(arg)?),
            "burn" => self.burn(parse_id(arg)?),
            "aggression" => {
                let v: f64 = arg
                    .unwrap_or("1")
                    .parse()
                    .map_err(|_| "Aggression must be a number".to_string())?;
                if self.state.over {
                    return Err("Run is over. Start a new identity.".into());
                }
                self.state.aggression = v.clamp(0.2, 2.0);
                self.state.push_log(
                    LogKind::Info,
                    format!("Script aggression set to {:.1}x.", self.state.aggression),
                );
                Ok(())
            }
            "face_sweep" => {
                if self.state.over {
                    return Err("This run is already over.".into());
                }
                if !self.state.sweep_pending {
                    return Err("No sweep scheduled yet.".into());
                }
                self.sweep();
                Ok(())
            }
            "prestige" => self.prestige(),
            "wait" => {
                let secs: f64 = arg
                    .unwrap_or("0")
                    .parse()
                    .map_err(|_| "wait needs seconds".to_string())?;
                self.tick(secs);
                Ok(())
            }
            other => Err(format!("Unknown command '{other}'")),
        }
    }

    fn require_running(&self) -> Result<(), String> {
        if self.state.over {
            Err("Run is over. Start a new identity.".into())
        } else {
            Ok(())
        }
    }

    fn spend(&mut self, cost: f64, what: &str) -> Result<(), String> {
        if self.state.cash < cost {
            return Err(format!("{what} costs ${} — you have ${}.", money(cost), money(self.state.cash)));
        }
        self.state.cash -= cost;
        Ok(())
    }

    fn buy_account(&mut self) -> Result<(), String> {
        self.require_running()?;
        if self.state.accounts.len() >= self.state.slots as usize {
            return Err("No free slots. Buy a slot first.".into());
        }
        let cost = self.account_cost();
        self.spend(cost, "Account")?;
        let now = self.state.now;
        let id = self.state.add_account(now);
        self.state.push_log(
            LogKind::Money,
            format!("{} acquired for ${}. Welcome aboard.", tag(id), money(cost)),
        );
        Ok(())
    }

    fn buy_slot(&mut self) -> Result<(), String> {
        self.require_running()?;
        let cost = self.slot_cost();
        self.spend(cost, "Slot")?;
        self.state.slots += 1;
        self.state.slots_bought += 1;
        self.state.push_log(
            LogKind::Money,
            format!("Rack space added for ${}. {} slots total.", money(cost), self.state.slots),
        );
        Ok(())
    }

    fn buy_script(&mut self) -> Result<(), String> {
        self.require_running()?;
        let cost = self.script_cost();
        self.spend(cost, "Script upgrade")?;
        self.state.script_level += 1;
        self.state.push_log(
            LogKind::Money,
            format!(
                "Script v{} deployed (+{:.0}% yield). They patch, we adapt.",
                self.state.script_level + 1,
                self.balance.upgrades.script_yield_per_level * 100.0 * self.state.script_level as f64
            ),
        );
        Ok(())
    }

    fn buy_proxy(&mut self) -> Result<(), String> {
        self.require_running()?;
        let cost = self.proxy_cost();
        self.spend(cost, "Proxy tier")?;
        self.state.proxy_level += 1;
        self.state.push_log(
            LogKind::Money,
            format!("Proxy pool upgraded to tier {}. Cheaper, quieter, slower.", self.state.proxy_level),
        );
        Ok(())
    }

    fn buy_humanizer(&mut self) -> Result<(), String> {
        self.require_running()?;
        let cost = self.humanizer_cost();
        self.spend(cost, "Humanizer")?;
        self.state.humanizer_level += 1;
        self.state.push_log(
            LogKind::Money,
            format!(
                "Humanizer tier {}. Random pauses, path jitter, plausible mistakes.",
                self.state.humanizer_level
            ),
        );
        Ok(())
    }

    fn sell(&mut self) -> Result<(), String> {
        self.require_running()?;
        if self.state.gold < 1.0 {
            return Err("Nothing in staging to sell.".into());
        }
        let gross = self.state.gold;
        let hit = (gross / self.balance.market.shard_turnover).min(1.0) * self.balance.market.dump_hit;
        let price = self.state.market_price;
        let proceeds = gross * price * (1.0 - hit);
        self.state.market_price = (price * (1.0 - hit))
            .clamp(self.balance.market.min_price, self.balance.market.max_price);
        self.state.gold = 0.0;
        self.state.cash += proceeds;
        self.state.run_cash += proceeds;
        self.state.lifetime_cash += proceeds;
        self.state.push_log(
            LogKind::Money,
            format!(
                "Sold {} gold to mule_04 — ${}. Price now ${:.4}/g.",
                num(gross),
                money(proceeds),
                self.state.market_price
            ),
        );
        Ok(())
    }

    fn toggle_pause(&mut self, id: u32) -> Result<(), String> {
        let acc = self
            .state
            .accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| format!("No account {id}"))?;
        acc.paused = !acc.paused;
        let paused = acc.paused;
        if paused {
            acc.last_heat_at = self.state.now;
        }
        self.state.push_log(
            LogKind::Info,
            if paused {
                format!("{} pulled from the field. Heat decaying in 10 minutes.", tag(id))
            } else {
                format!("{} back online.", tag(id))
            },
        );
        Ok(())
    }

    fn burn(&mut self, id: u32) -> Result<(), String> {
        let idx = self
            .state
            .accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| format!("No account {id}"))?;
        let acc = self.state.accounts.remove(idx);
        let salvage = self.balance.accounts.cost * (1.0 + 0.2 * (acc.level as f64 - 1.0)) * 0.4;
        self.state.cash += salvage;
        self.state.push_log(
            LogKind::Warn,
            format!(
                "{} wiped and sold for parts. ${} recovered, {} heat gone.",
                tag(id),
                money(salvage),
                acc.heat.round()
            ),
        );
        Ok(())
    }

    pub fn prestige(&mut self) -> Result<(), String> {
        let gain = contacts_preview(&self.state, &self.balance);
        if gain == 0 && !self.state.over {
            return Err("Not worth burning yet. Grow the operation first.".into());
        }
        let s = &mut self.state;
        s.contacts += gain;
        s.tier += 1;
        s.runs_completed += 1;
        s.push_log(
            LogKind::System,
            format!(
                "New identity. Run {} closed, +{} contacts ({} total).",
                s.runs_completed, gain, s.contacts
            ),
        );

        s.accounts.clear();
        s.slots = self.balance.accounts.base_slots;
        s.slots_bought = 0;
        s.script_level = 0;
        s.proxy_level = 0;
        s.humanizer_level = 0;
        s.aggression = 1.0;
        s.gold = 0.0;
        s.cash = self.balance.start.cash;
        s.run_cash = 0.0;
        s.exposure = 0.0;
        s.over = false;
        s.sweep_pending = false;
        s.run_started_at = s.now;
        s.add_account(s.now);
        s.push_log(LogKind::Info, "Fresh start. Same mistakes available at no extra cost.");
        Ok(())
    }

    // --------------------------------------------------------- save/load

    pub fn save(&mut self, unix_now: f64) -> String {
        self.state.saved_at_unix = unix_now;
        serde_json::to_string(&self.state).expect("state serializes")
    }

    pub fn load(json: &str, unix_now: f64) -> Result<Self, String> {
        let mut state: State = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if state.version != crate::SAVE_VERSION {
            return Err(format!(
                "Save version {} is not supported ({})",
                state.version,
                crate::SAVE_VERSION
            ));
        }
        let balance = Balance::load();
        let offline = (unix_now - state.saved_at_unix).max(0.0);
        state.last_offline_secs = offline.min(balance.offline.cap_secs);
        let mut g = Self { state, balance };
        if offline > 1.0 {
            g.tick(offline);
        }
        Ok(g)
    }

    pub fn view(&self) -> ViewModel {
        ViewModel::build(&self.state, &self.balance)
    }
}

// ------------------------------------------------------------- helpers

pub fn contacts_preview(s: &State, bal: &Balance) -> u32 {
    if s.run_secs() < bal.prestige.run_floor_secs {
        return 0;
    }
    let e = bal.contacts_exponent(s.tier);
    (s.run_cash / bal.prestige.contacts_base)
        .max(0.0)
        .powf(e)
        .floor() as u32
}

fn level_for(age: f64, bal: &Balance) -> u32 {
    let l = 1 + (age / bal.accounts.level_secs) as u32;
    l.min(bal.accounts.max_level)
}

fn band_for(heat: f64, bal: &Balance) -> HeatBand {
    if heat >= bal.heat.imminent_at {
        HeatBand::Imminent
    } else if heat >= bal.heat.watch_at {
        HeatBand::Watched
    } else if heat >= bal.heat.flag_at {
        HeatBand::Flagged
    } else {
        HeatBand::Quiet
    }
}

fn band_kind(b: HeatBand) -> LogKind {
    match b {
        HeatBand::Quiet => LogKind::Info,
        HeatBand::Flagged => LogKind::Warn,
        HeatBand::Watched => LogKind::Warn,
        HeatBand::Imminent => LogKind::Danger,
    }
}

fn band_message(id: u32, b: HeatBand, heat: f64) -> String {
    match b {
        HeatBand::Quiet => format!("{} cooled off. Back to quiet.", tag(id)),
        HeatBand::Flagged => format!(
            "GM opened a ticket on {} (heat {:.0}). Nothing happened. Yet.",
            tag(id),
            heat
        ),
        HeatBand::Watched => format!(
            "{} is being watched (heat {:.0}). Est. action within a few hours.",
            tag(id),
            heat
        ),
        HeatBand::Imminent => format!(
            "{} will be actioned shortly (heat {:.0}). Move it or lose it.",
            tag(id),
            heat
        ),
    }
}

pub fn tag(id: u32) -> String {
    format!("#A{id:02}")
}

fn parse_id(arg: Option<&str>) -> Result<u32, String> {
    arg.ok_or_else(|| "Account id required".to_string())?
        .trim_start_matches("#A")
        .trim_start_matches("#a")
        .parse()
        .map_err(|_| "Account id must be a number".to_string())
}

pub fn money(v: f64) -> String {
    let s = format!("{v:.2}");
    let (int, frac) = s.split_once('.').unwrap_or((s.as_str(), "00"));
    format!("{}.{}", commas(int), frac)
}

pub fn num(v: f64) -> String {
    commas(&format!("{:.0}", v))
}

fn commas(s: &str) -> String {
    let neg = s.starts_with('-');
    let digits = s.trim_start_matches('-');
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}
