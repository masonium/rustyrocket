//! Methods around spawning dying player data.

use rand::Rng;
use std::time::Duration;

use crate::{
    barrier::HitBarrierEvent,
    level::LevelSettings,
    player::{DecomposedSprite, OutOfBoundsEvent, Player, PLAYER_SCALE},
    GameState, ResetEvent,
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct PlayerDeathAnim {
    death_time: Timer,
}

#[derive(Component)]
pub struct PlayerDeathPiece;

/// Update the timer, and change the state when it ends
pub fn update_death_timer(
    mut da: Query<&mut PlayerDeathAnim>,
    mut resets: EventWriter<ResetEvent>,
    time: Res<Time>,
) {
    for mut pda in da.iter_mut() {
        pda.death_time.tick(time.delta());
        if pda.death_time.just_finished() {
            resets.send(ResetEvent);
            break;
        }
    }
}

pub fn kill_death_anim(
    mut commands: Commands,
    da: Query<Entity, With<PlayerDeathAnim>>,
    dap: Query<Entity, With<PlayerDeathPiece>>,
) {
    for ent in da.iter() {
        commands.entity(ent).despawn();
    }
    for ent in dap.iter() {
        commands.entity(ent).despawn();
    }
}

pub fn explode_player(
    mut commands: Commands,
    player: Query<(Entity, &Transform, &Velocity), With<Player>>,
    level: Res<LevelSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    ds: Res<DecomposedSprite>,
) {
    let mut rng = rand::thread_rng();
    // Get the existing player
    for (ent, t, v) in player.iter() {
        let trans = t.translation;
        // spawn the sprites around the velocity
        commands.spawn((PlayerDeathAnim {
            death_time: Timer::new(Duration::from_secs(3), TimerMode::Once),
        },));
        for pix in &ds.pixels {
            let rand_dir = Vec2::from_angle(rng.gen::<f32>() * std::f32::consts::TAU);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::splat(2.0)),
                        color: pix.1,
                        ..default()
                    },
                    transform: Transform::from_translation(
                        trans + t.rotation * (-pix.0 * PLAYER_SCALE).extend(10.0),
                    ),
                    ..default()
                },
                Velocity {
                    linvel: v.linvel + rand_dir * level.explosion_speed,
                    ..default()
                },
                RigidBody::Dynamic,
                PlayerDeathPiece,
                Collider::cuboid(PLAYER_SCALE / 2.0, PLAYER_SCALE / 2.0),
                ColliderMassProperties::Density(1.0),
            ));
        }
        commands.entity(ent).despawn_recursive();
    }
    next_state.set(GameState::Dying);
}

pub struct DyingPlayerPlugin;

impl Plugin for DyingPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                explode_player.run_if(on_event::<HitBarrierEvent>()),
                explode_player.run_if(on_event::<OutOfBoundsEvent>()),
                update_death_timer.run_if(in_state(GameState::Dying)),
            ),
        )
        .add_systems(OnExit(GameState::Dying), kill_death_anim);
    }
}
