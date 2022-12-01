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
    EndCutscene,
    PreLevelOneCutscene,
    LevelOneIntroCutscene,
    LevelOne,
    LevelOnePostCutscene,

    LevelTwoIntroCutscene,
    LevelTwo,
    LevelTwoPostCutscene,

    LevelThreeIntroCutscene,
    LevelThree,
    LevelThreePostCutscene,

    LevelFourIntroCutscene,
    LevelFour,
    LevelFourPostCutscene,

    LevelFiveIntroCutscene,
    LevelFive,
    LevelFivePostCutscene,
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
            GameScript::IntroCutscene => GameScript::PreLevelOneCutscene,
            GameScript::PreLevelOneCutscene => GameScript::LevelOneIntroCutscene,
            GameScript::LevelOneIntroCutscene => GameScript::LevelOne,
            GameScript::LevelOne => GameScript::LevelOnePostCutscene,
            GameScript::LevelOnePostCutscene => GameScript::LevelTwoIntroCutscene,

            GameScript::LevelTwoIntroCutscene => GameScript::LevelTwo,
            GameScript::LevelTwo => GameScript::LevelTwoPostCutscene,
            GameScript::LevelTwoPostCutscene => GameScript::LevelThreeIntroCutscene,

            GameScript::LevelThreeIntroCutscene => GameScript::LevelThree,
            GameScript::LevelThree => GameScript::LevelThreePostCutscene,
            GameScript::LevelThreePostCutscene => GameScript::EndCutscene,

            GameScript::LevelFourIntroCutscene => GameScript::LevelFour,
            GameScript::LevelFour => GameScript::LevelFourPostCutscene,
            GameScript::LevelFourPostCutscene => GameScript::LevelFiveIntroCutscene,

            GameScript::LevelFiveIntroCutscene => GameScript::LevelFive,
            GameScript::LevelFive => GameScript::LevelFivePostCutscene,
            GameScript::LevelFivePostCutscene => GameScript::IntroCutscene, // todo: end?

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
        GameScript::IntroCutscene 
        | GameScript::EndCutscene
        | GameScript::PreLevelOneCutscene
        | GameScript::LevelOneIntroCutscene 
        | GameScript::LevelOnePostCutscene 
        | GameScript::LevelTwoIntroCutscene 
        | GameScript::LevelTwoPostCutscene 
        | GameScript::LevelThreeIntroCutscene 
        | GameScript::LevelThreePostCutscene 
        | GameScript::LevelFourIntroCutscene
        | GameScript::LevelFourPostCutscene
        | GameScript::LevelFiveIntroCutscene
        | GameScript::LevelFivePostCutscene => assets_handler.load(AppState::Cutscene, &mut game_assets, &game_state),

        GameScript::LevelOne
        | GameScript::LevelTwo
        | GameScript::LevelThree 
        | GameScript::LevelFour
        | GameScript::LevelFive => assets_handler.load(AppState::InGame, &mut game_assets, &game_state),
        _ => ()
    }
}
