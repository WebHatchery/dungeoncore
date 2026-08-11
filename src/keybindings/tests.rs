use super::*;

#[test]
fn assigning_an_existing_key_swaps_the_two_actions() {
    let mut bindings = KeyBindings::default();
    bindings.assign(BindingAction::Pause, KeyCode::Q);
    assert_eq!(bindings.label(BindingAction::Pause), "Q");
    assert_eq!(bindings.label(BindingAction::Smite), "Space");
}

#[test]
fn malformed_saved_keys_fall_back_to_their_action_defaults() {
    let mut bindings = KeyBindings::default();
    bindings.keys[BindingAction::Help as usize] = "not-a-key".to_string();
    bindings.sanitize();
    assert_eq!(bindings.label(BindingAction::Help), "H");
}
