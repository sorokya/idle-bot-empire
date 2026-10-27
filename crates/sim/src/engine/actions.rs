//! Player actions and the costs that gate them.

use crate::numeric::{u32_to_i32, u32_to_usize, usize_to_i32};
use crate::state::LogKind;
use crate::text::{format_integer, money, tag};

use super::{contacts_preview, Command, EngineError, Game, Upgrade};

impl Game {
    /// Applies a player command.
    ///
    /// # Errors
    /// Returns [`EngineError`] when the action is invalid, unaffordable, or the
    /// current run has already ended.
    pub fn dispatch(&mut self, command: Command) -> Result<(), EngineError> {
        match command {
            Command::BuyAccount => self.buy_account(),
            Command::BuySlot => self.buy_slot(),
            Command::Sell => self.sell(),
            Command::Click => self.farm_by_hand(),
            Command::Upgrade(Upgrade::Script) => self.buy_script(),
            Command::Upgrade(Upgrade::Proxy) => self.buy_proxy(),
            Command::Upgrade(Upgrade::Humanizer) => self.buy_humanizer(),
            Command::TogglePause(account_id) => self.toggle_pause(account_id),
            Command::Burn(account_id) => self.burn(account_id),
            Command::SetAggression(value) => self.set_aggression(value),
            Command::FaceSweep => self.face_sweep(),
            Command::Prestige => self.prestige(),
            Command::Wait(seconds) => {
                self.tick(seconds);
                Ok(())
            }
        }
    }

    /// Returns the cash cost of the next account.
    pub fn account_cost(&self) -> f64 {
        self.balance.accounts.cost
            * self
                .balance
                .accounts
                .cost_growth
                .powi(usize_to_i32(self.state.accounts.len()))
    }

    /// Returns the cash cost of the next rack slot.
    pub fn slot_cost(&self) -> f64 {
        self.balance.slots.base_cost
            * self
                .balance
                .slots
                .cost_growth
                .powi(u32_to_i32(self.state.slots_bought))
    }

    /// Returns the cash cost of the next script level.
    pub fn script_cost(&self) -> f64 {
        self.balance.upgrades.script_base_cost
            * self
                .balance
                .upgrades
                .script_cost_growth
                .powi(u32_to_i32(self.state.script_level))
    }

    /// Returns the cash cost of the next proxy tier.
    pub fn proxy_cost(&self) -> f64 {
        self.balance.upgrades.proxy_base_cost
            * self
                .balance
                .upgrades
                .proxy_cost_growth
                .powi(u32_to_i32(self.state.proxy_level))
    }

    /// Returns the cash cost of the next humanizer tier.
    pub fn humanizer_cost(&self) -> f64 {
        self.balance.upgrades.humanizer_base_cost
            * self
                .balance
                .upgrades
                .humanizer_cost_growth
                .powi(u32_to_i32(self.state.humanizer_level))
    }

    /// Returns the staged gold needed to afford the first bot.
    pub fn first_bot_gold(&self) -> f64 {
        self.account_cost() / self.state.market_price.max(0.0001)
    }

    /// Starts a new identity after a run ends.
    ///
    /// # Errors
    /// Returns [`EngineError::PrestigeNotWorth`] when the run is too young or
    /// has not earned any contacts.
    pub fn prestige(&mut self) -> Result<(), EngineError> {
        let gain = contacts_preview(&self.state, &self.balance);
        if gain == 0 && !self.state.over {
            return Err(EngineError::PrestigeNotWorth);
        }

        self.state.contacts = self.state.contacts.saturating_add(gain);
        self.state.tier = self.state.tier.saturating_add(1);
        self.state.runs_completed = self.state.runs_completed.saturating_add(1);
        self.state.push_log(
            LogKind::System,
            format!(
                "New identity. Run {} closed, +{} contacts ({} total).",
                self.state.runs_completed, gain, self.state.contacts
            ),
        );

        self.state.accounts.clear();
        self.state.slots = self.balance.accounts.base_slots;
        self.state.slots_bought = 0;
        self.state.script_level = 0;
        self.state.proxy_level = 0;
        self.state.humanizer_level = 0;
        self.state.aggression = 1.0;
        self.state.gold = 0.0;
        self.state.cash = self.balance.start.cash;
        self.state.run_cash = 0.0;
        self.state.exposure = 0.0;
        self.state.over = false;
        self.state.sweep_pending = false;
        self.state.run_started_at = self.state.now;
        self.state.started = true;
        self.state.add_account(self.state.now);
        self.state.push_log(
            LogKind::Info,
            "Fresh start. Same mistakes available at no extra cost.",
        );
        Ok(())
    }

    fn require_running(&self) -> Result<(), EngineError> {
        if self.state.over {
            Err(EngineError::RunOver)
        } else {
            Ok(())
        }
    }

    fn spend(&mut self, cost: f64, item: &'static str) -> Result<(), EngineError> {
        if self.state.cash < cost {
            return Err(EngineError::InsufficientCash {
                item,
                cost,
                cash: self.state.cash,
            });
        }
        self.state.cash -= cost;
        Ok(())
    }

    fn buy_account(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        if self.state.accounts.len() >= u32_to_usize(self.state.slots) {
            return Err(EngineError::NoFreeSlots);
        }

        let cost = self.account_cost();
        self.spend(cost, "Account")?;
        let now = self.state.now;
        let account_id = self.state.add_account(now);

        if self.state.started {
            self.state.push_log(
                LogKind::Money,
                format!(
                    "{} acquired for ${}. Welcome aboard.",
                    tag(account_id),
                    money(cost)
                ),
            );
        } else {
            self.state.started = true;
            self.state.push_log(
                LogKind::System,
                format!(
                    "{} runs the script now. It farms while you sleep, which is the entire point.",
                    tag(account_id)
                ),
            );
        }
        Ok(())
    }

    fn buy_slot(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        let cost = self.slot_cost();
        self.spend(cost, "Slot")?;
        self.state.slots = self.state.slots.saturating_add(1);
        self.state.slots_bought = self.state.slots_bought.saturating_add(1);
        self.state.push_log(
            LogKind::Money,
            format!(
                "Rack space added for ${}. {} slots total.",
                money(cost),
                self.state.slots
            ),
        );
        Ok(())
    }

    fn buy_script(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        let cost = self.script_cost();
        self.spend(cost, "Script upgrade")?;
        self.state.script_level = self.state.script_level.saturating_add(1);
        self.state.push_log(
            LogKind::Money,
            format!(
                "Script v{} deployed (+{:.0}% yield). They patch, we adapt.",
                self.state.script_level + 1,
                self.balance.upgrades.script_yield_per_level
                    * 100.0
                    * f64::from(self.state.script_level)
            ),
        );
        Ok(())
    }

    fn buy_proxy(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        let cost = self.proxy_cost();
        self.spend(cost, "Proxy tier")?;
        self.state.proxy_level = self.state.proxy_level.saturating_add(1);
        self.state.push_log(
            LogKind::Money,
            format!(
                "Proxy pool upgraded to tier {}. Cheaper, quieter, slower.",
                self.state.proxy_level
            ),
        );
        Ok(())
    }

    fn buy_humanizer(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        let cost = self.humanizer_cost();
        self.spend(cost, "Humanizer")?;
        self.state.humanizer_level = self.state.humanizer_level.saturating_add(1);
        self.state.push_log(
            LogKind::Money,
            format!(
                "Humanizer tier {}. Random pauses, path jitter, plausible mistakes.",
                self.state.humanizer_level
            ),
        );
        Ok(())
    }

    fn farm_by_hand(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        let gain = self.balance.start.manual_click;
        let before = self.state.gold;
        self.state.gold += gain;

        let needed = self.first_bot_gold();
        if !self.state.started && before < needed && self.state.gold >= needed {
            self.state.push_log(
                LogKind::Money,
                "That is enough for a script. Sell the gold, buy the bot, get your evenings back.",
            );
        }
        Ok(())
    }

    fn sell(&mut self) -> Result<(), EngineError> {
        self.require_running()?;
        if self.state.gold < 1.0 {
            return Err(EngineError::NothingToSell);
        }

        let gross = self.state.gold;
        let hit =
            (gross / self.balance.market.shard_turnover).min(1.0) * self.balance.market.dump_hit;
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
                format_integer(gross),
                money(proceeds),
                self.state.market_price
            ),
        );
        Ok(())
    }

    fn set_aggression(&mut self, value: f64) -> Result<(), EngineError> {
        self.require_running()?;
        self.state.aggression = value.clamp(0.2, 2.0);
        self.state.push_log(
            LogKind::Info,
            format!("Script aggression set to {:.1}x.", self.state.aggression),
        );
        Ok(())
    }

    fn face_sweep(&mut self) -> Result<(), EngineError> {
        if self.state.over {
            return Err(EngineError::RunOver);
        }
        if !self.state.sweep_pending {
            return Err(EngineError::NoSweepScheduled);
        }
        self.sweep();
        Ok(())
    }

    fn toggle_pause(&mut self, account_id: u32) -> Result<(), EngineError> {
        let account = self
            .state
            .accounts
            .iter_mut()
            .find(|account| account.id == account_id)
            .ok_or(EngineError::AccountNotFound(account_id))?;
        account.paused = !account.paused;
        let paused = account.paused;
        if paused {
            account.last_heat_at = self.state.now;
        }
        self.state.push_log(
            LogKind::Info,
            if paused {
                format!(
                    "{} pulled from the field. Heat decaying in 10 minutes.",
                    tag(account_id)
                )
            } else {
                format!("{} back online.", tag(account_id))
            },
        );
        Ok(())
    }

    fn burn(&mut self, account_id: u32) -> Result<(), EngineError> {
        let index = self
            .state
            .accounts
            .iter()
            .position(|account| account.id == account_id)
            .ok_or(EngineError::AccountNotFound(account_id))?;
        let account = self.state.accounts.remove(index);
        let salvage =
            self.balance.accounts.cost * (1.0 + 0.2 * (f64::from(account.level) - 1.0)) * 0.4;
        self.state.cash += salvage;
        self.state.push_log(
            LogKind::Warn,
            format!(
                "{} wiped and sold for parts. ${} recovered, {:.0} heat gone.",
                tag(account_id),
                money(salvage),
                account.heat
            ),
        );
        Ok(())
    }
}
