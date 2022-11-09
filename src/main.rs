#![windows_subsystem = "windows"]
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::asset::AssetServerSettings;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::window::WindowMode;
use bevy_inspector_egui::{WorldInspectorPlugin, egui, bevy_egui};
use bevy_rapier3d::prelude::*;

mod asset_loading;
mod assets;
mod bull;
mod direction;
mod game_camera;
mod game_controller;
mod game_state;
mod ingame;
mod player;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bull::BullPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WindowDescriptor {
            fit_canvas_to_parent: true,
            ..default()
        })
        .add_plugin(asset_loading::AssetLoadingPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(game_controller::GameControllerPlugin)
        .add_plugin(game_state::GameStatePlugin)
        .add_plugin(ingame::InGamePlugin)
        .add_plugin(player::PlayerPlugin)

        .add_system(debug)
        .add_startup_system(window_settings)
        .add_state(AppState::Initial)
        .insert_resource(bevy_egui::EguiSettings { scale_factor: 1.8, ..default() })
        .add_system_set(SystemSet::on_update(AppState::Initial).with_system(bootstrap))
        .run();
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Initial,
    Pause,
    Cutscene,
    Debug,
    TitleScreen,
    Options,
    InGame,
    Splash,
    LevelOver,
    ResetInGame,
    Loading,
}

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn bootstrap(
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
    game_state: ResMut<game_state::GameState>,
    mut clear_color: ResMut<ClearColor>,
) {
    clear_color.0 = Color::hex("aaaaaa").unwrap();

    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
}

pub trait ZeroSignum {
    fn zero_signum(&self) -> Vec3;
}

impl ZeroSignum for Vec3 {
    fn zero_signum(&self) -> Vec3 {
        let convert = |n| {
            if n < 0.1 && n > -0.1 {
                0.0
            } else if n > 0.0 {
                1.0
            } else {
                -1.0
            }
        };

        Vec3::new(convert(self.x), convert(self.y), convert(self.z))
    }
}

fn debug(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    game_state: ResMut<game_state::GameState>,
    mut exit: ResMut<Events<AppExit>>,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }

    if keys.just_pressed(KeyCode::R) {
        assets_handler.load(AppState::ResetInGame, &mut game_assets, &game_state);
    }
}

fn window_settings(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_title(String::from("Charlotte's Independence: The Road to Uptown"));
        //        window.set_mode(WindowMode::BorderlessFullscreen);
    }
}
