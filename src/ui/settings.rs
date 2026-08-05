//! Persisted title-settings transitions, kept apart from the application loop.

use macroquad_toolkit::settings::GameSettings;

use super::title_screen::TitleSettingsAction;

pub fn apply_title_settings_action(
    settings: &mut GameSettings,
    action: TitleSettingsAction,
) -> (Option<String>, bool) {
    let notice = match action {
        TitleSettingsAction::ToggleFullscreen => {
            settings.toggle_fullscreen();
            Some(
                if settings.fullscreen {
                    "Fullscreen enabled."
                } else {
                    "Fullscreen disabled."
                }
                .to_string(),
            )
        }
        TitleSettingsAction::AdjustMasterVolume => {
            settings.master_volume = cycle_volume(settings.master_volume);
            Some(format!(
                "Master volume: {:.0}%.",
                settings.master_volume * 100.0
            ))
        }
        TitleSettingsAction::AdjustSfxVolume => {
            settings.sfx_volume = cycle_volume(settings.sfx_volume);
            Some(format!("SFX volume: {:.0}%.", settings.sfx_volume * 100.0))
        }
        TitleSettingsAction::AdjustMusicVolume => {
            settings.music_volume = cycle_volume(settings.music_volume);
            Some(format!(
                "Music volume: {:.0}%.",
                settings.music_volume * 100.0
            ))
        }
        TitleSettingsAction::AdjustUiScale(_) => {
            settings.ui_text_scale = if settings.ui_text_scale >= 1.5 {
                0.8
            } else {
                settings.ui_text_scale + 0.1
            };
            settings.sanitize();
            settings.apply_display();
            Some(format!(
                "UI scale set to {:.0}%.",
                settings.ui_text_scale * 100.0
            ))
        }
        TitleSettingsAction::ToggleReducedMotion => {
            settings.screen_shake = !settings.screen_shake;
            Some(
                if settings.screen_shake {
                    "Full motion enabled."
                } else {
                    "Reduced motion enabled."
                }
                .to_string(),
            )
        }
        TitleSettingsAction::AdjustAutosave(_) => {
            settings.autosave_interval = if settings.autosave_interval >= 120.0 {
                15.0
            } else {
                settings.autosave_interval + 15.0
            };
            settings.sanitize();
            Some(format!(
                "Autosave set to {:.0} seconds.",
                settings.autosave_interval
            ))
        }
        TitleSettingsAction::AdjustDefaultSpeed => {
            settings.default_speed = if settings.default_speed >= 4 {
                1
            } else {
                settings.default_speed + 1
            };
            Some(format!("New-run speed: {}x.", settings.default_speed))
        }
        TitleSettingsAction::Back => return (None, true),
        TitleSettingsAction::None => return (None, false),
    };
    let _ = settings.save("dungeon_core");
    (notice, false)
}

fn cycle_volume(volume: f32) -> f32 {
    if volume >= 0.99 {
        0.75
    } else if volume >= 0.74 {
        0.50
    } else if volume >= 0.49 {
        0.25
    } else {
        1.0
    }
}
