//! Numeric conversions used by the simulation and its display helpers.
//!
//! The simulation works in `f64` for economy math and `u32` for counters. These
//! helpers make boundary conversions explicit and saturating, so call sites do
//! not hide lossy casts.

/// Converts a non-negative float to `u32`, clamping and truncating.
///
/// This is intentionally used only for display values and proven-range sim
/// counters such as levels and contacts.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn f64_to_u32(value: f64) -> u32 {
    value.clamp(0.0, f64::from(u32::MAX)) as u32
}

/// Converts a collection length to `f64`.
pub(crate) fn count_as_f64(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// Converts a collection length to `u32`, saturating on overflow.
pub(crate) fn count_as_u32(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Converts a `u32` counter to `i32` for `f64::powi`.
pub(crate) fn u32_to_i32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Converts a collection length to `i32` for `f64::powi`.
pub(crate) fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Converts a `u32` count to `usize` for collection comparisons.
pub(crate) fn u32_to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}
