#![allow(clippy::type_complexity)]
pub mod background;
pub mod center_display;
pub mod dying_player;
pub mod fonts;
pub mod gravity_shift;
pub mod level;
pub mod obstacle;
pub mod obstacle_spawner;
pub mod player;
pub mod score;
pub mod score_display;
pub mod scoring_region;
use bevy::prelude::*;

#[derive(Resource, Reflect, Default)]
pub struct WorldSettings {
    /// Visible / bounds of the level world.
    pub bounds: Rect,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    Ready,
    Playing,
    Dying,
}

#[derive(Event, Default)]
pub struct ResetEvent;

/// Generic mechanism for sending default events.
pub fn send_event<T: Event + Default>(mut ev: EventWriter<T>) {
    ev.send(T::default());
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
