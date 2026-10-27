//! Typed player actions accepted by the engine.

/// A player action that changes the simulation.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// Buy a new account if a rack slot is free.
    BuyAccount,
    /// Buy an additional rack slot.
    BuySlot,
    /// Sell all staged gold at the current market price.
    Sell,
    /// Farm one click by hand during the manual phase.
    Click,
    /// Buy one of the upgrade tracks.
    Upgrade(Upgrade),
    /// Pause or resume an account by id.
    TogglePause(u32),
    /// Delete an account and salvage part of its value.
    Burn(u32),
    /// Set the global script aggression between 0.2 and 2.0.
    SetAggression(f64),
    /// Face a queued sweep and end the current run.
    FaceSweep,
    /// Start a new identity after a run ends.
    Prestige,
    /// Advance simulation time by the given number of seconds.
    Wait(f64),
}

/// The three upgrade tracks available to the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Upgrade {
    /// Increases gold yield per hour.
    Script,
    /// Improves stealth through proxy quality.
    Proxy,
    /// Improves stealth through humanizer quality.
    Humanizer,
}
