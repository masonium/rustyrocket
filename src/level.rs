use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    obstacle::{new_obstacle, HitObstacleEvent, ObstacleAssets, RegionRef},
    scoring_region::new_scoring_region,
    GameState, LevelSet, ResetEvent, WorldSet, WorldSettings,
};

#[derive(Resource, Reflect, Default)]
pub struct LevelSettings {
    vel: Vec2,
    center_y_range: [f32; 2],
    gap_height_range: [f32; 2],
    pub obstacle_width: f32,
    scoring_gap_width: f32,
    max_object_width: f32,
    pub gravity_width: f32,
    pub start_offset: f32,
}

#[derive(Component)]
pub struct RemoveWhenLeft;

#[derive(Component)]
pub struct RemoveOnReset;

#[derive(Resource, Reflect)]
pub struct LevelTimer {
    timer: Timer,
}

pub struct LevelPlugin;

fn setup_level_settings(
    world_settings: Res<WorldSettings>,
    mut level_settings: ResMut<LevelSettings>,
) {
    level_settings.vel = Vec2::new(-200.0, 0.0);
    level_settings.center_y_range = [-200.0, 200.0];
    level_settings.gap_height_range = [200.0, 300.0];
    level_settings.obstacle_width = 96.0;
    level_settings.scoring_gap_width = 32.0;
    level_settings.max_object_width = level_settings
        .obstacle_width
        .max(level_settings.scoring_gap_width);
    level_settings.start_offset = world_settings.bounds.max.x + 100.0;
}

fn update_timer(time: Res<Time>, mut timer: ResMut<LevelTimer>) {
    timer.timer.tick(time.delta());
}

fn spawn_obstacles(
    mut commands: Commands,
    level_timer: Res<LevelTimer>,
    level_settings: Res<LevelSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    obs_mat: Res<ObstacleAssets>,
) {
    if level_timer.timer.just_finished() {
        // create the level obstacles and the scoring region.
        let vel = Velocity {
            linvel: level_settings.vel,
            ..default()
        };

        let lvl = &level_settings;

        let gap_center = lvl.center_y_range[0]
            + fastrand::f32() * (lvl.center_y_range[1] - lvl.center_y_range[0]);
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
            .insert((RemoveWhenLeft, RemoveOnReset, vel.clone()))
            .id();

        commands
            .spawn(new_obstacle(
                true,
                top_height,
                &mut meshes,
                &play_world,
                &level_settings,
                &obs_mat,
            ))
            .insert((
                Name::new("top_obstacle"),
                RegionRef { region },
                RemoveWhenLeft,
                RemoveOnReset,
                vel.clone(),
            ));
        commands
            .spawn(new_obstacle(
                false,
                bottom_height,
                &mut meshes,
                &play_world,
                &level_settings,
                &obs_mat,
            ))
            .insert((
                Name::new("bottom_obstacle"),
                RegionRef { region },
                RemoveWhenLeft,
                RemoveOnReset,
                vel.clone(),
            ));
    }
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

fn reset_on_collision(
    mut resets: EventWriter<ResetEvent>,
    mut collisions: EventReader<HitObstacleEvent>,
) {
    for _ in collisions.iter() {
        resets.send(ResetEvent);
        return;
    }
}

/// Kill all level items on reset.
fn reset_level(
    mut commands: Commands,
    items: Query<Entity, With<RemoveOnReset>>,
    mut level_timer: ResMut<LevelTimer>,
    mut resets: EventReader<ResetEvent>,
) {
    for _ in resets.iter() {
        for ent in items.iter() {
            commands.entity(ent).despawn();
        }
        level_timer.timer.reset();
    }
}

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSettings::default())
            .insert_resource(LevelTimer {
                timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            })
            .add_systems(
                Startup,
                setup_level_settings.in_set(LevelSet).after(WorldSet),
            )
            .add_systems(PreUpdate, update_timer)
            .add_systems(
                Update,
                (
                    remove_invisible_objects,
                    spawn_obstacles,
                    reset_on_collision,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(PostUpdate, reset_level);
    }
}
