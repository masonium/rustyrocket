use bevy::prelude::*;

use crate::ResetEvent;

#[derive(Resource, Default, Reflect)]
pub struct Score {
    pub score: i32,
}

fn reset_score(mut resets: EventReader<ResetEvent>,
	       mut score: ResMut<Score>) {
    for _ in resets.iter() {
	score.score = 0;
    }
}

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
	app.register_type::<Score>()
	    .insert_resource(Score::default())
	    .add_systems(PostUpdate, reset_score);
    }
}
