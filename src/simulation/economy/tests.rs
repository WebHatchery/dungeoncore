use super::*;

#[test]
fn channel_converts_gold_to_mana() {
    let mut s = GameState::new();
    s.gold = 250;
    s.mana = 0;
    s.max_mana = 500;
    channel_gold_to_mana(&mut s).unwrap();
    assert_eq!(s.gold, 150);
    assert_eq!(s.mana, GOLD_CHANNEL_MANA);
}

#[test]
fn channel_never_overfills_mana() {
    let mut s = GameState::new();
    s.gold = 500;
    s.max_mana = 100;
    s.mana = 90;
    channel_gold_to_mana(&mut s).unwrap();
    // Capped at max even though 20 was offered.
    assert_eq!(s.mana, 100);
    // Gold is still spent (the transaction is fixed-cost).
    assert_eq!(s.gold, 400);
}

#[test]
fn channel_blocked_when_poor_or_full() {
    let mut s = GameState::new();
    s.gold = 50;
    s.mana = 0;
    s.max_mana = 100;
    assert!(channel_gold_to_mana(&mut s).is_err());
    assert!(!can_channel_gold(&s));
    s.gold = 500;
    s.mana = 100;
    assert!(channel_gold_to_mana(&mut s).is_err());
    assert!(!can_channel_gold(&s));
}
