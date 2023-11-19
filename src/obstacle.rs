use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{level::LevelSettings, WorldSettings};

/// Marker trait for obstacles.
#[derive(Component, Reflect)]
pub struct Obstacle;

#[derive(Resource, Default, Reflect)]
pub struct ObstacleAssets {
    /// basic quad mesh
    base_mesh: Handle<Mesh>,

    /// material to use when colliding
    enter_mat: Handle<ColorMaterial>,

    /// material to use when not colliding
    exit_mat: Handle<ColorMaterial>,
}

/// Event spawned when the player hits an obstacle.
#[derive(Event, Default)]
pub struct HitObstacleEvent;

#[derive(Component, Reflect)]
pub struct RegionRef {
    pub region: Entity,
}

/// Spawn an obstacle bundle off-screen, moving left.
pub fn new_obstacle(
    from_top: bool,
    height: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    play_world: &Res<WorldSettings>,
    level_settings: &Res<LevelSettings>,
    obs_mat: &Res<ObstacleAssets>,
) -> impl Bundle {
    let width = level_settings.obstacle_width;
    //let height = 100.0; //play_world.bounds.height();
    let b = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(width, height))));
    let c = obs_mat.exit_mat.clone();

    let top_mult = if from_top { 1.0 } else { -1.0 };
    let center_y = play_world.bounds.max.y - height / 2.0;
    (
        MaterialMesh2dBundle {
            mesh: b.into(),
            material: c,
            transform: Transform {
                translation: Vec3::new(
                    level_settings.start_offset + width / 2.0,
                    center_y * top_mult,
                    2.0,
                ),
                ..default()
            },
            ..default()
        },
        Obstacle,
        Collider::cuboid(width / 2.0, height / 2.0),
        ColliderMassProperties::Density(1.0),
        RigidBody::KinematicVelocityBased,
        //Sensor,
        ActiveEvents::COLLISION_EVENTS,
    )
}

/// Setup the collection of materials used for obstacles.
fn setup_obstacle_assets(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    level: Res<LevelSettings>,
    mut obs_mat: ResMut<ObstacleAssets>,
) {
    let height = play_world.bounds.height() / 2.0;
    let quad_dim = Vec2::new(level.obstacle_width, height);
    obs_mat.base_mesh = meshes.add(Mesh::from(shape::Quad::new(quad_dim)));

    obs_mat.enter_mat = materials.add(ColorMaterial {
        color: Color::rgba(0.6, 0.2, 0.0, 1.0),
        ..default()
    });
    obs_mat.exit_mat = materials.add(ColorMaterial {
        color: Color::rgba(0.6, 0.6, 0.0, 1.0),
        ..default()
    });
}

fn react_to_obstacle_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut hit_events: EventWriter<HitObstacleEvent>,
    query: Query<(Entity, Option<&RegionRef>), With<Obstacle>>,
) {
    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = event {
            for entity in [a, b] {
                if let Ok(ent) = query.get(*entity) {
                    // send the event that an obstacle as hit.
                    hit_events.send(HitObstacleEvent);

                    // Remove any scoring regions from the parent
                    if let Some(rr) = ent.1 {
                        if let Some(mut region) = commands.get_entity(rr.region) {
                            region.despawn();
                        }
                    }
                }
            }
        }
    }
}

/// Plugin for setting up obstacle-related systems and resources.
pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<ObstacleAssets>(ObstacleAssets::default())
            .register_type::<ObstacleAssets>()
            .register_type::<Obstacle>()
            .add_event::<HitObstacleEvent>()
            .add_systems(Startup, setup_obstacle_assets)
            .add_systems(Update, (react_to_obstacle_collision,));
    }
}
