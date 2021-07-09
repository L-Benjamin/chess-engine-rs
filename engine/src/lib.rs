// temporary
#![allow(dead_code, unused_variables, unused_macros)]

extern crate chess;

mod params;
mod engine;
mod eval;
mod movepick;
mod search;
mod table;

// Export the Engine struct.
pub use self::engine::Engine;