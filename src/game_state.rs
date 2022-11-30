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
    pub score: f32,
    pub live_score: f32,
    pub score_check_count: usize,
    pub shadows_on: bool,
    pub current_time: f32,
    pub graphics_high: bool,
    pub title_screen_cooldown: f32,
}

impl GameState {
    pub fn initialize(graphics: bool, shadows_on: bool) -> Self {
        GameState {
            score: 1.0,
            live_score: 1.0,
            score_check_count: 0,
            shadows_on,
            current_time: 120.0,
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
