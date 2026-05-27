// ===== External Crate Heuristics =====

/// Crate stability classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateStability {
    /// Rust language fundamentals (std, core, alloc) - always ignore
    Fundamental,
    /// Highly stable, ubiquitous crates (serde, thiserror) - low concern
    Stable,
    /// Infrastructure crates (tokio, tracing) - medium concern
    Infrastructure,
    /// Regular external crate - normal analysis
    Normal,
}

/// Check the stability classification of a crate
pub fn classify_crate_stability(crate_name: &str) -> CrateStability {
    // Extract the base crate name (before ::)
    let base_name = crate_name.split("::").next().unwrap_or(crate_name).trim();

    match base_name {
        // Rust fundamentals - always safe to depend on
        "std" | "core" | "alloc" => CrateStability::Fundamental,

        // Highly stable, de-facto standard crates
        "serde" | "serde_json" | "serde_yaml" | "toml" |  // Serialization
        "thiserror" | "anyhow" |                           // Error handling
        "log" |                                            // Logging trait
        "chrono" | "time" |                                // Date/time
        "uuid" |                                           // UUIDs
        "regex" |                                          // Regex
        "lazy_static" | "once_cell" |                      // Statics
        "bytes" | "memchr" |                               // Byte utilities
        "itertools" |                                      // Iterator utilities
        "derive_more" | "strum"                            // Derive macros
        => CrateStability::Stable,

        // Infrastructure crates - stable but architectural decisions
        "tokio" | "async-std" | "smol" |                   // Async runtimes
        "async-trait" |                                    // Async traits
        "futures" | "futures-util" |                       // Futures
        "tracing" | "tracing-subscriber" |                 // Tracing
        "tracing-opentelemetry" | "opentelemetry" |        // Observability
        "opentelemetry-otlp" | "opentelemetry_sdk" |
        "hyper" | "reqwest" | "http" |                     // HTTP
        "tonic" | "prost" |                                // gRPC
        "sqlx" | "diesel" | "sea-orm" |                    // Database
        "clap" | "structopt"                               // CLI
        => CrateStability::Infrastructure,

        // Everything else
        _ => CrateStability::Normal,
    }
}

/// Check if a crate should be excluded from issue detection
pub fn should_skip_crate(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Fundamental
    )
}

/// Check if a crate should have reduced severity
pub fn should_reduce_severity(crate_name: &str) -> bool {
    matches!(
        classify_crate_stability(crate_name),
        CrateStability::Stable | CrateStability::Infrastructure
    )
}

/// Check if this is an external crate (not part of the workspace)
/// External crates are identified by not containing "::" or starting with known external patterns
pub fn is_external_crate(target: &str, source: &str) -> bool {
    // If target doesn't have ::, it might be external
    // But we need to check if it's the same workspace member

    // Extract the crate/module prefix
    let target_prefix = target.split("::").next().unwrap_or(target);
    let source_prefix = source.split("::").next().unwrap_or(source);

    // If they have the same prefix, it's internal
    if target_prefix == source_prefix {
        return false;
    }

    // If target looks like an external crate pattern (no workspace prefix match)
    // Check if it's a known stable/infrastructure crate
    let stability = classify_crate_stability(target);
    matches!(
        stability,
        CrateStability::Fundamental | CrateStability::Stable | CrateStability::Infrastructure
    )
}
