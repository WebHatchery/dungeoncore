//! Transient floating-text feedback anchored to dungeon rooms — damage
//! numbers, ability call-outs, deaths, loot. Not persisted; cleared each
//! session and re-populated as combat/traps/abilities fire.
//!
//! Lifetime bookkeeping is delegated to [`macroquad_toolkit::timing::Timer`].
//! Screen position stays a UI concern (room tiles are laid out fresh every
//! frame), so effects only carry the room-anchoring data the UI needs to
//! place and stack them — see `ui::dungeon_view::room_art`.

use macroquad_toolkit::timing::Timer;

use super::GameState;

/// Seconds a floating effect stays visible before fully fading.
const EFFECT_TTL: f32 = 1.6;
const DUST_TTL: f32 = 0.82;
const SPARK_TTL: f32 = 0.30;

/// Kind of transient visual effect surfaced over a room
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectKind {
    Damage,
    Ability,
    MonsterDown,
    AdventurerDown,
    Loot,
    /// A central brawl cloud, spawned once per resolved combat tick.
    MeleeDust,
    /// A very short impact flash on the side that took damage.
    HitSpark,
}

/// Which side of the room a floating effect belongs over, so damage/deaths
/// rise above the units actually involved rather than all stacking centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EffectAnchor {
    Center,
    /// The defenders' (monster) side — left zone.
    Defenders,
    /// The invaders' (adventurer) side — right zone.
    Invaders,
}

/// A short-lived floating effect anchored to a room (not persisted)
#[derive(Clone, Debug)]
pub struct RoomEffect {
    pub floor: i32,
    pub room: usize,
    pub text: String,
    pub kind: EffectKind,
    pub anchor: EffectAnchor,
    timer: Timer,
}

impl RoomEffect {
    /// Fraction of life remaining: `1.0` fresh down to `0.0` expired. Drives
    /// the rise offset and fade-out alpha in the UI.
    pub fn life_fraction(&self) -> f32 {
        self.timer.fraction_remaining()
    }
}

impl GameState {
    /// Spawn a short-lived floating effect centred over a room.
    pub fn push_effect(
        &mut self,
        floor: i32,
        room: usize,
        text: impl Into<String>,
        kind: EffectKind,
    ) {
        self.push_effect_at(floor, room, text, kind, EffectAnchor::Center);
    }

    /// Spawn a floating effect over a specific side of a room, so damage and
    /// deaths appear above the units they concern.
    pub fn push_effect_at(
        &mut self,
        floor: i32,
        room: usize,
        text: impl Into<String>,
        kind: EffectKind,
        anchor: EffectAnchor,
    ) {
        self.effects.push(RoomEffect {
            floor,
            room,
            text: text.into(),
            kind,
            anchor,
            timer: Timer::new(effect_ttl(kind)),
        });
        if self.effects.len() > 48 {
            self.effects.remove(0);
        }
    }

    /// Age floating effects and drop expired ones.
    pub fn decay_effects(&mut self, dt: f32) {
        for effect in &mut self.effects {
            effect.timer.tick(dt);
        }
        self.effects.retain(|effect| !effect.timer.finished());
    }
}

fn effect_ttl(kind: EffectKind) -> f32 {
    match kind {
        EffectKind::MeleeDust => DUST_TTL,
        EffectKind::HitSpark => SPARK_TTL,
        _ => EFFECT_TTL,
    }
}
