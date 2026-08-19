pub mod adventurers;
pub mod constants;
pub mod difficulty;
pub mod elements;
pub mod equipment;
pub mod evolutions;
pub mod monsters;
pub mod strata;
pub mod traits;
pub mod upgrades;

pub use constants::*;
pub use upgrades::*;

// Every asset JSON is embedded at compile time and parsed lazily at runtime,
// so a malformed file only surfaces when its loader is first hit in-game.
// These tests force every loader (and the cross-file references) at test time.
#[cfg(test)]
mod tests;
