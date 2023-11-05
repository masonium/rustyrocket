use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier2d::prelude::*;
use bevy_tweening::{lens::TransformRotateZLens, Animator, EaseFunction, Tween, TweenCompleted};

use crate::{
    gravity_shift::GravityEvent, level::LevelSettings, GameState, ResetEvent, WorldSettings,
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

#[derive(Component, Default, Reflect, PartialEq, Eq, Clone, Copy, Debug)]
enum PlayerRotTarget {
    #[default]
    Up,
    Down,
}

impl PlayerRotTarget {
    fn rot(&self) -> f32 {
        match self {
            PlayerRotTarget::Down => std::f32::consts::PI,
            PlayerRotTarget::Up => 0.0,
        }
    }
}

#[derive(Component, Reflect, PartialEq)]
struct PlayerAnim {
    tick: f32,
    state: PlayerState,
    rotation_target: PlayerRotTarget,
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
    mut commands: Commands,
    mut player: Query<(Entity, &mut Transform, &mut Velocity, &mut PlayerAnim), With<Player>>,
    mut reset_events: EventReader<ResetEvent>,
) {
    for _ in reset_events.iter() {
        let mut count = 0;
        for (ent, mut trans, mut vel, mut anim) in player.iter_mut() {
            count += 1;
            trans.translation = Vec3::ZERO;
            vel.linvel = Vec2::ZERO;
            anim.rotation_target = PlayerRotTarget::Up;
            trans.rotation.z = 0.0;
            if let Some(mut e) = commands.get_entity(ent) {
                e.remove::<Animator<Transform>>();
            }
        }
        println!("Reset {count} player.");
    }
}

/// Change the rotation based on a gravit multiplier.
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
            println!("rotation change on player with target {target_rotation:?}");
            if anim.rotation_target != target_rotation {
                // Delete an existing tweener, then add a new a
                // tweener. with the correct target.
                if animator.is_some() {
                    commands.entity(ent).remove::<Animator<Transform>>();
                }

                let current_rot = trans.rotation.z;
                let anim_time = (current_rot - target_rotation.rot()).abs() / std::f32::consts::PI
                    * ROTATION_TIME;

                let new_tween = Tween::new(
                    EaseFunction::QuadraticInOut,
                    Duration::from_secs_f32(anim_time),
                    TransformRotateZLens {
                        start: current_rot,
                        end: target_rotation.rot(),
                    },
                );

                println!(
                    "changing tween to ({} -> {}) in {anim_time:.2}s",
                    current_rot,
                    target_rotation.rot()
                );

                // Add a new animator with the target proper target.
                commands.entity(ent).insert(Animator::new(new_tween));

                anim.rotation_target = target_rotation;
            }
        }
    }
}

pub fn anim_complete(
    mut ev_reader: EventReader<TweenCompleted>,
    player_q: Query<(&Animator<Transform>, &Transform), With<Player>>,
) {
    for p in player_q.iter() {
        //println!("{}", p.0.tweenable().times_completed());
    }
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
            .add_collection_to_loading_state::<_, SpriteCollection>(GameState::AssetLoading)
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    update_anim,
                    handle_input,
                    keep_player_in_bounds,
                    rotate_player_on_gravity_change,
                    anim_complete,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(PostUpdate, reset_player);
    }
}
