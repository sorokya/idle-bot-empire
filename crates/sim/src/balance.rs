use serde::{Deserialize, Serialize};

use crate::numeric::u32_to_i32;

/// Every tunable number in the game. Loaded from `balance.toml` at compile
/// time via `include_str!`, so tuning never requires touching code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub(crate) accounts: Accounts,
    pub(crate) slots: Slots,
    pub(crate) upgrades: Upgrades,
    pub(crate) heat: Heat,
    pub(crate) exposure: Exposure,
    pub(crate) market: Market,
    pub(crate) sweep: Sweep,
    pub(crate) prestige: Prestige,
    pub(crate) start: Start,
    pub(crate) offline: Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Accounts {
    pub(crate) base_slots: u32,
    pub(crate) base_yield: f64,
    pub(crate) level_growth: f64,
    pub(crate) level_secs: f64,
    pub(crate) max_level: u32,
    pub(crate) cost: f64,
    pub(crate) cost_growth: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Slots {
    pub(crate) base_cost: f64,
    pub(crate) cost_growth: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Upgrades {
    pub(crate) script_base_cost: f64,
    pub(crate) script_cost_growth: f64,
    pub(crate) script_yield_per_level: f64,
    pub(crate) proxy_base_cost: f64,
    pub(crate) proxy_cost_growth: f64,
    pub(crate) proxy_quality_per_level: f64,
    pub(crate) proxy_base_quality: f64,
    pub(crate) humanizer_base_cost: f64,
    pub(crate) humanizer_cost_growth: f64,
    pub(crate) humanizer_quality_per_level: f64,
    pub(crate) humanizer_base_quality: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Heat {
    pub(crate) base: f64,
    pub(crate) crowding: f64,
    pub(crate) decay: f64,
    pub(crate) decay_delay: f64,
    pub(crate) flag_at: f64,
    pub(crate) watch_at: f64,
    pub(crate) imminent_at: f64,
    pub(crate) aggression_yield_exp: f64,
    pub(crate) aggression_heat_exp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Exposure {
    pub(crate) per_account_hr: f64,
    pub(crate) ceiling_base: f64,
    pub(crate) ceiling_growth: f64,
    pub(crate) aggression_exp: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Market {
    pub(crate) price: f64,
    pub(crate) drift: f64,
    pub(crate) shard_turnover: f64,
    pub(crate) dump_hit: f64,
    pub(crate) min_price: f64,
    pub(crate) max_price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Sweep {
    pub(crate) ban_divisor: f64,
    pub(crate) min_ban_heat: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Prestige {
    pub(crate) contacts_base: f64,
    pub(crate) contacts_exp_start: f64,
    pub(crate) contacts_exp_step: f64,
    pub(crate) contacts_exp_floor: f64,
    pub(crate) mult_per_contact: f64,
    pub(crate) run_floor_secs: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Start {
    pub(crate) cash: f64,
    pub(crate) manual_click: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Offline {
    pub(crate) cap_secs: f64,
}

const BALANCE_TOML: &str = include_str!("../balance.toml");

impl Balance {
    #[allow(clippy::expect_used)]
    pub(crate) fn load() -> Self {
        toml::from_str(BALANCE_TOML).expect("balance.toml is malformed")
    }

    /// Returns the operation-wide exposure ceiling for a prestige tier.
    pub fn exposure_ceiling(&self, tier: u32) -> f64 {
        self.exposure.ceiling_base * self.exposure.ceiling_growth.powi(u32_to_i32(tier))
    }

    pub(crate) fn contacts_exponent(&self, tier: u32) -> f64 {
        (self.prestige.contacts_exp_start - self.prestige.contacts_exp_step * f64::from(tier))
            .max(self.prestige.contacts_exp_floor)
    }
}

impl Default for Balance {
    fn default() -> Self {
        Self::load()
    }
}
