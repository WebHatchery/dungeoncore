//! Player-configurable keyboard bindings. These preferences are cosmetic UI
//! state, separate from saves, and use the toolkit's native/WebGL key store.

use macroquad::prelude::{is_key_pressed, KeyCode};
use macroquad_toolkit::persistence::{load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

const STORAGE_KEY: &str = "keybindings";
const GAME_KEY: &str = "dungeon_core";

/// Every action the current keyboard surface exposes. Keeping this finite lets
/// the settings screen show every binding and prevents unknown save values from
/// producing unreachable controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingAction {
    Pause,
    NavigateLeft,
    NavigateRight,
    NavigateUp,
    NavigateDown,
    Smite,
    Codex,
    Goals,
    CorePowers,
    Help,
}

impl BindingAction {
    pub const ALL: [Self; 10] = [
        Self::Pause,
        Self::NavigateLeft,
        Self::NavigateRight,
        Self::NavigateUp,
        Self::NavigateDown,
        Self::Smite,
        Self::Codex,
        Self::Goals,
        Self::CorePowers,
        Self::Help,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Pause => "Pause / resume",
            Self::NavigateLeft => "Inspect left",
            Self::NavigateRight => "Inspect right",
            Self::NavigateUp => "Inspect previous floor",
            Self::NavigateDown => "Inspect next floor",
            Self::Smite => "Cast Core Smite",
            Self::Codex => "Open Codex",
            Self::Goals => "Open goals",
            Self::CorePowers => "Open Core Powers",
            Self::Help => "Open controls",
        }
    }

    fn default_key(self) -> KeyCode {
        match self {
            Self::Pause => KeyCode::Space,
            Self::NavigateLeft => KeyCode::Left,
            Self::NavigateRight => KeyCode::Right,
            Self::NavigateUp => KeyCode::Up,
            Self::NavigateDown => KeyCode::Down,
            Self::Smite => KeyCode::Q,
            Self::Codex => KeyCode::C,
            Self::Goals => KeyCode::K,
            Self::CorePowers => KeyCode::P,
            Self::Help => KeyCode::H,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    keys: [String; 10],
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            keys: BindingAction::ALL.map(|action| key_name(action.default_key()).to_string()),
        }
    }
}

impl KeyBindings {
    pub fn load() -> Self {
        let mut bindings: Self = load_json_key(GAME_KEY, STORAGE_KEY).unwrap_or_default();
        bindings.sanitize();
        bindings
    }

    pub fn save(&self) -> Result<(), String> {
        save_json_key(GAME_KEY, STORAGE_KEY, self)
    }

    pub fn pressed(&self, action: BindingAction) -> bool {
        is_key_pressed(self.key(action))
    }

    pub fn key(&self, action: BindingAction) -> KeyCode {
        parse_key(&self.keys[action as usize]).unwrap_or_else(|| action.default_key())
    }

    pub fn label(&self, action: BindingAction) -> &'static str {
        key_name(self.key(action))
    }

    pub fn supports(key: KeyCode) -> bool {
        !key_name(key).is_empty()
    }

    /// Assigning a key swaps it with any action already using that key, so no
    /// shortcut silently becomes a duplicate or stops working.
    pub fn assign(&mut self, action: BindingAction, key: KeyCode) {
        let target = action as usize;
        let previous = self.keys[target].clone();
        let replacement = key_name(key).to_string();
        if let Some(other) = self.keys.iter().position(|saved| saved == &replacement) {
            self.keys[other] = previous;
        }
        self.keys[target] = replacement;
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn sanitize(&mut self) {
        for action in BindingAction::ALL {
            let index = action as usize;
            if parse_key(&self.keys[index]).is_none() {
                self.keys[index] = key_name(action.default_key()).to_string();
            }
        }
    }
}

fn key_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Space",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Q => "Q",
        KeyCode::C => "C",
        KeyCode::K => "K",
        KeyCode::P => "P",
        KeyCode::H => "H",
        KeyCode::A => "A",
        KeyCode::D => "D",
        KeyCode::W => "W",
        KeyCode::S => "S",
        KeyCode::E => "E",
        KeyCode::R => "R",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::Z => "Z",
        KeyCode::X => "X",
        KeyCode::V => "V",
        KeyCode::B => "B",
        KeyCode::M => "M",
        KeyCode::Tab => "Tab",
        _ => "",
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    Some(match name {
        "Space" => KeyCode::Space,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Q" => KeyCode::Q,
        "C" => KeyCode::C,
        "K" => KeyCode::K,
        "P" => KeyCode::P,
        "H" => KeyCode::H,
        "A" => KeyCode::A,
        "D" => KeyCode::D,
        "W" => KeyCode::W,
        "S" => KeyCode::S,
        "E" => KeyCode::E,
        "R" => KeyCode::R,
        "F" => KeyCode::F,
        "G" => KeyCode::G,
        "Z" => KeyCode::Z,
        "X" => KeyCode::X,
        "V" => KeyCode::V,
        "B" => KeyCode::B,
        "M" => KeyCode::M,
        "Tab" => KeyCode::Tab,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
