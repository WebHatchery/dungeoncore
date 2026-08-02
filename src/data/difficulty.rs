//! Difficulty presets. One knob-set chosen at new-game time scales how tough
//! and frequent invaders are, how soon the realm sieges, how much income the
//! dungeon banks, and how sturdy the core starts — so the same systems can
//! serve a relaxed builder run or a punishing one.

use serde::{Deserialize, Serialize};

/// The chosen difficulty for a run. `Keeper` is the balanced default and the
/// serde fallback for saves written before difficulty existed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Difficulty {
    /// Gentle: weaker, rarer invaders, later sieges, richer income, sturdier core.
    Apprentice,
    /// The intended, balanced experience.
    #[default]
    Keeper,
    /// Punishing: stronger, more frequent invaders, early sieges, lean income.
    Overlord,
}

/// The concrete multipliers a [`Difficulty`] applies across the sim.
pub struct DifficultyProfile {
    pub name: &'static str,
    pub blurb: &'static str,
    /// Scales invader attack / defense / HP.
    pub invader_stat_mult: f32,
    /// Scales the per-tick chance a new party spawns.
    pub spawn_chance_mult: f32,
    /// Scales the death count that triggers a siege (higher = sieges later).
    pub siege_threshold_mult: f32,
    /// Scales mana income from invaders — both the trickle drawn from those
    /// still inside and the payout banked when one is slain.
    pub income_mult: f32,
    /// Scales the core's starting HP.
    pub core_hp_mult: f32,
}

impl Difficulty {
    /// Every difficulty, easiest first — for the selection UI.
    pub fn all() -> [Difficulty; 3] {
        [
            Difficulty::Apprentice,
            Difficulty::Keeper,
            Difficulty::Overlord,
        ]
    }

    /// The multiplier set for this difficulty.
    pub fn profile(self) -> DifficultyProfile {
        match self {
            Difficulty::Apprentice => DifficultyProfile {
                name: "Apprentice",
                blurb: "A forgiving delve. Weaker, rarer heroes; later sieges; richer income and a sturdier core. For learning the depths.",
                invader_stat_mult: 0.8,
                spawn_chance_mult: 0.85,
                siege_threshold_mult: 1.3,
                income_mult: 1.2,
                core_hp_mult: 1.3,
            },
            Difficulty::Keeper => DifficultyProfile {
                name: "Keeper",
                blurb: "The intended balance. Heroes, sieges, and income tuned as designed.",
                invader_stat_mult: 1.0,
                spawn_chance_mult: 1.0,
                siege_threshold_mult: 1.0,
                income_mult: 1.0,
                core_hp_mult: 1.0,
            },
            Difficulty::Overlord => DifficultyProfile {
                name: "Overlord",
                blurb: "A brutal reign. Stronger, more frequent heroes; early sieges; lean income and a fragile core. For those who have mastered the loop.",
                invader_stat_mult: 1.3,
                spawn_chance_mult: 1.15,
                siege_threshold_mult: 0.8,
                income_mult: 0.85,
                core_hp_mult: 0.8,
            },
        }
    }

    /// Short display name.
    pub fn name(self) -> &'static str {
        self.profile().name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeper_is_the_neutral_default() {
        assert_eq!(Difficulty::default(), Difficulty::Keeper);
        let p = Difficulty::Keeper.profile();
        assert_eq!(p.invader_stat_mult, 1.0);
        assert_eq!(p.spawn_chance_mult, 1.0);
        assert_eq!(p.siege_threshold_mult, 1.0);
        assert_eq!(p.income_mult, 1.0);
        assert_eq!(p.core_hp_mult, 1.0);
    }

    #[test]
    fn difficulty_ordering_is_monotonic() {
        let easy = Difficulty::Apprentice.profile();
        let hard = Difficulty::Overlord.profile();
        // Harder = tougher invaders, sooner sieges, leaner income, weaker core.
        assert!(easy.invader_stat_mult < hard.invader_stat_mult);
        assert!(easy.spawn_chance_mult < hard.spawn_chance_mult);
        assert!(easy.siege_threshold_mult > hard.siege_threshold_mult);
        assert!(easy.income_mult > hard.income_mult);
        assert!(easy.core_hp_mult > hard.core_hp_mult);
    }

    #[test]
    fn all_lists_every_difficulty_easiest_first() {
        assert_eq!(
            Difficulty::all(),
            [
                Difficulty::Apprentice,
                Difficulty::Keeper,
                Difficulty::Overlord
            ]
        );
    }
}
