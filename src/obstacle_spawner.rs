use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level::{RemoveOnReset, RemoveWhenLeft};
use crate::{
    gravity_shift::{new_gravity_region, GravityMaterials},
    obstacle::{new_obstacle, ObstacleAssets, RegionRef},
    scoring_region::new_scoring_region,
};
use crate::{level::LevelSettings, WorldSettings};
use crate::{GameState, ResetEvent};

#[derive(Resource, Reflect)]
pub struct LevelTimer {
    timer: Timer,
}

/// Update the level timer.
fn update_timer(time: Res<Time>, mut timer: ResMut<LevelTimer>) {
    timer.timer.tick(time.delta());
}

/// Settings for overall object spawning.
#[derive(Resource, Default)]
pub struct SpawnerSettings {
    item_vel: Vec2,
    start_offset_secs: f32,

    /// Spawn rate for obstacles and other spawned items in the level.
    seconds_per_item: f32,
}

impl SpawnerSettings {
    fn reset(&mut self) {
        self.item_vel = Vec2::new(-200.0, 0.0);
        self.start_offset_secs = 0.1;
        self.seconds_per_item = 2.0;
    }

    /// Return the x offset where obstacles should start.
    ///
    /// Most obstacles should be shifted so that left boundary begins at start_offset.
    fn start_offset_x(&self, play_world: &WorldSettings) -> f32 {
        play_world.bounds.max.x - self.item_vel.x * self.start_offset_secs
    }
}

/// Per instance settings for a gravity region.
pub struct GravityRegionSettings {
    gravity_width: f32,
}

/// Per instance settings for a tunnel barrier.
///
/// A tunnel consists of two objects and a scoring region between them.
pub struct TunnelSpawnSettings {
    center_y_range: [f32; 2],
    gap_height_range: [f32; 2],
    pub obstacle_width: f32,
    scoring_gap_width: f32,
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

#[derive(Resource, Reflect, Default)]
pub struct SpawnStats {
    /// Total number of logical items sent since reset.
    num_items: u32,
    since_last_gravity: u32,
}

impl SpawnStats {
    /// Reset the tracked statistics.
    fn reset(&mut self) {
        self.num_items = 0;
        self.since_last_gravity = 0;
    }
}

/// On a timer, spawn one of many items.
fn spawn_items(
    commands: Commands,
    mut spawn_stats: ResMut<SpawnStats>,
    spawn_settings: Res<SpawnerSettings>,
    level_settings: Res<LevelSettings>,
    meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<ObstacleAssets>,
    grav_mat: Res<GravityMaterials>,
    level_timer: Res<LevelTimer>,
) {
    if level_timer.timer.just_finished() {
        let should_spawn_tunnel = spawn_stats.num_items == 0 || spawn_stats.since_last_gravity < 5;
        spawn_stats.num_items += 1;
        if should_spawn_tunnel {
            let tunnel = TunnelSpawnSettings::default();
            spawn_stats.since_last_gravity += 1;
            spawn_tunnel(
                &tunnel,
                commands,
                spawn_settings,
                meshes,
                play_world,
                obs_mat,
            );
        } else {
            spawn_stats.since_last_gravity = 0;
            let gs = GravityRegionSettings {
                gravity_width: 32.0,
            };
            let start_x = spawn_settings.start_offset_x(&play_world) + gs.gravity_width * 0.5;
            spawn_gravity_region(
                commands,
                -level_settings.gravity_mult,
                start_x,
                gs,
                spawn_settings,
                play_world,
                grav_mat,
            );
        }
    }
}

/// Spawn a gravity region with the given gravity mult.
fn spawn_gravity_region(
    mut commands: Commands,
    gravity_mult: f32,
    start_x: f32,
    gs: GravityRegionSettings,
    spawn_settings: Res<SpawnerSettings>,
    play_world: Res<WorldSettings>,
    grav_mat: Res<GravityMaterials>,
) {
    let vel = Velocity {
        linvel: spawn_settings.item_vel,
        ..default()
    };

    let width = gs.gravity_width;
    commands
        .spawn(new_gravity_region(
            gravity_mult,
            start_x,
            width,
            &play_world,
            &grav_mat,
        ))
        .insert((
            Name::new(format!(
                "gravity {}",
                if gravity_mult > 0.0 { "down" } else { "up" }
            )),
            RemoveWhenLeft(width),
            RemoveOnReset,
            vel,
        ));
}

/// Spawn two barriers and a scoring region.
fn spawn_tunnel(
    tunnel: &TunnelSpawnSettings,
    mut commands: Commands,
    spawn: Res<SpawnerSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<ObstacleAssets>,
) {
    // create the level obstacles and the scoring region.
    let vel = Velocity {
        linvel: spawn.item_vel,
        ..default()
    };

    let gap_center = tunnel.center_y_range[0]
        + fastrand::f32() * (tunnel.center_y_range[1] - tunnel.center_y_range[0]);
    let gap_height = tunnel.gap_height_range[0]
        + fastrand::f32() * (tunnel.gap_height_range[1] - tunnel.gap_height_range[0]);

    let top_height = play_world.bounds.max.y - (gap_center + gap_height / 2.0);
    let bottom_height = (gap_center - gap_height / 2.0) - play_world.bounds.min.y;

    let scoring_gap_height = play_world.bounds.height() - top_height - bottom_height;
    let scoring_gap_width = tunnel.scoring_gap_width;
    let region = commands
        .spawn(new_scoring_region(
            1,
            Vec2::new(
                spawn.start_offset_x(&play_world) + tunnel.obstacle_width - scoring_gap_width / 2.0,
                gap_center,
            ),
            Vec2::new(scoring_gap_width, scoring_gap_height),
        ))
        .insert((RemoveWhenLeft(scoring_gap_width), RemoveOnReset, vel))
        .id();

    commands
        .spawn(new_obstacle(
            true,
            tunnel.obstacle_width,
            top_height,
            spawn.start_offset_x(&play_world) + tunnel.obstacle_width / 2.0,
            &mut meshes,
            &play_world,
            &obs_mat,
        ))
        .insert((
            Name::new("top_barrier"),
            RegionRef { region },
            RemoveWhenLeft(tunnel.obstacle_width),
            RemoveOnReset,
            vel,
        ));
    commands
        .spawn(new_obstacle(
            false,
            tunnel.obstacle_width,
            bottom_height,
            spawn.start_offset_x(&play_world) + tunnel.obstacle_width / 2.0,
            &mut meshes,
            &play_world,
            &obs_mat,
        ))
        .insert((
            Name::new("bottom_barrier"),
            RegionRef { region },
            RemoveWhenLeft(tunnel.obstacle_width),
            RemoveOnReset,
            vel,
        ));
}

fn reset_obstacle_spawner(
    mut stats: ResMut<SpawnStats>,
    mut spawn_settings: ResMut<SpawnerSettings>,
    mut level_timer: ResMut<LevelTimer>,
) {
    stats.reset();
    spawn_settings.reset();
    level_timer.timer.reset();
}

pub struct ObstacleSpawnerPlugin;

impl Plugin for ObstacleSpawnerPlugin {
    fn build(&self, app: &mut App) {
        let initial_secs_per_item = 2.0;

        let mut timer = Timer::from_seconds(initial_secs_per_item, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(initial_secs_per_item - 0.01));

        let mut ss = SpawnerSettings::default();
        ss.reset();

        app.insert_resource(LevelTimer { timer })
            .insert_resource(SpawnStats::default())
            .insert_resource(ss)
            .add_systems(PreUpdate, update_timer)
            .add_systems(
                Update,
                (
                    spawn_items,
                    // spawn_tunnel.run_if(input_just_pressed(KeyCode::O)),
                    // spawn_gravity_region.run_if(input_just_pressed(KeyCode::G)),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                reset_obstacle_spawner.run_if(on_event::<ResetEvent>()),
            );
    }
}
