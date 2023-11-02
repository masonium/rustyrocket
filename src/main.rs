use bevy::{prelude::*, window::close_on_esc};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{LoadingState, LoadingStateAppExt},
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use rustyrocket::{obstacle::{self, ObstaclePlugin, new_obstacle, ObstacleMaterials, Obstacle}, PhysicsSettings};

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

#[derive(Component)]
struct Player;

#[derive(Component, Reflect, PartialEq)]
struct PlayerAnim {
    tick: f32,
    state: PlayerState,
}


const JUMP_ANIM_FRAMES: u32 = 4;
const JUMP_ANIM_TIME: f32 = 0.1;

fn setup_camera(mut commands: Commands) {
    let camera = Camera2dBundle::default();

    commands.spawn(camera);
}

fn setup_physics(mut physics: ResMut<PhysicsSettings>,
		 mut rapier_config: ResMut<RapierConfiguration>,
		 window: Query<&Window>,
) {
    let w = window.single();
    rapier_config.gravity = Vec2::new(0.0, -500.0);

    physics.bounds.max = Vec2::new(w.width() / 2.0, w.height() / 2.0);
    physics.bounds.min = -physics.bounds.max;
}


fn setup(
    mut commands: Commands,
    sprites: Res<SpriteCollection>,
) {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                index: 0,
                ..default()
            },
	    transform: Transform::from_translation(Vec3::new(200.0, 0.0, 0.0)),
            texture_atlas: sprites.player_atlas.clone(),
            ..default()
        },
        PlayerAnim {
            tick: 0.0,
            state: PlayerState::Jumping,
        },
	Player,
	Collider::cuboid(20.0, 28.0),
	RigidBody::Dynamic,
	Velocity::default(),
	Sensor,
	//ActiveEvents::COLLISION_EVENTS,
        Name::new("Player"),
    ));
}

fn spawn_first_obstacle(meshes: ResMut<Assets<Mesh>>,
			mut commands: Commands,
			physics: Res<PhysicsSettings>,
			obs_mat: Res<ObstacleMaterials>,
) {
    commands.spawn(new_obstacle(true, 200.0, meshes, physics, obs_mat));
}

fn set_first_obstacle(mut query: Query<&mut Transform, With<Obstacle>>) {
    println!("updating");
    for mut t in  query.iter_mut() {
	t.translation.x = 300.0;
    }
}

fn handle_input(
    mut player: Query<(&mut PlayerAnim, &mut Velocity)>,
    keys: Res<Input<KeyCode>>,
    physics: Res<PhysicsSettings>,
) {
    let (mut p, mut v) = player.single_mut();
    if keys.just_pressed(KeyCode::Space) && p.state != PlayerState::Jumping {
        p.state = PlayerState::Jumping;
        v.linvel = physics.jump_vector;
    }
}

fn update_anim(mut player: Query<(&mut PlayerAnim, &mut TextureAtlasSprite)>, time: Res<Time>) {
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

fn update_position(
    mut player: Query<&mut Transform, With<Player>>,
    physics: Res<PhysicsSettings>,
) {
    for mut trans in player.iter_mut() {
        // trans.translation += moveable.linvel * time.delta_seconds();
        if !physics.bounds.contains(trans.translation.truncate()) {
            trans.translation = Vec3::ZERO;
        }
    }
}

fn on_changed(
    query: Query<(Entity, &Name, &Transform), Changed<Transform>> 
) {
    for q in query.iter() { 
	println!("{} changed to {:?}",q.1, q.2.translation);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(96.0),
		      RapierDebugRenderPlugin::default()))
        .add_plugins(ObstaclePlugin)
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
        .add_systems(Startup, (setup_camera, setup_physics))
        .add_systems(OnEnter(GameState::Playing), ((setup, spawn_first_obstacle), set_first_obstacle).chain())
        .add_systems(Update, close_on_esc)
        .add_systems(
            Update,
            (on_changed, update_anim, handle_input, update_position)
                .run_if(in_state(GameState::Playing)),
        )
        .run()
}
