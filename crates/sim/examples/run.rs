//! Headless playtest: runs a scripted "reasonable player" until the run ends
//! and prints the log. This is the balance-review tool.
//!
//! cargo run --example run -- 6

use ibe_sim::{contacts_preview, harness, Game, LogKind};

fn main() {
    let hours: f64 = std::env::args()
        .nth(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(6.0);
    let seed: u64 = std::env::args()
        .nth(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(7);

    let mut game = Game::new(seed);
    let played = harness::autoplay(&mut game, hours * 3600.0);

    println!("\n================ LOG ================");
    for entry in &game.state.log {
        let kind = match entry.kind {
            LogKind::Money => "$",
            LogKind::Warn => "!",
            LogKind::Danger => "!!",
            LogKind::System => "*",
            LogKind::Info => " ",
        };
        println!(
            "D{} {:>5} [{}] {}",
            format_day(entry.at),
            format_clock(entry.at),
            kind,
            entry.text
        );
    }

    let view = game.view();
    println!("\n============== SUMMARY ==============");
    println!("played        {:.2}h", played / 3600.0);
    println!(
        "accounts      {} / {} slots",
        view.accounts.len(),
        view.capacity
    );
    println!("gold staged   {:.0}", view.gold);
    println!(
        "cash          ${:.2}  (run total ${:.2})",
        view.cash, game.state.run_cash
    );
    println!("exposure      {:.0} / {:.0}", view.exposure, view.ceiling);
    println!(
        "tier {}  contacts {}  pending {}",
        view.tier,
        view.contacts,
        contacts_preview(&game.state, &game.balance)
    );
    println!("banned total  {}", view.banned_total);
    println!("run over      {}", view.over);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn format_day(seconds: f64) -> u32 {
    1 + (seconds / 86400.0) as u32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn format_clock(seconds: f64) -> String {
    let in_day = seconds.rem_euclid(86400.0);
    let hours = (in_day / 3600.0) as u32;
    let minutes = ((in_day % 3600.0) / 60.0) as u32;
    format!("{hours:02}:{minutes:02}")
}
