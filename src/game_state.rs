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

pub struct GameState {
    pub score: usize,
    pub shadows_on: bool,
    pub graphics_high: bool,
    pub title_screen_cooldown: f32,
}

impl GameState {
    pub fn initialize(graphics: bool, shadows_on: bool) -> Self {
        GameState {
            score: 0,
            shadows_on: shadows_on,
            graphics_high: graphics,
            title_screen_cooldown: 1.0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            score: 0,
            shadows_on: true,
            graphics_high: true,
            title_screen_cooldown: 1.0,
        }
    }
}
