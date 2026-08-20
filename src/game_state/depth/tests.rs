use super::*;

#[test]
fn depth_layers_cycle_every_four_floors() {
    assert_eq!(DepthLayer::for_floor(1), DepthLayer::Threshold);
    assert_eq!(DepthLayer::for_floor(2), DepthLayer::Hunt);
    assert_eq!(DepthLayer::for_floor(3), DepthLayer::Gauntlet);
    assert_eq!(DepthLayer::for_floor(4), DepthLayer::Apex);
    assert_eq!(DepthLayer::for_floor(8), DepthLayer::Apex);
}

#[test]
fn relics_follow_the_authored_strata() {
    assert_eq!(relic_for_floor(1).id, "rootbound_sigil");
    assert_eq!(relic_for_floor(5).id, "cinder_crown");
    assert_eq!(relic_for_floor(9).id, "tide_lens");
    assert_eq!(relic_for_floor(13).id, "prism_heart");
    assert_eq!(relic_for_floor(17).id, "ossuary_key");
}

#[test]
fn apex_relics_change_the_long_run_and_only_claim_once() {
    let mut state = GameState::new();
    let max_mana = state.max_mana;
    let relic = state.claim_depth_relic(1).expect("first stratum relic");
    assert_eq!(relic.id, "rootbound_sigil");
    assert_eq!(state.max_mana, max_mana + 50);
    assert!(state.claim_depth_relic(4).is_none());
    assert!(state.depth_pressure(4) > state.depth_pressure(1));
}
