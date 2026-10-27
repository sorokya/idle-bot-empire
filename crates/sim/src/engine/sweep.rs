//! The sweep (prestige) and its Contacts payout.

use crate::balance::Balance;
use crate::numeric::{count_as_u32, f64_to_u32};
use crate::state::{LogKind, State};
use crate::text::{format_integer, tag};

use super::Game;

/// Contacts that prestiging right now would award.
pub fn contacts_preview(state: &State, balance: &Balance) -> u32 {
    if state.run_secs() < balance.prestige.run_floor_secs {
        return 0;
    }
    let exponent = balance.contacts_exponent(state.tier);
    f64_to_u32(
        (state.run_cash / balance.prestige.contacts_base)
            .max(0.0)
            .powf(exponent)
            .floor(),
    )
}

impl Game {
    pub(super) fn sweep(&mut self) {
        let balance = &self.balance;
        let state = &mut self.state;

        state.push_log(
            LogKind::Danger,
            "PATCH DAY. Shard goes down for maintenance. Some accounts do not come back.",
        );

        let accounts = std::mem::take(&mut state.accounts);
        let mut kept = Vec::with_capacity(accounts.len());
        let mut banned = 0_u32;

        for account in accounts {
            let ban_chance = if account.heat < balance.sweep.min_ban_heat {
                0.0
            } else {
                (account.heat / balance.sweep.ban_divisor).clamp(0.0, 0.95)
            };

            if state.rng.f64() < ban_chance {
                banned = banned.saturating_add(1);
                state.push_log(
                    LogKind::Danger,
                    format!(
                        "{} banned. Level {}, heat {:.0}. Nothing recoverable.",
                        tag(account.id),
                        account.level,
                        account.heat
                    ),
                );
            } else {
                kept.push(account);
            }
        }

        let survivors = count_as_u32(kept.len());
        state.accounts = kept;
        let seized = state.gold;
        state.gold = 0.0;
        state.banned_total = state.banned_total.saturating_add(banned);
        state.over = true;
        state.sweep_pending = false;

        state.push_log(
            LogKind::Danger,
            format!(
                "{} of {} accounts banned. {} gold in staging seized.",
                banned,
                banned.saturating_add(survivors),
                format_integer(seized)
            ),
        );

        let gain = contacts_preview(state, balance);
        if survivors == 0 {
            state.push_log(LogKind::System, "Nothing left but the lesson.");
        }
        state.push_log(
            LogKind::System,
            format!("New identity available: +{gain} contacts."),
        );
    }
}
