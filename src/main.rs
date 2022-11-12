#![windows_subsystem = "windows"]
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::asset::AssetServerSettings;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::window::WindowMode;
use bevy_inspector_egui::{WorldInspectorPlugin, egui, bevy_egui};
use bevy_rapier3d::prelude::*;
use bevy_camera_shake::Shake3d;
use bevy::gltf::Gltf;
use bevy_scene_hook::{HookPlugin, SceneHook, HookedSceneBundle};

mod asset_loading;
mod assets;
mod bull;
mod direction;
mod game_camera;
mod game_controller;
mod game_state;
mod ingame;
mod player;
mod props;

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
//        .add_system(initial_damp_physics)
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

const TRAUMA_AMOUNT: f32 = 0.5;
fn debug(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    game_state: ResMut<game_state::GameState>,
    mut exit: ResMut<Events<AppExit>>,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
    mut shakeables: Query<&mut Shake3d>,
    mut rapier: ResMut<RapierConfiguration>,
    plates: Query<(Entity, &Parent, &Velocity), With<props::Plate>>,
    parent_transforms: Query<&Transform>,
    assets_gltf: Res<Assets<Gltf>>,
//   mut velocities: Query<(Entity, &mut Velocity), Without<bull::Bull>>,
) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }

    if keys.just_pressed(KeyCode::R) {
        assets_handler.load(AppState::ResetInGame, &mut game_assets, &game_state);
    }


    if keys.just_pressed(KeyCode::E) {
        for mut shakeable in shakeables.iter_mut() {
            shakeable.trauma = f32::min(shakeable.trauma + TRAUMA_AMOUNT, 1.0);
        }
    }

    if keys.just_pressed(KeyCode::P) {
        rapier.physics_pipeline_active = !rapier.physics_pipeline_active;
        rapier.query_pipeline_active = !rapier.query_pipeline_active;

        rapier.timestep_mode =  TimestepMode::Variable {
            time_scale: 1.0,
            max_dt: 1.0,
            substeps: 1,
        };
//          timestep_mode: TimestepMode::Variable {
//              max_dt: 1.0 / 60.0,
//              time_scale: 1.0,
//              substeps: 1,
//          },
    }

//  for (e,v) in &mut velocities {
//      if v.linvel.length() > 0.0 {
//          println!("{:?} V: {:?}", e, v);
//      }
//  }

    if keys.just_pressed(KeyCode::O) {
        println!("Pressed O");

        for (entity, parent, velocity) in &plates {
            println!("replacing a thing");
            let transform = parent_transforms.get(parent.get()).unwrap().clone();
            let velocity = velocity.clone();
            commands.entity(entity).despawn_recursive();

            if let Some(gltf) = assets_gltf.get(&game_assets.broken_plate.clone()) {
                commands.spawn_bundle(HookedSceneBundle {
                    scene: SceneBundle { scene: gltf.scenes[0].clone(), ..default() },
                    hook: SceneHook::new(move |entity, cmds, mesh| {
                        if let Some(name) = entity.get::<Name>().map(|t|t.as_str()) {
                            if name.contains("plate") {
                                use props::ComponentAdder;
                                props::BrokenPlate::add_components(cmds); 
                                cmds.insert(velocity.clone());
                                cmds.insert(transform.clone());
                            }
                        }
                    })
                });
            }
        }
    }
}

fn window_settings(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_title(String::from("Charlotte's Independence: The Road to Uptown"));
        //        window.set_mode(WindowMode::BorderlessFullscreen);
    }
}

#[derive(Component)]
pub struct DampPhysics(f32);
fn initial_damp_physics(
    mut commands: Commands,
    mut damps: Query<(Entity, &mut DampPhysics, &mut Velocity, &ReadMassProperties)>,
    time: Res<Time>,
) {
    for (entity, mut damp, mut velocity, r) in &mut damps {
        damp.0 -= time.delta_seconds(); 

        println!("dampening {:?}, {:?}", velocity.linvel, velocity.angvel);
        println!("{:?}", r);
        velocity.linvel = velocity.linvel.normalize() * 0.0000001;
        velocity.angvel = velocity.angvel.normalize() * 0.0000001;
        commands.entity(entity).insert(
            ColliderMassProperties::MassProperties(MassProperties {
                principal_inertia: r.0.principal_inertia.normalize() * 0.0000001,
                principal_inertia_local_frame: Quat::default(),
                ..default()
            })
        );

        if damp.0 < 0.0 {
           commands.entity(entity).remove::<DampPhysics>();
           commands.entity(entity).remove::<ColliderMassProperties>();
        }
    }
}

