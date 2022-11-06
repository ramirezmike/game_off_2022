use crate::{
    asset_loading, assets::GameAssets, cleanup, game_state, AppState, game_camera, player, 
};
use bevy::prelude::*;
use bevy::gltf::Gltf;
use leafwing_input_manager::prelude::*;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::f32::consts::TAU;
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
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
//                  .with_system(game_camera::follow_player.after(player::move_player))
//                  .with_system(game_camera::light_follow_camera.after(player::move_player))
//                    .with_system(pass_speed_to_shader.after(player::move_player))
                    .with_system(game_camera::pan_orbit_camera),
            );
    }
}

#[derive(Component, Copy, Clone)]
pub struct CleanupMarker;

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
    assets_handler.add_glb(&mut game_assets.matador, "models/person.glb");
    assets_handler.add_animation(&mut game_assets.matador_run,"models/person.glb#Animation2");
    assets_handler.add_animation(&mut game_assets.matador_idle,"models/person.glb#Animation1");
    assets_handler.add_glb(&mut game_assets.plate, "models/plate.glb");
    assets_handler.add_glb(&mut game_assets.level_one, "models/level_one.glb");
}

pub fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    assets_gltf: Res<Assets<Gltf>>,
    mut game_state: ResMut<game_state::GameState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Query<&mut Transform, With<game_camera::PanOrbitCamera>>,
) {
    println!("Setting up ingame!");
    game_state.title_screen_cooldown = 1.0;

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
                           cmds.insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap());
                       }
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
                           children.spawn()
                               .insert(Collider::cuboid(0.2, 1.0, 0.2))
                               // Position the collider relative to the rigid-body.
                               .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 1.0, 0.0)));
                       });
                   }
                   if name.contains("dynamic") {
                       if let Some(mesh) = mesh {
                           cmds.insert(Restitution::coefficient(0.2))
                               .insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap())
                               .insert(RigidBody::Dynamic);
                       }
                   }
                   if name.contains("plate") {
                       println!("adding collider plate");
                       cmds.insert(Restitution::coefficient(0.2))
                           .insert(Collider::cuboid(1.0, 0.1, 1.0))
                           .insert(RigidBody::Dynamic);
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
