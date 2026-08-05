//! Small procedural feedback set for actions where visual confirmation alone
//! is easy to miss. Generated WAVs keep native and WebGL packaging asset-free.

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use macroquad_toolkit::synth::{render_wav, SynthConfig, Voice, Wave};
use std::cell::Cell;

use crate::game_state::{GameState, SoundEvent};

#[derive(Clone, Copy)]
pub enum SoundCue {
    Ui,
    Place,
    Smite,
    Combat,
    Trap,
    Death,
    Income,
    Threat,
    Siege,
    CoreDamage,
    Prestige,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MusicLayer {
    Build,
    Raid,
    Siege,
}

pub struct GameAudio {
    ui: Option<Sound>,
    place: Option<Sound>,
    smite: Option<Sound>,
    combat: Vec<Sound>,
    trap: Vec<Sound>,
    death: Vec<Sound>,
    combat_variant: Cell<usize>,
    trap_variant: Cell<usize>,
    death_variant: Cell<usize>,
    income: Option<Sound>,
    threat: Option<Sound>,
    siege: Option<Sound>,
    core_damage: Option<Sound>,
    prestige: Option<Sound>,
    build_music: Option<Sound>,
    raid_music: Option<Sound>,
    siege_music: Option<Sound>,
    active_music: Cell<Option<MusicLayer>>,
}

impl GameAudio {
    pub async fn new() -> Self {
        Self {
            ui: load_effect(
                &[Voice::tone(0.0, 0.07, 540.0, 0.28).wave(Wave::Triangle)],
                1,
            )
            .await,
            place: load_effect(
                &[
                    Voice::tone(0.0, 0.12, 280.0, 0.34)
                        .glide(420.0)
                        .wave(Wave::Triangle),
                    Voice::tone(0.0, 0.05, 900.0, 0.10).wave(Wave::Noise),
                ],
                2,
            )
            .await,
            smite: load_effect(
                &[
                    Voice::tone(0.0, 0.18, 180.0, 0.30)
                        .glide(1080.0)
                        .wave(Wave::Square),
                    Voice::tone(0.02, 0.10, 1760.0, 0.16).wave(Wave::Triangle),
                ],
                3,
            )
            .await,
            combat: load_effect_variants(
                &[
                    Voice::tone(0.0, 0.06, 210.0, 0.16).wave(Wave::Noise),
                    Voice::tone(0.0, 0.05, 480.0, 0.12).wave(Wave::Square),
                ],
                4..=6,
            )
            .await,
            trap: load_effect_variants(
                &[
                    Voice::tone(0.0, 0.10, 720.0, 0.22)
                        .glide(280.0)
                        .wave(Wave::Triangle),
                    Voice::tone(0.02, 0.06, 1400.0, 0.10).wave(Wave::Noise),
                ],
                7..=9,
            )
            .await,
            death: load_effect_variants(
                &[Voice::tone(0.0, 0.16, 360.0, 0.24)
                    .glide(90.0)
                    .wave(Wave::Triangle)],
                10..=12,
            )
            .await,
            combat_variant: Cell::new(0),
            trap_variant: Cell::new(0),
            death_variant: Cell::new(0),
            income: load_effect(
                &[Voice::tone(0.0, 0.11, 520.0, 0.16)
                    .glide(790.0)
                    .wave(Wave::Triangle)],
                7,
            )
            .await,
            threat: load_effect(
                &[Voice::tone(0.0, 0.18, 220.0, 0.22)
                    .glide(145.0)
                    .wave(Wave::Square)],
                8,
            )
            .await,
            siege: load_effect(
                &[
                    Voice::tone(0.0, 0.30, 105.0, 0.30).wave(Wave::Square),
                    Voice::tone(0.04, 0.24, 208.0, 0.20).wave(Wave::Triangle),
                ],
                9,
            )
            .await,
            core_damage: load_effect(
                &[
                    Voice::tone(0.0, 0.14, 130.0, 0.26).wave(Wave::Noise),
                    Voice::tone(0.02, 0.10, 92.0, 0.20).wave(Wave::Square),
                ],
                10,
            )
            .await,
            prestige: load_effect(
                &[
                    Voice::tone(0.0, 0.28, 390.0, 0.18)
                        .glide(1170.0)
                        .wave(Wave::Triangle),
                    Voice::tone(0.10, 0.18, 780.0, 0.14)
                        .glide(1560.0)
                        .wave(Wave::Triangle),
                ],
                11,
            )
            .await,
            build_music: load_effect(
                &[
                    Voice::tone(0.0, 3.6, 146.8, 0.10).wave(Wave::Triangle),
                    Voice::tone(0.0, 3.6, 220.0, 0.05).wave(Wave::Triangle),
                ],
                31,
            )
            .await,
            raid_music: load_effect(
                &[
                    Voice::tone(0.0, 2.4, 98.0, 0.10).wave(Wave::Square),
                    Voice::tone(0.0, 2.4, 147.0, 0.07).wave(Wave::Triangle),
                    Voice::tone(0.0, 0.18, 720.0, 0.05).wave(Wave::Noise),
                ],
                32,
            )
            .await,
            siege_music: load_effect(
                &[
                    Voice::tone(0.0, 2.0, 65.4, 0.14).wave(Wave::Square),
                    Voice::tone(0.0, 2.0, 98.0, 0.09).wave(Wave::Triangle),
                    Voice::tone(0.0, 0.24, 180.0, 0.09).wave(Wave::Noise),
                ],
                33,
            )
            .await,
            active_music: Cell::new(None),
        }
    }

    /// Select and loop the appropriate renderer-owned music layer. Nothing in
    /// this method feeds back into the simulation or its saved state.
    pub fn update_music(&self, state: &GameState, volume: f32) {
        let wanted = if state.game_over {
            None
        } else if state.adventurer_parties.iter().any(|party| party.sieging) {
            Some(MusicLayer::Siege)
        } else if state.adventurer_parties.is_empty() {
            Some(MusicLayer::Build)
        } else {
            Some(MusicLayer::Raid)
        };
        if self.active_music.get() != wanted {
            for sound in [&self.build_music, &self.raid_music, &self.siege_music]
                .into_iter()
                .flatten()
            {
                stop_sound(sound);
            }
            if let Some(layer) = wanted {
                if let Some(sound) = self.music_sound(layer) {
                    play_sound(
                        sound,
                        PlaySoundParams {
                            looped: true,
                            volume: volume.clamp(0.0, 1.0),
                        },
                    );
                }
            }
            self.active_music.set(wanted);
        } else if let Some(layer) = wanted {
            if let Some(sound) = self.music_sound(layer) {
                set_sound_volume(sound, volume.clamp(0.0, 1.0));
            }
        }
    }

    fn music_sound(&self, layer: MusicLayer) -> Option<&Sound> {
        match layer {
            MusicLayer::Build => self.build_music.as_ref(),
            MusicLayer::Raid => self.raid_music.as_ref(),
            MusicLayer::Siege => self.siege_music.as_ref(),
        }
    }

    pub fn play(&self, cue: SoundCue, volume: f32) {
        let sound = match cue {
            SoundCue::Ui => self.ui.as_ref(),
            SoundCue::Place => self.place.as_ref(),
            SoundCue::Smite => self.smite.as_ref(),
            SoundCue::Combat => Self::next_variant(&self.combat, &self.combat_variant),
            SoundCue::Trap => Self::next_variant(&self.trap, &self.trap_variant),
            SoundCue::Death => Self::next_variant(&self.death, &self.death_variant),
            SoundCue::Income => self.income.as_ref(),
            SoundCue::Threat => self.threat.as_ref(),
            SoundCue::Siege => self.siege.as_ref(),
            SoundCue::CoreDamage => self.core_damage.as_ref(),
            SoundCue::Prestige => self.prestige.as_ref(),
        };
        if let Some(sound) = sound {
            play_sound(
                &sound,
                PlaySoundParams {
                    looped: false,
                    volume: volume.clamp(0.0, 1.0),
                },
            );
        }
    }

    /// Rotate only cosmetic samples. The cursor deliberately lives outside the
    /// simulation, so save/load and capture frames cannot change game RNG.
    fn next_variant<'a>(sounds: &'a [Sound], cursor: &Cell<usize>) -> Option<&'a Sound> {
        let index = cursor.get();
        let sound = sounds.get(index % sounds.len().max(1));
        if sound.is_some() {
            cursor.set(index.wrapping_add(1));
        }
        sound
    }
}

impl From<SoundEvent> for SoundCue {
    fn from(event: SoundEvent) -> Self {
        match event {
            SoundEvent::Combat => Self::Combat,
            SoundEvent::Trap => Self::Trap,
            SoundEvent::Death => Self::Death,
            SoundEvent::Income => Self::Income,
            SoundEvent::Threat => Self::Threat,
            SoundEvent::Siege => Self::Siege,
            SoundEvent::CoreDamage => Self::CoreDamage,
            SoundEvent::Prestige => Self::Prestige,
        }
    }
}

async fn load_effect(voices: &[Voice], seed: u64) -> Option<Sound> {
    let bytes = render_wav(voices, &SynthConfig::default(), seed);
    load_sound_from_bytes(&bytes).await.ok()
}

async fn load_effect_variants(
    voices: &[Voice],
    seeds: std::ops::RangeInclusive<u64>,
) -> Vec<Sound> {
    let mut effects = Vec::new();
    for seed in seeds {
        if let Some(effect) = load_effect(voices, seed).await {
            effects.push(effect);
        }
    }
    effects
}
