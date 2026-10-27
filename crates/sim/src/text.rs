//! Small text and number formatting helpers shared by the engine and the view.

/// A stable, human-readable account tag such as `#A07`.
pub(crate) fn tag(id: u32) -> String {
    format!("#A{id:02}")
}

/// Formats a cash amount as `1,234.56`.
pub(crate) fn money(value: f64) -> String {
    let formatted = format!("{value:.2}");
    let (integer, fraction) = formatted
        .split_once('.')
        .unwrap_or((formatted.as_str(), "00"));
    format!("{}.{}", commas(integer), fraction)
}

/// Formats a non-negative integer with thousands separators.
pub(crate) fn format_integer(value: f64) -> String {
    commas(&format!("{value:.0}"))
}

fn commas(digits: &str) -> String {
    let is_negative = digits.starts_with('-');
    let unsigned = digits.trim_start_matches('-');
    let mut output = String::with_capacity(digits.len());
    for (index, character) in unsigned.chars().enumerate() {
        if index > 0 && (unsigned.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    if is_negative {
        format!("-{output}")
    } else {
        output
    }
}
