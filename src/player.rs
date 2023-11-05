use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;

use crate::{GameState, ResetEvent, WorldSettings, level::LevelSettings};

const JUMP_ANIM_FRAMES: u32 = 4;
const JUMP_ANIM_TIME: f32 = 0.1;

#[derive(Resource, AssetCollection)]
struct SpriteCollection {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 4, rows = 1))]
    #[asset(path = "images/rocketman.png")]
    player_atlas: Handle<TextureAtlas>,
}

#[derive(Component, Reflect, PartialEq, Eq)]
enum PlayerState {
    Jumping,
    Falling,
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Reflect, PartialEq)]
struct PlayerAnim {
    tick: f32,
    state: PlayerState,
}

/// Create the initial player.
fn spawn_player(mut commands: Commands, sprites: Res<SpriteCollection>) {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                index: 0,
                ..default()
            },
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
        Name::new("Player"),
    ));
}

/// Handle jumping inputs for the player.
fn handle_input(
    mut player: Query<(&mut PlayerAnim, &mut Velocity)>,
    keys: Res<Input<KeyCode>>,
    level: Res<LevelSettings>,
) {
    let (mut p, mut v) = player.single_mut();
    if keys.just_pressed(KeyCode::Space) && p.state != PlayerState::Jumping {
        p.state = PlayerState::Jumping;
        v.linvel = level.jump_vector();
    }
}

/// Update the animation state of the player based on its action state.
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

/// Move the player back to the center of the play window if the
/// player center leaves the environment bounding box.
fn keep_player_in_bounds(
    mut player: Query<&mut Transform, With<Player>>,
    play_world: Res<WorldSettings>,
) {
    for mut trans in player.iter_mut() {
        // trans.translation += moveable.linvel * time.delta_seconds();
        if !play_world.bounds.contains(trans.translation.truncate()) {
            trans.translation = Vec3::ZERO;
        }
    }
}

/// Reset the player position.
fn reset_player(
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
    mut reset_events: EventReader<ResetEvent>,
) {
    for _ in reset_events.iter() {
        for (mut trans, mut vel) in player.iter_mut() {
            trans.translation = Vec3::ZERO;
            vel.linvel = Vec2::ZERO;
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            .register_type::<PlayerAnim>()
            .add_collection_to_loading_state::<_, SpriteCollection>(GameState::AssetLoading)
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (update_anim, handle_input, keep_player_in_bounds)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(PostUpdate, reset_player);
    }
}
