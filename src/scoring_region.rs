use crate::player::Player;
use crate::GameState;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::score::Score;

#[derive(Component, Reflect)]
pub struct ScoringRegion {
    score_delta: i32,
}

/// A scoring region is an area that can change your score by the specify amount.
pub fn new_scoring_region(score_delta: i32, offset: Vec2, dim: Vec2) -> impl Bundle {
    (
        ScoringRegion { score_delta },
        SpatialBundle {
            transform: Transform::from_translation(offset.extend(0.0)),
            ..default()
        },
        Collider::cuboid(dim.x * 0.5, dim.y * 0.5),
        Sensor,
        RigidBody::KinematicVelocityBased,
        ActiveEvents::COLLISION_EVENTS,
        Name::new("scoring_region"),
    )
}

/// Increment the score and despawn the region when intersecting.
fn check_scoring_region_collisions(
    mut commands: Commands,
    rapier: Res<RapierContext>,
    regions: Query<(Entity, &ScoringRegion)>,
    mut score: ResMut<Score>,
    player_q: Query<(Entity, &Player)>,
) {
    let player = player_q.single();
    for (region_entity, region) in regions.iter() {
        if rapier.intersection_pair(player.0, region_entity) == Some(true) {
            score.score += region.score_delta;

            // despawn the region, so this only happens once
            commands.entity(region_entity).despawn();
        }
    }
}

pub struct ScoringRegionPlugin;
impl Plugin for ScoringRegionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScoringRegion>().add_systems(
            Update,
            check_scoring_region_collisions.run_if(in_state(GameState::Playing)),
        );
    }
}
