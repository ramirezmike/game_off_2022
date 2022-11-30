use crate::{assets::GameAssets, player};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::camera::{PerspectiveProjection, RenderTarget, ScalingMode};
use bevy::render::view::RenderLayers;
use bevy::render::camera::Projection;
use std::f32::consts::TAU;

//) Vec3(-0.7463726, -2.7414508, -0.2843299)

pub const INGAME_CAMERA_X: f32 = -29.928919;
pub const INGAME_CAMERA_Y: f32 = 6.384182;
pub const INGAME_CAMERA_ROTATION_AXIS: Vec3 = Vec3::new(0.24503553, 0.93911797, 0.24086143);
pub const INGAME_CAMERA_ROTATION_ANGLE: f32 = 4.6667724;

#[derive(Component)]
pub struct PanOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}


/*

   -93 x was around 60 bottom

   */

pub fn light_follow_camera(
    mut lights: Query<&mut DirectionalLight>,
    cameras: Query<&Transform, With<PanOrbitCamera>>,
    time: Res<Time>,
) {
    for mut directional_light in lights.iter_mut() {
        for camera_transform in cameras.iter() {
            directional_light.shadow_projection.left = camera_transform.translation.x - 100.0;
            directional_light.shadow_projection.right = camera_transform.translation.x + 100.0;
        }
    }
}

pub fn pan_orbit_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    time: Res<Time>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let orbit_key = KeyCode::LShift;
    let pan_button = MouseButton::Middle;
    let pan_key = KeyCode::LAlt;
    let zoom_key = KeyCode::LControl;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) || keyboard_input.pressed(orbit_key) {
        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) || keyboard_input.pressed(pan_key) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.iter() {
            pan += ev.delta;
        }
    }
    if keyboard_input.pressed(zoom_key) {
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }
    }

    if input_mouse.just_released(orbit_button)
        || input_mouse.just_pressed(orbit_button)
        || keyboard_input.just_released(orbit_key)
        || keyboard_input.just_pressed(orbit_key)
    {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let camera_speed = 30.0;

        if keyboard_input.pressed(KeyCode::Up) {
            let forward = transform.forward();
            transform.translation += forward * camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Down) {
            let back = transform.back();
            transform.translation += back * camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Left) {
            let left = transform.left();
            transform.translation += left * camera_speed * time.delta_seconds();
        }
        if keyboard_input.pressed(KeyCode::Right) {
            let right = transform.right();
            transform.translation += right * camera_speed * time.delta_seconds();
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * TAU;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            match projection {
                Projection::Perspective(perspective) => {
                    pan *= Vec2::new(perspective.fov * perspective.aspect_ratio, perspective.fov) / window;
                },
                _ => ()
            }
            
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    Vec2::new(window.width() as f32, window.height() as f32)
}

pub fn look_at_player(
    players: Query<&Transform, (With<player::Player>, Without<Camera3d>)>,
    mut cameras: Query<&mut Transform, (With<Camera3d>, Without<player::Player>)>,
    time: Res<Time>,
) {
    for mut camera_transform in &mut cameras {
        for player_transform in &players {
            camera_transform.look_at(player_transform.translation, Vec3::Y);

            if player_transform.translation.x >= -2.0 {
                let target = Vec3::new(-15.0, INGAME_CAMERA_Y, 0.0);
                let diff = target - camera_transform.translation;
                camera_transform.translation += diff * time.delta_seconds();
            } else {
                let target = Vec3::new(INGAME_CAMERA_X, INGAME_CAMERA_Y, 0.0);
                let diff = target - camera_transform.translation;
                camera_transform.translation += diff * time.delta_seconds();
            }
        }
    }
}

pub fn spawn_camera<T: Component + Clone>(
    commands: &mut Commands,
    cleanup_marker: T,
    game_assets: &Res<GameAssets>,
    translation: Vec3,
    rotation: Quat,
) -> Entity {
    let radius = translation.length();

    println!("Spawning camera");
    let id = 
    commands
        .spawn(Camera3dBundle {
            transform: {
                let mut t = Transform::from_translation(translation);
                t.rotation = rotation;
//                t.rotate_x(-1.602);
//                t.rotate_y(-1.264);
//                t.rotate_z(-1.603);

                t
            },
            camera: Camera {
                priority: 0,
                ..default()
            },
            //      projection: OrthographicProjection {
            //          scale: 10.0,
            //          scaling_mode: ScalingMode::FixedVertical(1.0),
            //          near: -100.0,
            //          ..default()
            //      }.into(),
            ..default()
        })
//        .insert(cleanup_marker.clone())
        .insert(PanOrbitCamera {
            radius,
            //focus: Vec3::new(-1.0900329, -2.5414279, -1.4901161),
            ..Default::default()
        })
        .id();
    id
}
