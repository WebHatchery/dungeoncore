//! Data-driven unit-sheet mapping for the dungeon board.
//!
//! The sheet is intentionally optional: a failed asset load leaves the old
//! coloured initial-disc presentation in place rather than hiding a unit.

use macroquad::prelude::*;
use macroquad_toolkit::assets::AssetManager;
use macroquad_toolkit::sprite::{SpriteAtlas, SpriteClip};

pub const UNIT_SHEET_KEY: &str = "dungeon_unit_sheet";
pub const UNIT_SHEET_PATH: &str = "assets/sprites/dungeon_units.png";

pub struct DungeonSprites {
    atlas: Option<SpriteAtlas>,
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
        Self {
            atlas,
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
    ) -> bool {
        self.draw_frame(
            monster_frame(name),
            center,
            size,
            elapsed,
            flip_x,
            &self.idle,
        )
    }

    pub fn draw_adventurer(
        &self,
        class_name: &str,
        center: Vec2,
        size: f32,
        elapsed: f32,
        flip_x: bool,
        walking: bool,
    ) -> bool {
        let clip = if walking { &self.walk } else { &self.attack };
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
        let frame = if monster {
            monster_frame(key)
        } else {
            adventurer_frame(key)
        };
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
mod tests {
    use super::*;

    #[test]
    fn every_declared_unit_has_a_sprite_definition() {
        for monster in crate::data::monsters::get_monster_templates() {
            assert!(
                monster_frame(&monster.name).is_some(),
                "missing monster mapping: {}",
                monster.name
            );
        }
        for class in crate::data::adventurers::get_adventurer_classes() {
            assert!(
                adventurer_frame(&class.name).is_some(),
                "missing class mapping: {}",
                class.name
            );
        }
    }
}
