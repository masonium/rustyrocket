use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use futures_lite::AsyncReadExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::WorldSettings;

/// Settings for overall object spawning.
#[derive(Asset, TypePath, Debug, Serialize, Deserialize, Clone)]
pub struct SpawnerSettings {
    pub item_vel: Vec2,
    pub(crate) start_offset_secs: f32,

    /// Spawn rate for obstacles and other spawned items in the level.
    pub(crate) seconds_per_item: f32,

    pub(crate) tunnel_weight: f32,
    pub(crate) tunnel_settings: TunnelSpawnSettings,

    pub(crate) gravity_weight: f32,
    pub min_items_between_gravity: u32,
    pub(crate) gravity_settings: GravityRegionSettings,
}

impl SpawnerSettings {
    pub fn new() -> SpawnerSettings {
        SpawnerSettings {
            item_vel: Vec2::new(-200.0, 0.0),
            start_offset_secs: 0.1,
            seconds_per_item: 2.0,

            tunnel_weight: 0.8,
            tunnel_settings: TunnelSpawnSettings::default(),

            gravity_weight: 0.2,
            min_items_between_gravity: 3,
            gravity_settings: GravityRegionSettings {
                gravity_width: 32.0,
            },
        }
    }

    pub fn reset(&mut self) {
        *self = SpawnerSettings::new();
    }

    /// Return the x offset where obstacles should start.
    ///
    /// Most obstacles should be shifted so that left boundary begins at start_offset.
    pub fn start_offset_x(&self, play_world: &WorldSettings) -> f32 {
        play_world.bounds.max.x - self.item_vel.x * self.start_offset_secs
    }
}

/// Per instance settings for a gravity region.
#[derive(Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct GravityRegionSettings {
    pub gravity_width: f32,
}

/// Per instance settings for a tunnel barrier.
///
/// A tunnel consists of two objects and a scoring region between them.
#[derive(Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct TunnelSpawnSettings {
    pub center_y_range: [f32; 2],
    pub gap_height_range: [f32; 2],
    pub obstacle_width: f32,
    pub scoring_gap_width: f32,
}

impl Default for TunnelSpawnSettings {
    fn default() -> Self {
        Self {
            center_y_range: [-200.0, 200.0],
            gap_height_range: [200.0, 300.0],
            obstacle_width: 96.0,
            scoring_gap_width: 32.0,
        }
    }
}

#[derive(Default)]
pub struct SpawnerSettingsLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SpawnerSettingsLoaderError {
    /// An [IO](std::io) Error
    #[error("IO error while loading file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for SpawnerSettingsLoader {
    type Asset = SpawnerSettings;
    type Settings = ();
    type Error = SpawnerSettingsLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let custom_asset = ron::de::from_bytes::<SpawnerSettings>(&bytes)?;
            Ok(custom_asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["spawner.ron"]
    }
}

pub struct SpawnerSettingsPlugin;

impl Plugin for SpawnerSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SpawnerSettings>()
            .init_asset_loader::<SpawnerSettingsLoader>();
    }
}
