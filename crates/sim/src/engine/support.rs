//! Internal helpers shared by the tick and action implementations.

use crate::balance::Balance;
use crate::numeric::f64_to_u32;
use crate::state::{HeatBand, LogKind};
use crate::text::tag;

/// Derives an account level from its age.
pub(super) fn level_for_age(age_seconds: f64, balance: &Balance) -> u32 {
    let level = f64_to_u32(age_seconds / balance.accounts.level_secs).saturating_add(1);
    level.min(balance.accounts.max_level)
}

/// Maps a heat value to its warning band.
pub(super) fn heat_band_for(heat: f64, balance: &Balance) -> HeatBand {
    if heat >= balance.heat.imminent_at {
        HeatBand::Imminent
    } else if heat >= balance.heat.watch_at {
        HeatBand::Watched
    } else if heat >= balance.heat.flag_at {
        HeatBand::Flagged
    } else {
        HeatBand::Quiet
    }
}

/// Chooses the log severity for a heat-band change.
pub(super) fn log_kind_for_band(band: HeatBand) -> LogKind {
    match band {
        HeatBand::Quiet => LogKind::Info,
        HeatBand::Flagged | HeatBand::Watched => LogKind::Warn,
        HeatBand::Imminent => LogKind::Danger,
    }
}

/// Builds the player-facing message for a heat-band change.
pub(super) fn heat_band_message(account_id: u32, band: HeatBand, heat: f64) -> String {
    match band {
        HeatBand::Quiet => format!("{} cooled off. Back to quiet.", tag(account_id)),
        HeatBand::Flagged => format!(
            "GM opened a ticket on {} (heat {:.0}). Nothing happened. Yet.",
            tag(account_id),
            heat
        ),
        HeatBand::Watched => format!(
            "{} is being watched (heat {:.0}). Est. action within a few hours.",
            tag(account_id),
            heat
        ),
        HeatBand::Imminent => format!(
            "{} will be actioned shortly (heat {:.0}). Move it or lose it.",
            tag(account_id),
            heat
        ),
    }
}
