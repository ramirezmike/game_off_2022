#![windows_subsystem = "windows"]
use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::window::WindowMode;
use bevy_inspector_egui::{WorldInspectorPlugin, egui, bevy_egui};
use bevy_rapier3d::prelude::*;
use bevy_camera_shake::Shake3d;
use bevy::gltf::Gltf;
use bevy_scene_hook::{HookPlugin, SceneHook, HookedSceneBundle};
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlinePlugin, 
};

use bevy_flycam::{NoCameraPlayerPlugin, FlyCam};

mod asset_loading;
mod assets;
mod audio;
mod bull;
mod billboard;
mod direction;
mod dust;
mod cutscene;
mod game_camera;
mod game_controller;
mod follow_text;
mod fishmonger;
mod game_state;
mod game_script;
mod groups;
mod ingame;
mod ingame_ui;
mod menus;
mod player;
mod shopkeeper;
mod splash;
mod score;
mod props;
mod title_screen;
mod ui;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
          watch_for_changes: true,
          ..default()
        })
         .set(WindowPlugin {
          window: WindowDescriptor {
            fit_canvas_to_parent: true,
            ..default()
          },
          ..default()
        })
                     )
        .add_plugin(bull::BullPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default().with_physics_scale(10.0))
//      .add_plugin(RapierDebugRenderPlugin::default())
//      .add_plugin(LogDiagnosticsPlugin::default())
//      .add_plugin(FrameTimeDiagnosticsPlugin::default())
//      .add_plugin(NoCameraPlayerPlugin)
//      .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(OutlinePlugin)
        .add_plugin(AutoGenerateOutlineNormalsPlugin)
        .add_plugin(billboard::BillboardPlugin)
        .add_plugin(dust::DustPlugin)
        .add_plugin(audio::GameAudioPlugin)
        .add_plugin(cutscene::CutscenePlugin)
        .add_plugin(asset_loading::AssetLoadingPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(game_controller::GameControllerPlugin)
        .add_plugin(follow_text::FollowTextPlugin)
        .add_plugin(game_state::GameStatePlugin)
        .add_plugin(game_script::GameScriptPlugin)
        .add_plugin(groups::GroupPlugin)
        .add_plugin(ingame::InGamePlugin)
        .add_plugin(ingame_ui::InGameUIPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(props::PropsPlugin)
        .add_plugin(score::ScorePlugin)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(fishmonger::FishMongerPlugin)
        .add_plugin(shopkeeper::ShopKeeperPlugin)
        .add_plugin(title_screen::TitlePlugin)
        .add_plugin(ui::text_size::TextSizePlugin)

//      .add_system(debug)
//      .add_system(debug_2)
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
    LoadWorld,
    InGame,
    Splash,
    ScoreDisplay,
    LevelOver,
    ResetInGame,
    Loading,
}

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.get_or_spawn(entity).despawn_recursive();
    }
}

fn bootstrap(
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
    game_state: ResMut<game_state::GameState>,
    mut clear_color: ResMut<ClearColor>,
    mut audio: audio::GameAudio,
) {
    audio.set_volume();
    clear_color.0 = Color::hex("aaaaaa").unwrap();

    assets_handler.load(AppState::Splash, &mut game_assets, &game_state);
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
    mut game_state: ResMut<game_state::GameState>,
    mut exit: ResMut<Events<AppExit>>,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
    mut shakeables: Query<&mut Shake3d>,
    mut rapier: ResMut<RapierConfiguration>,
    mut restore_group_event_writer: EventWriter<groups::RestoreGroupEvent>,
    cameras: Query<(Entity, &Transform, &game_camera::PanOrbitCamera), With<Camera3d>>,
    plates: Query<(Entity, &Parent, &Velocity), With<props::Plate>>,
    parent_transforms: Query<&Transform>,
    assets_gltf: Res<Assets<Gltf>>,
    mut dust_spawn_event_writer: EventWriter<dust::DustSpawnEvent>,
    mut cutscene_state: ResMut<cutscene::CutsceneState>,
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
            commands.get_or_spawn(entity).despawn_recursive();

            if let Some(gltf) = assets_gltf.get(&game_assets.broken_plate.clone()) {
                commands.spawn(HookedSceneBundle {
                    scene: SceneBundle { scene: gltf.scenes[0].clone(), ..default() },
                    hook: SceneHook::new(move |entity, cmds, mesh| {
                        if let Some(name) = entity.get::<Name>().map(|t|t.as_str()) {
                            if name.contains("plate") {
                                use props::ComponentAdder;
                                props::BrokenPlate::add_components(cmds, mesh); 
                                cmds.insert(velocity.clone());
                                cmds.insert(transform.clone());
                            }
                        }
                    })
                });
            }
        }
    }

    if keys.just_pressed(KeyCode::G) {
        println!("sending group event");
        restore_group_event_writer.send(groups::RestoreGroupEvent {
            group_id: 1
        });
    }

    if keys.just_pressed(KeyCode::U) {
        for (_, transform, pan) in &cameras {
            println!("C: {:?} {:?} {:?}", transform.translation, transform.rotation.to_axis_angle(), pan.focus);
            println!("Forward: {:?}", transform.forward());
        }
    }

    if keys.just_pressed(KeyCode::C) {
        dust_spawn_event_writer.send(dust::DustSpawnEvent {
            position: Vec3::new(0.0, 0.5, 0.0),
            count: 10,
            spread: 6.0,
            rate: 0.5,
            dust_time_to_live: 3.0,
            emitter_time_to_live: 0.0,
            size: 2.0,
            ..default()
        });
    }

    if keys.just_pressed(KeyCode::V) {
        // skip cutscene
        cutscene_state.cutscene_index = 99999;
        cutscene_state.waiting_on_input = false;
        game_state.current_time = 0.0;
    }

    if keys.just_pressed(KeyCode::F) {
        for (entity, _, _) in &cameras {
            commands.entity(entity).insert(FlyCam);
        }
    }
}

fn debug_2(
    keys: Res<Input<KeyCode>>,
    mut follow_text_event_writer: EventWriter<follow_text::FollowTextEvent>,
    mut chase_event_writer: EventWriter<fishmonger::ChaseEvent>,
    mut hit_player_event_writer: EventWriter<player::HitPlayerEvent>,
    players: Query<Entity, With<player::Player>>,
) {
    if keys.just_pressed(KeyCode::H) {
        for e in &players {
            follow_text_event_writer.send(follow_text::FollowTextEvent {
                follow: follow_text::FollowThing::Entity(e),
                text: "oh geez!".to_string(),
                color: Color::WHITE,
                time_to_live: 6.0,
            });
        }
    }


    if keys.just_pressed(KeyCode::T) {
        hit_player_event_writer.send(player::HitPlayerEvent);
    }

    if keys.just_pressed(KeyCode::Y) {
        chase_event_writer.send(fishmonger::ChaseEvent);
    }
}

fn window_settings(mut windows: ResMut<Windows>) {
    for window in windows.iter_mut() {
        window.set_title(String::from(""));
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
