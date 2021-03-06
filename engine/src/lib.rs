extern crate chess;

mod params;
mod engine;
mod eval;
mod movepick;
mod search;
mod table;
mod utils;

/// The version of the engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Export the Engine struct.
pub use self::engine::Engine;