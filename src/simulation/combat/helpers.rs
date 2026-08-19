//! Stat, trait, and targeting helpers shared across the combat submodules.

use crate::data::traits::get_trait;
use crate::game_state::{ActiveTrait, Adventurer, AdventurerParty, Monster, RoomBattleOrder};
use macroquad_toolkit::rng::SeededRng;

/// Whether a monster has a passive trait with the given effect type.
pub(super) fn has_passive(monster: &Monster, effect_type: &str) -> bool {
    monster.active_traits.iter().any(|t| {
        get_trait(&t.id).is_some_and(|d| d.trait_type == "Passive" && d.effect_type == effect_type)
    })
}

/// Summed value of a monster's passive traits with the given effect type.
pub(super) fn passive_value(monster: &Monster, effect_type: &str) -> f32 {
    monster
        .active_traits
        .iter()
        .filter_map(|t| get_trait(&t.id))
        .filter(|d| d.trait_type == "Passive" && d.effect_type == effect_type)
        .map(|d| d.value)
        .sum()
}

/// Index of the monster adventurers hit next: taunters first, then the front.
pub(super) fn target_monster_idx(monsters: &[Monster]) -> Option<usize> {
    monsters
        .iter()
        .position(|m| m.alive && has_passive(m, "Taunt"))
        .or_else(|| monsters.iter().position(|m| m.alive))
}

/// Build the tier-1 monster a slain splitter breaks into (half HP).
pub(super) fn split_spawn(rng: &mut SeededRng, parent_type: &str, floor: i32) -> Option<Monster> {
    let parent = crate::data::monsters::get_monster_template(parent_type)?;
    let candidates: Vec<_> = crate::data::monsters::get_monster_templates()
        .into_iter()
        .filter(|t| t.species == parent.species && t.tier == 1)
        .collect();
    let template = candidates
        .iter()
        .find(|t| t.element == parent.element)
        .or_else(|| candidates.first())?
        .clone();

    let scaled = crate::data::get_scaled_stats(
        crate::game_state::Stats {
            hp: template.hp,
            attack: template.attack,
            defense: template.defense,
        },
        floor,
        false,
    );

    Some(Monster {
        id: rng.next_u64(),
        type_name: template.name.clone(),
        hp: (scaled.hp / 2).max(1),
        max_hp: scaled.hp,
        alive: true,
        is_boss: false,
        fusion_rank: 1,
        scaled_stats: scaled,
        active_traits: template
            .traits
            .iter()
            .map(|trait_id| ActiveTrait {
                id: trait_id.clone(),
                name: get_trait(trait_id)
                    .map(|t| t.name)
                    .unwrap_or_else(|| trait_id.clone()),
                cooldown_timer: 0,
            })
            .collect(),
    })
}

/// Damage element of an adventurer, from their class. Empty = neutral.
pub(super) fn adventurer_element(class_name: &str) -> String {
    crate::data::adventurers::get_adventurer_class(class_name)
        .map(|c| c.element)
        .unwrap_or_default()
}

/// Element of a monster, from its template. Empty = neutral.
pub(super) fn monster_element(type_name: &str) -> String {
    crate::data::monsters::get_monster_template(type_name)
        .and_then(|t| t.element)
        .unwrap_or_default()
}

/// Stat multiplier a room attunement grants to a monster of `element`.
pub(super) fn attunement_mult(attunement: &Option<(String, f32)>, element: &str) -> f32 {
    match attunement {
        Some((attuned, mult)) if !element.is_empty() && attuned == element => *mult,
        _ => 1.0,
    }
}

/// Damage multiplier from a monster's defensive passive traits.
pub(super) fn monster_damage_taken_mult(monster: &Monster) -> f32 {
    let mut mult = 1.0;
    for trait_data in &monster.active_traits {
        if let Some(def) = get_trait(&trait_data.id) {
            if def.trait_type == "Passive"
                && def.applies_to == "OnDefense"
                && def.effect_type == "DamageReductionMult"
            {
                mult *= 1.0 - def.value;
            }
        }
    }
    mult
}

/// Attack multiplier from temporary room conditions on an adventurer.
pub(super) fn adventurer_attack_mult(adventurer: &Adventurer) -> f32 {
    let elemental = adventurer_element(&adventurer.class_name);
    let condition_mult = adventurer
        .conditions
        .iter()
        .filter(|condition| {
            condition.kind == "Weakened"
                || condition.kind == "Rallied"
                || (condition.kind == "ArcaneWeakened" && elemental == "Arcane")
        })
        .map(|condition| condition.multiplier)
        .product::<f32>()
        .max(0.0);
    let resolve_mult = 0.9 + adventurer.resolve.clamp(0, 100) as f32 / 500.0;
    condition_mult * adventurer.drive.attack_multiplier() * resolve_mult
}

/// Damage multiplier from temporary brittle conditions on an adventurer.
pub(super) fn adventurer_damage_taken_mult(adventurer: &Adventurer) -> f32 {
    let multiplier = adventurer
        .conditions
        .iter()
        .filter(|condition| condition.kind == "Brittle")
        .map(|condition| condition.multiplier)
        .product::<f32>();
    let condition_mult = if multiplier == 0.0 { 1.0 } else { multiplier };
    condition_mult * adventurer.drive.damage_taken_multiplier()
}

/// Effective attack including offensive passives and room reinforcement.
pub(super) fn monster_attack_value(
    monster: &Monster,
    allies_alive: usize,
    enemies_alive: usize,
    reinforcement_mult: f32,
) -> i32 {
    let mut attack = monster.scaled_stats.attack as f32;
    for trait_data in &monster.active_traits {
        if let Some(def) = get_trait(&trait_data.id) {
            if def.trait_type == "Passive"
                && def.applies_to == "OnAttack"
                && def.effect_type == "AttackBonus"
            {
                match def.scaling_type.as_str() {
                    "PerAlly" => attack += def.value * allies_alive.saturating_sub(1) as f32,
                    "PerEnemy" => attack += def.value * enemies_alive as f32,
                    _ => attack += def.value,
                }
            }
        }
    }
    (attack * reinforcement_mult).round() as i32
}

/// Index of a random living party member.
pub(super) fn random_alive_idx(rng: &mut SeededRng, party: &AdventurerParty) -> Option<usize> {
    let alive: Vec<usize> = party
        .members
        .iter()
        .enumerate()
        .filter(|(_, a)| a.alive)
        .map(|(i, _)| i)
        .collect();
    if alive.is_empty() {
        None
    } else {
        Some(alive[rng.below(alive.len())])
    }
}

/// Target selected by a room's standing order. Cull rooms concentrate every
/// strike on the most wounded living hero; other rooms preserve the existing
/// seeded spread across the party.
pub(super) fn target_adventurer_idx(
    rng: &mut SeededRng,
    party: &AdventurerParty,
    order: RoomBattleOrder,
) -> Option<usize> {
    if order == RoomBattleOrder::CullWounded {
        party
            .members
            .iter()
            .enumerate()
            .filter(|(_, hero)| hero.alive)
            .min_by_key(|(index, hero)| {
                let health_per_mille = hero.hp.max(0) as i64 * 1_000 / hero.max_hp.max(1) as i64;
                (health_per_mille, hero.hp, *index)
            })
            .map(|(index, _)| index)
    } else {
        random_alive_idx(rng, party)
    }
}
