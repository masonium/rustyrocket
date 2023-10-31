use bevy::{prelude::*, window::close_on_esc};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

#[derive(Resource, AssetCollection)]
struct SpriteCollection {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 4, rows = 1))]
    #[asset(path = "images/rocketman.png")]
    player_atlas: Handle<TextureAtlas>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    AssetLoading,
    Playing,
}

#[derive(Component, Reflect, PartialEq, Eq)]
enum PlayerState {
    Jumping,
    Falling,
}

#[derive(Component, Reflect, PartialEq)]
struct PlayerAnim {
    tick: f32,
    state: PlayerState,
}

#[derive(Default, Component, Reflect)]
struct Moveable {
    velocity: Vec3,
}

#[derive(Component, Reflect)]
struct Gravity;

#[derive(Resource, Reflect)]
pub struct PhysicsSettings {
    gravity: Vec3,
    jump_vector: Vec3,
    bounds: Rect,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self { gravity: Vec3::new(0.0, -200.0, 0.0), 
	       jump_vector: Vec3::new(0.0, 300.0, 0.0),
	       bounds: Rect::default(),
	}
    }
}


const JUMP_ANIM_FRAMES: u32 = 4;
const JUMP_ANIM_TIME: f32 = 0.1;

fn setup_camera(mut commands: Commands) {
    let camera = Camera2dBundle::default();
    
    commands.spawn(camera);
 }

fn setup(mut commands: Commands, 
	 images: Res<SpriteCollection>,
	 cameras: Query<(&Camera, &GlobalTransform)>,
	 mut physics: ResMut<PhysicsSettings>) {

    let (camera, camera_gt) = cameras.single();
    physics.bounds.min = camera.ndc_to_world(&camera_gt, Vec3::new(-1.0, -1.0, 0.0)).unwrap().truncate();
    physics.bounds.max = camera.ndc_to_world(&camera_gt, Vec3::new(1.0, 1.0, 0.0)).unwrap().truncate();
   
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
		index: 0,
                ..default()
            },
            texture_atlas: images.player_atlas.clone(),
            ..default()
        },
        PlayerAnim {
            tick: 0.0,
            state: PlayerState::Jumping,
        },
	Moveable::default(),
	Gravity,
	Name::new("Player"),
    ));
}

fn handle_input(mut player: Query<(&mut PlayerAnim, &mut Moveable)>, 
		keys: Res<Input<KeyCode>>,
		physics: Res<PhysicsSettings>) {
    let (mut p, mut m) = player.single_mut();
    if keys.just_pressed(KeyCode::Space) && p.state != PlayerState::Jumping {
	p.state = PlayerState::Jumping;
	m.velocity = physics.jump_vector;
    }
}

fn update_anim(mut player: Query<(&mut PlayerAnim, &mut TextureAtlasSprite)>,
	       time: Res<Time>) {
    for (mut anim, mut sprite) in player.iter_mut() {
	if anim.state == PlayerState::Jumping {
            anim.tick += time.delta_seconds() / JUMP_ANIM_TIME;
	} else {
	    anim.tick -= time.delta_seconds() / JUMP_ANIM_TIME;
	}
	anim.tick = anim.tick.clamp(0.0, 1.0);
	if anim.tick == 1.0 {
	    anim.state = PlayerState::Falling;
	}
	
	sprite.index = (anim.tick * (JUMP_ANIM_FRAMES - 1) as f32) as usize;
    }
}

fn update_gravity(mut objs: Query<&mut Moveable, With<Gravity>>,
		  time: Res<Time>,
		  physics: Res<PhysicsSettings>) {
    for mut moveable in objs.iter_mut() {
	moveable.velocity += physics.gravity * time.delta_seconds();
    }
}

fn update_position(mut player: Query<(&mut Transform, &Moveable)>,
		   time: Res<Time>,
		   physics:  Res<PhysicsSettings>) {
    for (mut trans, moveable) in player.iter_mut() {
	trans.translation += moveable.velocity * time.delta_seconds();
	if !physics.bounds.contains(trans.translation.truncate()) {
	    trans.translation = Vec3::ZERO;
	}
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .register_type::<PlayerState>()
        .register_type::<PlayerAnim>()
        .insert_resource(PhysicsSettings::default())
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<PhysicsSettings>::default())
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, SpriteCollection>(GameState::AssetLoading)
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Playing), setup)
        .add_systems(Update, close_on_esc)
        .add_systems(Update, (update_anim, handle_input, update_gravity, update_position).run_if(in_state(GameState::Playing)))
        .run()
}
