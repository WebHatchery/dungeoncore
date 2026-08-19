use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const RUNTIME_ASSETS: [&str; 7] = [
    "assets/title_screen.png",
    "assets/sprites/dungeon_adventurers_animated.png",
    "assets/sprites/dungeon_monsters_animated.png",
    "assets/sprites/dungeon_monsters_full_animated.png",
    "assets/sprites/monster_giant_rat.png",
    "assets/sprites/dungeon_units.png",
    "assets/sprites/dungeon_units_animated.png",
];

#[test]
fn asset_registry_matches_external_runtime_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let json = fs::read_to_string(root.join("asset_registry.json"))
        .expect("asset_registry.json must be readable");
    let registry: Value = serde_json::from_str(&json).expect("asset registry must be valid JSON");
    assert_eq!(registry["version"], 1);

    let registered: BTreeSet<&str> = registry["assets"]
        .as_array()
        .expect("asset registry needs an assets array")
        .iter()
        .map(|entry| entry.as_str().expect("asset paths must be strings"))
        .collect();
    let expected: BTreeSet<&str> = RUNTIME_ASSETS.into_iter().collect();
    assert_eq!(registered, expected);

    for relative in registered {
        assert!(
            root.join(relative).is_file(),
            "registered runtime asset is missing: {relative}"
        );
    }
}
