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
const EVENT_TTL: f32 = 1.25;

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
    /// Lingering venom bubbles around the afflicted invader side.
    PoisonCloud,
    /// A warning sigil pulsing over the Core as the realm's siege begins.
    SiegeArrival,
    /// A burst of Core shards after the siege is broken and prestige earned.
    Prestige,
}

/// A semantic, one-shot sound request emitted by the authoritative simulation.
/// It is transient like visual effects: audio never affects saves or outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEvent {
    Combat,
    Trap,
    Death,
    Income,
    Threat,
    Siege,
    CoreDamage,
    Prestige,
    /// A short renderer-only sting keyed to an elemental combat or trap hit.
    ElementalHit(ElementSound),
}

/// Compact, serialisation-free element identity for procedural sound selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementSound {
    Fire,
    Water,
    Nature,
    Earth,
    Air,
    Spirit,
    Death,
    Arcane,
    Body,
}

impl ElementSound {
    pub fn from_id(element: &str) -> Option<Self> {
        match element {
            "Fire" => Some(Self::Fire),
            "Water" => Some(Self::Water),
            "Nature" => Some(Self::Nature),
            "Earth" => Some(Self::Earth),
            "Air" => Some(Self::Air),
            "Spirit" => Some(Self::Spirit),
            "Death" => Some(Self::Death),
            "Arcane" => Some(Self::Arcane),
            "Body" => Some(Self::Body),
            _ => None,
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Fire => 0,
            Self::Water => 1,
            Self::Nature => 2,
            Self::Earth => 3,
            Self::Air => 4,
            Self::Spirit => 5,
            Self::Death => 6,
            Self::Arcane => 7,
            Self::Body => 8,
        }
    }
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
    /// Optional casualty identity for renderer-only death-pose playback.
    pub visual_unit: Option<String>,
    /// Optional damage element used only to choose an impact treatment.
    pub visual_element: Option<String>,
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
    /// Queue a cosmetic sound for the interactive renderer to consume once.
    pub fn queue_sound(&mut self, event: SoundEvent) {
        self.sound_events.push(event);
        // Avoid an extreme combat tick scheduling an unbounded chorus.
        if self.sound_events.len() > 12 {
            self.sound_events.remove(0);
        }
    }

    /// Drain the cosmetic queue. Capture rendering deliberately leaves it alone.
    pub fn take_sound_events(&mut self) -> Vec<SoundEvent> {
        std::mem::take(&mut self.sound_events)
    }

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
            visual_unit: None,
            visual_element: None,
            timer: Timer::new(effect_ttl(kind)),
        });
        if self.effects.len() > 48 {
            self.effects.remove(0);
        }
    }

    /// Spawn an effect carrying a short-lived unit identity for cosmetic art.
    pub fn push_unit_effect_at(
        &mut self,
        floor: i32,
        room: usize,
        text: impl Into<String>,
        kind: EffectKind,
        anchor: EffectAnchor,
        visual_unit: impl Into<String>,
    ) {
        self.effects.push(RoomEffect {
            floor,
            room,
            text: text.into(),
            kind,
            anchor,
            visual_unit: Some(visual_unit.into()),
            visual_element: None,
            timer: Timer::new(effect_ttl(kind)),
        });
        if self.effects.len() > 48 {
            self.effects.remove(0);
        }
    }

    /// Spawn an impact effect with an element-specific cosmetic treatment.
    /// The element is never read by simulation logic or persisted in saves.
    pub fn push_element_effect_at(
        &mut self,
        floor: i32,
        room: usize,
        text: impl Into<String>,
        kind: EffectKind,
        anchor: EffectAnchor,
        element: impl Into<String>,
    ) {
        self.effects.push(RoomEffect {
            floor,
            room,
            text: text.into(),
            kind,
            anchor,
            visual_unit: None,
            visual_element: Some(element.into()),
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
        EffectKind::PoisonCloud | EffectKind::SiegeArrival | EffectKind::Prestige => EVENT_TTL,
        _ => EFFECT_TTL,
    }
}
