use bevy::{
    prelude::{shape::Quad, *},
    reflect::{TypePath, TypeUuid},
    render::render_resource::AsBindGroup,
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_rapier2d::prelude::RapierConfiguration;

use crate::{GameState, WorldSet, WorldSettings};

#[derive(AsBindGroup, Clone, TypeUuid, TypePath, Debug, Asset)]
#[uuid = "476f30fe-bed3-4495-9603-aaedb35ba69b"]
struct BackgroundMaterial {
    #[uniform(0)]
    pub c1: Color,
    #[uniform(1)]
    pub c2: Color,

    #[uniform(2)]
    time: f32,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/background.wgsl".into()
    }
}

#[derive(Component)]
pub struct Background;

fn spawn_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<BackgroundMaterial>>,
    world: Res<WorldSettings>,
) {
    println!("world bounds: {}", world.bounds.size());
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(Quad::new(world.bounds.size())))
                .into(),
            material: mats.add(BackgroundMaterial {
                c1: Color::rgba(0.4, 0.4, 0.4, 1.0),
                c2: Color::rgba(0.7, 0.7, 0.7, 1.0),
                time: 0.0,
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Background,
    ));
}

fn update_background(
    mut mats: ResMut<Assets<BackgroundMaterial>>,
    back: Query<&Handle<BackgroundMaterial>, With<Background>>,
    rapier: Res<RapierConfiguration>,
    time: Res<Time>,
) {
    let mat_handle = back.single();
    let mat = mats.get_mut(mat_handle).unwrap();
    if rapier.physics_pipeline_active {
        mat.time += time.delta_seconds();
    }
}

fn reset_background(
    mut mats: ResMut<Assets<BackgroundMaterial>>,
    back: Query<&Handle<BackgroundMaterial>, With<Background>>,
) {
    let mat_handle = back.single();
    let mat = mats.get_mut(mat_handle).unwrap();
    mat.time = 0.0;
}

pub struct GameBackgroundPlugin;

impl Plugin for GameBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .add_systems(Startup, spawn_background.after(WorldSet))
            .add_systems(Update, update_background)
            .add_systems(OnEnter(GameState::Ready), reset_background);
    }
}
