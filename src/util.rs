use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tweening::Lens;

pub struct LinearVelocityLens {
    pub start_linvel: Vec2,
    pub end_linvel: Vec2,
}

impl Lens<Velocity> for LinearVelocityLens {
    fn lerp(&mut self, target: &mut Velocity, ratio: f32) {
        target.linvel = self.start_linvel * (1.0 - ratio) + self.end_linvel * ratio;
    }
}
