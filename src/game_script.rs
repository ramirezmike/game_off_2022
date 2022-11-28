use bevy::prelude::*;
use crate::{
    asset_loading, assets::GameAssets, game_state, AppState, 
};

pub struct GameScriptPlugin;
impl Plugin for GameScriptPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameScriptState>()
           .add_system_set(SystemSet::on_update(AppState::LoadWorld).with_system(load_state));
    }
}

#[derive(Debug)]
pub enum GameScript {
    IntroCutscene,
    LevelOneIntroCutscene,
    LevelOne,
}

#[derive(Resource)]
pub struct GameScriptState {
    pub current: GameScript,
}

impl Default for GameScriptState {
    fn default() -> Self {
        GameScriptState {
            current: GameScript::IntroCutscene,
        }
    }
}

impl GameScriptState {
    pub fn next(&mut self) {
        println!("Moving from {:?}", self.current);
        self.current = match self.current {
            GameScript::IntroCutscene => GameScript::LevelOneIntroCutscene,
            GameScript::LevelOneIntroCutscene => GameScript::LevelOne,
            GameScript::LevelOne => GameScript::IntroCutscene,
            _ => GameScript::IntroCutscene
        };
        println!("to {:?}", self.current);
    }
}

fn load_state(
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<GameAssets>,
    game_state: ResMut<game_state::GameState>,
    game_script_state: Res<GameScriptState>,
) {
    println!("Loading state {:?}", game_script_state.current);
    match game_script_state.current {
        GameScript::IntroCutscene => assets_handler.load(AppState::Cutscene, &mut game_assets, &game_state),
        GameScript::LevelOneIntroCutscene => assets_handler.load(AppState::Cutscene, &mut game_assets, &game_state),
        GameScript::LevelOne => assets_handler.load(AppState::InGame, &mut game_assets, &game_state),
        _ => ()
    }
}
