pub mod gravity_shift;
pub mod level;
pub mod obstacle;
pub mod player;
pub mod score;
pub mod scoring_region;
use bevy::prelude::*;

#[derive(Resource, Reflect)]
pub struct WorldSettings {
    /// Visible / bounds of the level world.
    pub bounds: Rect,
}

impl Default for WorldSettings {
    fn default() -> Self {
        Self {
            bounds: Rect::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(Event)]
pub struct ResetEvent;

pub fn send_reset_event(mut ev: EventWriter<ResetEvent>) {
    ev.send(ResetEvent);
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, SystemSet)]
pub struct WorldSet;

#[derive(Clone, PartialEq, Eq, Debug, Hash, SystemSet)]
pub struct LevelSet;

#[allow(unused)]
const OTHER_COLLISION_LAYER: u32 = 0b001;
#[allow(unused)]
const PLAYER_COLLISION_LAYER: u32 = 0b010;
#[allow(unused)]
const WORLD_COLLISION_LAYER: u32 = 0b100;
