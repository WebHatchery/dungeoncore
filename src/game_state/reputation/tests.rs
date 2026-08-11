use super::*;

#[test]
fn raid_results_reward_deep_escapes_and_penalize_shallow_wipes() {
    assert!(raid_change(3, 2, 80, 1) > 0);
    assert!(raid_change(1, 0, 0, 0) < 0);
}
