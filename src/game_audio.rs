//! Small procedural feedback set for actions where visual confirmation alone
//! is easy to miss. Generated WAVs keep native and WebGL packaging asset-free.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use macroquad_toolkit::synth::{render_wav, SynthConfig, Voice, Wave};

use crate::game_state::SoundEvent;

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

pub struct GameAudio {
    ui: Option<Sound>,
    place: Option<Sound>,
    smite: Option<Sound>,
    combat: Option<Sound>,
    trap: Option<Sound>,
    death: Option<Sound>,
    income: Option<Sound>,
    threat: Option<Sound>,
    siege: Option<Sound>,
    core_damage: Option<Sound>,
    prestige: Option<Sound>,
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
            combat: load_effect(
                &[
                    Voice::tone(0.0, 0.06, 210.0, 0.16).wave(Wave::Noise),
                    Voice::tone(0.0, 0.05, 480.0, 0.12).wave(Wave::Square),
                ],
                4,
            )
            .await,
            trap: load_effect(
                &[
                    Voice::tone(0.0, 0.10, 720.0, 0.22)
                        .glide(280.0)
                        .wave(Wave::Triangle),
                    Voice::tone(0.02, 0.06, 1400.0, 0.10).wave(Wave::Noise),
                ],
                5,
            )
            .await,
            death: load_effect(
                &[Voice::tone(0.0, 0.16, 360.0, 0.24)
                    .glide(90.0)
                    .wave(Wave::Triangle)],
                6,
            )
            .await,
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
        }
    }

    pub fn play(&self, cue: SoundCue, volume: f32) {
        let sound = match cue {
            SoundCue::Ui => self.ui.as_ref(),
            SoundCue::Place => self.place.as_ref(),
            SoundCue::Smite => self.smite.as_ref(),
            SoundCue::Combat => self.combat.as_ref(),
            SoundCue::Trap => self.trap.as_ref(),
            SoundCue::Death => self.death.as_ref(),
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
