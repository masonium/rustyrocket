use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    gravity_shift::{new_gravity_region, GravityMaterials},
    obstacle::{new_obstacle, HitObstacleEvent, ObstacleAssets, RegionRef},
    player::{OutOfBoundsEvent, PlayerSet},
    scoring_region::new_scoring_region,
    GameState, LevelSet, ResetEvent, WorldSet, WorldSettings, send_event,
};

#[derive(Resource, Reflect, Default)]
pub struct LevelSettings {
    /// Base velocity of items in the level.
    item_vel: Vec2,

    /// Base velocity jump vector (set when initializing a jump). Can
    /// be modified by the gravity mult.
    base_jump_vel: Vec2,

    pub explosion_speed: f32,

    /// Base gravity acceleration vector. Typically not modified in
    /// game, but is effectively tranformed by gravity mult.
    base_gravity: Vec2,

    center_y_range: [f32; 2],
    gap_height_range: [f32; 2],
    pub obstacle_width: f32,
    scoring_gap_width: f32,
    max_object_width: f32,
    pub gravity_width: f32,
    pub start_offset: f32,

    /// Spawn rate for obstacles and other spawned items in the level.
    seconds_per_item: f32,

    pub gravity_mult: f32,

    /// Total number of logical items sent since reset.
    num_items: u32,
    since_last_gravity: u32,
}

impl LevelSettings {
    /// Settings that should be reset on level start
    fn reset(&mut self) {
        self.item_vel = Vec2::new(-200.0, 0.0);
        self.seconds_per_item = 2.0;
        self.num_items = 0;
        self.since_last_gravity = 0;
        self.gravity_mult = 1.0;
    }

    /// Return the current jump vector, taking the gravity mult into account.
    pub fn jump_vector(&self) -> Vec2 {
        self.base_jump_vel * self.gravity_mult
    }

    /// return the current gravity vector, taking the gravity mult into account.
    pub fn gravity_vector(&self) -> Vec2 {
        self.base_gravity * self.gravity_mult
    }

    /// Sync the level settings to rapier.
    pub fn sync_to_rapier(&self, rc: &mut ResMut<RapierConfiguration>) {
        rc.gravity = self.base_gravity * self.gravity_mult;
    }
}

/// Market component for objects that should be removed when the reach
/// the left of the screen.
#[derive(Component)]
pub struct RemoveWhenLeft;

/// Market component for objects that should be removed when the game is reset.
#[derive(Component)]
pub struct RemoveOnReset;

#[derive(Resource, Reflect)]
pub struct LevelTimer {
    timer: Timer,
}

/// Initialize the level settings.
fn setup_level_settings(
    world_settings: Res<WorldSettings>,
    mut level_settings: ResMut<LevelSettings>,
) {
    level_settings.reset();

    level_settings.base_jump_vel = Vec2::new(0.0, 300.0);
    level_settings.explosion_speed = 600.0;
    level_settings.base_gravity = Vec2::new(0.0, -500.0);
    level_settings.center_y_range = [-200.0, 200.0];
    level_settings.gap_height_range = [200.0, 300.0];
    level_settings.obstacle_width = 96.0;
    level_settings.scoring_gap_width = 32.0;
    level_settings.max_object_width = level_settings
        .obstacle_width
        .max(level_settings.scoring_gap_width);
    level_settings.start_offset = world_settings.bounds.max.x + 100.0;
}

/// Update the level timer.
fn update_timer(time: Res<Time>, mut timer: ResMut<LevelTimer>) {
    timer.timer.tick(time.delta());
}

/// On a timer, spawn one of many items.
fn spawn_items(
    commands: Commands,
    mut level_settings: ResMut<LevelSettings>,
    meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<ObstacleAssets>,
    grav_mat: Res<GravityMaterials>,
    level_timer: Res<LevelTimer>,
) {
    if level_timer.timer.just_finished() {
        let should_spawn_obstacle =
            level_settings.num_items == 0 || level_settings.since_last_gravity < 5;
        level_settings.num_items += 1;
        if should_spawn_obstacle {
            level_settings.since_last_gravity += 1;
            spawn_obstacles(commands, level_settings, meshes, play_world, obs_mat);
        } else {
            level_settings.since_last_gravity = 0;
            spawn_gravity_region(commands, level_settings, play_world, grav_mat);
        }
    }
}

fn spawn_gravity_region(
    mut commands: Commands,
    level_settings: ResMut<LevelSettings>,
    play_world: Res<WorldSettings>,
    grav_mat: Res<GravityMaterials>,
) {
    let vel = Velocity {
        linvel: level_settings.item_vel,
        ..default()
    };

    let current_up = level_settings.base_gravity.y > 0.0;

    commands
        .spawn(new_gravity_region(
            -level_settings.gravity_mult,
            &play_world,
            &level_settings.into(),
            &grav_mat,
        ))
        .insert((
            Name::new(format!(
                "gravity {}",
                if current_up { "down" } else { "up" }
            )),
            RemoveWhenLeft,
            RemoveOnReset,
            vel,
        ));
}

fn spawn_obstacles(
    mut commands: Commands,
    level_settings: ResMut<LevelSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<ObstacleAssets>,
) {
    // create the level obstacles and the scoring region.
    let vel = Velocity {
        linvel: level_settings.item_vel,
        ..default()
    };

    let lvl = &level_settings;

    let gap_center =
        lvl.center_y_range[0] + fastrand::f32() * (lvl.center_y_range[1] - lvl.center_y_range[0]);
    let gap_height = lvl.gap_height_range[0]
        + fastrand::f32() * (lvl.gap_height_range[1] - lvl.gap_height_range[0]);

    let top_height = play_world.bounds.max.y - (gap_center + gap_height / 2.0);
    let bottom_height = (gap_center - gap_height / 2.0) - play_world.bounds.min.y;

    let scoring_gap_height = play_world.bounds.height() - top_height - bottom_height;
    let scoring_gap_width = level_settings.scoring_gap_width;
    let region = commands
        .spawn(new_scoring_region(
            1,
            Vec2::new(
                level_settings.start_offset + level_settings.obstacle_width
                    - scoring_gap_width / 2.0,
                gap_center,
            ),
            Vec2::new(scoring_gap_width, scoring_gap_height),
        ))
        .insert((RemoveWhenLeft, RemoveOnReset, vel))
        .id();

    let ls: Res<LevelSettings> = level_settings.into();

    commands
        .spawn(new_obstacle(
            true,
            top_height,
            &mut meshes,
            &play_world,
            &ls,
            &obs_mat,
        ))
        .insert((
            Name::new("top_obstacle"),
            RegionRef { region },
            RemoveWhenLeft,
            RemoveOnReset,
            vel,
        ));
    commands
        .spawn(new_obstacle(
            false,
            bottom_height,
            &mut meshes,
            &play_world,
            &ls,
            &obs_mat,
        ))
        .insert((
            Name::new("bottom_obstacle"),
            RegionRef { region },
            RemoveWhenLeft,
            RemoveOnReset,
            vel,
        ));
}

/// Remove obstacles once they move out of the world view.
fn remove_invisible_objects(
    mut commands: Commands,
    query: Query<
        (Entity, &GlobalTransform, Option<&Handle<Mesh>>),
        (With<RemoveWhenLeft>, With<Collider>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    level_settings: Res<LevelSettings>,
) {
    for (ent, global, maybe_mesh) in query.iter() {
        if global.translation().x < play_world.bounds.min.x - level_settings.max_object_width {
            commands.entity(ent).despawn();
            if let Some(mesh_handle) = maybe_mesh {
                meshes.remove(mesh_handle);
            }
        }
    }
}

/// Kill all marked level items on reset.
fn reset_level(
    mut commands: Commands,
    items: Query<Entity, With<RemoveOnReset>>,
    mut level_timer: ResMut<LevelTimer>,
    mut level: ResMut<LevelSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for ent in items.iter() {
        commands.entity(ent).despawn();
    }
    level_timer.timer.reset();
    level.reset();
    level.sync_to_rapier(&mut rapier_config);

    app_state.set(GameState::Ready);
}

fn in_ready_level(mut rapier: ResMut<RapierConfiguration>) {
    rapier.physics_pipeline_active = false;
    bevy::log::warn!(rapier.physics_pipeline_active);
}

fn in_start_level(mut rapier: ResMut<RapierConfiguration>) {
    rapier.physics_pipeline_active = true;
    bevy::log::warn!(rapier.physics_pipeline_active);
}

fn start_level(mut app_state: ResMut<NextState<GameState>>) {
    app_state.set(GameState::Playing);
}

/// struct for level-based plugins
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let initial_secs_per_item = 2.0;

        let mut timer = Timer::from_seconds(initial_secs_per_item, TimerMode::Repeating);
        timer.tick(Duration::from_secs_f32(initial_secs_per_item - 0.01));

        app.insert_resource(LevelSettings::default())
            .insert_resource(LevelTimer { timer })
            .add_systems(
                Startup,
                setup_level_settings.in_set(LevelSet).after(WorldSet),
            )
            .add_systems(PreUpdate, update_timer)
            .add_systems(
                OnExit(GameState::AssetLoading),
		send_event::<ResetEvent>,
            )
            .add_systems(
                Update,
                (
                    remove_invisible_objects,
                    spawn_items,
                    spawn_obstacles.run_if(input_just_pressed(KeyCode::O)),
                    spawn_gravity_region.run_if(input_just_pressed(KeyCode::G)),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(GameState::Ready), in_ready_level)
            .add_systems(
                Update,
                start_level.run_if(
                    in_state(GameState::Ready).and_then(input_just_pressed(KeyCode::Space)),
                ),
            )
            .add_systems(OnEnter(GameState::Playing), in_start_level)
            .add_systems(PostUpdate, reset_level.run_if(on_event::<ResetEvent>()));
    }
}
