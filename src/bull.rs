use bevy::prelude::*;
use rand::thread_rng;
use bevy::render::view::NoFrustumCulling;
use std::collections::HashMap;
use bevy_rapier3d::prelude::*;
use bevy_camera_shake::Shake3d;

use crate::{
    assets::GameAssets,
    AppState,
    ZeroSignum,
    player,
    ingame,
};

const TRAUMA_AMOUNT: f32 = 0.5;
pub struct BullPlugin;
impl Plugin for BullPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(update_bull_minds)
                .with_system(animate_bull)
                .with_system(handle_collisions)
                .with_system(update_bulls)
                .with_system(handle_bull_charge_event)
                .with_system(handle_reset_bull_event_handler.before(update_bulls))
        )
        .add_event::<BullChargeEvent>()
        .add_event::<ResetBullEvent>()
        .add_event::<BullMoveEvent>();
    }
}

const CHARGE_LIMIT: f32 = 2.5;

pub struct BullChargeEvent {
    pub charging: bool
}

#[derive(PartialEq)]
pub enum BullState {
    Idle,
    Charging,
    Walking,
    Running,
    Collision,
}

impl Default for BullState {
    fn default() -> BullState {
        BullState::Idle
    }
}

#[derive(Component)]
pub struct Bull {
    pub current_animation: Handle<AnimationClip>,
    pub state: BullState,
    pub random: f32,
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub mind_cooldown: f32,
    pub heading_to: Option::<Vec2>,
    pub charging_cooldown: f32,
}

impl Bull {
    pub fn can_think(&self) -> bool {
        self.mind_cooldown <= 0.0 && self.charging_cooldown <= 0.0
    }
}

impl Default for Bull {
    fn default() -> Bull {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Bull {
            current_animation: Handle::<AnimationClip>::default(),
            state: BullState::Idle,
            speed: 20.0,
            rotation_speed: 1.0,
            friction: 0.01,
            random: rng.gen_range(0.5..1.0),
            mind_cooldown: 0.0,
            charging_cooldown: 0.0,
            heading_to: None,
        }
    }
}


fn handle_collisions(
    mut contact_force_events: EventReader<ContactForceEvent>,
    mut bulls: Query<(&mut Bull, &mut Velocity)>,
    walls: Query<&ingame::Wall>,
    mut shakeables: Query<&mut Shake3d>,
) {
    for e in contact_force_events.iter() {
        println!("e: {}", e.total_force_magnitude);
        let is_wall = walls.get(e.collider1).is_ok() || walls.get(e.collider2).is_ok();
        if is_wall {
            println!("hit wall");
            for (mut bull, mut velocity) in &mut bulls {
                if bull.state == BullState::Running {
                    for mut shakeable in shakeables.iter_mut() {
                        shakeable.trauma = f32::min(shakeable.trauma + TRAUMA_AMOUNT, 1.0);
                    }
                    bull.state = BullState::Collision;
                    bull.charging_cooldown = 1.0;

                    velocity.linvel = -velocity.linvel;
                }
            }
        }
    }
}

pub struct ResetBullEvent(Entity);

pub struct BullMoveEvent {
    pub entity: Entity,
    pub direction: Vec2,
}

fn handle_bull_charge_event(
    mut charge_event_reader: EventReader<BullChargeEvent>,
    mut bulls: Query<&mut Bull>,
) {
    for event in charge_event_reader.iter() {
        for mut bull in &mut bulls {
            if event.charging && bull.charging_cooldown <= 0.0 {
                bull.charging_cooldown = CHARGE_LIMIT;
                bull.state = BullState::Charging;
            } else if !event.charging {
                // ?
            }
        }
    }
}

fn animate_bull( 
    mut bulls: Query<(Entity, &mut Bull)>,
    mut animations: Query<&mut AnimationPlayer>,
    game_assets: ResMut<GameAssets>,
) {
    for (entity, mut bull) in &mut bulls {
        let mut animation = animations.get_mut(entity).unwrap();

        match bull.state {
            BullState::Collision => {
                if bull.current_animation != game_assets.bull_collide {
                    animation.play(game_assets.bull_collide.clone_weak());
                    animation.resume();
                    bull.current_animation = game_assets.bull_collide.clone_weak();
                }
                animation.set_speed(2.0);
            },
            BullState::Walking => {
                if bull.current_animation != game_assets.bull_walk {
                    animation.play(game_assets.bull_walk.clone_weak()).repeat();
                    animation.resume();
                    bull.current_animation = game_assets.bull_walk.clone_weak();
                }
                animation.set_speed(2.0);
            },
            BullState::Charging => {
                if bull.current_animation != game_assets.bull_charge {
                    animation.play(game_assets.bull_charge.clone_weak()).repeat();
                    animation.resume();
                    bull.current_animation = game_assets.bull_charge.clone_weak();
                }
                animation.set_speed(2.0);
            },
            BullState::Running => {
                if bull.current_animation != game_assets.bull_run {
                    animation.play(game_assets.bull_run.clone_weak()).repeat();
                    animation.resume();
                    bull.current_animation = game_assets.bull_run.clone_weak();
                }
                animation.set_speed(3.0);
            },
            _ => {
                if bull.current_animation != game_assets.bull_idle {
                    animation.play(game_assets.bull_idle.clone_weak()).repeat();
                    animation.resume();
                    bull.current_animation = game_assets.bull_idle.clone_weak();
                }
                animation.set_speed(2.0);
            }
        }
    }
}

fn handle_reset_bull_event_handler(
    mut commands: Commands,
    mut reset_bull_event_writer: EventReader<ResetBullEvent>,
    mut bulls: Query<(&mut Transform, &mut GlobalTransform, &mut Velocity), With<Bull>>,
) {
    for event in reset_bull_event_writer.iter() {
        println!("AHHH");

        for (mut transform, mut global_transform, mut velocity) in &mut bulls {
            velocity.linvel = Vec3::default();
            velocity.angvel = Vec3::default();
            *transform = Transform::from_xyz(0.0, 2.0, 0.0);
            *global_transform = GlobalTransform::from_xyz(0.0, 2.0, 0.0);
        }
//      commands.entity(event.0)
//              .insert(RigidBody::Dynamic)
//              .insert(Collider::cuboid(1.0, 1.0, 1.0));
//      if let Ok(mut t) = transforms.get_mut(event.0) {
//          t.translation = Vec3::new(0.0, 2.0, 0.0);
//          println!("set transform");
//      }
    }
}

fn update_bulls(
    mut commands: Commands,
    time: Res<Time>,
    mut bulls: Query<(Entity, &mut Transform, &mut Bull, &mut Velocity), Without<player::Player>>,
    mut bull_move_event_reader: EventReader<BullMoveEvent>,
    players: Query<(&Transform, &player::Player), Without<Bull>>,
    mut reset_bull_event_writer: EventWriter<ResetBullEvent>,
) {
    let mut move_events = HashMap::new();
    for move_event in bull_move_event_reader.iter() {
        move_events.entry(move_event.entity).or_insert(move_event);
    }

    for (entity, mut transform, mut bull, mut velocity) in bulls.iter_mut() {
        if transform.translation.is_nan() {
            println!("is nan!");
            println!("A{:?} {:?} {:?}", transform.translation, velocity.linvel, velocity.angvel);
            velocity.linvel = Vec3::default();
            velocity.angvel = Vec3::default();

            println!("B{:?} {:?} {:?}", transform.translation, velocity.linvel, velocity.angvel);
            reset_bull_event_writer.send(ResetBullEvent(entity));
            continue;
        }

        if bull.state == BullState::Idle {
            continue;
        }

        if bull.state == BullState::Charging {
            for (player_transform, _) in &players {
                let player_translation = player_transform.translation;
                let bull_translation = transform.translation;
                let angle = (-(player_translation.z - bull_translation.z))
                       .atan2(player_translation.x - bull_translation.x);
                let rotation = Quat::from_axis_angle(Vec3::Y, angle);
                transform.rotation = rotation;
            }
            bull.charging_cooldown -= time.delta_seconds();

            if bull.charging_cooldown <= 0.0 {
                bull.state = BullState::Running;
                bull.charging_cooldown = CHARGE_LIMIT * 2.0;
            }

            continue;
        }

        let speed: f32 = match bull.state {
                             BullState::Running => bull.speed * 2.5,
                             BullState::Collision => bull.speed * 1.5,
                             _ => bull.speed,
                         };
        let rotation_speed: f32 = bull.rotation_speed;
        let friction: f32 = bull.friction;

        velocity.linvel *= friction.powf(time.delta_seconds());
        match bull.state {
            BullState::Running => {
                bull.charging_cooldown -= time.delta_seconds();
                if bull.charging_cooldown <= CHARGE_LIMIT {
                    let direction = velocity.linvel.normalize();
                    velocity.linvel += (direction * speed) * time.delta_seconds();
                } else {
                    for (player_transform, _) in &players {
                        let acceleration = player_transform.translation - transform.translation;
                        let acceleration = Vec3::new(acceleration.x, 0.0, acceleration.z);
                        velocity.linvel += (acceleration.normalize() * speed) * time.delta_seconds();
                    }
                }

                if bull.charging_cooldown <= 0.0 {
                    bull.state = BullState::Idle;
                }
            },
            BullState::Collision => {
                bull.charging_cooldown -= time.delta_seconds();
                let direction = velocity.linvel.normalize();
                velocity.linvel += (direction * speed) * time.delta_seconds();

                if bull.charging_cooldown <= 0.0 {
                    bull.state = BullState::Idle;
                }
            },
            _ => {
                if let Some(move_event) = move_events.get(&entity) {
                    let acceleration = Vec3::new(move_event.direction.x, 0.0, move_event.direction.y);
                    velocity.linvel += (acceleration.zero_signum() * speed) * time.delta_seconds();
                }
            }
        }

        velocity.linvel = velocity.linvel.clamp_length_max(speed);

        let linvel = if bull.state == BullState::Collision { -velocity.linvel } else { velocity.linvel };
        let mut new_translation = transform.translation + (linvel * time.delta_seconds());

        let angle = (-(new_translation.z - transform.translation.z))
            .atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);

        let new_rotation = transform
            .rotation
            .lerp(rotation, time.delta_seconds() * rotation_speed);

        // don't rotate if we're not moving or if uhh rotation isnt a number?? why isn't it a number? who did this
        if !rotation.is_nan() && linvel.length() > 1.0 {
            transform.rotation = rotation;
        }
    }
}

fn update_bull_minds(
    time: Res<Time>,
    mut bulls: Query<(Entity, &mut Transform, &mut Bull)>,
    mut bull_move_event_writer: EventWriter<BullMoveEvent>,
) {
    for (entity, mut transform, mut bull) in bulls.iter_mut() {
        // handling mind cool down
        bull.mind_cooldown -= time.delta_seconds();
        bull.mind_cooldown = bull.mind_cooldown.clamp(-10.0, 30.0);

        if let Some(heading_to) = bull.heading_to {
            bull_move_event_writer.send(BullMoveEvent {
                entity,
                direction: heading_to,
            });
        }

        if !bull.can_think() {
            continue;
        }

        bull.state = match bull.state {
            BullState::Walking => {
                BullState::Idle
            },
            _ => {
                let random_direction = get_random_direction();
                bull.heading_to = Some(random_direction);
                BullState::Walking
            },
        };

        bull.mind_cooldown = 2.0;
    }
}

fn get_random_direction() -> Vec2 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let x: f32 = rng.gen_range(-100.0..100.0);
    let z: f32 = rng.gen_range(-100.0..100.0);

    Vec2::new(x, z).normalize()
}
