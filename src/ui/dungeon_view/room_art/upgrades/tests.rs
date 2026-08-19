use super::{upgrade_art_key, UpgradeArtKey};

#[test]
fn every_configured_upgrade_has_named_map_art() {
    for upgrade in crate::data::upgrades::get_all_upgrades() {
        assert!(
            upgrade_art_key(&upgrade.name).is_some(),
            "missing map sprite for upgrade {:?}",
            upgrade.name
        );
    }
}

#[test]
fn stone_walls_uses_a_physical_reinforcement_sprite() {
    assert_eq!(
        upgrade_art_key("Stone Walls"),
        Some(UpgradeArtKey::StoneWalls)
    );
    assert_ne!(upgrade_art_key("Stone Walls"), upgrade_art_key("Dark Aura"));
}
