use crate::{
    asset_loading, assets::GameAssets, audio::GameAudio, cleanup, game_controller, menus, 
    ui::text_size, AppState, menus::HOVERED_BUTTON, menus::NORMAL_BUTTON, game_state, assets,
};
use bevy::app::AppExit;
use bevy::ecs::event::Events;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct TitlePlugin;
impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<MenuAction>::default())
            .add_system_set(SystemSet::on_enter(AppState::TitleScreen).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::TitleScreen)
                    .with_system(update_menu_buttons.after("handle_input"))
                    .with_system(
                        handle_controllers
                            .label("handle_input")
                            .after("store_controller_inputs"),
                    ),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::TitleScreen).with_system(cleanup::<TitleScreenCleanupMarker>),
            );
    }
}

#[derive(Component)]
struct TitleScreenCleanupMarker;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum MenuAction {
    Up,
    Down,
    Left,
    Right,
    Select,
}
impl MenuAction {
    pub fn default_input_map() -> InputMap<MenuAction> {
        use MenuAction::*;
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad { id: 0 });

        input_map.insert(KeyCode::Up, Up);
        input_map.insert(KeyCode::W, Up);
        input_map.insert(GamepadButtonType::DPadUp, Up);

        input_map.insert(KeyCode::Down, Down);
        input_map.insert(KeyCode::S, Down);
        input_map.insert(GamepadButtonType::DPadDown, Down);

        input_map.insert(KeyCode::Left, Left);
        input_map.insert(KeyCode::A, Left);
        input_map.insert(GamepadButtonType::DPadLeft, Left);

        input_map.insert(KeyCode::Right, Right);
        input_map.insert(KeyCode::D, Right);
        input_map.insert(GamepadButtonType::DPadRight, Right);

//        input_map.insert(KeyCode::Space, Select);
        input_map.insert(KeyCode::Return, Select);
        input_map.insert(GamepadButtonType::South, Select);

        input_map
    }
}

pub fn load(
    assets_handler: &mut asset_loading::AssetsHandler,
    game_assets: &mut ResMut<GameAssets>,
) {
    println!("loading assets");
    assets_handler.add_audio(&mut game_assets.title_screen_bgm, "audio/title_screen_bgm.ogg");
    assets_handler.add_audio(&mut game_assets.blip, "audio/blip.wav");
    assets_handler.add_font(&mut game_assets.font, "fonts/monogram.ttf");
    assets_handler.add_material(
        &mut game_assets.title_screen_logo,
        "textures/logo.png",
        true,
    );
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut audio: GameAudio,
    mut clear_color: ResMut<ClearColor>,
    text_scaler: text_size::TextScaler,
) {
    println!("Setting up camera");
    commands
        .spawn(InputManagerBundle {
            input_map: MenuAction::default_input_map(),
            action_state: ActionState::default(),
        })
        .insert(TitleScreenCleanupMarker);

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.00,
    });

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, -0.0001).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(TitleScreenCleanupMarker);

    clear_color.0 = Color::hex("fffffa").unwrap(); 
    let image_height = 512.0;
    let scale = (text_scaler.window_size.height * 0.8) / image_height;

//        commands.spawn(ImageBundle {
//  //        transform: {
//  //            let height = (text_scaler.window_size.height / 2.0) * 0.224;
//  //            let mut t = Transform::from_translation(Vec3::new(0.0, height, 0.0));
//  //            t.scale *= scale;
//  //            t
//  //        },
//            texture: ,
//            ..Default::default()
//          })
//      .insert(TitleScreenCleanupMarker);
     commands 
             .spawn(NodeBundle {
                 style: Style {
                     size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                     position_type: PositionType::Absolute,
                     justify_content: JustifyContent::Center,
                     align_items: AlignItems::Center,
                     flex_direction: FlexDirection::Row,
                     margin: UiRect {
                         left: Val::Auto,
                         right: Val::Auto,
                         ..Default::default()
                     },
                     ..Default::default()
                 },
                 background_color: Color::NONE.into(),
                 ..Default::default()
             })
             .with_children(|parent| {
                 parent.spawn(ImageBundle {
                     style: Style {
                         size: Size::new(Val::Percent(50.0), Val::Auto),
                         ..Default::default()
                     },
                     image: game_assets.title_screen_logo.image.clone().into(),
                     ..Default::default()
                 });
             })
          .insert(TitleScreenCleanupMarker);

    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::from_section(
                "by michael ramirez".to_string(),
                TextStyle {
                    font: game_assets.font.clone(),
                    font_size: text_scaler.scale(menus::BY_LINE_FONT_SIZE),
                    color: Color::rgba(0.0, 0.0, 0.0, 1.0),
                }
            ),
            ..Default::default()
        })
        .insert(TitleScreenCleanupMarker);

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(30.0), Val::Percent(25.0)),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Percent(40.0),
                    ..Default::default()
                },
                align_items: AlignItems::FlexEnd,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(TitleScreenCleanupMarker)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        margin: UiRect::all(Val::Auto),
                        size: Size::new(Val::Percent(100.0), Val::Percent(40.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Start",
                            TextStyle {
                                font: game_assets.font.clone(),
                                font_size: text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE),
                                color: Color::rgb(0.0, 0.0, 0.0),
                            }
                        ),
                        ..Default::default()
                    });
                })
                .insert(TitleScreenCleanupMarker);

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(40.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        ..Default::default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Quit",
                            TextStyle {
                                font: game_assets.font.clone(),
                                font_size: text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE),
                                color: Color::rgb(0.0, 0.0, 0.0),
                            }
                        ),
                        ..Default::default()
                    });
                })
                .insert(TitleScreenCleanupMarker);
        });

    audio.play_bgm(&game_assets.title_screen_bgm);
}

fn update_menu_buttons(
    mut selected_button: Local<usize>,
    mut exit: ResMut<Events<AppExit>>,
    buttons: Query<Entity, With<Button>>,
    mut button_colors: Query<&mut BackgroundColor, With<Button>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    action_state: Query<&ActionState<MenuAction>>,
    //mut assets_handler: asset_loading::AssetsHandler,
    mut audio: GameAudio,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    time: Res<Time>,
) {
    if game_state.title_screen_cooldown > 0.0 {
        game_state.title_screen_cooldown -= time.delta_seconds();
        return; 
    }

    let action_state = action_state.single();
    let number_of_buttons = buttons.iter().count();
    let mut pressed_button = action_state.pressed(MenuAction::Select);

    if action_state.just_pressed(MenuAction::Up) {
        audio.play_sfx(&game_assets.blip);
        *selected_button = selected_button
            .checked_sub(1)
            .unwrap_or(number_of_buttons - 1);
    }
    if action_state.just_pressed(MenuAction::Down) {
        audio.play_sfx(&game_assets.blip);
        let new_selected_button = selected_button.checked_add(1).unwrap_or(0);
        *selected_button = if new_selected_button > number_of_buttons - 1 {
            0
        } else {
            new_selected_button
        };
    }


    for (i, mut color) in button_colors.iter_mut().enumerate() {
        if i == *selected_button {
            *color = HOVERED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }

    if pressed_button {
        if *selected_button == 0 {
            audio.play_sfx(&game_assets.blip);
            assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
        }
        if *selected_button == 1 {
            exit.send(AppExit);
        }
    }
}

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    mut players: Query<(Entity, &mut ActionState<MenuAction>)>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, just_pressed) in controllers.just_pressed.iter() {
            // release all buttons
            // this probably affects durations but for
            // this game it might not be a big deal
            action_state.release(MenuAction::Up);
            action_state.release(MenuAction::Down);

            action_state.release(MenuAction::Select);

            if just_pressed.contains(&game_controller::GameButton::Up) {
                action_state.press(MenuAction::Up);
            }
            if just_pressed.contains(&game_controller::GameButton::Down) {
                action_state.press(MenuAction::Down);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionDown)
                || just_pressed.contains(&game_controller::GameButton::Start)
            {
                action_state.press(MenuAction::Select);
            }
        }
    }
}
