use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{WorldSettings, level::LevelSettings, player::Player};

/// Marker trait for obstacles.
#[derive(Component, Reflect)]
pub struct Obstacle;

#[derive(Resource, Default, Reflect)]
pub struct ObstacleMaterials {
    /// material to use when colliding
    enter: Handle<ColorMaterial>,

    /// material to use when not colliding
    exit: Handle<ColorMaterial>,
}

/// Event spawned when the player hits an obstacle.
#[derive(Event)]
pub struct HitObstacleEvent {
    obstacle: Entity,

    /// location of the player when the collision occurred.
    hit_location: Vec2,
}

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
    obs_mat: &Res<ObstacleMaterials>,
) -> impl Bundle {
    let width = level_settings.obstacle_width;
    let b = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(width, height))));
    let c = obs_mat.exit.clone();

    let top_mult = if from_top { 1.0 } else { -1.0 };
    let center_y = play_world.bounds.max.y - height / 2.0;

    (
        MaterialMesh2dBundle {
            mesh: b.into(),
            material: c,
            transform: Transform::from_translation(Vec3::new(level_settings.start_offset + width / 2.0, 
							     center_y * top_mult, 0.0)),
            ..default()
        },
        Obstacle,
        Collider::cuboid(width / 2.0, height / 2.0),
        RigidBody::KinematicVelocityBased,
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    )
}

/// Setup the collection of materials used for obstacles.
fn setup_obstacle_materials(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut obs_mat: ResMut<ObstacleMaterials>,
) {
    obs_mat.enter = materials.add(ColorMaterial {
        color: Color::from(Color::rgba(0.6, 0.2, 0.0, 1.0)),
        ..default()
    });
    obs_mat.exit = materials.add(ColorMaterial {
        color: Color::from(Color::rgba(0.6, 0.6, 0.0, 1.0)),
        ..default()
    });
}

fn react_to_obstacle_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut hit_events: EventWriter<HitObstacleEvent>,
    obs_mat: Res<ObstacleMaterials>,
    query: Query<(Entity, Option<&RegionRef>), With<Obstacle>>,
    player_q: Query<&Transform, With<Player>>,
) {

    for event in events.iter() {
        match event {
            CollisionEvent::Started(a, b, _) => {
                for entity in [a, b] {
                    if let Ok(ent) = query.get(*entity) {
			// change the color of the event during the collision
                        commands.entity(ent.0).remove::<Handle<ColorMaterial>>();
                        commands.entity(ent.0).insert(obs_mat.enter.clone());

			// send the event that an obstacle as hit.
			hit_events.send(HitObstacleEvent { 
			    obstacle: ent.0, 
			    hit_location: player_q.single().translation.truncate()
			});

			// Remove any scoring regions from the parent
			if let Some(rr) = ent.1 {
			    if let Some(mut region) = commands.get_entity(rr.region) {
				region.despawn();
			    }
			}
                    }
                }
            }
            CollisionEvent::Stopped(a, b, _) => {
                for entity in [a, b] {
                    if let Ok(ent) = query.get(*entity) {
                        commands.entity(ent.0).remove::<Handle<ColorMaterial>>();
                        commands.entity(ent.0).insert(obs_mat.exit.clone());
                    }
                }
            }
        }
    }
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<ObstacleMaterials>(ObstacleMaterials::default())
            .register_type::<ObstacleMaterials>()
            .register_type::<Obstacle>()
            .add_event::<HitObstacleEvent>()
            .add_systems(Startup, setup_obstacle_materials)
            .add_systems(Update, (react_to_obstacle_collision,));
    }
}
