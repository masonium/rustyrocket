use bevy::{prelude::*, sprite::Anchor};

use crate::{fonts::FontsCollection, score::Score, GameState, WorldSettings};

pub struct ScoreDisplayPlugin;

#[derive(Component)]
struct ScoreDisplay;

fn setup_score(mut commands: Commands, world: Res<WorldSettings>, fonts: Res<FontsCollection>) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "Score: 000".to_string(),
                TextStyle {
                    font: fonts.score_font.clone(),
                    font_size: 24.0,
                    color: Color::BLACK,
                },
            ),
            text_anchor: Anchor::TopLeft,
            transform: Transform::from_translation(Vec3::new(
                world.bounds.min.x + 12.0,
                world.bounds.max.y,
                10.0,
            )),
            ..default()
        },
        ScoreDisplay,
    ));
}

/// System to update the score display.
fn update_score(score: ResMut<Score>, mut query: Query<&mut Text, With<ScoreDisplay>>) {
    if score.is_changed() {
        for mut score_text in query.iter_mut() {
            score_text.sections[0].value = format!("Score: {:03}", score.score);
        }
    }
}

impl Plugin for ScoreDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), setup_score)
            .add_systems(Update, update_score.run_if(in_state(GameState::Playing)));
    }
}
