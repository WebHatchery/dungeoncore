//! Player-driven economy sinks. Currently: channelling surplus gold into mana.
//!
//! Gold used to pile up dead once species were unlocked — its only sinks were
//! one-time. "Channel the Hoard" turns that stagnant gold into an ongoing sink
//! *and* a mana safety-valve: a deliberately inefficient conversion so slain-
//! invader mana stays the primary income, but hoarded gold can still be poured
//! back into building when mana runs short.

use crate::game_state::{GameState, LogEntry};

/// Gold spent per Channel the Hoard transaction.
pub const GOLD_CHANNEL_COST: i32 = 100;
/// Mana granted per Channel the Hoard transaction (5:1 — intentionally lossy).
pub const GOLD_CHANNEL_MANA: i32 = 20;

/// Can the player channel gold into mana right now?
pub fn can_channel_gold(state: &GameState) -> bool {
    state.gold >= GOLD_CHANNEL_COST && state.mana < state.max_mana
}

/// Channel a fixed chunk of gold into mana (capped at max mana). Returns `Ok`
/// on success or a short reason it could not proceed.
pub fn channel_gold_to_mana(state: &mut GameState) -> Result<(), String> {
    if state.mana >= state.max_mana {
        return Err("Mana is already full.".into());
    }
    if state.gold < GOLD_CHANNEL_COST {
        return Err(format!("Not enough gold! Need {}.", GOLD_CHANNEL_COST));
    }
    state.gold -= GOLD_CHANNEL_COST;
    let before = state.mana;
    state.mana = (state.mana + GOLD_CHANNEL_MANA).min(state.max_mana);
    let gained = state.mana - before;
    state.add_log(LogEntry::system(format!(
        "Channelled {} gold into {} mana.",
        GOLD_CHANNEL_COST, gained
    )));
    Ok(())
}

#[cfg(test)]
mod tests;
