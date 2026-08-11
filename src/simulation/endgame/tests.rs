use super::*;

#[test]
fn siege_musters_at_peak_threat() {
    let mut s = GameState::new();
    s.total_deaths = 100; // tier 4
    assert_eq!(s.threat_tier(), 4);
    maybe_launch_siege(&mut s);
    assert!(s.siege_active);
    assert_eq!(s.adventurer_parties.len(), 1);
    assert!(s.adventurer_parties[0].sieging);
    assert!(s.take_sound_events().contains(&SoundEvent::Siege));
    // A second call must not stack another siege.
    maybe_launch_siege(&mut s);
    assert_eq!(s.adventurer_parties.len(), 1);
}

#[test]
fn repel_grants_prestige_and_resets_threat() {
    let mut s = GameState::new();
    s.total_deaths = 100;
    s.threat_warned = 4;
    s.siege_active = true;
    let hp_before = s.core_max_hp;
    repel_siege(&mut s);
    assert_eq!(s.prestige, 1);
    assert!(!s.siege_active);
    assert_eq!(s.total_deaths, 0);
    assert!(s.core_max_hp > hp_before);
    assert!(s.take_sound_events().contains(&SoundEvent::Prestige));
}

#[test]
fn buying_bulwark_raises_core_hp() {
    let mut s = GameState::new();
    s.souls = 10;
    let hp_before = s.core_max_hp;
    buy_core_power(&mut s, "bulwark_core").unwrap();
    assert!(s.core_max_hp > hp_before);
    assert!(s.has_core_power("bulwark_core"));
    // Can't buy twice.
    assert!(buy_core_power(&mut s, "bulwark_core").is_err());
}

#[test]
fn tier_two_power_gated_behind_prerequisite() {
    let mut s = GameState::new();
    s.souls = 100;
    // Aquifer needs Wellspring, which needs Deep Roots.
    assert!(buy_core_power(&mut s, "aquifer").is_err());
    buy_core_power(&mut s, "deep_roots").unwrap();
    assert!(buy_core_power(&mut s, "aquifer").is_err());
    buy_core_power(&mut s, "wellspring").unwrap();
    assert!(buy_core_power(&mut s, "aquifer").is_ok());
}

#[test]
fn regen_bonus_sums_across_owned_powers() {
    let mut s = GameState::new();
    s.souls = 100;
    assert_eq!(core_mana_regen_bonus(&s), 0.0);
    buy_core_power(&mut s, "deep_roots").unwrap();
    buy_core_power(&mut s, "wellspring").unwrap();
    assert_eq!(core_mana_regen_bonus(&s), 2.0);
}

#[test]
fn smite_bonuses_stack_and_max_mana_bakes_in() {
    let mut s = GameState::new();
    s.souls = 100;
    buy_core_power(&mut s, "dread_aura").unwrap();
    buy_core_power(&mut s, "searing_smite").unwrap();
    buy_core_power(&mut s, "cataclysm").unwrap();
    assert_eq!(core_smite_damage_bonus(&s), 65);
    // Dread stacks: dread_aura only so far.
    assert_eq!(core_dread_bonus(&s), 1);
    // MaxMana effect is baked in at purchase time.
    let mana_before = s.max_mana;
    buy_core_power(&mut s, "deep_roots").unwrap();
    buy_core_power(&mut s, "mana_font").unwrap();
    assert_eq!(s.max_mana, mana_before + 150);
}

#[test]
fn tree_prerequisites_reference_real_shallower_nodes() {
    for power in CORE_POWERS.iter() {
        for req in power.requires {
            let dep = core_power(req)
                .unwrap_or_else(|| panic!("{} requires unknown '{}'", power.id, req));
            assert!(
                dep.tier < power.tier,
                "{} (tier {}) must require a shallower node, not {} (tier {})",
                power.id,
                power.tier,
                dep.id,
                dep.tier
            );
        }
    }
}
