use bevy::prelude::Component;

pub mod barrier;
pub mod gravity_shift;
pub mod spawner_settings;

#[derive(Component)]
pub struct Obstacle;
