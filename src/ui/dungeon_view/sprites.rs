//! Data-driven unit-sheet mapping for the dungeon board.
//!
//! The sheet is intentionally optional: a failed asset load leaves the old
//! coloured initial-disc presentation in place rather than hiding a unit.

use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::sprite::{SpriteAtlas, SpriteClip};

pub const UNIT_SHEET_KEY: &str = "dungeon_unit_sheet";
pub const UNIT_SHEET_PATH: &str = "assets/sprites/dungeon_units.png";
pub const ANIMATED_UNIT_SHEET_KEY: &str = "dungeon_unit_sheet_animated";
pub const ANIMATED_UNIT_SHEET_PATH: &str = "assets/sprites/dungeon_units_animated.png";
pub const ANIMATED_ADVENTURER_SHEET_KEY: &str = "dungeon_adventurers_animated";
pub const ANIMATED_ADVENTURER_SHEET_PATH: &str = "assets/sprites/dungeon_adventurers_animated.png";
pub const ANIMATED_MONSTER_SHEET_KEY: &str = "dungeon_monsters_animated";
pub const ANIMATED_MONSTER_SHEET_PATH: &str = "assets/sprites/dungeon_monsters_animated.png";
pub const ANIMATED_FULL_MONSTER_SHEET_KEY: &str = "dungeon_monsters_full_animated";
pub const ANIMATED_FULL_MONSTER_SHEET_PATH: &str =
    "assets/sprites/dungeon_monsters_full_animated.png";

pub struct DungeonSprites {
    atlas: Option<SpriteAtlas>,
    animated_atlas: Option<SpriteAtlas>,
    animated_adventurer_atlas: Option<SpriteAtlas>,
    animated_monster_atlas: Option<SpriteAtlas>,
    animated_full_monster_atlas: Option<SpriteAtlas>,
    idle: SpriteClip,
    walk: SpriteClip,
    attack: SpriteClip,
    death: SpriteClip,
}

impl DungeonSprites {
    pub fn from_assets(assets: &AssetManager) -> Self {
        let atlas = assets
            .get_texture(UNIT_SHEET_KEY)
            .cloned()
            .map(|texture| SpriteAtlas::new(texture, 313.5, 313.5));
        let animated_atlas = assets
            .get_texture(ANIMATED_UNIT_SHEET_KEY)
            .cloned()
            .map(|texture| {
                let frame_w = texture.width() / 4.0;
                let frame_h = texture.height() / 4.0;
                SpriteAtlas::new(texture, frame_w, frame_h)
            });
        let animated_monster_atlas =
            assets
                .get_texture(ANIMATED_MONSTER_SHEET_KEY)
                .cloned()
                .map(|texture| {
                    let frame_w = texture.width() / 4.0;
                    let frame_h = texture.height() / 4.0;
                    SpriteAtlas::new(texture, frame_w, frame_h)
                });
        let animated_adventurer_atlas = assets
            .get_texture(ANIMATED_ADVENTURER_SHEET_KEY)
            .cloned()
            .map(|texture| {
                let frame_w = texture.width() / 4.0;
                let frame_h = texture.height() / 4.0;
                SpriteAtlas::new(texture, frame_w, frame_h)
            });
        let animated_full_monster_atlas = assets
            .get_texture(ANIMATED_FULL_MONSTER_SHEET_KEY)
            .cloned()
            .map(|texture| {
                let frame_w = texture.width() / 4.0;
                let frame_h = texture.height() / 4.0;
                SpriteAtlas::new(texture, frame_w, frame_h)
            });
        Self {
            atlas,
            animated_atlas,
            animated_adventurer_atlas,
            animated_monster_atlas,
            animated_full_monster_atlas,
            // The supplied art is a pose atlas. Keeping named clips now makes
            // future animated sheets a manifest-only asset change.
            idle: SpriteClip::new("idle", 0, 1, 2.0),
            walk: SpriteClip::new("walk", 0, 1, 7.0),
            attack: SpriteClip::new("attack", 0, 1, 10.0).one_shot(),
            death: SpriteClip::new("death", 0, 1, 8.0).one_shot(),
        }
    }

    pub fn has_art(&self) -> bool {
        self.atlas.is_some()
    }

    pub fn draw_monster(
        &self,
        name: &str,
        center: Vec2,
        size: f32,
        elapsed: f32,
        flip_x: bool,
        fighting: bool,
    ) -> bool {
        let clip = if fighting { &self.attack } else { &self.idle };
        if let Some(frame) = animated_monster_frame(name) {
            return self.draw_monster_animated_frame(frame, center, size, flip_x, clip);
        }
        if let Some(frame) = animated_full_monster_frame(name) {
            return self.draw_full_monster_frame(frame, center, size, flip_x, clip);
        }
        self.draw_frame(monster_frame(name), center, size, elapsed, flip_x, clip)
    }

    pub fn draw_adventurer(
        &self,
        class_name: &str,
        center: Vec2,
        size: f32,
        elapsed: f32,
        flip_x: bool,
        walking: bool,
        fighting: bool,
    ) -> bool {
        let clip = if walking {
            &self.walk
        } else if fighting {
            &self.attack
        } else {
            &self.idle
        };
        if let Some(frame) = animated_adventurer_frame(class_name) {
            return self.draw_animated_frame(frame, center, size, elapsed, flip_x, clip);
        }
        if let Some(frame) = animated_late_adventurer_frame(class_name) {
            return self.draw_late_adventurer_frame(frame, center, size, flip_x, clip);
        }
        self.draw_frame(
            adventurer_frame(class_name),
            center,
            size,
            elapsed,
            flip_x,
            clip,
        )
    }

    pub fn draw_death(
        &self,
        monster: bool,
        key: &str,
        center: Vec2,
        size: f32,
        elapsed: f32,
        flip_x: bool,
    ) -> bool {
        if monster {
            if let Some(frame) = animated_monster_frame(key) {
                return self.draw_monster_animated_frame(frame, center, size, flip_x, &self.death);
            }
            if let Some(frame) = animated_full_monster_frame(key) {
                return self.draw_full_monster_frame(frame, center, size, flip_x, &self.death);
            }
        }
        let frame = if monster {
            monster_frame(key)
        } else {
            adventurer_frame(key)
        };
        if let Some(frame) = if monster {
            animated_monster_frame(key)
        } else {
            animated_adventurer_frame(key)
        } {
            return self.draw_animated_frame(frame, center, size, elapsed, flip_x, &self.death);
        }
        if !monster {
            if let Some(frame) = animated_late_adventurer_frame(key) {
                return self.draw_late_adventurer_frame(frame, center, size, flip_x, &self.death);
            }
        }
        self.draw_frame(frame, center, size, elapsed, flip_x, &self.death)
    }

    fn draw_frame(
        &self,
        frame: Option<usize>,
        center: Vec2,
        size: f32,
        elapsed: f32,
        flip_x: bool,
        clip: &SpriteClip,
    ) -> bool {
        let (Some(atlas), Some(frame)) = (&self.atlas, frame) else {
            return false;
        };
        // Each definition has a stable base frame. The named clip contributes
        // its local animation phase without changing which identity is drawn.
        let animated = frame + clip.frame_at(elapsed).saturating_sub(clip.start_frame);
        atlas.draw_frame(animated, center, vec2(size, size), flip_x, WHITE);
        true
    }

    fn draw_animated_frame(
        &self,
        base_frame: usize,
        center: Vec2,
        size: f32,
        _elapsed: f32,
        flip_x: bool,
        clip: &SpriteClip,
    ) -> bool {
        let Some(atlas) = &self.animated_atlas else {
            return false;
        };
        // Each animated row contains idle, walk, attack, then death. The
        // existing clip names supply the intentional pose index while phase
        // remains cosmetic and stable for this render frame.
        let pose = match clip.name.as_str() {
            "walk" => 1,
            "attack" => 2,
            "death" => 3,
            _ => 0,
        };
        atlas.draw_frame(base_frame + pose, center, vec2(size, size), flip_x, WHITE);
        true
    }

    fn draw_monster_animated_frame(
        &self,
        base: usize,
        center: Vec2,
        size: f32,
        flip_x: bool,
        clip: &SpriteClip,
    ) -> bool {
        let Some(atlas) = &self.animated_monster_atlas else {
            return false;
        };
        let pose = match clip.name.as_str() {
            "walk" => 1,
            "attack" => 2,
            "death" => 3,
            _ => 0,
        };
        atlas.draw_frame(base + pose, center, vec2(size, size), flip_x, WHITE);
        true
    }

    fn draw_late_adventurer_frame(
        &self,
        base_frame: usize,
        center: Vec2,
        size: f32,
        flip_x: bool,
        clip: &SpriteClip,
    ) -> bool {
        let Some(atlas) = &self.animated_adventurer_atlas else {
            return false;
        };
        let pose = match clip.name.as_str() {
            "walk" => 1,
            "attack" => 2,
            "death" => 3,
            _ => 0,
        };
        atlas.draw_frame(base_frame + pose, center, vec2(size, size), flip_x, WHITE);
        true
    }

    fn draw_full_monster_frame(
        &self,
        base_frame: usize,
        center: Vec2,
        size: f32,
        flip_x: bool,
        clip: &SpriteClip,
    ) -> bool {
        let Some(atlas) = &self.animated_full_monster_atlas else {
            return false;
        };
        let pose = match clip.name.as_str() {
            "walk" => 1,
            "attack" => 2,
            "death" => 3,
            _ => 0,
        };
        atlas.draw_frame(base_frame + pose, center, vec2(size, size), flip_x, WHITE);
        true
    }
}

fn animated_monster_frame(name: &str) -> Option<usize> {
    let species = crate::data::monsters::get_monster_template(name)?.species;
    match species.as_str() {
        "Slime" => Some(0),
        "Undead" => Some(4),
        "Beast" => Some(8),
        "Demon" => Some(12),
        _ => None,
    }
}

fn animated_adventurer_frame(class_name: &str) -> Option<usize> {
    match class_name {
        "Warrior" => Some(4),
        "Rogue" => Some(8),
        "Mage" => Some(12),
        _ => None,
    }
}

fn animated_late_adventurer_frame(class_name: &str) -> Option<usize> {
    match class_name {
        "Cleric" => Some(0),
        "Ranger" => Some(4),
        "Paladin" => Some(8),
        "Alchemist" => Some(12),
        _ => None,
    }
}

fn animated_full_monster_frame(name: &str) -> Option<usize> {
    let species = crate::data::monsters::get_monster_template(name)?.species;
    match species.as_str() {
        "Goblinoid" => Some(0),
        "Draconic" => Some(4),
        "Elemental" => Some(8),
        "Construct" => Some(12),
        _ => None,
    }
}

/// One recognizable body per species; evolutions use palette/scale in the
/// board composition and intentionally share their line's silhouette.
pub fn monster_frame(name: &str) -> Option<usize> {
    let species = crate::data::monsters::get_monster_template(name)?.species;
    Some(match species.as_str() {
        "Goblinoid" => 0,
        "Slime" => 2,
        "Undead" => 3,
        "Beast" => 5,
        "Demon" => 6,
        "Draconic" => 6,
        "Elemental" => 2,
        "Construct" => 1,
        _ => return None,
    })
}

pub fn adventurer_frame(class_name: &str) -> Option<usize> {
    Some(match class_name {
        "Warrior" => 8,
        "Rogue" => 9,
        "Mage" => 10,
        "Cleric" => 11,
        "Paladin" => 12,
        "Alchemist" => 13,
        "Ranger" => 15,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
