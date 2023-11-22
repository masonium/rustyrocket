use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::WorldSettings;

/// Marker trait for obstacles.
#[derive(Component, Reflect)]
pub struct Barrier;

#[derive(Resource, Default, Reflect)]
pub struct BarrierAssets {
    /// basic quad mesh
    base_mesh: Handle<Mesh>,

    /// material to use when colliding
    enter_mat: Handle<ColorMaterial>,

    /// material to use when not colliding
    exit_mat: Handle<ColorMaterial>,
}

/// Event spawned when the player hits an obstacle.
#[derive(Event, Default)]
pub struct HitBarrierEvent;

#[derive(Component, Reflect)]
pub struct RegionRef {
    pub region: Entity,
}

/// Spawn an barrier bundle off-screen
pub fn new_barrier(
    from_top: bool,
    width: f32,
    height: f32,
    start_x: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    play_world: &Res<WorldSettings>,
    obs_mat: &Res<BarrierAssets>,
) -> impl Bundle {
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
                translation: Vec3::new(start_x, center_y * top_mult, 2.0),
                scale: Vec3::new(1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        Barrier,
        Collider::cuboid(width / 2.0, height / 2.0),
        ColliderMassProperties::Density(1.0),
        RigidBody::KinematicVelocityBased,
        //Sensor,
        ActiveEvents::COLLISION_EVENTS,
    )
}

/// Setup the collection of materials used for obstacles.
fn setup_barrier_assets(
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    play_world: Res<WorldSettings>,
    mut obs_mat: ResMut<BarrierAssets>,
) {
    let height = play_world.bounds.height() / 2.0;
    let quad_dim = Vec2::new(1.0, height);
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

fn react_to_barrier_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut hit_events: EventWriter<HitBarrierEvent>,
    query: Query<(Entity, Option<&RegionRef>), With<Barrier>>,
) {
    for event in events.read() {
        if let CollisionEvent::Started(a, b, _) = event {
            for entity in [a, b] {
                if let Ok(ent) = query.get(*entity) {
                    // send the event that a barrier as hit.
                    hit_events.send(HitBarrierEvent);

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
pub struct BarrierPlugin;

impl Plugin for BarrierPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource::<BarrierAssets>(BarrierAssets::default())
            .register_type::<BarrierAssets>()
            .register_type::<Barrier>()
            .add_event::<HitBarrierEvent>()
            .add_systems(Startup, setup_barrier_assets)
            .add_systems(Update, (react_to_barrier_collision,));
    }
}
