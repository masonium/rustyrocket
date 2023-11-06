use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{lens::TransformRotationLens, Animator, EaseFunction, Tween, TweenCompleted};

use crate::{
    gravity_shift::GravityEvent, level::LevelSettings, GameState, ResetEvent, WorldSettings, LevelSet,
};

const JUMP_ANIM_FRAMES: u32 = 4;
const JUMP_ANIM_TIME: f32 = 0.1;

/// Time in seconds to complete a full rotation.
const ROTATION_TIME: f32 = 0.25;

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
pub struct OutOfBounds;

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
fn signal_player_out_of_bounds(
    mut player: Query<&mut Transform, With<Player>>,
    mut oob: EventWriter<OutOfBounds>,
    play_world: Res<WorldSettings>,
) {
    for trans in player.iter_mut() {
        if !play_world.bounds.contains(trans.translation.truncate()) {
	    oob.send(OutOfBounds);
        }
    }
}

/// Reset the player position, animation, rotation state.
fn reset_player(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Velocity, &mut PlayerAnim), With<Player>>,
) {
    for (ent, mut trans, mut vel, mut anim) in player.iter_mut() {
        trans.translation = Vec3::ZERO;
        vel.linvel = Vec2::ZERO;
        anim.rotation_target = PlayerRotTarget::Up;
        trans.rotation.z = 0.0;
        if let Some(mut e) = commands.get_entity(ent) {
            e.remove::<Animator<Transform>>();
        }
    }
}

/// System to kill and spawn the player.
fn respawn_player(
    mut commands: Commands,
    sprites: Res<SpriteCollection>,
    player: Query<Entity, With<Player>>,) {
    for ent in player.iter() {
	commands.entity(ent).despawn();
    }
    spawn_player(commands, sprites);
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

/// System for when the animation is complete.
pub fn anim_complete(
    mut ev_reader: EventReader<TweenCompleted>,
    player_q: Query<(&Animator<Transform>, &Transform), With<Player>>,
) {
    for ev in ev_reader.iter() {
        println!("Completed animation");
        if let Ok(_) = player_q.get(ev.entity) {}
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerState>()
            .register_type::<PlayerAnim>()
            .add_event::<OutOfBounds>()
            .add_collection_to_loading_state::<_, SpriteCollection>(GameState::AssetLoading)
            .add_systems(OnEnter(GameState::Ready), respawn_player.after(LevelSet))
            .add_systems(Update, (
                    update_anim,
                    handle_input,
                    signal_player_out_of_bounds,
                    rotate_player_on_gravity_change,
                    anim_complete,
            ).in_set(PlayerSet).run_if(in_state(GameState::Playing)))
            .add_systems(PostUpdate, reset_player.run_if(on_event::<ResetEvent>()));
    }
}
