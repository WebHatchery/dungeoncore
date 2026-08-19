use super::*;

#[test]
fn resource_panel_data_is_clamped_and_keeps_all_player_values_visible() {
    let mut state = GameState::new();
    state.mana = 500;
    state.max_mana = 200;
    state.mana_regen = 2.5;
    state.gold = 81;
    state.souls = 7;

    let data = resource_panel_data(&state);
    assert_eq!(data.mana_label, "500/200");
    assert_eq!(data.mana_fraction, 1.0);
    assert_eq!(data.regen_label, "(+2.5/tick)");
    assert!(data.gold_label.contains("81"));
    assert!(data.souls_label.contains("7"));
}
