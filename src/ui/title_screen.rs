use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect};
use macroquad_toolkit::settings::GameSettings;

use crate::data::difficulty::Difficulty;

use super::theme::*;
use macroquad_toolkit::colors::with_alpha;

pub const TITLE_BACKGROUND_KEY: &str = "title_background";
pub const TITLE_BACKGROUND_PATH: &str = "assets/title_screen.png";
const BUILD_STAMP: &str = concat!("Dungeon Core v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleAction {
    None,
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleSettingsAction {
    None,
    ToggleFullscreen,
    AdjustMasterVolume,
    AdjustSfxVolume,
    AdjustMusicVolume,
    AdjustUiScale(i8),
    ToggleReducedMotion,
    AdjustAutosave(i8),
    AdjustDefaultSpeed,
    OpenKeybindings,
    Back,
}

pub fn draw_title_screen(
    assets: &AssetManager,
    has_save: bool,
    notice: Option<&str>,
) -> TitleAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);

    let panel_w = sw.min(1280.0) * 0.29;
    let panel_w = panel_w.clamp(280.0, 380.0);
    let button_h = 48.0;
    let gap = 12.0;
    let panel_h = button_h * 4.0 + gap * 3.0 + 88.0;
    let x = (sw * 0.075).clamp(32.0, 118.0);
    let y = (sh - panel_h - 48.0).max(sh * 0.42);
    let panel = Rect::new(x, y, panel_w, panel_h.min(sh - y - 28.0));

    draw_title_panel(panel);
    draw_text_fit(
        "COMMAND THE CORE",
        panel.x + 26.0,
        panel.y + 31.0,
        panel.w - 52.0,
        15.0,
        with_alpha(TEXT, 0.92),
    );
    let btn_x = panel.x + 22.0;
    let btn_w = panel.w - 44.0;
    let mut btn_y = panel.y + 82.0;

    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "New Game",
        true,
        ButtonTone::Arcane,
    ) {
        return TitleAction::NewGame;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Load Game",
        has_save,
        ButtonTone::Ghost,
    ) {
        return TitleAction::LoadGame;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Settings",
        true,
        ButtonTone::Ghost,
    ) {
        return TitleAction::Settings;
    }

    btn_y += button_h + gap;
    if draw_title_button(
        Rect::new(btn_x, btn_y, btn_w, button_h),
        "Exit",
        true,
        ButtonTone::Danger,
    ) {
        return TitleAction::Exit;
    }

    if !has_save {
        draw_text_fit(
            "No saved dungeon found.",
            btn_x,
            btn_y + button_h + 24.0,
            btn_w,
            12.0,
            TEXT_DIM,
        );
    }

    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }
    draw_text_fit(BUILD_STAMP, 18.0, sh - 16.0, 180.0, 10.0, TEXT_DIM);

    TitleAction::None
}

/// Action from the new-game difficulty picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewGameSetupAction {
    None,
    Back,
    Start(Difficulty, Option<u64>),
}

/// Difficulty-selection screen shown when the player starts a new game.
pub fn draw_new_game_setup(
    assets: &AssetManager,
    seed_input: &mut String,
    notice: Option<&str>,
) -> NewGameSetupAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);

    let panel_w = (sw * 0.5).clamp(460.0, 720.0);
    let panel_h = (sh * 0.72).clamp(360.0, 500.0);
    let panel = Rect::new((sw - panel_w) * 0.5, (sh - panel_h) * 0.5, panel_w, panel_h);
    draw_title_panel(panel);

    draw_text_fit(
        "CHOOSE YOUR REIGN",
        panel.x + 28.0,
        panel.y + 38.0,
        panel.w - 56.0,
        22.0,
        TEXT,
    );
    draw_text_fit(
        "Tap a difficulty to begin. Your choice fixes invaders, siege timing, income, and core resilience for the run.",
        panel.x + 28.0,
        panel.y + 62.0,
        panel.w - 56.0,
        12.0,
        TEXT_MUTED,
    );

    while let Some(character) = get_char_pressed() {
        if character.is_ascii_hexdigit() && seed_input.len() < 16 {
            seed_input.push(character.to_ascii_uppercase());
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        seed_input.pop();
    }
    let challenge_seed = if seed_input.is_empty() {
        Some(None)
    } else {
        u64::from_str_radix(seed_input, 16).ok().map(Some)
    };

    let mut action = NewGameSetupAction::None;
    let card_x = panel.x + 28.0;
    let card_w = panel.w - 56.0;
    let card_h = 84.0;
    let gap = 12.0;
    let mut cy = panel.y + 88.0;
    for diff in Difficulty::all() {
        let profile = diff.profile();
        let card = Rect::new(card_x, cy, card_w, card_h);
        let hovered = is_hovered_rect(card);
        let accent = match diff {
            Difficulty::Apprentice => EMERALD,
            Difficulty::Keeper => SOUL,
            Difficulty::Overlord => DANGER,
        };
        draw_card(
            card,
            with_alpha(accent, if hovered { 0.18 } else { 0.08 }),
            with_alpha(accent, if hovered { 0.7 } else { 0.32 }),
        );
        draw_text_fit(
            profile.name,
            card.x + 16.0,
            card.y + 26.0,
            card.w - 112.0,
            17.0,
            accent,
        );
        draw_pill(
            Rect::new(card.x + card.w - 82.0, card.y + 12.0, 66.0, 20.0),
            "BEGIN",
            accent,
        );
        draw_wrapped_blurb(profile.blurb, card.x + 16.0, card.y + 46.0, card.w - 32.0);
        if was_clicked_rect(card) && challenge_seed.is_some() {
            action = NewGameSetupAction::Start(diff, challenge_seed.flatten());
        }
        cy += card_h + gap;
    }

    draw_text_fit(
        "Challenge seed (hex, optional):",
        card_x,
        panel.y + panel.h - 96.0,
        card_w,
        11.0,
        TEXT_MUTED,
    );
    draw_card(
        Rect::new(card_x, panel.y + panel.h - 84.0, card_w, 24.0),
        CARD,
        if challenge_seed.is_some() {
            BORDER
        } else {
            DANGER
        },
    );
    draw_text_fit(
        if seed_input.is_empty() {
            "Random seed"
        } else {
            seed_input
        },
        card_x + 10.0,
        panel.y + panel.h - 68.0,
        card_w - 20.0,
        11.0,
        if seed_input.is_empty() {
            TEXT_DIM
        } else {
            TEXT
        },
    );

    if draw_title_button(
        Rect::new(card_x, panel.y + panel.h - 46.0, card_w, 32.0),
        "Back",
        true,
        ButtonTone::Ghost,
    ) || is_key_pressed(KeyCode::Escape)
    {
        action = NewGameSetupAction::Back;
    }

    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }

    action
}

/// Draw a short blurb wrapped to at most two lines within `max_w`.
fn draw_wrapped_blurb(text: &str, x: f32, y: f32, max_w: f32) {
    // Rough char budget per line for the blurb font size.
    let per_line = (max_w / 6.4).floor().max(8.0) as usize;
    let mut line = String::new();
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        if line.len() + word.len() + 1 > per_line && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
        if lines.len() == 2 {
            break;
        }
    }
    if lines.len() < 2 && !line.is_empty() {
        lines.push(line);
    }
    for (i, l) in lines.iter().take(2).enumerate() {
        draw_text_fit(l, x, y + i as f32 * 15.0, max_w, 11.0, TEXT_MUTED);
    }
}

pub fn draw_title_settings_screen(
    assets: &AssetManager,
    settings: &GameSettings,
    notice: Option<&str>,
) -> TitleSettingsAction {
    let sw = screen_width();
    let sh = screen_height();
    draw_title_background(assets, sw, sh);

    let panel_w = sw.min(1280.0) * 0.32;
    let panel_h = (sh - 40.0).min(680.0);
    let panel = Rect::new(
        (sw - panel_w.clamp(320.0, 430.0)) * 0.5,
        (sh - panel_h) * 0.5,
        panel_w.clamp(320.0, 430.0),
        panel_h,
    );
    draw_title_panel(panel);

    draw_text_fit(
        "SETTINGS",
        panel.x + 28.0,
        panel.y + 38.0,
        panel.w - 56.0,
        24.0,
        TEXT,
    );
    draw_text_fit(
        "Preferences are saved for every future delve.",
        panel.x + 28.0,
        panel.y + 64.0,
        panel.w - 56.0,
        13.0,
        TEXT_MUTED,
    );

    let button_w = panel.w - 56.0;
    let button_x = panel.x + 28.0;
    if draw_title_button(
        Rect::new(button_x, panel.y + 96.0, button_w, 48.0),
        if settings.fullscreen {
            "Fullscreen: On"
        } else {
            "Fullscreen: Off"
        },
        true,
        ButtonTone::Arcane,
    ) {
        return TitleSettingsAction::ToggleFullscreen;
    }

    if draw_title_button(
        Rect::new(button_x, panel.y + 158.0, button_w, 42.0),
        &format!("Master volume: {:.0}%", settings.master_volume * 100.0),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustMasterVolume;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 212.0, button_w, 42.0),
        &format!("SFX volume: {:.0}%", settings.sfx_volume * 100.0),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustSfxVolume;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 266.0, button_w, 42.0),
        &format!("Music volume: {:.0}%", settings.music_volume * 100.0),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustMusicVolume;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 320.0, button_w, 42.0),
        &format!("UI scale: {:.0}%", settings.ui_text_scale * 100.0),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustUiScale(1);
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 374.0, button_w, 42.0),
        if settings.screen_shake {
            "Reduced motion: Off"
        } else {
            "Reduced motion: On"
        },
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::ToggleReducedMotion;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 428.0, button_w, 42.0),
        &format!("Autosave: {:.0}s", settings.autosave_interval),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustAutosave(1);
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 482.0, button_w, 42.0),
        &format!("New-run speed: {}x", settings.default_speed),
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::AdjustDefaultSpeed;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 536.0, button_w, 42.0),
        "Keyboard bindings",
        true,
        ButtonTone::Ghost,
    ) {
        return TitleSettingsAction::OpenKeybindings;
    }
    if draw_title_button(
        Rect::new(button_x, panel.y + 590.0, button_w, 48.0),
        "Back",
        true,
        ButtonTone::Ghost,
    ) || is_key_pressed(KeyCode::Escape)
    {
        return TitleSettingsAction::Back;
    }

    if let Some(message) = notice {
        draw_title_notice(message, sw, sh);
    }

    TitleSettingsAction::None
}

pub(super) fn draw_title_background(assets: &AssetManager, sw: f32, sh: f32) {
    clear_background(BG_DEEP);

    if let Some(texture) = assets.get_texture(TITLE_BACKGROUND_KEY) {
        let scale = (sw / texture.width()).max(sh / texture.height());
        let draw_w = texture.width() * scale;
        let draw_h = texture.height() * scale;
        let x = (sw - draw_w) * 0.5;
        let y = (sh - draw_h) * 0.5;
        draw_texture_ex(
            texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );
    } else {
        draw_game_background(sw, sh);
    }

    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.16));
    draw_rectangle(0.0, 0.0, sw * 0.42, sh, Color::new(0.0, 0.0, 0.0, 0.28));
    draw_rectangle(
        0.0,
        sh * 0.72,
        sw,
        sh * 0.28,
        Color::new(0.0, 0.0, 0.0, 0.34),
    );
}

pub(super) fn draw_title_panel(rect: Rect) {
    draw_card(
        rect,
        Color::new(0.014, 0.017, 0.029, 0.88),
        with_alpha(SOUL, 0.28),
    );
    draw_rectangle(rect.x, rect.y, 3.0, rect.h, with_alpha(SOUL, 0.68));
}

pub(super) fn draw_title_button(rect: Rect, text: &str, enabled: bool, tone: ButtonTone) -> bool {
    let hovered = enabled && is_hovered_rect(rect);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
    let clicked = enabled && was_clicked_rect(rect);

    let (base, border, text_color) = match tone {
        ButtonTone::Primary => (EMERALD, EMERALD, TEXT),
        ButtonTone::Danger => (DANGER, DANGER, Color::new(1.0, 0.91, 0.91, 1.0)),
        ButtonTone::Ghost => (PANEL_ALT, BORDER, TEXT),
        ButtonTone::Arcane => (SOUL, SOUL, Color::new(0.96, 0.90, 1.0, 1.0)),
    };
    let alpha = if enabled {
        if pressed {
            0.34
        } else if hovered {
            0.28
        } else {
            0.18
        }
    } else {
        0.08
    };
    draw_card(
        rect,
        with_alpha(base, alpha),
        with_alpha(border, if hovered { 0.78 } else { 0.38 }),
    );
    draw_centered_text(
        text,
        Rect::new(rect.x + 10.0, rect.y, rect.w - 20.0, rect.h),
        18.0,
        if enabled { text_color } else { TEXT_DIM },
    );

    clicked
}

pub(super) fn draw_title_notice(message: &str, sw: f32, sh: f32) {
    let rect = Rect::new((sw - 430.0) * 0.5, sh - 76.0, 430.0, 42.0);
    draw_card(
        rect,
        Color::new(0.018, 0.020, 0.034, 0.82),
        with_alpha(TREASURE, 0.32),
    );
    draw_centered_text(
        message,
        Rect::new(rect.x + 16.0, rect.y, rect.w - 32.0, rect.h),
        13.0,
        TEXT,
    );
}
