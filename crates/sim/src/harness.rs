//! A scripted "reasonable player" used by the balance harness and the CLI.
//! Not shipped to players — this is the tuning instrument.

use crate::engine::Game;

/// Play like a competent-but-greedy operator: triage heat, sell in batches,
/// reinvest in slots before width, buy stealth when heat runs hot.
pub fn autoplay(g: &mut Game, total_secs: f64) -> f64 {
    let mut elapsed = 0.0;
    while elapsed < total_secs && !g.state.over {
        g.tick(60.0);
        elapsed += 60.0;
        decide(g);
    }
    elapsed
}

pub fn decide(g: &mut Game) {
    if g.state.over {
        return;
    }

    // Manual phase: click until a bot is affordable, then automate.
    if !g.state.started {
        let need = g.first_bot_gold();
        let mut guard = 0;
        while g.state.gold < need && guard < 500 {
            let _ = g.dispatch("click");
            guard += 1;
        }
        let _ = g.dispatch("sell");
        let _ = g.dispatch("buy_account");
        return;
    }
    // A queued sweep is a decision: liquidate, then take the hit.
    if g.state.sweep_pending {
        if g.state.gold >= 1.0 {
            let _ = g.dispatch("sell");
        }
        let _ = g.dispatch("face_sweep");
        return;
    }

    let snapshot: Vec<(u32, f64, bool)> = g
        .state
        .accounts
        .iter()
        .map(|a| (a.id, a.heat, a.paused))
        .collect();

    for (id, heat, paused) in &snapshot {
        if *heat >= 92.0 {
            let _ = g.dispatch(&format!("burn:{id}"));
        } else if (*heat >= 78.0 && !*paused) || (*paused && *heat < 30.0) {
            let _ = g.dispatch(&format!("pause:{id}"));
        }
    }

    if g.state.gold > 30000.0 {
        let _ = g.dispatch("sell");
    }

    if g.state.cash > g.slot_cost() * 1.5 && g.state.accounts.len() >= g.state.slots as usize {
        let _ = g.dispatch("buy_slot");
    }
    if g.state.cash > g.account_cost() * 1.2 && (g.state.accounts.len() as u32) < g.state.slots {
        let _ = g.dispatch("buy_account");
    }

    let avg_heat = average_heat(g);
    if avg_heat > 45.0 && g.state.cash > g.proxy_cost() * 1.5 {
        let _ = g.dispatch("upgrade:proxy");
    }
    if g.state.script_level < 4 && g.state.cash > g.script_cost() * 2.0 {
        let _ = g.dispatch("upgrade:script");
    }
    if avg_heat > 40.0 && g.state.cash > g.humanizer_cost() * 2.0 {
        let _ = g.dispatch("upgrade:humanizer");
    }

    let target = if avg_heat > 70.0 {
        0.6
    } else if avg_heat < 18.0 {
        1.4
    } else {
        g.state.aggression
    };
    if (g.state.aggression - target).abs() > 0.05 {
        let _ = g.dispatch(&format!("aggression:{target}"));
    }
}

pub fn average_heat(g: &Game) -> f64 {
    let heats: Vec<f64> = g.state.accounts.iter().map(|a| a.heat).collect();
    if heats.is_empty() {
        0.0
    } else {
        heats.iter().sum::<f64>() / heats.len() as f64
    }
}
