pub mod obstacle;

use bevy::prelude::*;

#[derive(Resource, Reflect)]
pub struct PhysicsSettings {
    pub jump_vector: Vec2,
    pub bounds: Rect,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
	Self {
	    jump_vector: Vec2::new(0.0, 300.0),
	    bounds: Rect::default(),
	}
    }
}
