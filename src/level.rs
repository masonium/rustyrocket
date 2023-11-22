use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{send_event, GameState, LevelSet, ResetEvent, WorldSet, WorldSettings};

#[derive(Resource, Reflect, Default)]
pub struct LevelSettings {
    /// Base velocity jump vector (set when initializing a jump). Can
    /// be modified by the gravity mult.
    base_jump_vel: Vec2,

    pub explosion_speed: f32,

    /// Base gravity acceleration vector. Typically not modified in
    /// game, but is effectively tranformed by gravity mult.
    base_gravity: Vec2,

    max_object_width: f32,
    pub start_offset: f32,

    pub gravity_mult: f32,
}

impl LevelSettings {
    /// Settings that should be reset on level start
    fn reset(&mut self) {
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
pub struct RemoveWhenLeft(pub f32);

/// Market component for objects that should be removed when the game is reset.
#[derive(Component)]
pub struct RemoveOnReset;

/// Initialize the level settings.
fn setup_level_settings(
    world_settings: Res<WorldSettings>,
    mut level_settings: ResMut<LevelSettings>,
) {
    level_settings.reset();

    level_settings.base_jump_vel = Vec2::new(0.0, 300.0);
    level_settings.explosion_speed = 600.0;
    level_settings.base_gravity = Vec2::new(0.0, -500.0);
    level_settings.start_offset = world_settings.bounds.max.x + 100.0;
}

/// Remove obstacles once they move out of the world view.
fn remove_invisible_objects(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &GlobalTransform,
            &RemoveWhenLeft,
            Option<&Handle<Mesh>>,
        ),
        With<Collider>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
) {
    for (ent, global, rwl, maybe_mesh) in query.iter() {
        if global.translation().x < play_world.bounds.min.x - rwl.0 {
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
    mut level: ResMut<LevelSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for ent in items.iter() {
        commands.entity(ent).despawn();
    }
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
            .add_systems(
                Startup,
                setup_level_settings.in_set(LevelSet).after(WorldSet),
            )
            .add_systems(OnExit(GameState::AssetLoading), send_event::<ResetEvent>)
            .add_systems(
                Update,
                (remove_invisible_objects,).run_if(in_state(GameState::Playing)),
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
