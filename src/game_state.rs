use crate::AppState;
use bevy::prelude::*;

pub struct GameStatePlugin;
impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameState::default());
    }
}

#[derive(Component)]
pub struct LevelOverCleanupMarker;

#[derive(Resource)]
pub struct GameState {
    pub score: usize,
    pub shadows_on: bool,
    pub current_time: f32,
    pub graphics_high: bool,
    pub title_screen_cooldown: f32,
}

impl GameState {
    pub fn initialize(graphics: bool, shadows_on: bool) -> Self {
        GameState {
            score: 0,
            shadows_on: shadows_on,
            current_time: 0.0,
            graphics_high: graphics,
            title_screen_cooldown: 1.0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState::initialize(true, true)
    }
}
