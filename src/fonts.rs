use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::GameState;

#[derive(AssetCollection, Resource)]
pub struct FontsCollection {
    /// font for displaying the score
    #[asset(path = "fonts/SpeedrushRegular-qZWp6.otf")]
    pub score_font: Handle<Font>,

    /// font for the pause menu, possibly
    #[asset(path = "fonts/SpeedrushRegular-qZWp6.otf")]
    menu_font: Handle<Font>,
}

pub struct GameFontsPlugin;

impl Plugin for GameFontsPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, FontsCollection>(GameState::AssetLoading);
    }
}
