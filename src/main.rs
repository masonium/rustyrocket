use bevy::{
    input::common_conditions::input_toggle_active,
    log::{Level, LogPlugin},
    prelude::*,
    render::render_resource::{FilterMode, SamplerDescriptor},
    window::{close_on_esc, WindowResolution},
};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::{prelude::*, render::RapierDebugRenderPlugin};
use rustyrocket::{
    gravity_shift::GravityShiftPlugin,
    level::{LevelPlugin, LevelSettings},
    obstacle::ObstaclePlugin,
    player::PlayerPlugin,
    score::{Score, ScorePlugin},
    scoring_region::ScoringRegionPlugin,
    ResetEvent, WorldSet, WorldSettings,
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

fn reset_on_r(keys: Res<Input<KeyCode>>, mut reset_events: EventWriter<ResetEvent>) {
    if keys.just_pressed(KeyCode::R) {
        reset_events.send(ResetEvent);
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Rusty Rocket".to_string(),
                        resolution: WindowResolution::new(1024.0, 1024.0 * 9.0 / 16.0),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: {
                        SamplerDescriptor {
                            mag_filter: FilterMode::Nearest,
                            ..default()
                        }
                    },
                })
                .set(LogPlugin {
                    level: Level::INFO,
                    ..default()
                }),
        )
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(96.0),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins(ObstaclePlugin)
        .insert_resource(WorldSettings::default())
        .add_event::<ResetEvent>()
        .add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(false, KeyCode::I)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<LevelSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::L)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<WorldSettings>::default()
                .run_if(input_toggle_active(false, KeyCode::W)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<Score>::default()
                .run_if(input_toggle_active(false, KeyCode::S)),
        )
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Playing),
        )
        .add_plugins(PlayerPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(ScorePlugin)
        .add_plugins(ScoringRegionPlugin)
        .add_plugins(GravityShiftPlugin)
        .add_systems(Startup, (setup_camera, setup_physics).in_set(WorldSet))
        .add_systems(Update, reset_on_r.run_if(in_state(GameState::Playing)))
        .add_systems(Update, (close_on_esc,))
        .run()
}
