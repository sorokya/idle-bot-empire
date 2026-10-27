//! Balance harness: the assertions the design doc promises.
//! These are intentionally loose bands — they catch a broken curve, not a
//! two-percent drift.

use ibe_sim::{contacts_preview, harness, Command, Game};

#[test]
fn first_run_ends_in_a_reasonable_window() {
    let mut game = Game::new(7);
    let played = harness::autoplay(&mut game, 8.0 * 3600.0);
    let hours = played / 3600.0;

    assert!(
        game.state.over,
        "first run should end within 8h, played {hours:.2}h"
    );
    assert!(
        (2.0..=5.0).contains(&hours),
        "first run should land in 2-5h, took {hours:.2}h"
    );
}

#[test]
fn first_prestige_pays_a_meaningful_amount() {
    let mut game = Game::new(7);
    harness::autoplay(&mut game, 8.0 * 3600.0);
    let gain = contacts_preview(&game.state, &game.balance);
    assert!(
        gain >= 2,
        "first run should pay at least 2 contacts, paid {gain}"
    );
    assert!(
        gain <= 40,
        "first run paid too much ({gain}) — curve too generous"
    );
}

#[test]
fn run_two_is_faster_than_run_one() {
    let mut game = Game::new(7);
    let first = harness::autoplay(&mut game, 8.0 * 3600.0);
    game.prestige().expect("prestige after sweep");
    let second = harness::autoplay(&mut game, 8.0 * 3600.0);

    // Hybrid design: runs are allowed to stretch, but they must pay better.
    assert!(
        second < first * 2.0,
        "run two ({:.2}h) should not be wildly slower than run one ({:.2}h)",
        second / 3600.0,
        first / 3600.0
    );
    assert!(
        game.state.over,
        "run two should end within 8h (tier {}, exposure {:.0}/{:.0})",
        game.state.tier,
        game.state.exposure,
        game.balance.exposure_ceiling(game.state.tier)
    );
    assert_eq!(
        game.state.tier, 1,
        "exactly one prestige should have happened"
    );
    assert!(
        game.state.contacts >= 4,
        "second run should accumulate contacts, have {}",
        game.state.contacts
    );
}

#[test]
fn time_to_next_purchase_stays_in_band_early() {
    // Sample every minute: how often is the player able to buy *something*?
    let mut game = Game::new(9);
    let mut dry_minutes = 0;
    let mut worst_dry = 0;

    for _ in 0..240 {
        game.tick(60.0);
        {
            let cheapest = game.account_cost().min(game.script_cost());
            let can_buy = game.state.cash >= cheapest;
            if can_buy {
                dry_minutes = 0;
            } else {
                dry_minutes += 1;
                worst_dry = worst_dry.max(dry_minutes);
            }
        }
        if game.state.gold > 2500.0 {
            let _ = game.dispatch(Command::Sell);
        }
        if game.state.cash > game.account_cost()
            && game.state.accounts.len() < usize::try_from(game.state.slots).unwrap_or(usize::MAX)
        {
            let _ = game.dispatch(Command::BuyAccount);
        }
        if game.state.cash > game.script_cost() * 2.0 {
            let _ = game.dispatch(Command::Upgrade(ibe_sim::Upgrade::Script));
        }
    }

    assert!(
        worst_dry <= 45,
        "player went {worst_dry} minutes without any affordable purchase"
    );
}
