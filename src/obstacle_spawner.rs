use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::loading_state::LoadingStateAppExt;
use rand::Rng;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::level::{RemoveOnReset, RemoveWhenLeft};
use crate::obstacle::spawner_settings::{SpawnerSettings, GravityRegionSettings, TunnelSpawnSettings};
use crate::{
    barrier::{new_barrier, BarrierAssets, RegionRef},
    gravity_shift::{new_gravity_region, GravityMaterials},
    scoring_region::new_scoring_region,
};
use crate::{level::LevelSettings, WorldSettings};
use crate::{GameState, ResetEvent};

/// Available options for spawning from a spawner.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SpawnOption {
    Tunnel,
    Gravity,
}

#[derive(AssetCollection, Resource)]
pub struct Levels {
    #[asset(path = "levels/base.spawner.ron")]
    pub base_level: Handle<SpawnerSettings>,

    #[asset(path = "levels/fast.spawner.ron")]
    pub fast_level: Handle<SpawnerSettings>,
}

/// Track statistics based on spawning, for determining later spawns.
#[derive(Reflect, Default)]
pub struct SpawnStats {
    /// Total number of logical items sent since reset.
    num_items: u32,

    /// Number of items spawned since the last gravity shfit.
    since_last_gravity: u32,
}

impl SpawnStats {
    /// Reset the tracked statistics.
    fn reset(&mut self) {
        self.num_items = 0;
        self.since_last_gravity = 0;
    }
}

/// Obstacle spawning component.
#[derive(Component)]
pub struct ObstacleSpawner {
    timer: Timer,
    level: SpawnerSettings,
    stats: SpawnStats,
}

impl ObstacleSpawner {
    fn reset(&mut self) {
	self.timer = Timer::from_seconds(self.level.seconds_per_item, TimerMode::Repeating);
	self.stats.reset();
    }
}

/// Update the timers on the obstacle spawners
fn update_spawner_timers(time: Res<Time>, mut query: Query<&mut ObstacleSpawner>) {
    for mut spawner in query.iter_mut() {
        spawner.timer.tick(time.delta());
    }
}

/// On a timer, spawn one of many items.
fn spawn_items(
    mut commands: Commands,
    mut spawner_query: Query<&mut ObstacleSpawner>,
    level_settings: Res<LevelSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<BarrierAssets>,
    grav_mat: Res<GravityMaterials>,
) {
    let mut rng = rand::thread_rng();
    for mut spawner in spawner_query.iter_mut() {
        if spawner.timer.just_finished() {
            let mut choices = vec![(SpawnOption::Tunnel, spawner.level.tunnel_weight)];

            if spawner.stats.since_last_gravity >= spawner.level.min_items_between_gravity {
                choices.push((SpawnOption::Gravity, spawner.level.gravity_weight));
            }
            spawner.stats.num_items += 1;
            let dist = rand::distributions::WeightedIndex::new(choices.iter().map(|x| x.1)).unwrap();
            match choices[rng.sample(dist)].0 {
                SpawnOption::Tunnel => {
                    spawner.stats.since_last_gravity += 1;
                    spawn_tunnel(
                        &spawner.level.tunnel_settings,
                        &mut commands,
                        &spawner.level,
                        &mut meshes,
                        &play_world,
                        &obs_mat,
                    );
                }
                SpawnOption::Gravity => {
                    spawner.stats.since_last_gravity = 0;
                    let gs = &spawner.level.gravity_settings;
                    let start_x = spawner.level.start_offset_x(&play_world) + gs.gravity_width * 0.5;
                    spawn_gravity_region(
                        &mut commands,
                        -level_settings.gravity_mult,
                        start_x,
                        gs,
                        &spawner.level,
                        &play_world,
                        &grav_mat,
                    );
                }
            }
        }
    }
}
/// Spawn a gravity region with the given gravity mult.
fn spawn_gravity_region(
    commands: &mut Commands,
    gravity_mult: f32,
    start_x: f32,
    gs: &GravityRegionSettings,
    spawn_settings: &SpawnerSettings,
    play_world: &Res<WorldSettings>,
    grav_mat: &Res<GravityMaterials>,
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
    commands: &mut Commands,
    spawn: &SpawnerSettings,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    play_world: &Res<WorldSettings>,
    obs_mat: &Res<BarrierAssets>,
) {
    // create the level obstacles and the scoring region.
    let vel = Velocity {
        linvel: spawn.item_vel,
        ..default()
    };
    let mut rng = rand::thread_rng();

    let gap_center = tunnel.center_y_range[0]
        + rng.gen::<f32>() * (tunnel.center_y_range[1] - tunnel.center_y_range[0]);
    let gap_height = tunnel.gap_height_range[0]
        + rng.gen::<f32>() * (tunnel.gap_height_range[1] - tunnel.gap_height_range[0]);

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
        .spawn(new_barrier(
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
        .spawn(new_barrier(
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

/// Reset the state of the obstacle spawners.
fn reset_obstacle_spawner(mut spawners: Query<&mut ObstacleSpawner>,
			  levels: Res<Levels>,
			  s: Res<Assets<SpawnerSettings>>
) {
    for mut spawner in spawners.iter_mut() {
	// reset the level back to the base level.
        spawner.level = s.get(&levels.base_level).unwrap().clone();
        spawner.reset();
    }
}

/// Initial setup of the obstacle spawner.
fn setup_obstacle_spawner(mut commands: Commands,
			  levels: Res<Levels>,
			  s: Res<Assets<SpawnerSettings>>) {
    let settings = s.get(&levels.base_level).unwrap();
    commands.spawn(ObstacleSpawner {
        timer: Timer::from_seconds(settings.seconds_per_item, TimerMode::Repeating),
        level: s.get(&levels.base_level).unwrap().clone(),
        stats: SpawnStats::default(),
    });
}

pub struct ObstacleSpawnerPlugin;

impl Plugin for ObstacleSpawnerPlugin {
    fn build(&self, app: &mut App) {
        let initial_secs_per_item = 2.0;

        let mut timer = Timer::from_seconds(initial_secs_per_item, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(initial_secs_per_item - 0.01));

        app.add_collection_to_loading_state::<_, Levels>(GameState::AssetLoading)
	    .add_systems(OnExit(GameState::AssetLoading), setup_obstacle_spawner)
            .add_systems(PreUpdate, update_spawner_timers)
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
