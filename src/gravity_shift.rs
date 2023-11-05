//! Gravity shifting 'obstacle. When the user runs into it, their gravity is shifted in teh corresponding direction.
use crate::{level::LevelSettings, GameState, WorldSettings};
use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        render_resource::{AddressMode, AsBindGroup, SamplerDescriptor},
        texture::ImageSampler,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;

/// Sent when the player hits a gravity event.
#[derive(Event, Reflect)]
pub struct GravityEvent {
    region: Entity,

    /// new applied gravity scale
    direction: f32,
}

/// Textures for gravity assets
#[derive(Resource, AssetCollection)]
pub struct GravityAssets {
    #[asset(path = "images/grav_arrow_down.png")]
    arrow: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct GravityMaterials {
    scrolling_down_mat: Handle<ScrollingMaterial>,
    scrolling_up_mat: Handle<ScrollingMaterial>,

    mesh: Handle<Mesh>,
}

#[derive(AsBindGroup, Clone, TypeUuid, TypePath, Debug)]
#[uuid = "313dfd8f-51a7-4cf2-a5f2-8b1491988974"]
pub struct ScrollingMaterial {
    #[texture(0)]
    #[sampler(1)]
    base_texture: Option<Handle<Image>>,

    #[uniform(2)]
    pub color: Color,
    #[uniform(3)]
    scroll_speed: f32,
    #[uniform(4)]
    scroll_direction: f32,
}

impl Material2d for ScrollingMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        println!("loading shaders/anim.wgsl");
        "shaders/anim.wgsl".into()
    }
}

fn setup_gravity_assets(
    mut images: ResMut<Assets<Image>>,
    grav_assets: Res<GravityAssets>,
    play_world: Res<WorldSettings>,
    mut materials: ResMut<Assets<ScrollingMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut level_settings: ResMut<LevelSettings>,
    mut grav_mat: ResMut<GravityMaterials>,
) {
    let image = images.get_mut(&grav_assets.arrow).unwrap();
    level_settings.gravity_width = image.texture_descriptor.size.width as f32;

    let mut sampler = SamplerDescriptor::default();
    sampler.address_mode_v = AddressMode::Repeat;
    image.sampler_descriptor = ImageSampler::Descriptor(sampler);

    grav_mat.scrolling_down_mat = materials.add(ScrollingMaterial {
        color: Color::RED,
        scroll_speed: 1.0,
        scroll_direction: -1.0,
        base_texture: Some(grav_assets.arrow.clone()),
    });

    grav_mat.scrolling_up_mat = materials.add(ScrollingMaterial {
        color: Color::BLUE,
        scroll_speed: 1.0,
        scroll_direction: 1.0,
        base_texture: Some(grav_assets.arrow.clone()),
    });

    let width = level_settings.gravity_width;
    let height = play_world.bounds.height();
    grav_mat.mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(width, height))));
}

/// Create a new gravity region.
pub fn new_gravity_region(
    down: bool,
    play_world: &Res<WorldSettings>,
    level: &Res<LevelSettings>,
    grav_mat: &Res<GravityMaterials>,
) -> impl Bundle {
    let width = level.gravity_width;
    let height = play_world.bounds.height();
    let q = grav_mat.mesh.clone();

    let material = if down {
        grav_mat.scrolling_down_mat.clone()
    } else {
        grav_mat.scrolling_up_mat.clone()
    };
    (
        MaterialMesh2dBundle {
            mesh: q.into(),
            material,
            transform: Transform::from_translation(Vec3::new(level.start_offset + width / 2.0, 0.0, 3.0)),
            ..default()
        },
        Collider::cuboid(width * 0.5, height * 0.5),
        Sensor,
        RigidBody::KinematicVelocityBased,
    )
}

pub struct GravityShiftPlugin;

impl Plugin for GravityShiftPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, GravityAssets>(GameState::AssetLoading)
            .add_plugins(Material2dPlugin::<ScrollingMaterial>::default())
            .insert_resource(GravityMaterials::default())
            .add_asset::<ScrollingMaterial>()
            .add_systems(
                OnEnter(GameState::Playing),
                setup_gravity_assets,
            );
    }
}
