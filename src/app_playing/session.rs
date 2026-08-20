use macroquad::prelude::get_time;

use crate::ui::{DrawerTab, UpgradeSection};

/// Mutable state owned by the playing screen rather than by the simulation.
/// Keeping these controls together prevents the application loop from passing
/// a long list of unrelated UI references into every frame.
pub struct PlayingSession {
    pub drawer_tab: DrawerTab,
    pub upgrade_section: UpgradeSection,
    pub drawer_open: bool,
    pub event_log_expanded: bool,
    pub species_scroll: f32,
    pub defender_scroll: f32,
    pub heroes_scroll: f32,
    pub show_codex: bool,
    pub show_controls: bool,
    pub codex_scroll: f32,
    pub show_core_tree: bool,
    pub show_milestones: bool,
    pub milestones_scroll: f32,
    pub timing: PlayingTiming,
}

impl PlayingSession {
    pub fn new() -> Self {
        Self {
            drawer_tab: DrawerTab::Monsters,
            upgrade_section: UpgradeSection::Traps,
            drawer_open: false,
            event_log_expanded: false,
            species_scroll: 0.0,
            defender_scroll: 0.0,
            heroes_scroll: 0.0,
            show_codex: false,
            show_controls: false,
            codex_scroll: 0.0,
            show_core_tree: false,
            show_milestones: false,
            milestones_scroll: 0.0,
            timing: PlayingTiming::now(),
        }
    }
}

impl Default for PlayingSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Wall-clock checkpoints used by the playing screen's periodic simulation.
/// They are session state, not part of a saved dungeon.
pub struct PlayingTiming {
    pub last_time_advance: f64,
    pub last_adventure_tick: f64,
    pub last_save: f64,
}

impl PlayingTiming {
    fn now() -> Self {
        let now = get_time();
        Self {
            last_time_advance: now,
            last_adventure_tick: now,
            last_save: now,
        }
    }

    pub fn reset(&mut self) {
        let now = get_time();
        self.last_time_advance = now;
        self.last_adventure_tick = now;
        self.last_save = now;
    }
}

/// Runtime settings that affect one playing frame but are not mutable screen
/// state. Capture scenes use the same shape as interactive frames so gameplay
/// orchestration cannot accidentally diverge between the two paths.
#[derive(Clone, Copy)]
pub struct PlayingFrameSettings<'a> {
    pub simulate: bool,
    pub autosave_interval: f64,
    pub save_slot: &'a str,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl<'a> PlayingFrameSettings<'a> {
    pub fn new(
        simulate: bool,
        autosave_interval: f64,
        save_slot: &'a str,
        sfx_volume: f32,
        music_volume: f32,
    ) -> Self {
        Self {
            simulate,
            autosave_interval,
            save_slot,
            sfx_volume,
            music_volume,
        }
    }
}
