use crate::{
    assets::GameAssets,
    direction,
    game_controller,
    game_state,
    AppState,
    ZeroSignum,
    bull,
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;
use std::collections::HashMap;
use bevy_rapier3d::prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_event::<PlayerMoveEvent>()
            .add_event::<HitPlayerEvent>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(handle_controllers.before(handle_input))
                    .with_system(handle_input)
                    .with_system(handle_hit_player_event)
                    .with_system(move_player.after(handle_input)),
            );
    }
}

pub struct HitPlayerEvent;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Player {
    pub speed: f32,
    pub rotation_speed: f32,
    pub friction: f32,
    pub random: f32,
    pub current_animation: Handle<AnimationClip>,
    pub state: PlayerState,
    pub dive_cooldown: f32,
    pub hit_cooldown: f32,
}

impl Player {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        Player {
            speed: 40.0,
            rotation_speed: 1.0,
            friction: 0.10,
            random: rng.gen_range(0.5..1.0),
            current_animation: Handle::<AnimationClip>::default(),
            state: PlayerState::Normal,
            dive_cooldown: 0.0,
            hit_cooldown: 0.0,
        }
    }
}

#[derive(Reflect, Clone, PartialEq)]
pub enum PlayerState {
    Normal,
    Diving,
    Charging,
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState::Normal
    }
}

pub fn handle_hit_player_event(
    mut hit_player_event_reader: EventReader<HitPlayerEvent>,
    mut animations: Query<&mut AnimationPlayer>,
    game_assets: ResMut<GameAssets>,
    mut players: Query<(Entity, &mut Player)>,
) {
    for _ in hit_player_event_reader.iter() {
        println!("hit player event!");
        for (entity, mut player) in &mut players {
            let mut animation = animations.get_mut(entity).unwrap();
            if player.current_animation != game_assets.matador_dive {
                animation.play(game_assets.matador_dive.clone_weak());
                player.current_animation = game_assets.matador_dive.clone_weak();
                animation.set_speed(5.25);
            }
            player.hit_cooldown = 3.0;
        }
    }
}

pub fn move_player(
    time: Res<Time>,
    mut players: Query<(Entity, &mut Transform, &mut Player, &mut Velocity), (Without<bull::Bull>, Without<Camera3d>)>,
    bull: Query<(&Transform, &bull::Bull), Without<Player>>,
    mut player_move_event_reader: EventReader<PlayerMoveEvent>,
    mut animations: Query<&mut AnimationPlayer>,
    mut game_state: ResMut<game_state::GameState>,
    cameras: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    game_assets: ResMut<GameAssets>,
) {
    let mut move_events = HashMap::new();
    for move_event in player_move_event_reader.iter() {
        move_events.entry(move_event.entity).or_insert(move_event);
    }

    for (entity, mut transform, mut player, mut velocity) in players.iter_mut() {
        player.hit_cooldown -= time.delta_seconds();
        player.hit_cooldown = player.hit_cooldown.clamp(0.0, 10.0);
        if player.hit_cooldown > 0.0 {
            continue;
        }

        let speed: f32 = match player.state {
                             PlayerState::Normal => player.speed,
                             PlayerState::Charging => player.speed * 0.25,
                             PlayerState::Diving => player.speed * 2.25,
                         };

        let rotation_speed: f32 = player.rotation_speed;
        let friction: f32 = player.friction + if player.state == PlayerState::Diving { 0.1 } else { 0.0 };

        velocity.linvel *= friction.powf(time.delta_seconds());
        match &player.state {
            PlayerState::Diving => {
                let acceleration;
                if player.dive_cooldown >= 2.0 {
                    acceleration = 
                        if let Some(move_event) = move_events.get(&entity) {
                            match move_event.movement {
                                Movement::Normal(direction) => Vec3::from(direction)
                            }
                        } else {
                            velocity.linvel.normalize()
                        };
                } else {
                    acceleration = velocity.linvel.normalize();
                }

                if player.dive_cooldown > 1.6 {
                    velocity.linvel += (acceleration.zero_signum() * speed) * time.delta_seconds();
                }

                let mut animation = animations.get_mut(entity).unwrap();
                if player.current_animation != game_assets.matador_dive {
                    animation.play(game_assets.matador_dive.clone_weak());
                    player.current_animation = game_assets.matador_dive.clone_weak();
                    animation.set_speed(1.25);
                }

                player.dive_cooldown -= time.delta_seconds();
                player.dive_cooldown = player.dive_cooldown.clamp(0.0, 10.0);
                println!("dive cooldown: {:?}", player.dive_cooldown);
            },
            _ => {
                if let Some(move_event) = move_events.get(&entity) {
                    match move_event.movement {
                        Movement::Normal(direction) => {

                            /*
                            let camera_transform = cameras.single();
                            let direction = Vec3::from(direction);
                            let mut acceleration = Vec3::ZERO;
                            if direction.z >= 0.5 {
                                acceleration += camera_transform.right();
                            }
                            if direction.z <= -0.5 {
                                acceleration += camera_transform.left();
                            }
                            if direction.x >= 0.5 {
                                acceleration += camera_transform.forward();
                            }
                            if direction.x <= -0.5 {
                                acceleration += camera_transform.back();
                            }
                            acceleration.y = 0.0; 

                            println!("GOing: {}", acceleration);
                            */
                            let acceleration = Vec3::from(direction);
                            velocity.linvel += (acceleration.zero_signum() * speed) * time.delta_seconds();
                        }
                    }
                }
            }
        }

        velocity.linvel = velocity.linvel.clamp_length_max(speed);
        if player.state == PlayerState::Diving {
            if player.dive_cooldown > 0.0 {
                println!("diving.. {:?}", velocity.linvel.length());
                continue;
            } else {
                println!("dive ended! {:?}", velocity.linvel.length());
                player.state = PlayerState::Normal;
            }
        } 
//      player.velocity.z *= if player.velocity.x > 0.0 { 1.0 } else { 0.0 };
//      player.velocity.y *= if player.velocity.x > 0.0 { 1.0 } else { 0.0 };
//      game_state.driving_speed = player.velocity.x * 0.1;

        let mut new_translation = transform.translation + (velocity.linvel * time.delta_seconds());

        let angle = (-(new_translation.z - transform.translation.z))
            .atan2(new_translation.x - transform.translation.x);
        let rotation = Quat::from_axis_angle(Vec3::Y, angle);
//       velocity.angvel = rotation.to_scaled_axis();
//        transform.translation = new_translation;
//        velocity.linvel = player.velocity * time.delta_seconds();

//        transform.translation.x = 0.0; // hardcoding for now

        if player.state == PlayerState::Charging {
            let mut animation = animations.get_mut(entity).unwrap();
            if player.current_animation != game_assets.matador_pose {
                animation.play(game_assets.matador_pose.clone_weak());
                animation.resume();
                player.current_animation = game_assets.matador_pose.clone_weak();
                animation.set_speed(4.0);
            } 
        } else if velocity.linvel.length() > 0.1 {
            let mut animation = animations.get_mut(entity).unwrap();
            if player.current_animation != game_assets.matador_run {
                animation.play(game_assets.matador_run.clone_weak()).repeat();
                animation.resume();
                player.current_animation = game_assets.matador_run.clone_weak();
            }
            animation.set_speed(velocity.linvel.length() / 2.0);
        } else {
            let mut animation = animations.get_mut(entity).unwrap();
            if player.current_animation != game_assets.matador_idle {
                animation.play(game_assets.matador_idle.clone_weak()).repeat();
                animation.resume();
                player.current_animation = game_assets.matador_idle.clone_weak();
                animation.set_speed(4.0);
            } 
        }

        match player.state {
             PlayerState::Charging => {
                 for (bull_transform, _) in &bull {
                     let bull_translation = bull_transform.translation;
                     let player_translation = transform.translation;
                     let angle = (-(bull_translation.z - player_translation.z))
                            .atan2(bull_translation.x - player_translation.x);
                     let rotation = Quat::from_axis_angle(Vec3::Y, angle);
                     transform.rotation = rotation;
                 }
             },
             _ => {
                let new_rotation = transform
                    .rotation
                    .lerp(Quat::from_axis_angle(Vec3::Y, TAU * 0.75), time.delta_seconds() * rotation_speed);

                // don't rotate if we're not moving or if rotation isnt a number
                if !rotation.is_nan() && velocity.linvel.length() > 1.0 {
                    transform.rotation = rotation;
                }
             }
        };

    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,

    ActionUp,
    ActionDown,
    ActionLeft,
    ActionRight,
}

impl PlayerAction {
    const DIRECTIONS: [Self; 4] = [
        PlayerAction::Up,
        PlayerAction::Down,
        PlayerAction::Left,
        PlayerAction::Right,
    ];

    fn direction(self) -> direction::Direction {
        match self {
            PlayerAction::Up => direction::Direction::UP,
            PlayerAction::Down => direction::Direction::DOWN,
            PlayerAction::Left => direction::Direction::LEFT,
            PlayerAction::Right => direction::Direction::RIGHT,
            _ => direction::Direction::NEUTRAL,
        }
    }
}

pub struct PlayerMoveEvent {
    pub entity: Entity,
    pub movement: Movement,
}

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
}

impl PlayerBundle {
    pub fn new() -> Self {
        PlayerBundle {
            player: Player::new(),
            input_manager: InputManagerBundle {
                input_map: PlayerBundle::default_input_map(),
                action_state: ActionState::default(),
            },
        }
    }

    fn default_input_map() -> InputMap<PlayerAction> {
        use PlayerAction::*;
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad { id: 0 });

        // Movement
//        input_map.insert(KeyCode::Up, Up);
        input_map.insert(KeyCode::W, Up);
        input_map.insert(KeyCode::Z, Up);
        input_map.insert(GamepadButtonType::DPadUp, Up);

//        input_map.insert(KeyCode::Down, Down);
        input_map.insert(KeyCode::S, Down);
        input_map.insert(GamepadButtonType::DPadDown, Down);

//        input_map.insert(KeyCode::Left, Left);
        input_map.insert(KeyCode::A, Left);
        input_map.insert(KeyCode::Q, Left);
        input_map.insert(GamepadButtonType::DPadLeft, Left);

//        input_map.insert(KeyCode::Right, Right);
        input_map.insert(KeyCode::D, Right);
        input_map.insert(GamepadButtonType::DPadRight, Right);

        // Actions
        input_map.insert(KeyCode::I, ActionUp);
        input_map.insert(GamepadButtonType::North, ActionUp);

        input_map.insert(KeyCode::K, ActionDown);
        input_map.insert(GamepadButtonType::South, ActionDown);

        input_map.insert(KeyCode::J, ActionLeft);
        input_map.insert(GamepadButtonType::West, ActionLeft);

        input_map.insert(KeyCode::L, ActionRight);
        input_map.insert(GamepadButtonType::East, ActionRight);

        input_map
    }
}

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    game_state: Res<game_state::GameState>,
    mut players: Query<(Entity, &mut ActionState<PlayerAction>), With<Player>>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, pressed) in controllers.pressed.iter() {
            // release all buttons
            // this probably affects durations but for
            // this game it might not be a big deal
            action_state.release(PlayerAction::Left);
            action_state.release(PlayerAction::Right);
            action_state.release(PlayerAction::Up);
            action_state.release(PlayerAction::Down);

            if pressed.contains(&game_controller::GameButton::Left) {
                action_state.press(PlayerAction::Left);
            }
            if pressed.contains(&game_controller::GameButton::Right) {
                action_state.press(PlayerAction::Right);
            }
            if pressed.contains(&game_controller::GameButton::Up) {
                action_state.press(PlayerAction::Up);
            }
            if pressed.contains(&game_controller::GameButton::Down) {
                action_state.press(PlayerAction::Down);
            }
            if pressed.contains(&game_controller::GameButton::ActionDown) {
                action_state.press(PlayerAction::ActionDown);
            } else {
                action_state.release(PlayerAction::ActionDown);
            }
            if pressed.contains(&game_controller::GameButton::ActionUp) {
                action_state.press(PlayerAction::ActionUp);
            } else {
                action_state.release(PlayerAction::ActionUp);
            }
            if pressed.contains(&game_controller::GameButton::ActionLeft) {
                action_state.press(PlayerAction::ActionLeft);
            } else {
                action_state.release(PlayerAction::ActionLeft);
            }
            if pressed.contains(&game_controller::GameButton::ActionRight) {
                action_state.press(PlayerAction::ActionRight);
            } else {
                action_state.release(PlayerAction::ActionRight);
            }
        }

        for (_, just_pressed) in controllers.just_pressed.iter() {
            if just_pressed.contains(&game_controller::GameButton::ActionUp) {
                action_state.release(PlayerAction::ActionUp);
                action_state.press(PlayerAction::ActionUp);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionDown) {
                action_state.release(PlayerAction::ActionDown);
                action_state.press(PlayerAction::ActionDown);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionRight) {
                action_state.release(PlayerAction::ActionRight);
                action_state.press(PlayerAction::ActionRight);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionLeft) {
                action_state.release(PlayerAction::ActionLeft);
                action_state.press(PlayerAction::ActionLeft);
            }
        }
    }
}

pub enum Movement {
    Normal(direction::Direction),
}

fn handle_input(
    mut app_state: ResMut<State<AppState>>,
    mut players: Query<(Entity, &ActionState<PlayerAction>, &Transform, &mut Player, &mut Velocity)>,
    game_state: Res<game_state::GameState>,
    mut player_move_event_writer: EventWriter<PlayerMoveEvent>,
    mut bull_charge_event_writer: EventWriter<bull::BullChargeEvent>,
) {
    for (entity, action_state, transform, mut player, mut velocity) in &mut players {
        //println!("T: {:?}", transform.translation);
        let mut direction = direction::Direction::NEUTRAL;

        for input_direction in PlayerAction::DIRECTIONS {
            if action_state.pressed(input_direction) {
                direction += input_direction.direction();
            }
        }

        if direction != direction::Direction::NEUTRAL {
            player_move_event_writer.send(PlayerMoveEvent {
                entity,
                movement: Movement::Normal(direction),
            });
        }

        if action_state.just_pressed(PlayerAction::ActionUp) {}
        if action_state.pressed(PlayerAction::ActionUp) {}

        if action_state.just_pressed(PlayerAction::ActionDown) && player.state != PlayerState::Diving {
            player.state = PlayerState::Diving;
            velocity.linvel = velocity.linvel.normalize();
            player.dive_cooldown = 2.0;
            //audio.play_sfx(&game_assets.dive);
        }

        if action_state.pressed(PlayerAction::ActionDown) {}

        if action_state.just_pressed(PlayerAction::ActionLeft) {}

        if action_state.pressed(PlayerAction::ActionLeft) {}

        if action_state.just_pressed(PlayerAction::ActionRight) {
            bull_charge_event_writer.send(bull::BullChargeEvent { charging: true });
        }

        if action_state.pressed(PlayerAction::ActionRight) {
            player.state = PlayerState::Charging;
        } 
        if action_state.just_released(PlayerAction::ActionRight) {
            player.state = PlayerState::Normal;
        }
    }
}
