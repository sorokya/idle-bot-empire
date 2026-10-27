use serde::{Deserialize, Serialize};

/// Every tunable number in the game. Loaded from `balance.toml` at compile
/// time via `include_str!`, so tuning never requires touching code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub accounts: Accounts,
    pub slots: Slots,
    pub upgrades: Upgrades,
    pub heat: Heat,
    pub exposure: Exposure,
    pub market: Market,
    pub sweep: Sweep,
    pub prestige: Prestige,
    pub start: Start,
    pub offline: Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Accounts {
    pub base_slots: u32,
    pub base_yield: f64,
    pub level_growth: f64,
    pub level_secs: f64,
    pub max_level: u32,
    pub cost: f64,
    pub cost_growth: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Slots {
    pub base_cost: f64,
    pub cost_growth: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Upgrades {
    pub script_base_cost: f64,
    pub script_cost_growth: f64,
    pub script_yield_per_level: f64,
    pub proxy_base_cost: f64,
    pub proxy_cost_growth: f64,
    pub proxy_quality_per_level: f64,
    pub proxy_base_quality: f64,
    pub humanizer_base_cost: f64,
    pub humanizer_cost_growth: f64,
    pub humanizer_quality_per_level: f64,
    pub humanizer_base_quality: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Heat {
    pub base: f64,
    pub crowding: f64,
    pub decay: f64,
    pub decay_delay: f64,
    pub flag_at: f64,
    pub watch_at: f64,
    pub imminent_at: f64,
    pub aggression_yield_exp: f64,
    pub aggression_heat_exp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exposure {
    pub per_account_hr: f64,
    pub ceiling_base: f64,
    pub ceiling_growth: f64,
    pub aggression_exp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Market {
    pub price: f64,
    pub drift: f64,
    pub shard_turnover: f64,
    pub dump_hit: f64,
    pub min_price: f64,
    pub max_price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sweep {
    pub ban_divisor: f64,
    pub min_ban_heat: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prestige {
    pub contacts_base: f64,
    pub contacts_exp_start: f64,
    pub contacts_exp_step: f64,
    pub contacts_exp_floor: f64,
    pub mult_per_contact: f64,
    pub run_floor_secs: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Start {
    pub cash: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offline {
    pub cap_secs: f64,
}

pub const BALANCE_TOML: &str = include_str!("../balance.toml");

impl Balance {
    pub fn load() -> Self {
        toml::from_str(BALANCE_TOML).expect("balance.toml is malformed")
    }

    pub fn exposure_ceiling(&self, tier: u32) -> f64 {
        self.exposure.ceiling_base * self.exposure.ceiling_growth.powi(tier as i32)
    }

    pub fn contacts_exponent(&self, tier: u32) -> f64 {
        (self.prestige.contacts_exp_start - self.prestige.contacts_exp_step * tier as f64)
            .max(self.prestige.contacts_exp_floor)
    }
}

impl Default for Balance {
    fn default() -> Self {
        Balance::load()
    }
}
