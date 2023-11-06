use bevy::prelude::*;

use crate::ResetEvent;

#[derive(Resource, Default, Reflect)]
pub struct Score {
    pub score: i32,
}

fn reset_score(mut score: ResMut<Score>) {
    score.score = 0;
}

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Score>()
            .insert_resource(Score::default())
            .add_systems(Update, reset_score.run_if(on_event::<ResetEvent>()));
    }
}
