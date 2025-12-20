//! Web visualization module for cargo-coupling
//!
//! Provides an interactive web-based visualization of coupling metrics,
//! allowing exploration of the 5 coupling dimensions:
//! - Strength (Contract, Model, Functional, Intrusive)
//! - Distance (SameFunction, SameModule, DifferentModule, DifferentCrate)
//! - Volatility (Low, Medium, High)
//! - Balance Score (0.0-1.0)
//! - Connascence (Name, Type, Meaning, Position, Algorithm)

pub mod graph;
pub mod routes;
pub mod server;

pub use graph::GraphData;
pub use server::{ServerConfig, start_server};
