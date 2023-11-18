use std::time::Duration;

use bevy::{ecs::schedule::AnonymousSet, prelude::*};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{lens::TransformRotationLens, Animator, EaseFunction, Tween};

use crate::{
    gravity_shift::GravityEvent, level::LevelSettings, GameState, LevelSet, ResetEvent,
    WorldSettings,
};

const JUMP_ANIM_FRAMES: u32 = 4;
const JUMP_ANIM_TIME: f32 = 0.1;

pub const PLAYER_SCALE: f32 = 2.0;

/// Time in seconds to complete a full rotation.
const ROTATION_TIME: f32 = 0.25;

#[derive(Resource, AssetCollection)]
struct PlayerSprites {
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct PlayerSet;

#[derive(Component, Default, Reflect, PartialEq, Eq, Clone, Copy, Debug)]
enum PlayerRotTarget {
    #[default]
    Up,
    Down,
}

impl PlayerRotTarget {
    fn rot(&self) -> Quat {
        match self {
            PlayerRotTarget::Down => Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI),
            PlayerRotTarget::Up => Quat::from_axis_angle(Vec3::Z, 0.0),
        }
    }
}

#[derive(Component, Reflect, PartialEq)]
struct PlayerAnim {
    tick: f32,
    state: PlayerState,
    rotation_target: PlayerRotTarget,
}

#[derive(Event)]
pub struct OutOfBoundsEvent;

/// Create the initial player.
fn spawn_player(
    mut commands: Commands,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<PlayerSprites>,
) {
    let r = atlases.get(&sprites.player_atlas).unwrap();
    let cs = r.textures[0].size() * PLAYER_SCALE;
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                custom_size: Some(cs),
                index: 0,
                ..default()
            },
	    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
            texture_atlas: sprites.player_atlas.clone(),
            ..default()
        },
        PlayerAnim {
            tick: 0.0,
            state: PlayerState::Jumping,
            rotation_target: PlayerRotTarget::Up,
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
    for (mut p, mut v) in player.iter_mut() {
        if keys.just_pressed(KeyCode::Space) && p.state != PlayerState::Jumping {
            p.state = PlayerState::Jumping;
            v.linvel = level.jump_vector();
        }
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
fn signal_player_out_of_bounds(
    mut player: Query<&mut Transform, With<Player>>,
    mut oob: EventWriter<OutOfBoundsEvent>,
    play_world: Res<WorldSettings>,
) {
    for trans in player.iter_mut() {
        if !play_world.bounds.contains(trans.translation.truncate()) {
            oob.send(OutOfBoundsEvent);
        }
    }
}


/// System to kill and spawn the player.
fn respawn_player(
    mut commands: Commands,
    atlases: Res<Assets<TextureAtlas>>,
    sprites: Res<PlayerSprites>,
    player: Query<Entity, With<Player>>,
) {
    for ent in player.iter() {
        commands.entity(ent).despawn();
    }
    spawn_player(commands, atlases, sprites);
}

/// Change the rotation based on a gravity multiplier.
fn rotate_player_on_gravity_change(
    mut commands: Commands,
    mut player_q: Query<
        (
            Entity,
            &Transform,
            &mut PlayerAnim,
            Option<&Animator<Transform>>,
        ),
        With<Player>,
    >,
    mut gevs: EventReader<GravityEvent>,
) {
    // Check the current ratio, and see if we need to add a tweener.
    for ev in gevs.iter() {
        let target_rotation = if ev.gravity_mult > 0.0 {
            PlayerRotTarget::Up
        } else {
            PlayerRotTarget::Down
        };

        // check to see if we're already rotating to there
        for (ent, trans, mut anim, animator) in player_q.iter_mut() {
            if anim.rotation_target != target_rotation {
                // Delete an existing tweener, then add a new a
                // tweener. with the correct target.
                if animator.is_some() {
                    commands.entity(ent).remove::<Animator<Transform>>();
                }

                let current_rot = trans.rotation;
                let target_rot = target_rotation.rot();
                let anim_time = current_rot.angle_between(target_rot).abs() / std::f32::consts::PI
                    * ROTATION_TIME;

                let new_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(anim_time),
                    TransformRotationLens {
                        start: current_rot,
                        end: target_rotation.rot(),
                    },
                );

                // Add a new animator with the target proper target.
                commands.entity(ent).insert(Animator::new(new_tween));

                anim.rotation_target = target_rotation;
            }
        }
    }
}

#[derive(Resource)]
pub struct DecomposedSprite {
    pub pixels: Vec<(Vec2, Color)>,
}

impl DecomposedSprite {
    fn from_img_rect(img: &Image, rect: Rect) -> anyhow::Result<DecomposedSprite> {
        let center = rect.center();
        let dynamic_image = img.clone().try_into_dynamic()?;
        let buf = dynamic_image.into_rgba8();
        Ok(DecomposedSprite {
            pixels: buf
                .enumerate_pixels()
                .filter(|(x, y, _)| rect.contains(Vec2::new(*x as f32, *y as f32)))
                .filter_map(|(x, y, c)| {
                    if c.0[3] != 0 {
                        Some((
                            Vec2::new(x as f32 - center.x, y as f32 - center.y),
                            Color::rgba_u8(c.0[0], c.0[1], c.0[2], c.0[3]),
                        ))
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}

/// Insert the decomposed version of the player sprite.
fn insert_decomposed_sprite(world: &mut World) {
    let atlas: &Assets<TextureAtlas> = world.get_resource().unwrap();
    let ps: &PlayerSprites = world.get_resource().unwrap();
    let images: &Assets<Image> = world.get_resource().unwrap();

    let ta = atlas.get(&ps.player_atlas).unwrap();
    // grab the image reprented by the first text
    let img = images.get(&ta.texture).unwrap();
    let rect = ta.textures[0];

    let ds = DecomposedSprite::from_img_rect(img, rect).unwrap();
    bevy::log::warn!("{}", ds.pixels.len());
    world.insert_resource(ds);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            .register_type::<PlayerAnim>()
            .add_event::<OutOfBoundsEvent>()
            .add_collection_to_loading_state::<_, PlayerSprites>(GameState::AssetLoading)
            .add_systems(OnExit(GameState::AssetLoading), insert_decomposed_sprite)
            .add_systems(OnEnter(GameState::Ready), respawn_player.after(LevelSet))
            .add_systems(
                Update,
                (
                    update_anim,
                    handle_input,
                    signal_player_out_of_bounds,
                    rotate_player_on_gravity_change,
                )
                    .in_set(PlayerSet)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
