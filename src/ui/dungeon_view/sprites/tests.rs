use super::*;
use std::collections::HashSet;

#[test]
fn every_declared_unit_has_a_sprite_definition() {
    for monster in crate::data::monsters::get_monster_templates() {
        assert!(
            monster_frame(&monster.name).is_some(),
            "missing monster mapping: {}",
            monster.name
        );
    }
    for class in crate::data::adventurers::get_adventurer_classes() {
        assert!(
            adventurer_frame(&class.name).is_some(),
            "missing class mapping: {}",
            class.name
        );
    }
}

#[test]
fn every_class_has_an_animated_pose_row() {
    for class in crate::data::adventurers::get_adventurer_classes() {
        assert!(
            animated_adventurer_frame(&class.name).is_some()
                || animated_late_adventurer_frame(&class.name).is_some(),
            "missing animated adventurer pose row: {}",
            class.name
        );
    }
}

#[test]
fn every_monster_has_an_animated_pose_row() {
    for monster in crate::data::monsters::get_monster_templates() {
        assert!(
            animated_monster_frame(&monster.name).is_some()
                || animated_full_monster_frame(&monster.name).is_some(),
            "missing animated monster pose row: {}",
            monster.name
        );
    }
}

#[test]
fn every_placeable_monster_has_a_unique_sprite_identity() {
    let mut identities = HashSet::new();
    for monster in crate::data::monsters::get_monster_templates() {
        let style = monster_sprite_style(&monster.name)
            .unwrap_or_else(|| panic!("missing creature sprite identity: {}", monster.name));
        assert!(
            identities.insert(style.identity),
            "duplicate creature sprite identity: {}",
            monster.name
        );
    }
}

#[test]
fn giant_rat_uses_its_own_sprite_instead_of_the_beast_row() {
    let wolf = monster_sprite_style("Wolf").expect("wolf sprite identity");
    let rat = monster_sprite_style("Giant Rat").expect("rat sprite identity");

    assert_ne!(wolf.identity, rat.identity);
    assert_eq!(rat.identity.source, MonsterSpriteSource::GiantRat);
}
