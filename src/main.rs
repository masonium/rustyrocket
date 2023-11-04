use bevy::{
    prelude::*,
    window::{close_on_esc, WindowResolution},
};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use rustyrocket::{
    obstacle::ObstaclePlugin,
    player::PlayerPlugin,
    WorldSettings, WorldSet, level::{LevelPlugin, LevelSettings}, score::{Score, ScorePlugin}, scoring_region::ScoringRegionPlugin, ResetEvent,
};

use rustyrocket::GameState;

fn setup_camera(mut commands: Commands) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
}

fn setup_physics(
    mut physics: ResMut<WorldSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
    window: Query<&Window>,
) {
    let w = window.single();
    rapier_config.gravity = Vec2::new(0.0, -500.0);

    physics.bounds.max = Vec2::new(w.width() / 2.0, w.height() / 2.0);
    physics.bounds.min = -physics.bounds.max;
}

fn reset_on_r(keys: Res<Input<KeyCode>>,
	      mut reset_events: EventWriter<ResetEvent>,
) {
    if keys.just_pressed(KeyCode::R) {
	reset_events.send(ResetEvent);
    }
}
	      

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Rocket".to_string(),
                resolution: WindowResolution::new(1024.0, 1024.0 * 9.0 / 16.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(96.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(ObstaclePlugin)
        .insert_resource(WorldSettings::default())
        .add_event::<ResetEvent>()
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        // .add_plugins(ResourceInspectorPlugin::<LevelSettings>::default())
        // .add_plugins(ResourceInspectorPlugin::<WorldSettings>::default())
        // .add_plugins(ResourceInspectorPlugin::<Score>::default())
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Playing),
        )
        .add_plugins(PlayerPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(ScoringRegionPlugin)
        .add_systems(Startup, (setup_camera, setup_physics).in_set(WorldSet))
        .add_systems(Update, reset_on_r.run_if(in_state(GameState::Playing)))
        .add_systems(Update, (close_on_esc, ))
        .run()
}
