use crate::{
    asset_loading, assets::GameAssets, cleanup, game_state, AppState, game_camera, player, bull, 
    DampPhysics, props::*,
};
use bevy::prelude::*;
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

pub struct InGamePlugin;
impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
            .add_system_set(
                SystemSet::on_exit(AppState::InGame).with_system(cleanup::<CleanupMarker>),
            )
            .add_system_set(SystemSet::on_update(AppState::ResetInGame).with_system(reset_ingame))
            .add_plugin(HookPlugin)
            .add_plugin(CameraShakePlugin)
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
//                  .with_system(game_camera::follow_player.after(player::move_player))
//                  .with_system(game_camera::light_follow_camera.after(player::move_player))
//                    .with_system(pass_speed_to_shader.after(player::move_player))
                    .with_system(collision_report)
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
) {
    assets_handler.load(AppState::InGame, &mut game_assets, &mut game_state);
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
    assets_handler.add_glb(&mut game_assets.level_one, "models/level_one.glb");
    assets_handler.add_glb(&mut game_assets.broken_plate, "models/broken_plate.glb");
    assets_handler.add_glb(&mut game_assets.broken_mug, "models/broken_mug.glb");
    assets_handler.add_font(&mut game_assets.font, "fonts/monogram.ttf");
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
    mut rapier: ResMut<RapierConfiguration>
) {
    println!("Setting up ingame!");
    game_state.title_screen_cooldown = 1.0;
//  rapier.gravity = Vec3::new(0.0, -9.81, 0.0);
//  rapier.physics_pipeline_active = false;
//  rapier.query_pipeline_active = false;
//  rapier.timestep_mode =  TimestepMode::Variable {
//      time_scale: 0.0,
//      max_dt: 1.0,
//      substeps: 0,
//  };

//  if let Some(gltf) = assets_gltf.get(&game_assets.matador.clone()) {
//      .insert(CleanupMarker);
//  }

    if let Some(gltf) = assets_gltf.get(&game_assets.level_one.clone()) {
        commands.spawn_bundle(HookedSceneBundle {

           scene: SceneBundle { scene: gltf.scenes[0].clone(), ..default() },
           hook: SceneHook::new(move |entity, cmds, mesh| {
               if let Some(name) = entity.get::<Name>().map(|t|t.as_str()) {
                   println!("Name: {} Mesh: {:?}", name, mesh.is_some());
                   if name.contains("static") {
                       if let Some(mesh) = mesh {
                           println!("adding collider");
                           cmds.insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap())
                               .insert(BullCollide);
                       }
                   }
                   if name.contains("invisible") {
                       cmds.insert(Visibility {
                           is_visible: false
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
                       cmds.insert_bundle(player::PlayerBundle::new())
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
                   if name.contains("collide") {
                       cmds.insert((BullCollide, ExternalImpulse::default(), ExternalForce::default()));
                   }
                   if name.contains("mug") {
                       Mug::add_components(cmds);
                   }
                   if name.contains("plate") {
                       Plate::add_components(cmds);
                   }

                   cmds.insert(CleanupMarker);
               }
           }),
        });
    }

    // lights
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    const HALF_SIZE: f32 = 100.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100000.0,
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
            t.rotate_x(5.00);
            t
        },        
        ..Default::default()
    })
    .insert(CleanupMarker);

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
