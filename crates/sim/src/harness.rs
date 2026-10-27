//! A scripted "reasonable player" used by the balance harness and the CLI.
//! Not shipped to players — this is the tuning instrument.

use crate::engine::{Command, Game, Upgrade};
use crate::numeric::{count_as_f64, count_as_u32, u32_to_usize};

/// Play like a competent-but-greedy operator: triage heat, sell in batches,
/// reinvest in slots before width, buy stealth when heat runs hot.
pub fn autoplay(game: &mut Game, total_seconds: f64) -> f64 {
    let mut elapsed = 0.0;
    while elapsed < total_seconds && !game.state.over {
        game.tick(60.0);
        elapsed += 60.0;
        decide(game);
    }
    elapsed
}

/// Applies one minute of scripted decisions.
pub fn decide(game: &mut Game) {
    if game.state.over {
        return;
    }

    // Manual phase: click until a bot is affordable, then automate.
    if !game.state.started {
        let needed = game.first_bot_gold();
        let mut guard = 0;
        while game.state.gold < needed && guard < 500 {
            let _ = game.dispatch(Command::Click);
            guard += 1;
        }
        let _ = game.dispatch(Command::Sell);
        let _ = game.dispatch(Command::BuyAccount);
        return;
    }

    // A queued sweep is a decision: liquidate, then take the hit.
    if game.state.sweep_pending {
        if game.state.gold >= 1.0 {
            let _ = game.dispatch(Command::Sell);
        }
        let _ = game.dispatch(Command::FaceSweep);
        return;
    }

    let snapshot: Vec<(u32, f64, bool)> = game
        .state
        .accounts
        .iter()
        .map(|account| (account.id, account.heat, account.paused))
        .collect();

    for (account_id, heat, paused) in &snapshot {
        if *heat >= 92.0 {
            let _ = game.dispatch(Command::Burn(*account_id));
        } else if (*heat >= 78.0 && !*paused) || (*paused && *heat < 30.0) {
            let _ = game.dispatch(Command::TogglePause(*account_id));
        }
    }

    if game.state.gold > 30_000.0 {
        let _ = game.dispatch(Command::Sell);
    }

    if game.state.cash > game.slot_cost() * 1.5
        && game.state.accounts.len() >= u32_to_usize(game.state.slots)
    {
        let _ = game.dispatch(Command::BuySlot);
    }
    if game.state.cash > game.account_cost() * 1.2
        && count_as_u32(game.state.accounts.len()) < game.state.slots
    {
        let _ = game.dispatch(Command::BuyAccount);
    }

    let average_heat = average_heat(game);
    if average_heat > 45.0 && game.state.cash > game.proxy_cost() * 1.5 {
        let _ = game.dispatch(Command::Upgrade(Upgrade::Proxy));
    }
    if game.state.script_level < 4 && game.state.cash > game.script_cost() * 2.0 {
        let _ = game.dispatch(Command::Upgrade(Upgrade::Script));
    }
    if average_heat > 40.0 && game.state.cash > game.humanizer_cost() * 2.0 {
        let _ = game.dispatch(Command::Upgrade(Upgrade::Humanizer));
    }

    let target = if average_heat > 70.0 {
        0.6
    } else if average_heat < 18.0 {
        1.4
    } else {
        game.state.aggression
    };
    if (game.state.aggression - target).abs() > 0.05 {
        let _ = game.dispatch(Command::SetAggression(target));
    }
}

/// Average heat across all accounts, or zero when there are none.
pub fn average_heat(game: &Game) -> f64 {
    let heats: Vec<f64> = game
        .state
        .accounts
        .iter()
        .map(|account| account.heat)
        .collect();
    if heats.is_empty() {
        0.0
    } else {
        heats.iter().sum::<f64>() / count_as_f64(heats.len())
    }
}
