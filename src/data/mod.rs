pub mod adventurers;
pub mod constants;
pub mod difficulty;
pub mod elements;
pub mod equipment;
pub mod evolutions;
pub mod monsters;
pub mod traits;
pub mod upgrades;

pub use constants::*;
pub use upgrades::*;

// Every asset JSON is embedded at compile time and parsed lazily at runtime,
// so a malformed file only surfaces when its loader is first hit in-game.
// These tests force every loader (and the cross-file references) at test time.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monsters_json_parses() {
        assert!(!monsters::get_monster_templates().is_empty());
        assert!(!monsters::get_all_species().is_empty());
    }

    #[test]
    fn traits_json_parses() {
        assert!(!traits::get_all_traits().is_empty());
    }

    #[test]
    fn upgrades_json_parses() {
        assert!(!upgrades::get_all_upgrades().is_empty());
    }

    #[test]
    fn equipment_json_parses() {
        assert!(!equipment::get_all_equipment().is_empty());
    }

    #[test]
    fn adventurers_json_parses() {
        assert!(!adventurers::get_adventurer_classes().is_empty());
        assert!(!adventurers::get_adventurer_names().is_empty());
        assert!(!adventurers::get_victory_quotes().is_empty());
        assert!(!adventurers::get_entry_quotes().is_empty());
        assert!(!adventurers::get_exit_quotes().is_empty());
    }

    #[test]
    fn evolution_trees_json_parses() {
        assert!(!evolutions::get_evolution_trees().is_empty());
        assert!(!evolutions::get_starting_monsters().is_empty());
    }

    #[test]
    fn pooled_variant_thresholds_require_more_than_a_single_quick_raid() {
        for path in evolutions::get_evolution_trees().values().flatten() {
            assert!(
                path.experience_required >= 150,
                "{} -> {} still has a single-creature XP threshold of {}",
                path.from_monster,
                path.to_monster,
                path.experience_required
            );
        }
    }

    #[test]
    fn constants_json_parses() {
        assert!(constants::max_rooms_per_floor() > 0);
        assert!(constants::max_log_entries() > 0);
        assert!(constants::adventurer_spawn_chance() > 0.0);
        assert!(constants::get_floor_scaling(1).is_some());
        assert!(constants::get_deep_floor_scaling().adventurer_level_increase >= 0);
    }

    #[test]
    fn monsters_reference_valid_species_and_traits() {
        let species: Vec<String> = monsters::get_species_names();
        for template in monsters::get_monster_templates() {
            assert!(
                species.contains(&template.species),
                "monster '{}' references unknown species '{}'",
                template.name,
                template.species
            );
            for trait_id in &template.traits {
                assert!(
                    traits::get_trait(trait_id).is_some(),
                    "monster '{}' references unknown trait '{}'",
                    template.name,
                    trait_id
                );
            }
        }
    }

    #[test]
    fn evolution_trees_reference_valid_monsters() {
        let species = monsters::get_species_names();
        for (tree_species, paths) in evolutions::get_evolution_trees() {
            assert!(
                species.contains(&tree_species),
                "evolution tree keyed by unknown species '{}'",
                tree_species
            );
            for path in paths {
                assert!(
                    monsters::get_monster_template(&path.from_monster).is_some(),
                    "evolution path from unknown monster '{}'",
                    path.from_monster
                );
                assert!(
                    monsters::get_monster_template(&path.to_monster).is_some(),
                    "evolution path to unknown monster '{}'",
                    path.to_monster
                );
            }
        }
        for (species_name, starter) in evolutions::get_starting_monsters() {
            assert!(
                species.contains(&species_name),
                "starting monster keyed by unknown species '{}'",
                species_name
            );
            assert!(
                monsters::get_monster_template(&starter).is_some(),
                "starting monster '{}' has no template",
                starter
            );
        }
    }

    #[test]
    fn elements_json_parses() {
        assert!(!elements::get_all_elements().is_empty());
    }

    #[test]
    fn element_matrix_is_consistent() {
        let all = elements::get_all_elements();
        let ids: Vec<&str> = all.iter().map(|e| e.id.as_str()).collect();
        for element in &all {
            for target in &element.strong_against {
                assert!(
                    ids.contains(&target.as_str()),
                    "element '{}' strong against unknown element '{}'",
                    element.id,
                    target
                );
                assert_ne!(
                    &element.id, target,
                    "element '{}' cannot be strong against itself",
                    element.id
                );
                let other = all.iter().find(|e| &e.id == target).unwrap();
                assert!(
                    !other.strong_against.contains(&element.id),
                    "elements '{}' and '{}' are strong against each other",
                    element.id,
                    target
                );
            }
        }
    }

    #[test]
    fn element_multiplier_sanity() {
        assert_eq!(
            elements::element_multiplier("Fire", "Nature"),
            elements::STRONG_MULT
        );
        assert_eq!(
            elements::element_multiplier("Nature", "Fire"),
            elements::WEAK_MULT
        );
        assert_eq!(elements::element_multiplier("Body", "Fire"), 1.0);
        assert_eq!(elements::element_multiplier("", "Fire"), 1.0);
        assert_eq!(elements::element_multiplier("Nonsense", "Fire"), 1.0);
    }

    #[test]
    fn monsters_and_classes_use_known_elements() {
        for template in monsters::get_monster_templates() {
            if let Some(element) = &template.element {
                assert!(
                    elements::get_element(element).is_some(),
                    "monster '{}' has unknown element '{}'",
                    template.name,
                    element
                );
            }
        }
        for class in adventurers::get_adventurer_classes() {
            assert!(
                elements::get_element(&class.element).is_some(),
                "class '{}' has unknown element '{}'",
                class.name,
                class.element
            );
        }
    }

    #[test]
    fn core_powers_are_soul_priced() {
        for power in crate::simulation::endgame::CORE_POWERS.iter() {
            assert!(power.cost > 0, "core power '{}' must cost souls", power.id);
            assert!(!power.name.is_empty() && !power.description.is_empty());
        }
    }

    #[test]
    fn adventurer_races_present() {
        let races = adventurers::get_all_races();
        assert!(
            races.len() >= 4,
            "expected the four core races, found {}",
            races.len()
        );
        for want in ["Human", "Elf", "Dwarf", "Halfling"] {
            assert!(
                adventurers::get_race(want).is_some(),
                "missing race '{}'",
                want
            );
        }
    }

    #[test]
    fn upgrades_use_known_types_and_elements() {
        const KNOWN_TYPES: [&str; 5] = [
            "trap",
            "treasure",
            "reinforcement",
            "evolution",
            "attunement",
        ];
        for upgrade in upgrades::get_all_upgrades() {
            assert!(
                KNOWN_TYPES.contains(&upgrade.upgrade_type.as_str()),
                "upgrade '{}' has unknown type '{}' (parse_upgrade_type would silently fall back to Trap)",
                upgrade.name,
                upgrade.upgrade_type
            );
            if upgrade.upgrade_type == "attunement" {
                assert!(
                    upgrade.element.is_some(),
                    "attunement '{}' missing element",
                    upgrade.name
                );
            }
            // Any upgrade may be elemental (attunements, elemental traps);
            // whatever is set must exist in the matrix.
            if let Some(element) = &upgrade.element {
                assert!(
                    elements::get_element(element).is_some(),
                    "upgrade '{}' keyed to unknown element '{}'",
                    upgrade.name,
                    element
                );
            }
        }
    }

    #[test]
    fn trap_catalog_is_valid() {
        const KNOWN_KINDS: [&str; 7] = [
            "Damage",
            "Poison",
            "Burn",
            "Snare",
            "Alarm",
            "ManaSiphon",
            "GoldSteal",
        ];
        let traps: Vec<_> = upgrades::get_all_upgrades()
            .into_iter()
            .filter(|u| u.upgrade_type == "trap")
            .collect();
        assert!(
            traps.len() >= 8,
            "expected a full trap catalog, found {}",
            traps.len()
        );
        for trap in &traps {
            assert!(
                KNOWN_KINDS.contains(&trap.effect_kind.as_str()),
                "trap '{}' has unknown effect_kind '{}' (would fall back to legacy damage)",
                trap.name,
                trap.effect_kind
            );
            assert!(
                trap.multiplier > 0.0,
                "trap '{}' has no effect value",
                trap.name
            );
        }
        // The soul economy keeps a sink in the top trap tier.
        assert!(
            traps.iter().any(|t| t.souls_cost >= 2),
            "no soul-gated top-tier trap in the catalog"
        );
    }

    #[test]
    fn exactly_three_starter_species() {
        let starters: Vec<String> = monsters::get_all_species()
            .into_iter()
            .filter(|s| s.starter)
            .map(|s| s.name)
            .collect();
        assert_eq!(
            starters.len(),
            3,
            "expected 3 starter species, got {:?}",
            starters
        );
    }

    #[test]
    fn demons_cost_souls() {
        for template in monsters::get_monster_templates() {
            if template.species == "Demon" {
                assert!(
                    template.souls_cost > 0,
                    "demon '{}' should cost souls to summon",
                    template.name
                );
            } else {
                assert_eq!(
                    template.souls_cost, 0,
                    "non-demon '{}' should not cost souls",
                    template.name
                );
            }
        }
    }

    #[test]
    fn branching_evolutions_exist() {
        assert_eq!(
            evolutions::get_evolutions_for_monster("Imp").len(),
            2,
            "Imp should branch into Hellhound and Shadow Fiend"
        );
        assert_eq!(
            evolutions::get_evolutions_for_monster("Skeleton").len(),
            2,
            "Skeleton should branch into Vampire and Bone Mage"
        );
    }

    #[test]
    fn splitters_break_into_non_splitting_tier_ones() {
        // split_on_death must terminate: no tier-1 monster may carry it.
        for template in monsters::get_monster_templates() {
            if template.tier == 1 {
                assert!(
                    !template.traits.iter().any(|t| t == "split_on_death"),
                    "tier-1 '{}' with split_on_death would split forever",
                    template.name
                );
            }
        }
    }

    #[test]
    fn every_species_has_a_boss_unique() {
        for species in monsters::get_all_species() {
            let unique: Vec<_> = monsters::get_monster_templates()
                .into_iter()
                .filter(|t| t.species == species.name && t.boss_only)
                .collect();
            assert_eq!(
                unique.len(),
                1,
                "species '{}' should have exactly one boss unique, found {:?}",
                species.name,
                unique.iter().map(|t| &t.name).collect::<Vec<_>>()
            );
            assert_eq!(
                unique[0].tier, 4,
                "boss unique '{}' should be tier 4",
                unique[0].name
            );
        }
    }

    #[test]
    fn boss_uniques_are_reachable_by_evolution() {
        for template in monsters::get_monster_templates() {
            if template.boss_only {
                let reachable = evolutions::get_evolution_trees()
                    .values()
                    .flatten()
                    .any(|p| p.to_monster == template.name);
                assert!(
                    reachable,
                    "boss unique '{}' has no evolution path leading to it",
                    template.name
                );
            }
        }
    }

    #[test]
    fn every_species_is_summonable_after_unlock() {
        // unlock_species grants the tier-1 roster, falling back to the
        // starting_monsters map (e.g. Draconic only has the tier-3 Dragon).
        let starting = evolutions::get_starting_monsters();
        for species in monsters::get_all_species() {
            let has_tier_one =
                !monsters::get_starter_monsters_for_species(&species.name).is_empty();
            assert!(
                has_tier_one || starting.contains_key(&species.name),
                "species '{}' has neither tier-1 monsters nor a starting_monsters entry",
                species.name
            );
        }
    }
}
