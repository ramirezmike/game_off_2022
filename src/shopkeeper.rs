use bevy::prelude::*;
use bevy::ecs::system::EntityCommands;
use crate::{
    AppState, assets::GameAssets, game_state, groups, ZeroSignum, dust, score, follow_text,
};
use std::collections::HashMap;

const DUST_RATE: f32 = 0.2;
const REPAIR_TIME: f32 = 5.0;
pub struct ShopKeeperPlugin;
impl Plugin for ShopKeeperPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(think_shopkeepers)
                .with_system(move_shopkeepers)
            );
    }
}

enum ShopKeeperState {
    Normal,
    Repairing(usize, f32),
}

#[derive(Component)]
struct ShopKeeper {
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub velocity: Vec3,
    pub cleanup_cooldown: f32,
    pub state: ShopKeeperState,
    pub dust_cooldown: f32,
    pub current_animation: Handle<AnimationClip>,
    pub target: Option::<(usize, Vec3)>,
    pub initial_position: Option::<Vec3>,
}

impl Default for ShopKeeper {
    fn default() -> ShopKeeper {
        ShopKeeper {
            speed: 42.0,
            rotation_speed: 1.0,
            friction: 0.01,
            velocity: Vec3::default(),
            state: ShopKeeperState::Normal,
            cleanup_cooldown: 10.0,
            current_animation: Handle::<AnimationClip>::default(),
            dust_cooldown: 0.0,
            target: None,
            initial_position: None,
        }
    }
}

pub fn spawn(name: &str, commands: &mut EntityCommands) {
    if name.contains("shopkeeper") {
        commands.insert((
            ShopKeeper::default()
        ));
    }
}

fn move_shopkeepers(
    mut shopkeepers: Query<(Entity, &mut ShopKeeper, &mut Transform)>,
    mut animations: Query<&mut AnimationPlayer>,
    mut game_state: ResMut<game_state::GameState>,
    time: Res<Time>,
    game_assets: ResMut<GameAssets>,
    mut restore_group_event_writer: EventWriter<groups::RestoreGroupEvent>,
    mut dust_spawn_event_writer: EventWriter<dust::DustSpawnEvent>,
) {
    for (entity, mut keeper, mut keeper_transform) in &mut shopkeepers {
        match keeper.state {
            ShopKeeperState::Repairing(group_id, remaining_time) => {
                let remaining_time = remaining_time - time.delta_seconds();

                if remaining_time <= 0.0 {
                    restore_group_event_writer.send(groups::RestoreGroupEvent {
                        group_id
                    });

                    // go to first spot or just wait
                    if group_id != 0 {
                        keeper.target = Some((0, keeper.initial_position.expect("Initial position missing")));
                    } else {
                        keeper.target = None;
                    }

                    keeper.cleanup_cooldown = 0.0;
                    keeper.state = ShopKeeperState::Normal;
                } else {
                    keeper.state = ShopKeeperState::Repairing(group_id, remaining_time);
                    continue;
                }
            },
            _ => ()
        }
        if keeper.initial_position.is_none() {
            keeper.initial_position = Some(keeper_transform.translation);
        }

        let speed: f32 = keeper.speed;
        let rotation_speed: f32 = keeper.rotation_speed;
        let friction: f32 = keeper.friction;

        keeper.velocity *= friction.powf(time.delta_seconds());

        if let Some((group_id, target)) = keeper.target {
            let target = Vec3::new(target.x, 0.0, target.z);
            if keeper_transform.translation.distance(target) < 0.8 {

                // println!("AT TARGET {:?} {:?}", keeper_transform.translation, target);
                keeper.state = ShopKeeperState::Repairing(group_id, REPAIR_TIME);
                dust_spawn_event_writer.send(dust::DustSpawnEvent {
                    position: keeper_transform.translation,
                    count: 3,
                    spread: 6.0,
                    rate: 0.5,
                    dust_time_to_live: 3.0,
                    emitter_time_to_live: REPAIR_TIME,
                    size: 2.0,
                    image: game_assets.cloud_texture.image.clone(),
                    ..default()
                });
                dust_spawn_event_writer.send(dust::DustSpawnEvent {
                    position: keeper_transform.translation,
                    count: 1,
                    spread: 6.0,
                    speed: 2.0,
                    rate: 0.2,
                    dust_time_to_live: 3.0,
                    emitter_time_to_live: REPAIR_TIME,
                    size: 1.5,
                    image: game_assets.wrench_texture.image.clone(),
                    ..default()
                });
            } else {
                //println!("Moving to target? {} to {}", keeper_transform.translation, target);
                let direction = target - keeper_transform.translation;
                let acceleration = Vec3::from(direction);

                keeper.velocity += (acceleration.zero_signum() * speed) * time.delta_seconds();
                keeper.velocity = keeper.velocity.clamp_length_max(speed);

                let angle = (-(target.z - keeper_transform.translation.z))
                    .atan2(target.x - keeper_transform.translation.x);
                let rotation = Quat::from_axis_angle(Vec3::Y, angle);

                if !rotation.is_nan() {
                    keeper_transform.rotation = rotation;
                }

                keeper.dust_cooldown -= time.delta_seconds();
                keeper.dust_cooldown = keeper.dust_cooldown.clamp(0.0, 10.0);
                if keeper.dust_cooldown <= 0.0 {
                    dust_spawn_event_writer.send(dust::DustSpawnEvent {
                        position: keeper_transform.translation,
                        count: 1,
                        image: game_assets.cloud_texture.image.clone(),
                        ..default()
                    });
                    keeper.dust_cooldown = DUST_RATE;
                }
            }
        }
            
        let new_translation = keeper_transform.translation + (keeper.velocity * time.delta_seconds());
        keeper_transform.translation = new_translation;

        let mut animation = animations.get_mut(entity).unwrap();
        if keeper.velocity.length() > 1.0 {
            if keeper.current_animation != game_assets.matador_run {
                animation.play(game_assets.matador_run.clone_weak()).repeat();
                animation.resume();
                keeper.current_animation = game_assets.matador_run.clone_weak();
            } 
            animation.set_speed(keeper.velocity.length() / 2.0);
        } else {
            if keeper.current_animation != game_assets.matador_idle {
                animation.play(game_assets.matador_idle.clone_weak()).repeat();
                animation.resume();
                keeper.current_animation = game_assets.matador_idle.clone_weak();
                animation.set_speed(4.0);
            } 
        }
    }
}

fn think_shopkeepers(
    mut shopkeepers: Query<(Entity, &mut ShopKeeper)>,
    time: Res<Time>,
    group_members: Query<(&groups::GroupMember, &Transform, &GlobalTransform)>,
    mut follow_text_event_writer: EventWriter<follow_text::FollowTextEvent>,
) {
    for (entity, mut keeper) in &mut shopkeepers {
// this makes keeper check constantly
//        if keeper.target.is_some() && keeper.target.unwrap().0 != 0 { continue; } 
        if keeper.target.is_some() { continue; }

        keeper.cleanup_cooldown -= time.delta_seconds();
        keeper.cleanup_cooldown = keeper.cleanup_cooldown.clamp(0.0, 10.0);

        if keeper.cleanup_cooldown <= 0.0 {

            let grouped_translations = 
                group_members.iter()
                             .fold(HashMap::<usize, Vec::<(Vec3, Vec3, Vec3)>>::new(),
                             |mut acc, (group_member, transform, global_transform)| {
                                 acc.entry(group_member.group_id)
                                    .or_insert(vec!())
                                    .push((transform.translation, group_member.original_transform.translation, global_transform.translation()));

                                 acc
                             });
            'outer: for (group_id, translations) in grouped_translations.iter() {
                for (current, original, target) in translations.iter() {
                    let distance = (current.y - original.y).abs();
                    if distance > score::BREAK_DISTANCE {
                        println!("setting target! {}", target);
                        keeper.target = Some((*group_id, target.clone()));
                        follow_text_event_writer.send(follow_text::FollowTextEvent {
                            follow: follow_text::FollowThing::Entity(entity),
                            text: "I can fix that!".to_string(),
                            color: Color::WHITE,
                            time_to_live: 6.0,
                        });
                        break 'outer;
                    }
                }
            }

            keeper.cleanup_cooldown = 10.0;
        }
    }
}
