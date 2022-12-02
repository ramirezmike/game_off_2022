use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use bevy::gltf::Gltf;
use crate::{
    AppState, assets::GameAssets, game_state, groups, ZeroSignum, dust, score, player, follow_text,
};
use std::collections::HashMap;
use std::str::FromStr;

const DUST_RATE: f32 = 0.2;
const REPAIR_TIME: f32 = 5.0;
pub struct FishMongerPlugin;
impl Plugin for FishMongerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(handle_chase_event)
                .with_system(move_fishmongers)
            )
            .add_event::<ChaseEvent>();
    }
}

enum FishMongerState {
    Normal,
    Returning,
    Chasing,
}

#[derive(Component)]
struct FishMongerMeshMarker;

#[derive(Component)]
struct FishMonger {
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub velocity: Vec3,
    pub cleanup_cooldown: f32,
    pub state: FishMongerState,
    pub dust_cooldown: f32,
    pub current_animation: Handle<AnimationClip>,
    pub target: Option::<Vec3>,
    pub initial_position: Option::<Vec3>,
}

impl Default for FishMonger {
    fn default() -> FishMonger {
        FishMonger {
            speed: 42.0,
            rotation_speed: 1.0,
            friction: 0.01,
            velocity: Vec3::default(),
            state: FishMongerState::Normal,
            cleanup_cooldown: 10.0,
            current_animation: Handle::<AnimationClip>::default(),
            dust_cooldown: 0.0,
            target: None,
            initial_position: None,
        }
    }
}

pub fn spawn(name: &str, commands: &mut EntityCommands) {
    if name.contains("fishmonger") {
        commands.insert(FishMonger::default());
    }

    if name.contains("aquariumherring") {
        commands.insert(AquariumFishMarker);
    }

    if name.contains("fishherring") {
        commands.insert(FishMongerFishMarker)
                .insert(Visibility { is_visible: false });
    }
}

#[derive(Component)]
struct AquariumFishMarker;

#[derive(Component)]
struct FishMongerFishMarker;

fn move_fishmongers( //, &mut Handle<Gltf>
    mut fishmongers: Query<(Entity, &mut FishMonger, &mut Transform), Without<player::Player>>,
    mut fishmonger_fishes: Query<(&FishMongerFishMarker, &mut Visibility), Without<AquariumFishMarker>>,
    mut aquarium_fishes: Query<(&AquariumFishMarker, &mut Visibility), Without<FishMongerFishMarker>>,
    mut animations: Query<&mut AnimationPlayer>,
    mut game_state: ResMut<game_state::GameState>,
    mut player: Query<&Transform, (With<player::Player>, Without<FishMonger>)>,
    time: Res<Time>,
    game_assets: ResMut<GameAssets>,
    mut restore_group_event_writer: EventWriter<groups::RestoreGroupEvent>,
    mut dust_spawn_event_writer: EventWriter<dust::DustSpawnEvent>,
    mut hit_player_event_writer: EventWriter<player::HitPlayerEvent>,
) {
//
    for (entity, mut monger, mut monger_transform) in &mut fishmongers {

        let mut animation = animations.get_mut(entity).unwrap();
        match monger.state {
            FishMongerState::Returning => {
                let target = monger.initial_position.expect("Initial position missing");
                if monger_transform.translation.distance(target) < 0.2 {
                    monger.state = FishMongerState::Normal;
                    // turn back gltf
                } else {
                    monger.target = Some(target);
                }
            },
            FishMongerState::Chasing => {
                let mut target = Vec3::default();
                for p in &player {
                    target = p.translation;
                }

                for (_, mut v) in &mut fishmonger_fishes {
                    v.is_visible = true;
                }
                for (_, mut v) in &mut aquarium_fishes {
                    v.is_visible = false;
                }

                if monger_transform.translation.distance(target) < 0.2 {
                    monger.state = FishMongerState::Returning;
                    hit_player_event_writer.send(player::HitPlayerEvent);
                } else {
                    monger.target = Some(target);
                }
            },
            _ =>  {
                for (_, mut v) in &mut fishmonger_fishes {
                    v.is_visible = false;
                }
                for (_, mut v) in &mut aquarium_fishes {
                    v.is_visible = true;
                }
                if monger.current_animation != game_assets.matador_idle {
                    animation.play(game_assets.matador_idle.clone_weak()).repeat();
                    animation.resume();
                    monger.current_animation = game_assets.matador_idle.clone_weak();
                    animation.set_speed(4.0);
                } 
                continue;
            }
        }
        if monger.initial_position.is_none() {
            monger.initial_position = Some(monger_transform.translation);
        }

        let speed: f32 = monger.speed;
        let rotation_speed: f32 = monger.rotation_speed;
        let friction: f32 = monger.friction;

        monger.velocity *= friction.powf(time.delta_seconds());

        if let Some(target) = monger.target {
            let target = Vec3::new(target.x, 0.0, target.z);
            //println!("Moving to target? {} to {}", monger_transform.translation, target);
            let direction = target - monger_transform.translation;
            let acceleration = Vec3::from(direction);

            monger.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
            monger.velocity = monger.velocity.clamp_length_max(speed);

            let angle = (-(target.z - monger_transform.translation.z))
                .atan2(target.x - monger_transform.translation.x);
            let rotation = Quat::from_axis_angle(Vec3::Y, angle);

            if !rotation.is_nan() {
                monger_transform.rotation = rotation;
            }

            monger.dust_cooldown -= time.delta_seconds();
            monger.dust_cooldown = monger.dust_cooldown.clamp(0.0, 10.0);
            if monger.dust_cooldown <= 0.0 {
                dust_spawn_event_writer.send(dust::DustSpawnEvent {
                    position: monger_transform.translation,
                    count: 1,
                    image: game_assets.cloud_texture.image.clone(),
                    ..default()
                });
                monger.dust_cooldown = DUST_RATE;
            }
        }
            
        let new_translation = monger_transform.translation + (monger.velocity * time.delta_seconds());
        monger_transform.translation = new_translation;

        if monger.velocity.length() > 1.0 {
            if monger.current_animation != game_assets.matador_run {
                animation.play(game_assets.matador_run.clone_weak()).repeat();
                animation.resume();
                monger.current_animation = game_assets.matador_run.clone_weak();
            } 
            animation.set_speed(monger.velocity.length() / 2.0);
        } else {
            if monger.current_animation != game_assets.matador_idle {
                animation.play(game_assets.matador_idle.clone_weak()).repeat();
                animation.resume();
                monger.current_animation = game_assets.matador_idle.clone_weak();
                animation.set_speed(4.0);
            } 
        }
    }
}

#[derive(Default)]
pub struct ChaseEvent;

fn handle_chase_event(
    mut chase_event_reader: EventReader<ChaseEvent>,
    mut fishmongers: Query<(Entity, &mut FishMonger)>,
    mut follow_text_event_writer: EventWriter<follow_text::FollowTextEvent>,
) {
    for _ in chase_event_reader.iter() {
        for (entity, mut monger) in &mut fishmongers {
            monger.state = FishMongerState::Chasing;
            follow_text_event_writer.send(follow_text::FollowTextEvent {
                follow: follow_text::FollowThing::Entity(entity),
                text: "YOU MONSTER!".to_string(),
                color: Color::WHITE,
                time_to_live: 6.0,
            });
        }
    }
}
