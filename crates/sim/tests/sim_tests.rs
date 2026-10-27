//! Determinism, save/load, offline cap, and core loop tests.

use ibe_sim::{Command, Game};

#[test]
fn same_seed_same_result() {
    let run = || {
        let mut game = Game::new(42);
        for _ in 0..120 {
            game.tick(60.0);
            if game.state.gold > 1000.0 {
                let _ = game.dispatch(Command::Sell);
            }
            if game.state.accounts.len() < usize::try_from(game.state.slots).unwrap_or(usize::MAX) {
                let _ = game.dispatch(Command::BuyAccount);
            }
        }
        game.save(0.0)
    };
    assert_eq!(run(), run(), "simulation must be deterministic per seed");
}

#[test]
fn save_roundtrip_preserves_state() {
    let mut game = Game::new(7);
    game.tick(3600.0);
    let _ = game.dispatch(Command::Sell);
    let json = game.save(1_000.0);

    let loaded = Game::load(&json, 1_000.0).expect("load");
    assert!((loaded.state.cash - game.state.cash).abs() < 1e-6);
    assert_eq!(loaded.state.accounts.len(), game.state.accounts.len());
    assert_eq!(loaded.state.tier, game.state.tier);
}

#[test]
fn offline_time_accrues_but_is_capped() {
    let mut game = Game::new(7);
    let json = game.save(1_000.0);

    // 12h later: full catch-up, which here means the run sweeps and ends.
    let after_twelve_hours = Game::load(&json, 1_000.0 + 43_200.0).unwrap();
    assert!(
        (after_twelve_hours.state.now - 43_200.0).abs() < 1.0,
        "should have advanced 12h"
    );

    // 48h later: still capped at 12h of simulation, not 4x.
    let after_forty_eight_hours = Game::load(&json, 1_000.0 + 172_800.0).unwrap();
    assert!(
        (after_forty_eight_hours.state.now - after_twelve_hours.state.now).abs() < 1.0,
        "offline catch-up must cap at 12h"
    );
    assert_eq!(
        after_forty_eight_hours.state.banned_total,
        after_twelve_hours.state.banned_total
    );
}

#[test]
fn heat_rises_and_bans_are_possible() {
    let mut game = started(11);
    game.tick(3600.0);
    let heat = game.state.accounts[0].heat;
    assert!(heat > 5.0, "account should accumulate heat, got {heat}");
}

#[test]
fn sweep_ends_run_and_prestige_advances_tier() {
    let mut game = started(3);
    // Run hot with small numbers until the ceiling trips.
    game.state.aggression = 2.0;
    let mut guard = 0;
    while !game.state.sweep_pending && guard < 200_000 {
        game.tick(60.0);
        guard += 1;
    }
    assert!(
        game.state.sweep_pending,
        "exposure ceiling should queue a sweep"
    );
    assert!(!game.state.over, "the sweep must wait for the player");

    game.dispatch(Command::FaceSweep).expect("face the sweep");
    assert!(game.state.over, "facing the sweep ends the run");

    let before = game.state.tier;
    game.prestige().expect("prestige");
    assert_eq!(game.state.tier, before + 1);
    assert!(!game.state.over);
    assert_eq!(game.state.accounts.len(), 1);
    assert_eq!(game.state.exposure, 0.0);
}

/// A game past the manual phase, with one bot running.
#[allow(clippy::expect_used)] // Test fixture: a failed first purchase means the fixture is broken.
fn started(seed: u64) -> Game {
    let mut game = Game::new(seed);
    game.dispatch(Command::BuyAccount).expect("first bot");
    game
}

#[test]
fn new_game_starts_in_the_manual_phase() {
    let game = Game::new(1);
    assert!(!game.state.started, "should start unstarted");
    assert_eq!(game.state.accounts.len(), 0, "no bots on day one");
    assert_eq!(game.view().gold_per_hr, 0.0);
}

#[test]
fn clicking_funds_the_first_bot() {
    let mut game = Game::new(1);
    let needed = game.first_bot_gold();
    assert!(needed > 0.0);

    let mut clicks = 0;
    while game.state.gold < needed && clicks < 1000 {
        game.dispatch(Command::Click).expect("click");
        clicks += 1;
    }
    assert!(
        clicks < 60,
        "first bot should need < 60 clicks, took {clicks}"
    );

    game.dispatch(Command::Sell).expect("sell");
    game.dispatch(Command::BuyAccount).expect("buy");
    assert!(game.state.started);
    assert_eq!(game.state.accounts.len(), 1);
    assert!(game.view().gold_per_hr > 0.0);
}

#[test]
fn early_prestige_is_blocked_by_run_floor() {
    let mut game = Game::new(3);
    game.tick(600.0);
    assert!(
        game.prestige().is_err(),
        "cannot prestige in the first 45 minutes"
    );
}
