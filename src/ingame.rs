use crate::{
    asset_loading, assets::GameAssets, cleanup, game_state, AppState, game_camera, player, bull, 
    DampPhysics, props::*, groups, shopkeeper, billboard, game_script, cutscene, dust, fishmonger,
};
use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::gltf::Gltf;
use bevy::render::view::NoFrustumCulling;
use leafwing_input_manager::prelude::*;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::f32::consts::TAU;
use bevy_rapier3d::geometry::ColliderMassProperties;
use bevy_camera_shake::{CameraShakePlugin, RandomSource, Shake3d};
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::{HookPlugin, SceneHook, HookedSceneBundle};
use std::str::FromStr;
use bevy_mod_outline::{
OutlineBundle, OutlineVolume,
};

pub const RENDER_TEXTURE_SIZE: u32 = 512;
pub struct InGamePlugin;
impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Cutscene).with_system(setup))
           .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
            .add_system_set(
                SystemSet::on_exit(AppState::InGame).with_system(cleanup::<CleanupMarker>),
            )
            .add_system_set(SystemSet::on_update(AppState::ResetInGame).with_system(reset_ingame))
            .add_system(animate_fire)
            .add_plugin(HookPlugin)
            .add_plugin(CameraShakePlugin)
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
//                  .with_system(game_camera::follow_player.after(player::move_player))
//                  .with_system(game_camera::light_follow_camera.after(player::move_player))
//                    .with_system(pass_speed_to_shader.after(player::move_player))
                    .with_system(collision_report)
                    .with_system(game_camera::look_at_player)
                    .with_system(rotate_rotate_entities)
                    .with_system(game_camera::pan_orbit_camera),
            );
    }
}

#[derive(Component, Copy, Clone)]
pub struct CleanupMarker;

fn collision_report(
    mut events: EventReader<CollisionEvent> 
) {
    for event in events.iter() {
        println!("Collision: {:?}", event);
    }
}

fn random_number() -> f32 {
    let mut rng = thread_rng();
    let x: f32 = rng.gen();
    x * 2.0 - 1.0
}
struct MyRandom;

impl RandomSource for MyRandom {
    fn rand(&self, _time: f32) -> f32 {
        random_number()
    }
}

fn reset_ingame(
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    mut game_script_state: ResMut<game_script::GameScriptState>,
) {
    game_script_state.current = game_script::GameScript::LevelThreeIntroCutscene;
    assets_handler.load(AppState::LoadWorld, &mut game_assets, &mut game_state);
}

pub fn load(
    assets_handler: &mut asset_loading::AssetsHandler,
    game_assets: &mut ResMut<GameAssets>,
    game_state: &ResMut<game_state::GameState>,
) {
    println!("loading for ingame");
    assets_handler.add_glb(&mut game_assets.matador, "models/matador.glb");
    assets_handler.add_animation(&mut game_assets.matador_run,"models/matador.glb#Animation3");
    assets_handler.add_animation(&mut game_assets.matador_idle,"models/matador.glb#Animation1");
    assets_handler.add_animation(&mut game_assets.matador_dive,"models/matador.glb#Animation0");
    assets_handler.add_animation(&mut game_assets.matador_pose,"models/matador.glb#Animation2");
    assets_handler.add_glb(&mut game_assets.bull, "models/bull.glb");
    assets_handler.add_animation(&mut game_assets.bull_charge,"models/bull.glb#Animation0");
    assets_handler.add_animation(&mut game_assets.bull_collide,"models/bull.glb#Animation1");
    assets_handler.add_animation(&mut game_assets.bull_idle,"models/bull.glb#Animation2");
    assets_handler.add_animation(&mut game_assets.bull_run,"models/bull.glb#Animation3");
    assets_handler.add_animation(&mut game_assets.bull_walk,"models/bull.glb#Animation4");
    assets_handler.add_glb(&mut game_assets.plate, "models/plate.glb");
    assets_handler.add_glb(&mut game_assets.broken_plate, "models/broken_plate.glb");
    assets_handler.add_glb(&mut game_assets.broken_mug, "models/broken_mug.glb");
    assets_handler.add_glb(&mut game_assets.broken_fishbowl, "models/fishbowl_empty.glb");
    assets_handler.add_glb(&mut game_assets.fishmonger, "models/fishmonger.glb");
    assets_handler.add_glb(&mut game_assets.fishmonger_with_fish, "models/fishmonger_with_fish.glb");
    assets_handler.add_animation(&mut game_assets.bull_charge,"models/bull.glb#Animation0");
    assets_handler.add_font(&mut game_assets.font, "fonts/monogram.ttf");

    assets_handler.add_standard_mesh(&mut game_assets.dust, Mesh::from(shape::Plane { size: 2.0 }));

    assets_handler.add_material(&mut game_assets.cloud_texture, "textures/cloud.png", true);
    assets_handler.add_material(&mut game_assets.wrench_texture, "textures/wrench.png", true);
    assets_handler.add_material(&mut game_assets.fire_texture, "textures/fire.png", true);
    assets_handler.add_material(&mut game_assets.star_full_texture, "textures/star_full.png", true);
    assets_handler.add_material(&mut game_assets.star_half_texture, "textures/star_half.png", true);
    assets_handler.add_material(&mut game_assets.star_empty_texture, "textures/star_empty.png", true);

    assets_handler.add_material(&mut game_assets.mat_idle, "textures/matador_idle.png", true);
    assets_handler.add_material(&mut game_assets.mat_talk, "textures/matador_mouth.png", true);
    assets_handler.add_material(&mut game_assets.pa_no_mouth, "textures/pa_nomouth.png", true);
    assets_handler.add_material(&mut game_assets.pa_mouth, "textures/pa_mouth.png", true);
    assets_handler.add_material(&mut game_assets.pa_lookleft, "textures/pa_lookleft.png", true);

    assets_handler.add_glb(&mut game_assets.intro_level, "models/intro.glb");
    assets_handler.add_glb(&mut game_assets.outro_level, "models/outro.glb");
    assets_handler.add_glb(&mut game_assets.pregame, "models/pregame.glb");
    assets_handler.add_glb(&mut game_assets.level_one, "models/level_one.glb");
    assets_handler.add_glb(&mut game_assets.level_two, "models/level_two.glb");
    assets_handler.add_glb(&mut game_assets.level_three, "models/level_three.glb");
//  assets_handler.add_glb(&mut game_assets.level_four, "models/level_four.glb");
//  assets_handler.add_glb(&mut game_assets.level_five, "models/level_five.glb");

    assets_handler.add_audio(&mut game_assets.mat_speak, "audio/mat_speak.wav");
    assets_handler.add_audio(&mut game_assets.pa_speak, "audio/pa_speak.wav");
    assets_handler.add_audio(&mut game_assets.clop_sfx, "audio/clop.wav");
    assets_handler.add_audio(&mut game_assets.break_sfx, "audio/break.wav");
    assets_handler.add_audio(&mut game_assets.crash_sfx, "audio/crash.wav");
    assets_handler.add_audio(&mut game_assets.intro_bgm, "audio/intro.ogg");
    assets_handler.add_audio(&mut game_assets.intro_end_bgm, "audio/intro_end.ogg");
    assets_handler.add_audio(&mut game_assets.pregame_bgm, "audio/pregame_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.intro_end_bgm, "audio/intro_end_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.level_one_bgm, "audio/level_one_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.level_two_bgm, "audio/level_two_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.level_three_bgm, "audio/level_three_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.title_screen_bgm, "audio/title_screen_bgm.ogg");
}

#[derive(Component)]
pub struct BullCollide;

pub fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    mut game_state: ResMut<game_state::GameState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Query<&mut Transform, With<game_camera::PanOrbitCamera>>,
    mut rapier: ResMut<RapierConfiguration>,
    mut clear_color: ResMut<ClearColor>,
    game_script_state: Res<game_script::GameScriptState>,
) {
    clear_color.0 = Color::hex("000000").unwrap(); 
    game_state.title_screen_cooldown = 1.0;
    game_state.current_time = 90.0;
    game_state.live_score = 1.0;

    let gltf = 
        match game_script_state.current {
            game_script::GameScript::IntroCutscene => assets_gltf.get(&game_assets.intro_level.clone()),
            game_script::GameScript::EndCutscene => assets_gltf.get(&game_assets.outro_level.clone()),
            game_script::GameScript::PreLevelOneCutscene => assets_gltf.get(&game_assets.pregame.clone()),
            game_script::GameScript::LevelOneIntroCutscene 
                | game_script::GameScript::LevelOnePostCutscene 
                | game_script::GameScript::LevelOne => assets_gltf.get(&game_assets.level_one.clone()),
            game_script::GameScript::LevelTwoIntroCutscene 
                | game_script::GameScript::LevelTwoPostCutscene 
                | game_script::GameScript::LevelTwo => assets_gltf.get(&game_assets.level_two.clone()),
            game_script::GameScript::LevelThreeIntroCutscene 
                | game_script::GameScript::LevelThreePostCutscene 
                | game_script::GameScript::LevelThree => assets_gltf.get(&game_assets.level_three.clone()),
            game_script::GameScript::LevelFourIntroCutscene 
                | game_script::GameScript::LevelFourPostCutscene 
                | game_script::GameScript::LevelFour => assets_gltf.get(&game_assets.level_four.clone()),
            game_script::GameScript::LevelFiveIntroCutscene 
                | game_script::GameScript::LevelFivePostCutscene 
                | game_script::GameScript::LevelFive => assets_gltf.get(&game_assets.level_five.clone()),
            _ => None  
        };

    if let Some(gltf) = gltf {
        println!("got gltf");
        commands.spawn(HookedSceneBundle {
           scene: SceneBundle { scene: gltf.scenes[0].clone(), ..default() },
           hook: SceneHook::new(move |entity, cmds, mesh| {
               if let Some(name) = entity.get::<Name>().map(|t|t.as_str()) {
//                   println!("Name: {} Mesh: {:?}", name, mesh.is_some());

                   shopkeeper::spawn(name, cmds);
                   fishmonger::spawn(name, cmds);

                   if name.contains("static") {
                       if let Some(mesh) = mesh {
                           println!("adding collider");
                           cmds.insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap())
                               .insert(BullCollide);
                       }
                   }

                   if name.contains("rotate") {
                       if let Some(mesh) = mesh {
                           cmds.insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap())
                               .insert(RotateEntityMarker)
                               .insert(RigidBody::KinematicPositionBased);
                       }
                   }

                   if name.contains("PointLight") {
                       handle_lights(cmds, name);
                   }

                   if name.contains("noshadowcast") {
                       cmds.insert(bevy::pbr::NotShadowCaster);
                   }
                   if name.contains("invisible") {
//                     cmds.insert(Visibility {
//                         is_visible: false
//                     });
                   }

                   if name.contains("light") {
                      println!("Inserting light!");
                      cmds.with_children(|children| {
                        children 
                            .spawn(PointLightBundle {
                                point_light: PointLight {
                                  intensity: 90.0, // lumens - roughly a 100W non-halogen incandescent bulb
                                  color: Color::rgba(255.0, 255.0, 255.0, 255.0),
                                  shadows_enabled: true,
                                    ..default()
                                },
                                ..default()
                            });
                      });
                   }
                   if name.contains("bull") {
                       let BULL_COLLISION_THRESHOLD: f32 = 0.40001;
                       cmds.insert(NoFrustumCulling)
                           .insert(Collider::cuboid(2.0, 2.0, 2.0))
                           .insert(ColliderMassProperties::Density(5.0))
                           .insert(Velocity::default())
                           .insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z | LockedAxes::ROTATION_LOCKED_Y) 
                           .insert(Ccd::enabled())
                           .insert(ActiveEvents::CONTACT_FORCE_EVENTS)
                           .insert(ContactForceEventThreshold(BULL_COLLISION_THRESHOLD))
                           .insert(RigidBody::Dynamic)
                           .insert(bull::Bull::default());
                   }
                   if name.contains("player") {
                       cmds.insert(player::PlayerBundle::new())
                       .insert(Restitution::coefficient(0.2))
                       .insert(RigidBody::Dynamic)
                       .insert(Velocity::default())
//                       .insert(Damping { linear_damping: 0.9, angular_damping: 0.0 })
                       .insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z | LockedAxes::ROTATION_LOCKED_Y) 
                       .insert(Ccd::enabled())
                       .with_children(|children| {
                           children.spawn(
                               (Collider::cuboid(0.2, 1.0, 0.2),
                               TransformBundle::from(Transform::from_xyz(0.0, 1.0, 0.0)))
                           );
                               // Position the collider relative to the rigid-body.
                       });
                   }
                   if name.contains("dynamic") {
                       if let Some(mesh) = mesh {
                           cmds.insert(Restitution::coefficient(0.2))
                               .insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap())
                               .insert(Velocity::default())
                               //.insert(DampPhysics(2.0))
                               //.insert(ReadMassProperties::default())
                               //.insert(Damping { linear_damping: 100.0, angular_damping: 100.0 })
//                             .insert(Sleeping {
//                                 linear_threshold: 15000.0,
//                                 angular_threshold: 15000.0,
//                                 sleeping: true,
//                                 ..default()
//                             })
                               .insert(RigidBody::Dynamic);
                       }
                   }
                   if name.contains("boutline") {
                       cmds.insert(OutlineBundle {
                            outline: OutlineVolume {
                                visible: true,
                                width: 3.0,
                                colour: Color::WHITE,
                            },
                            ..default()
                        });
                   }
                   if name.contains("routline") {
                       cmds.insert(OutlineBundle {
                            outline: OutlineVolume {
                                visible: true,
                                width: 3.0,
                                colour: Color::RED,
                            },
                            ..default()
                        });
                   }
                   if name.contains("collide") {
                       cmds.insert((BullCollide, ExternalImpulse::default(), ExternalForce::default()));
                   }

                   if name.contains("Tran") {
                       let split_by_tran = name.split("Tran")
                                           .collect::<Vec::<_>>();
                       let transform_info = split_by_tran.last()
                                               .expect("Tran missing entries");
                       let transform_info = transform_info.split("_").collect::<Vec::<_>>();
                       let x = f32::from_str(transform_info[0]).expect("Transform missing X");
                       let z = f32::from_str(transform_info[1]).expect("Transform missing Z");
                       let mut t = Transform::from_xyz(x, 0.0, z);
                       if !name.contains("player") && !name.contains("bull") {
                           t.rotate_y(TAU / 2.0);
                       }
                       cmds.insert(t);
                   }

                   if name.contains("Group") {
                       let split_by_group = name.split("Group")
                                                .collect::<Vec::<_>>();
                       let group_id = split_by_group.last()
                                                    .expect("Group name missing ID");
                       let group_id = usize::from_str(group_id)
                                           .expect("Group ID not a number");

                       cmds.insert(groups::GroupMarker(group_id));
                       /* 
                          need to break items that have groups
                          then add a debug to throw event that restores the groups
                          to where it was
                      */
                   }
                   if name.contains("AnimationPaMarker") {
                       cmds.insert(cutscene::PaTalkMarker);
                   }
                   if name.contains("AnimationMatMarker") {
                       cmds.insert(cutscene::MatTalkMarker);
                   }
                   if name.contains("mug") {
                       Mug::add_components(cmds, mesh);
                   }
                   if name.contains("plate") {
                       Plate::add_components(cmds, mesh);
                   }
                   if name.contains("fishbowl") {
                       FishBowl::add_components(cmds, mesh);
                   }

                   cmds.insert(CleanupMarker);
               }
           }),
        });

        match game_script_state.current {
            game_script::GameScript::IntroCutscene => {
                commands.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.00,
                });
            },
            game_script::GameScript::PreLevelOneCutscene |
            game_script::GameScript::EndCutscene 
                => {
                commands.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.50,
                });
            },
            game_script::GameScript::LevelThree => {
                commands.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.50,
                });
            },
            game_script::GameScript::LevelTwo => {
                commands.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.50,
                });
            },
            _ => {
                // lights
                commands.insert_resource(AmbientLight {
                    color: Color::WHITE,
                    brightness: 0.50,
                });

                const HALF_SIZE: f32 = 100.0;
                commands.spawn(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        illuminance: 50000.0,
                        color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                        shadow_projection: OrthographicProjection {
                            left: -HALF_SIZE,
                            right: HALF_SIZE,
                            bottom: -HALF_SIZE,
                            top: HALF_SIZE,
                            near: -10.0 * HALF_SIZE,
                            far: 10.0 * HALF_SIZE,
                            ..Default::default()
                        },
                        shadows_enabled: game_state.shadows_on,
                        ..Default::default()
                    },
                    transform: {
                        let mut t = Transform::default();
                        t.rotate_x(-1.6);
                        t
                    },        
                    ..Default::default()
                })
                .insert(cutscene::CutsceneCleanupMarker)
                .insert(CleanupMarker);
            }
        }

        if camera.iter().len() == 0 {
            let shake_id = commands
                .spawn((Shake3d {
                    max_offset: Vec3::new(0.0, 0.0, 0.0),
                    max_yaw_pitch_roll: Vec3::new(0.1, 0.1, 0.1),
                    trauma: 0.0,
                    trauma_power: 2.0,
                    decay: 0.8,
                    random_sources: [
                        Box::new(MyRandom),
                        Box::new(MyRandom),
                        Box::new(MyRandom),
                        Box::new(MyRandom),
                        Box::new(MyRandom),
                        Box::new(MyRandom),
                    ],
                },
                SpatialBundle::default()))
                .id();

            let camera_id =
                game_camera::spawn_camera(
                    &mut commands,
                    CleanupMarker,
                    &game_assets,
                    Vec3::new(
                        game_camera::INGAME_CAMERA_X,
                        game_camera::INGAME_CAMERA_Y,
                        0.0,
                    ),
                    Quat::from_axis_angle(
                        game_camera::INGAME_CAMERA_ROTATION_AXIS,
                        game_camera::INGAME_CAMERA_ROTATION_ANGLE,
                    ),
                );

            commands.entity(shake_id).push_children(&[camera_id]);
        } else {
            // Commented this so that when I refresh the camera stays in the same place
    //      for mut camera in &mut camera {
    //          camera.translation = Vec3::new(
    //              game_camera::INGAME_CAMERA_X,
    //              game_camera::INGAME_CAMERA_Y,
    //              0.0,
    //          );
    //          camera.rotation = Quat::from_axis_angle(
    //              game_camera::INGAME_CAMERA_ROTATION_AXIS,
    //              game_camera::INGAME_CAMERA_ROTATION_ANGLE,
    //          );
    //      }
        }
    }
//  rapier.gravity = Vec3::new(0.0, -9.81, 0.0);
//  rapier.physics_pipeline_active = false;
//  rapier.query_pipeline_active = false;
//  rapier.timestep_mode =  TimestepMode::Variable {
//      time_scale: 0.0,
//      max_dt: 1.0,
//      substeps: 0,
//  };
}

fn handle_lights(
    entity_commands: &mut EntityCommands,
    name: &str,
) {
    if name.contains("Fire") {
        entity_commands
            .insert(PointLight {
                color: Color::rgb(0.78, 0.474, 0.0),
                intensity: 90.0,
                shadows_enabled: true,
                ..default()
            })
            .insert(FireLight {
                light_going_up: true,
                original_translation: None,
                new_target: None,
                fire_jump_time: 0.0,
            });
    }
    if name.contains("Window") {
        entity_commands
            .insert(PointLight {
                color: Color::rgb(1.00, 0.962, 0.779),
                intensity: 100000.0,
                range: 60.6,
                radius: 12.2,
                shadows_enabled: true,
                ..default()
            });
    }
    if name.contains("Jail") {
        entity_commands
            .insert(PointLight {
                color: Color::rgb(1.00, 1.0, 1.0),
                intensity: 2274.0,
                range: 12.5,
                radius: 0.0,
                shadows_enabled: true,
                ..default()
            });
    }

    if name.contains("Fish") {
        entity_commands
            .insert(PointLight {
                color: Color::rgb(0.00, 0.474, 0.78),
                intensity: 2274.0,
                shadows_enabled: true,
                ..default()
            });
    }
//    color:  1.0, .279, 0
//    intensity: 58
// range 20.9
// shadows_enabled
// shadow_depth_bias
// shadow_normal_bias
}

#[derive(Component)]
struct FireLight {
    light_going_up: bool,
    original_translation: Option::<Vec3>,
    new_target: Option::<Vec3>,
    fire_jump_time: f32,
}

fn animate_fire(
    mut fires: Query<(&mut Transform, &GlobalTransform, &mut PointLight, &mut FireLight)>,
    mut dust_spawn_event_writer: EventWriter<dust::DustSpawnEvent>,
    game_assets: ResMut<GameAssets>,
    time: Res<Time>,
) {
    for (mut transform, &global, mut point_light, mut fire) in &mut fires {
        if fire.original_translation.is_none() {
            fire.original_translation = Some(transform.translation);

            dust_spawn_event_writer.send(dust::DustSpawnEvent {
                position: global.compute_transform().translation,
                count: 2,
                spread: 0.5,
                speed: 0.2,
                rate: 0.5,
                dust_time_to_live: 1.5,
                emitter_time_to_live: 99999.0,
                size: 0.6,
                image: game_assets.fire_texture.image.clone(),
                ..default()
            });

            dust_spawn_event_writer.send(dust::DustSpawnEvent {
                position: global.compute_transform().translation,
                count: 1,
                spread: 0.5,
                speed: 2.0,
                rate: 0.7,
                dust_time_to_live: 3.0,
                emitter_time_to_live: 99999.0,
                size: 0.5,
                image: game_assets.cloud_texture.image.clone(),
                ..default()
            });
        }

        let light_speed = 40.0;
        if fire.light_going_up {
            point_light.intensity += time.delta_seconds() * light_speed;
            if point_light.intensity > 150.0 {
                fire.light_going_up = false;
            }
        } else {
            point_light.intensity -= time.delta_seconds() * light_speed;
            if point_light.intensity < 90.0 {
                fire.light_going_up = true;
            }
        }

        fire.fire_jump_time -= time.delta_seconds();
        
        if fire.fire_jump_time < 0.0 {
            let x = fire.original_translation.expect("just set this x").x + random_number() * time.delta_seconds() * 2.0;
            let z = fire.original_translation.expect("just set this z").z + random_number() * time.delta_seconds() * 2.0;
            transform.translation.x = x;
            transform.translation.z = z;
            fire.fire_jump_time = 0.2;
        }
    }
}

#[derive(Component)]
struct RotateEntityMarker;

fn rotate_rotate_entities(
    mut items: Query<&mut Transform, With<RotateEntityMarker>>,
    time: Res<Time>,
) {
    for mut transform in &mut items {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
}
