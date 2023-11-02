use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::PhysicsSettings;

#[derive(Component, Reflect)]
pub struct Obstacle;

#[derive(Resource, Default, Reflect)]
pub struct ObstacleMaterials {
    /// material to use when colliding
    enter: Handle<ColorMaterial>,

    /// material to use when not colliding
    exit: Handle<ColorMaterial>,
}

pub fn new_obstacle(
    from_top: bool,
    height: f32,
    mut meshes: ResMut<Assets<Mesh>>,
    physics: Res<PhysicsSettings>,
    obs_mat: Res<ObstacleMaterials>) -> impl Bundle {

    let size = 64.0;
    let b = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(size, height))));
    let c = obs_mat.exit.clone();

    let top_mult = if from_top { 1.0 } else { 1.0 };
    let center_height = physics.bounds.max.y - height / 2.0;
    println!("Creating at {:?} {}", physics.bounds, center_height);

    ( MaterialMesh2dBundle {
	mesh: b.into(),
	material: c,
	transform: Transform::from_translation(Vec3::new(0.0, center_height * top_mult, 0.0)),
	..default()
    },
      Obstacle,
      Collider::cuboid(size / 2.0, height / 2.0),
      RigidBody::Fixed,
      Sensor,
      //Velocity { linvel: Vec2::new(-200.0, 0.0), ..default() },
      //GravityScale(0.1),
      ActiveEvents::COLLISION_EVENTS,
      Name::new("obstacle"),
    )
}

/// Setup the collection of materials used for obstacles.
fn setup_obstacle_materials(mut materials: ResMut<Assets<ColorMaterial>>,
			    mut obs_mat: ResMut<ObstacleMaterials>) {
    obs_mat.enter = materials.add(ColorMaterial {
	color: Color::from(Color::rgba(0.6, 0.2, 0.0, 1.0)),
	..default()
    });
    obs_mat.exit = materials.add(ColorMaterial {
	color: Color::from(Color::rgba(0.6, 0.6, 0.0, 1.0)),
	..default()
    });
}
fn change_color_on_collision(mut commands: Commands,
			     mut events: EventReader<CollisionEvent>,
			     obs_mat: Res<ObstacleMaterials>,
			     query: Query<Entity, With<Obstacle>> ) {
    for event in events.iter() {
	println!("{event:?}");
	match event {
	    CollisionEvent::Started(a, b, _) => {
		for entity in [a, b] {
		    if let Ok(ent) = query.get(*entity) {
			commands.entity(ent).remove::<Handle<ColorMaterial>>();
			commands.entity(ent).insert(obs_mat.enter.clone());
		    }
		}
	    }
	    CollisionEvent::Stopped(a, b, _) => {
		for entity in [a, b] {
		    if let Ok(ent) = query.get(*entity) {
			commands.entity(ent).remove::<Handle<ColorMaterial>>();
			commands.entity(ent).insert(obs_mat.exit.clone());
		    }
		}
	    }
	}
    }
}


fn reset_obstacles(mut query: Query<&mut Transform, With<Obstacle>>,
		   keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::R) {
	for mut t in query.iter_mut() {
	    t.translation.x = 0.0;
	}
    }
}

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
	app.insert_resource::<ObstacleMaterials>(ObstacleMaterials::default())
	    .register_type::<ObstacleMaterials>()
	    .register_type::<Obstacle>()
	    .add_systems(Startup, setup_obstacle_materials)
	    .add_systems(Update, (reset_obstacles, change_color_on_collision));
    }
}
