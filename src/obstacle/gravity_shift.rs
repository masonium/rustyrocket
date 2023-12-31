//! Gravity shifting 'obstacle. When the user runs into it, their gravity is shifted in teh corresponding direction.
use crate::{level::LevelSettings, player::Player, GameState, WorldSettings};
use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        render_resource::AsBindGroup,
        texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;

/// Sent when the player hits a gravity event.
#[derive(Event, Reflect)]
pub struct GravityEvent {
    region: Entity,

    /// new gravity multipler to set in settings
    pub gravity_mult: f32,
}

#[derive(Default, Resource)]
pub struct GravityMaterials {
    scrolling_down_mat: Handle<GravityShiftMaterial>,
    scrolling_up_mat: Handle<GravityShiftMaterial>,

    mesh: Handle<Mesh>,
}

/// Textures for rendering gravity regions.
#[derive(Resource, AssetCollection)]
struct GravityAssets {
    #[asset(path = "images/grav_arrow_down.png")]
    arrow: Handle<Image>,
}

#[derive(Component, Resource)]
struct GravityRegion(f32);

#[derive(AsBindGroup, Clone, TypeUuid, TypePath, Debug, Asset)]
#[uuid = "313dfd8f-51a7-4cf2-a5f2-8b1491988974"]
struct GravityShiftMaterial {
    #[texture(0)]
    #[sampler(1)]
    base_texture: Option<Handle<Image>>,

    #[uniform(2)]
    pub color: Color,
    #[uniform(3)]
    scroll_speed: f32,
    #[uniform(4)]
    scroll_direction: f32,
    #[uniform(5)]
    texture_y_mult: f32,
}

impl Material2d for GravityShiftMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        println!("loading shaders/anim.wgsl");
        "shaders/anim.wgsl".into()
    }
}

fn setup_gravity_assets(
    mut images: ResMut<Assets<Image>>,
    grav_assets: Res<GravityAssets>,
    play_world: Res<WorldSettings>,
    mut materials: ResMut<Assets<GravityShiftMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut grav_mat: ResMut<GravityMaterials>,
) {
    let image = images.get_mut(&grav_assets.arrow).unwrap();
    let width = 32.0;

    let sampler = ImageSamplerDescriptor {
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    };

    image.sampler = ImageSampler::Descriptor(sampler);

    let texture_y_mult = play_world.bounds.height() / width;

    grav_mat.scrolling_down_mat = materials.add(GravityShiftMaterial {
        color: Color::RED,
        scroll_speed: 1.0,
        scroll_direction: -1.0,
        base_texture: Some(grav_assets.arrow.clone()),
        texture_y_mult,
    });

    grav_mat.scrolling_up_mat = materials.add(GravityShiftMaterial {
        color: Color::BLUE,
        scroll_speed: 1.0,
        scroll_direction: 1.0,
        base_texture: Some(grav_assets.arrow.clone()),
        texture_y_mult,
    });

    let height = play_world.bounds.height();
    grav_mat.mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(width, height))));
}

/// Create a new gravity region.
pub fn new_gravity_region(
    new_gravity_mult: f32,
    start_x: f32,
    width: f32,
    play_world: &Res<WorldSettings>,
    grav_mat: &Res<GravityMaterials>,
) -> impl Bundle {
    let down = new_gravity_mult > 0.0;

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
            transform: Transform::from_xyz(start_x, 0.0, 3.0),
            ..default()
        },
        Collider::cuboid(width * 0.5, height * 0.5),
        Sensor,
        RigidBody::KinematicVelocityBased,
        GravityRegion(new_gravity_mult),
    )
}

/// Check for player interactions with any active gravity regions.
fn check_gravity_region_collisions(
    mut commands: Commands,
    rapier: Res<RapierContext>,
    regions: Query<(Entity, &GravityRegion)>,
    player_q: Query<(Entity, &Player)>,
    mut gevs: EventWriter<GravityEvent>,
) {
    for player in player_q.iter() {
        for (region_entity, region) in regions.iter() {
            if rapier.intersection_pair(player.0, region_entity) == Some(true) {
                // send a gravity changing event.
                gevs.send(GravityEvent {
                    region: region_entity,
                    gravity_mult: region.0,
                });

                // kill the gravity region marker so we don't keep sending events.
                commands.entity(region_entity).remove::<GravityRegion>();
            }
        }
    }
}

/// Change the level gravity mult.
fn on_gravity_event(
    mut level: ResMut<LevelSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut gevs: EventReader<GravityEvent>,
) {
    for ev in gevs.read() {
        level.gravity_mult = ev.gravity_mult;
        rapier_config.gravity = level.gravity_vector();
    }
}

pub struct GravityShiftPlugin;

impl Plugin for GravityShiftPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, GravityAssets>(GameState::AssetLoading)
            .add_plugins(Material2dPlugin::<GravityShiftMaterial>::default())
            .insert_resource(GravityMaterials::default())
            .add_event::<GravityEvent>()
            .add_systems(OnEnter(GameState::Playing), setup_gravity_assets)
            .add_systems(
                Update,
                (check_gravity_region_collisions, on_gravity_event)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
