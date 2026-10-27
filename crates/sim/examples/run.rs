//! Headless playtest: runs a scripted "reasonable player" until the run ends
//! and prints the log. This is the balance-review tool.
//!
//! cargo run --example run -- 6

use ibe_sim::engine::contacts_preview;
use ibe_sim::{harness, Game, LogKind};

fn main() {
    let hours: f64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6.0);
    let seed: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);

    let mut g = Game::new(seed);
    let played = harness::autoplay(&mut g, hours * 3600.0);

    println!("\n================ LOG ================");
    for e in &g.state.log {
        let kind = match e.kind {
            LogKind::Money => "$",
            LogKind::Warn => "!",
            LogKind::Danger => "!!",
            LogKind::System => "*",
            LogKind::Info => " ",
        };
        println!("D{} {:>5} [{}] {}", day(e.at), hhmm(e.at), kind, e.text);
    }

    let v = g.view();
    println!("\n============== SUMMARY ==============");
    println!("played        {:.2}h", played / 3600.0);
    println!("accounts      {} / {} slots", v.accounts.len(), v.capacity);
    println!("gold staged   {:.0}", v.gold);
    println!("cash          ${:.2}  (run total ${:.2})", v.cash, g.state.run_cash);
    println!("exposure      {:.0} / {:.0}", v.exposure, v.ceiling);
    println!(
        "tier {}  contacts {}  pending {}",
        v.tier, v.contacts, contacts_preview(&g.state, &g.balance)
    );
    println!("banned total  {}", v.banned_total);
    println!("run over      {}", v.over);
}

fn day(secs: f64) -> u32 {
    1 + (secs / 86400.0) as u32
}

fn hhmm(secs: f64) -> String {
    let d = secs.rem_euclid(86400.0);
    format!("{:02}:{:02}", (d / 3600.0) as u32, ((d % 3600.0) / 60.0) as u32)
}
