use crate::metrics::dimensions::{Distance, IntegrationStrength};
use crate::volatility::Volatility;

pub(crate) fn extract_type_name(path: &str) -> String {
    path.split("::")
        .last()
        .unwrap_or(path)
        .chars()
        .enumerate()
        .map(|(i, c)| if i == 0 { c.to_ascii_uppercase() } else { c })
        .collect()
}

/// Get human-readable label for integration strength
pub fn strength_label(strength: IntegrationStrength) -> &'static str {
    match strength {
        IntegrationStrength::Intrusive => "Intrusive",
        IntegrationStrength::Functional => "Functional",
        IntegrationStrength::Model => "Model",
        IntegrationStrength::Contract => "Contract",
    }
}

/// Get human-readable label for distance
pub fn distance_label(distance: Distance) -> &'static str {
    match distance {
        Distance::SameFunction => "same function",
        Distance::SameModule => "same module",
        Distance::DifferentModule => "different module",
        Distance::DifferentCrate => "external crate",
    }
}

/// Get human-readable label for volatility
pub fn volatility_label(volatility: Volatility) -> &'static str {
    match volatility {
        Volatility::Low => "rarely",
        Volatility::Medium => "sometimes",
        Volatility::High => "frequently",
    }
}
