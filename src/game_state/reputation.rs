//! The dungeon's standing beyond its walls. Threat measures the realm's will
//! to destroy the Core; reputation measures whether valuable parties seek it.

/// Reputation is bounded so a long campaign cannot make visitor quality run
/// away forever. Older saves deliberately begin at the neutral score.
pub const REPUTATION_MIN: i32 = -100;
pub const REPUTATION_MAX: i32 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReputationBand {
    Shunned,
    Unknown,
    Noted,
    Renowned,
}

impl ReputationBand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Shunned => "Shunned",
            Self::Unknown => "Unknown",
            Self::Noted => "Noted",
            Self::Renowned => "Renowned",
        }
    }

    /// The next score at which the realm changes its view, if any.
    pub fn next_threshold(self) -> Option<i32> {
        match self {
            Self::Shunned => Some(-24),
            Self::Unknown => Some(25),
            Self::Noted => Some(60),
            Self::Renowned => None,
        }
    }
}

/// The deterministic spawn adjustments attached to a reputation band. Random
/// member selection happens later; this is deliberately pure and testable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisitorQuality {
    pub level_bonus: i32,
    pub spawn_chance_mult: f32,
    pub returning_slot_stride: usize,
}

pub fn band(score: i32) -> ReputationBand {
    match score {
        ..=-25 => ReputationBand::Shunned,
        -24..=24 => ReputationBand::Unknown,
        25..=59 => ReputationBand::Noted,
        _ => ReputationBand::Renowned,
    }
}

pub fn visitor_quality(score: i32) -> VisitorQuality {
    match band(score) {
        ReputationBand::Shunned => VisitorQuality {
            level_bonus: -1,
            spawn_chance_mult: 0.70,
            returning_slot_stride: 3,
        },
        ReputationBand::Unknown => VisitorQuality {
            level_bonus: 0,
            spawn_chance_mult: 1.0,
            returning_slot_stride: 2,
        },
        ReputationBand::Noted => VisitorQuality {
            level_bonus: 1,
            spawn_chance_mult: 1.10,
            returning_slot_stride: 2,
        },
        ReputationBand::Renowned => VisitorQuality {
            level_bonus: 2,
            spawn_chance_mult: 1.20,
            returning_slot_stride: 1,
        },
    }
}

/// Score a concluded raid. Escapes advertise the dungeon, especially after a
/// deep, lucrative delve with returning witnesses; a shallow wipe leaves it
/// looking like an uninteresting deathtrap.
pub fn raid_change(floor: i32, survivors: i32, loot: i32, returning_survivors: i32) -> i32 {
    if survivors == 0 {
        return if floor <= 1 { -12 } else { -3 };
    }

    let depth_bonus = (floor - 1).max(0) * 4;
    let loot_bonus = if loot > 0 { 3 } else { 0 };
    let returning_bonus = returning_survivors.max(0) * 2;
    (4 + depth_bonus + loot_bonus + returning_bonus).min(20)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raid_results_reward_deep_escapes_and_penalize_shallow_wipes() {
        assert!(raid_change(3, 2, 80, 1) > 0);
        assert!(raid_change(1, 0, 0, 0) < 0);
    }
}
