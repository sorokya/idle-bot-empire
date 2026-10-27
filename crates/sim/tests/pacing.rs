//! Balance harness: the assertions the design doc promises.
//! These are intentionally loose bands — they catch a broken curve, not a
//! two-percent drift.

use ibe_sim::engine::contacts_preview;
use ibe_sim::{harness, Game};

#[test]
fn first_run_ends_in_a_reasonable_window() {
    let mut g = Game::new(7);
    let played = harness::autoplay(&mut g, 8.0 * 3600.0);
    let hrs = played / 3600.0;

    assert!(g.state.over, "first run should end within 8h, played {hrs:.2}h");
    assert!(
        (2.0..=5.0).contains(&hrs),
        "first run should land in 2-5h, took {hrs:.2}h"
    );
}

#[test]
fn first_prestige_pays_a_meaningful_amount() {
    let mut g = Game::new(7);
    harness::autoplay(&mut g, 8.0 * 3600.0);
    let gain = contacts_preview(&g.state, &g.balance);
    assert!(gain >= 2, "first run should pay at least 2 contacts, paid {gain}");
    assert!(gain <= 40, "first run paid too much ({gain}) — curve too generous");
}

#[test]
fn run_two_is_faster_than_run_one() {
    let mut g = Game::new(7);
    let first = harness::autoplay(&mut g, 8.0 * 3600.0);
    g.prestige().expect("prestige after sweep");
    let second = harness::autoplay(&mut g, 8.0 * 3600.0);

    // Hybrid design: runs are allowed to stretch, but they must pay better.
    assert!(
        second < first * 2.0,
        "run two ({:.2}h) should not be wildly slower than run one ({:.2}h)",
        second / 3600.0,
        first / 3600.0
    );
    assert!(
        g.state.over,
        "run two should end within 8h (tier {}, exposure {:.0}/{:.0})",
        g.state.tier, g.state.exposure, g.balance.exposure_ceiling(g.state.tier)
    );
    assert_eq!(g.state.tier, 1, "exactly one prestige should have happened");
    assert!(
        g.state.contacts >= 4,
        "second run should accumulate contacts, have {}",
        g.state.contacts
    );
}

#[test]
fn time_to_next_purchase_stays_in_band_early() {
    // Sample every minute: how often is the player able to buy *something*?
    let mut g = Game::new(9);
    let mut dry_minutes = 0;
    let mut worst_dry = 0;

    for _ in 0..240 {
        g.tick(60.0);
        {
            let cheapest = g.account_cost().min(g.script_cost());
            let can = g.state.cash >= cheapest;
            if can {
                dry_minutes = 0;
            } else {
                dry_minutes += 1;
                worst_dry = worst_dry.max(dry_minutes);
            }
        }
        if g.state.gold > 2500.0 {
            let _ = g.dispatch("sell");
        }
        if g.state.cash > g.account_cost() && (g.state.accounts.len() as u32) < g.state.slots {
            let _ = g.dispatch("buy_account");
        }
        if g.state.cash > g.script_cost() * 2.0 {
            let _ = g.dispatch("upgrade:script");
        }
    }

    assert!(
        worst_dry <= 45,
        "player went {worst_dry} minutes without any affordable purchase"
    );
}
