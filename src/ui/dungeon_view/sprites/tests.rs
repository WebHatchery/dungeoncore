use super::*;

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
