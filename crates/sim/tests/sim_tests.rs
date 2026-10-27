use ibe_sim::Game;

#[test]
fn same_seed_same_result() {
    let run = || {
        let mut g = Game::new(42);
        for _ in 0..120 {
            g.tick(60.0);
            if g.state.gold > 1000.0 {
                let _ = g.dispatch("sell");
            }
            if (g.state.accounts.len() as u32) < g.state.slots {
                let _ = g.dispatch("buy_account");
            }
        }
        g.save(0.0)
    };
    assert_eq!(run(), run(), "simulation must be deterministic per seed");
}

#[test]
fn save_roundtrip_preserves_state() {
    let mut g = Game::new(7);
    g.tick(3600.0);
    let _ = g.dispatch("sell");
    let json = g.save(1_000.0);

    let loaded = Game::load(&json, 1_000.0).expect("load");
    assert!((loaded.state.cash - g.state.cash).abs() < 1e-6);
    assert_eq!(loaded.state.accounts.len(), g.state.accounts.len());
    assert_eq!(loaded.state.tier, g.state.tier);
}

#[test]
fn offline_time_accrues_but_is_capped() {
    let mut g = Game::new(7);
    let json = g.save(1_000.0);

    // 12h later: full catch-up, which here means the run sweeps and ends.
    let g2 = Game::load(&json, 1_000.0 + 43_200.0).unwrap();
    assert!(
        (g2.state.now - 43_200.0).abs() < 1.0,
        "should have advanced 12h"
    );

    // 48h later: still capped at 12h of simulation, not 4x.
    let g3 = Game::load(&json, 1_000.0 + 172_800.0).unwrap();
    assert!(
        (g3.state.now - g2.state.now).abs() < 1.0,
        "offline catch-up must cap at 12h"
    );
    assert_eq!(g3.state.banned_total, g2.state.banned_total);
}

#[test]
fn heat_rises_and_bans_are_possible() {
    let mut g = started(11);
    g.tick(3600.0);
    let heat = g.state.accounts[0].heat;
    assert!(heat > 5.0, "account should accumulate heat, got {heat}");
}

#[test]
fn sweep_ends_run_and_prestige_advances_tier() {
    let mut g = started(3);
    // Run hot with small numbers until the ceiling trips.
    g.state.aggression = 2.0;
    let mut guard = 0;
    while !g.state.sweep_pending && guard < 200_000 {
        g.tick(60.0);
        guard += 1;
    }
    assert!(
        g.state.sweep_pending,
        "exposure ceiling should queue a sweep"
    );
    assert!(!g.state.over, "the sweep must wait for the player");

    g.dispatch("face_sweep").expect("face the sweep");
    assert!(g.state.over, "facing the sweep ends the run");

    let before = g.state.tier;
    g.prestige().expect("prestige");
    assert_eq!(g.state.tier, before + 1);
    assert!(!g.state.over);
    assert_eq!(g.state.accounts.len(), 1);
    assert_eq!(g.state.exposure, 0.0);
}

/// A game past the manual phase, with one bot running.
fn started(seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.dispatch("buy_account").expect("first bot");
    g
}

#[test]
fn new_game_starts_in_the_manual_phase() {
    let g = Game::new(1);
    assert!(!g.state.started, "should start unstarted");
    assert_eq!(g.state.accounts.len(), 0, "no bots on day one");
    assert_eq!(g.view().gold_per_hr, 0.0);
}

#[test]
fn clicking_funds_the_first_bot() {
    let mut g = Game::new(1);
    let need = g.first_bot_gold();
    assert!(need > 0.0);

    let mut clicks = 0;
    while g.state.gold < need && clicks < 1000 {
        g.dispatch("click").expect("click");
        clicks += 1;
    }
    assert!(
        clicks < 60,
        "first bot should need < 60 clicks, took {clicks}"
    );

    g.dispatch("sell").expect("sell");
    g.dispatch("buy_account").expect("buy");
    assert!(g.state.started);
    assert_eq!(g.state.accounts.len(), 1);
    assert!(g.view().gold_per_hr > 0.0);
}

#[test]
fn early_prestige_is_blocked_by_run_floor() {
    let mut g = Game::new(3);
    g.tick(600.0);
    assert!(
        g.prestige().is_err(),
        "cannot prestige in the first 45 minutes"
    );
}
