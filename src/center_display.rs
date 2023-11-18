pub use bevy::prelude::*;

use crate::{fonts::FontsCollection, GameState};

#[derive(Component)]
pub struct CenterDisplay;

pub fn spawn_display(mut commands: Commands, fonts: Res<FontsCollection>) {
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "",
                TextStyle {
                    font: fonts.menu_font.clone(),
                    font_size: 70.0,
                    color: Color::ANTIQUE_WHITE,
                },
            ),
            transform: Transform::from_xyz(0.0, 0.0, 20.0),
            ..default()
        },
        CenterDisplay,
    ));
}

pub fn show_game_over(mut text: Query<(&mut Text, &mut Visibility), With<CenterDisplay>>) {
    for (mut t, mut v) in text.iter_mut() {
        *v = Visibility::Visible;
        t.sections[0].value = "GAME OVER".to_string();
    }
}

pub fn show_ready(mut text: Query<(&mut Text, &mut Visibility), With<CenterDisplay>>) {
    for (mut t, mut v) in text.iter_mut() {
        *v = Visibility::Visible;
        t.sections[0].value = "READY".to_string();
    }
}

pub fn hide_display(mut text: Query<&mut Visibility, With<CenterDisplay>>) {
    for mut v in text.iter_mut() {
        *v = Visibility::Hidden;
    }
}

pub struct CenterDisplayPlugin;

impl Plugin for CenterDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), spawn_display)
            .add_systems(OnEnter(GameState::Dying), show_game_over)
            .add_systems(OnExit(GameState::Dying), hide_display)
            .add_systems(OnEnter(GameState::Ready), show_ready)
            .add_systems(OnExit(GameState::Ready), hide_display);
    }
}
