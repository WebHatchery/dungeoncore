//! Player-facing descriptions for the inspector: what an upgrade will do, what
//! a trap does when it springs, and how far a monster line is from its next
//! variant. Text only — nothing here draws.

use macroquad::prelude::*;

use crate::data::evolutions::get_evolution_for_monster;
use crate::data::traits::get_trait;
use crate::data::upgrades::UpgradeTemplate;
use crate::game_state::{GameState, Monster, Room, RoomUpgrade, RoomUpgradeType};
use crate::ui::theme::*;

/// Human description of a trap's behavior from its effect kind and value.
fn trap_preview(effect_kind: &str, value: f32) -> String {
    match effect_kind {
        "Damage" => format!("{:.0} damage on trigger", value),
        "Poison" => format!("Poison: {:.0} dmg/tick", value),
        "Burn" => format!("Burn: {:.0} dmg/tick", value),
        "Snare" => format!("Holds party {:.0} ticks", value),
        "Alarm" => "Alerts defenders: +25% attack".to_string(),
        "ManaSiphon" => format!("Siphons {:.0} mana per trigger", value),
        "GoldSteal" => format!("Steals {:.0} gold per trigger", value),
        _ => format!("Trap damage x{:.2}", value),
    }
}

pub(super) fn upgrade_preview(upgrade: &UpgradeTemplate) -> String {
    match upgrade.upgrade_type.as_str() {
        "trap" => trap_preview(&upgrade.effect_kind, upgrade.multiplier),
        "treasure" => format!("Gold drops x{:.2}", upgrade.multiplier),
        "reinforcement" => format!("Monster survival x{:.2}", upgrade.multiplier),
        "evolution" => format!("Monster XP x{:.2}", upgrade.multiplier),
        "attunement" => format!(
            "{} monsters x{:.2}",
            upgrade.element.as_deref().unwrap_or("Attuned"),
            upgrade.multiplier
        ),
        _ => upgrade.effect.clone(),
    }
}

pub(super) fn room_upgrade_preview(upgrade: &RoomUpgrade) -> String {
    match &upgrade.upgrade_type {
        RoomUpgradeType::Trap => {
            let mut text = trap_preview(&upgrade.effect_kind, upgrade.multiplier);
            if upgrade.disarmed {
                text.push_str(" (disarmed)");
            }
            text
        }
        RoomUpgradeType::Treasure => {
            format!("{} Gold drops x{:.2}", upgrade.effect, upgrade.multiplier)
        }
        RoomUpgradeType::Reinforcement => {
            format!(
                "{} Monster survival x{:.2}",
                upgrade.effect, upgrade.multiplier
            )
        }
        RoomUpgradeType::Evolution => {
            format!("{} Monster XP x{:.2}", upgrade.effect, upgrade.multiplier)
        }
        RoomUpgradeType::Attunement => {
            format!(
                "{} {} monsters x{:.2}",
                upgrade.effect,
                upgrade.element.as_deref().unwrap_or("Attuned"),
                upgrade.multiplier
            )
        }
    }
}

/// What this defender's *line* is learning. Identical for every creature of the
/// same type — progress pools across the line, not the individual.
pub(super) fn monster_variant_status(
    state: &GameState,
    room: &Room,
    monster: &Monster,
) -> (String, Color) {
    let Some(path) = get_evolution_for_monster(&monster.type_name) else {
        return ("Final".to_string(), TEXT_DIM);
    };

    if state.unlocked_monsters.contains(&path.to_monster) {
        return (format!("{} unlocked", path.to_monster), EMERALD);
    }
    let pooled = state.type_experience(&monster.type_name);
    if pooled < path.experience_required {
        return (
            format!(
                "{}/{} XP -> {}",
                pooled, path.experience_required, path.to_monster
            ),
            MANA,
        );
    }
    if room.floor_number < path.conditions.min_floor {
        return (format!("floor {}", path.conditions.min_floor), WARNING);
    }
    (format!("Ready -> {}", path.to_monster), EMERALD)
}

pub(super) fn template_trait_summary(trait_ids: &[String]) -> String {
    if trait_ids.is_empty() {
        return "None".to_string();
    }

    trait_ids
        .iter()
        .take(3)
        .map(|trait_id| {
            get_trait(trait_id)
                .map(|trait_def| trait_def.name)
                .unwrap_or_else(|| trait_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// What this line unlocks next, and what it takes. The cost is paid in pooled
/// experience across every creature of the type, not by any one of them.
pub(super) fn template_variant_hint(state: &GameState, monster_name: &str) -> String {
    get_evolution_for_monster(monster_name)
        .map(|path| {
            format!(
                "Variant: {}/{} XP pooled, fielded on floor {} -> {}",
                state.type_experience(monster_name),
                path.experience_required,
                path.conditions.min_floor,
                path.to_monster
            )
        })
        .unwrap_or_else(|| "Variant: final form".to_string())
}
