use crate::{
    assets::GameAssets, cleanup, game_state, menus, AppState, ui::text_size, ingame, 
    game_camera, ingame_ui, asset_loading, title_screen::MenuAction, audio::GameAudio,
    game_script, player, bull,
};
use std::mem;
use bevy::prelude::*;
use rand::Rng;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::InputManagerBundle;

pub struct CutscenePlugin;
impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::Cutscene)
           .with_system(play_cutscene)
           .with_system(display_textbox)
           .with_system(animate_textures)
           .with_system(handle_input)
           .with_system(move_camera)
        )
        .insert_resource(TextBox::default())
        .insert_resource(CutsceneTextureState::default())
        .add_system_set(SystemSet::on_enter(AppState::Cutscene)
//           .with_system(cleanup::<ingame_ui::CleanupMarker>)
           .with_system(setup_cutscene)
        )
        .add_system_set(SystemSet::on_exit(AppState::Cutscene)
           .with_system(cleanup::<CutsceneCleanupMarker>)
           .with_system(cleanup::<ingame::CleanupMarker>)
        )
        .insert_resource(CutsceneState::default());
    }
}

#[derive(Component)]
pub struct CutsceneCleanupMarker;

#[derive(Default, Resource)]
pub struct CutsceneState {
    pub cutscene_index: usize,
    cooldown: f32,
    input_cooldown: f32,
    pub waiting_on_input: bool,
    target_camera_translation: Option::<Vec3>,
    target_camera_rotation: Option::<Quat>,
    camera_speed: f32,
}

impl CutsceneState {
    pub fn init(&mut self) {
        self.cutscene_index = 0;
        self.cooldown = 0.0;
        self.input_cooldown = 0.0;
        self.target_camera_translation = None;
        self.target_camera_rotation = None;
        self.waiting_on_input = false;
    }
}

#[derive(Component)]
pub struct CutsceneTextBoxContainer;
#[derive(Component)]
struct CutsceneTextContainerMarker;

fn setup_cutscene(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    text_scaler: text_size::TextScaler,
) {
    commands
        .spawn(InputManagerBundle {
            input_map: MenuAction::default_input_map(),
            action_state: ActionState::default(),
        })
        .insert(CutsceneCleanupMarker);

    let scale = (text_scaler.window_size.width * 0.1) / ingame::RENDER_TEXTURE_SIZE as f32;

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(CutsceneCleanupMarker)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..Default::default()
                    },
                    background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                    ..Default::default()
                })
                .insert(CutsceneTextBoxContainer)
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::FlexStart,
                                flex_wrap: FlexWrap::Wrap,
                                overflow: Overflow::Hidden,
                                ..Default::default()
                            },
                            background_color: Color::hex("8d99ae").unwrap().into(),
                            ..Default::default()
                        })
                        .insert(CutsceneTextContainerMarker);
                });
        });
}

#[derive(Resource)]
pub struct TextBox {
    texts: Option::<Vec::<TextBoxText>>,
    queued_text: Option::<TextBoxText>,
    index: usize,
    cooldown: f32,
}

impl Default for TextBox {
    fn default() -> Self {
        TextBox {
            texts: None,
            queued_text: None,
            index: 0,
            cooldown: 0.0,
        }
    }
}

impl TextBox {
    fn take_next_text(&mut self) -> Option::<TextBoxText> {
        if let Some(texts) = &mut self.texts {
            if texts.is_empty() {
                None
            } else {
                Some(texts.remove(0))
            }
        } else {
            None
        }
    }
}

enum CutsceneTexture {
    MatIdle,
    MatTalk,
    PaIdle,
    PaTalk,
    PaLook,
}


#[derive(Resource, Default)]
struct CutsceneTextureState {
    mat: Vec::<CutsceneTexture>,
    pa: Vec::<CutsceneTexture>,
}

pub struct TextBoxText {
    text: String,
    speed: f32,
    auto: bool,
    speaking: DisplayCharacter,
}

enum DisplayCharacter {
    Mat, Pa, None
}

fn queue_initial_text(
    mut textbox: ResMut<TextBox>,
) {
    textbox.queued_text = textbox.take_next_text();
    textbox.cooldown = textbox.queued_text.as_ref().unwrap().speed;
}

fn move_camera(
    cutscene_state: Res<CutsceneState>,
    mut camera: Query<&mut Transform, With<game_camera::PanOrbitCamera>>,
    time: Res<Time>
) {
    if let Some(target) = cutscene_state.target_camera_translation {
        let mut camera = camera.single_mut();
        let camera_translation = camera.translation;
        camera.translation += (target - camera_translation) * (time.delta_seconds() * cutscene_state.camera_speed.max(0.1));
    }
    if let Some(target) = cutscene_state.target_camera_rotation {
        let mut camera = camera.single_mut();
        let new_rotation = camera.rotation
                            .lerp(target, time.delta_seconds() * cutscene_state.camera_speed.max(0.1));
        if !new_rotation.is_nan() {
            camera.rotation = new_rotation;
        }
    }
}

fn display_textbox(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut textbox: ResMut<TextBox>,
    mut text_container: Query<Entity, With<CutsceneTextContainerMarker>>,
    text_scaler: text_size::TextScaler,
    time: Res<Time>,
    mut audio: GameAudio,
) {
    textbox.cooldown -= time.delta_seconds();     
    textbox.cooldown = textbox.cooldown.clamp(-3.0, 3.0);
    if textbox.cooldown > 0.0 { return; }

    let mut current_speed = None;

    if let Ok(container) = text_container.get_single() {
        if let Some(current_text) = &mut textbox.queued_text {
            let maybe_space_index = current_text.text.find(' ');

            let text_to_display: String =
                if let Some(space_index) = maybe_space_index {
                    let mut temp = current_text.text.split_off(space_index + 1);
                    mem::swap(&mut temp, &mut current_text.text);
                    temp
                } else {
                    current_text.text.drain(..).collect()
                };

            let base_font_size = 50.0;
            let font_size = text_scaler.scale(base_font_size);
            commands.entity(container)
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect {
                                    right: Val::Percent(1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text::from_section(
                                text_to_display.trim(),
                                TextStyle {
                                    font: game_assets.font.clone(),
                                    font_size,
                                    color: Color::WHITE,
                                }
                            ),
                            ..Default::default()
                        });
                    });
            match current_text.speaking {
                DisplayCharacter::Mat => audio.play_talk(&game_assets.mat_speak),
                DisplayCharacter::Pa => audio.play_talk(&game_assets.pa_speak),
                _ => ()
            }

            current_speed = Some(current_text.speed);
            if current_text.text.is_empty() {
                textbox.queued_text = None;
            }
        }
    }

    textbox.cooldown = current_speed.unwrap_or(textbox.cooldown);
}

#[derive(Component)]
pub struct PaTalkMarker;
#[derive(Component)]
pub struct MatTalkMarker;

fn animate_textures(
    mut materials: ResMut<Assets<StandardMaterial>>,
    pas: Query<&Handle<StandardMaterial>, With<PaTalkMarker>>,
    mats: Query<&Handle<StandardMaterial>, With<MatTalkMarker>>,
    mut cutscene_texture_state: ResMut<CutsceneTextureState>,
    game_assets: Res<GameAssets>,
    mut current_time: Local<f32>,
    time: Res<Time>,
) {
    *current_time += time.delta_seconds();
    for pas_material_handle in &pas {
        if cutscene_texture_state.pa.len() > 1 && *current_time > 0.2 {
            cutscene_texture_state.pa.rotate_left(1);
        }

        if cutscene_texture_state.pa.len() > 0 {
            match cutscene_texture_state.pa[0] {
                CutsceneTexture::PaIdle => {
                    if let Some(mut material) = materials.get_mut(pas_material_handle) {
                        material.base_color_texture = Some(game_assets.pa_no_mouth.image.clone());
                    }
                },
                CutsceneTexture::PaTalk => {
                    if let Some(mut material) = materials.get_mut(pas_material_handle) {
                        material.base_color_texture = Some(game_assets.pa_mouth.image.clone());
                    }
                },
                CutsceneTexture::PaLook => {
                    if let Some(mut material) = materials.get_mut(pas_material_handle) {
                        material.base_color_texture = Some(game_assets.pa_lookleft.image.clone());
                    }
                },
                _ => ()
            }; //: (usize, Vec::<CutsceneTexture>),
        }
    }

    for mat_material_handle in &mats {
        if cutscene_texture_state.mat.len() > 1 && *current_time > 0.2 {
            cutscene_texture_state.mat.rotate_left(1);
        }

        if cutscene_texture_state.mat.len() > 0 {
            match cutscene_texture_state.mat[0] {
                CutsceneTexture::MatIdle => {
                    if let Some(mut material) = materials.get_mut(mat_material_handle) {
                        material.base_color_texture = Some(game_assets.mat_idle.image.clone());
                    }
                },
                CutsceneTexture::MatTalk => {
                    if let Some(mut material) = materials.get_mut(mat_material_handle) {
                        material.base_color_texture = Some(game_assets.mat_talk.image.clone());
                    }
                },
                _ => ()
            };
        }
    }

    if *current_time > 0.2 {
        *current_time = 0.0; 
    }
}

fn handle_input(
    mut commands: Commands,
    action_state: Query<&ActionState<MenuAction>>,
    mut textbox: ResMut<TextBox>,
    text_container: Query<&Children, With<CutsceneTextContainerMarker>>,
    mut state: ResMut<State<AppState>>,
    mut cutscene_state: ResMut<CutsceneState>,
    time: Res<Time>,
) {
    if !cutscene_state.waiting_on_input { return; }

    cutscene_state.input_cooldown -= time.delta_seconds();     
    cutscene_state.input_cooldown = cutscene_state.input_cooldown.clamp(-3.0, 3.0);
    if cutscene_state.input_cooldown > 0.0 { return; }

    if let Ok(action_state) = action_state.get_single() {
        if action_state.just_pressed(MenuAction::Select) {
            cutscene_state.input_cooldown = 0.5;
            cutscene_state.waiting_on_input = false;
            cutscene_state.cutscene_index += 1;
            // clear out existing text
            for children in text_container.iter() {
                for entity in children.iter() {
                    commands.get_or_spawn(*entity).despawn_recursive();
                }
            }
        }
    }
}

fn play_cutscene(
    mut cutscene_state: ResMut<CutsceneState>,
    mut camera: Query<&mut Transform, With<game_camera::PanOrbitCamera>>,
    mut textbox: ResMut<TextBox>,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    mut game_script_state: ResMut<game_script::GameScriptState>,
    mut cutscene_texture_state: ResMut<CutsceneTextureState>,
    mut animations: Query<&mut AnimationPlayer>,
    players: Query<Entity, With<player::Player>>,
    bulls: Query<Entity, With<bull::Bull>>,
//    mut ingame_ui_textbox: ResMut<ingame_ui::TextBox>,
    mut audio: GameAudio,
) {
    let mut camera = camera.single_mut();
//    println!("{:?} {:?}", camera.translation, camera.rotation.to_axis_angle());
    if cutscene_state.waiting_on_input { return; }

    cutscene_state.camera_speed = 2.0;
    cutscene_state.waiting_on_input = true;
    let text_speed = 0.10;

    println!("Cutscene: matching {:?}", game_script_state.current);
//    *ingame_ui_textbox = ingame_ui::TextBox::default(); // clear out any banter or commentary
    match game_script_state.current {
        game_script::GameScript::IntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    println!("Cutscene: 0 in intro");
                    camera.translation = Vec3::new(19.605387, -10.73157346, 20.539804);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.030079605, -0.99812686, -0.05320679), 2.071217);
                    cutscene_state.camera_speed = 1.0; 
                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.intro_bgm);
                    cutscene_state.target_camera_translation = Some(Vec3::new(18.70001, 1.6389314, 20.061293));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.07476175, -0.98985404, -0.12082917), 2.0425286));
//                  cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
//                                                   CutsceneTexture::PaTalk);
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: We didn't have a single customer come in this week.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                1 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: I don't know how we're going to make rent this month.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                2 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: There's too much competition! We need to think outside of the soap box.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                3 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: What are our strengths? Our resources?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                4 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle); 
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: All I have is this cape from my halloween costume.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                5 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle); 
                    cutscene_state.target_camera_translation = Some(Vec3::new(16.895172, 1.6389309, 21.279295));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.10611979, -0.9900699, -0.09219614), 1.440496));
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: and my fully-trained 2,000 lb bull.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                6 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Ah yes.. your pet bull. I keep forgetting.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                7 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);

                    cutscene_state.target_camera_translation = Some(Vec3::new(18.70001, 1.6389314, 20.061293));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.07476175, -0.98985404, -0.12082917), 2.0425286));

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: hmm... hold on.. that gives me an idea.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                8 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);
                    camera.translation = Vec3::new(20.503273, 1.9418178, 19.837416);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(0.033396073, 0.99048394, 0.1335149), 3.6273122);
                    cutscene_state.target_camera_translation = Some(Vec3::new(21.84123, 1.9418178, 22.34461));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(0.033396073, 0.99048394, 0.1335149), 3.6273122));
                    cutscene_state.camera_speed = 0.25; 

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Yes... YES...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                9 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);
                    cutscene_state.camera_speed = 0.25; 

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: I think we're going to be alright.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                10 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle); 
                                                     
                    cutscene_state.camera_speed = 0.25; 

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: ...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::None,
                    });
                },
                11 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaLook); 

                    audio.stop_bgm();
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: we should probably put out this fire now.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                12 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);

                    camera.translation = Vec3::new(16.895172, 1.6389309, 21.279295);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.10611979, -0.9900699, -0.09219614), 1.440496);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Yeah, we need to stop making these in here..".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    println!("Cutscene: Done in intro");
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::PreLevelOneCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    camera.translation = Vec3::new(6.8157444, 2.6143255, -2.526128);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.051877804, 0.9947759, 0.08791898), 2.080009);
                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.pregame_bgm);

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: OK! Here's the plan.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        println!("setting player animation");
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_run.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }

                    for entity in &bulls {
                        println!("setting bull animation");
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_run.clone_weak()).repeat();
                        animation.set_speed(3.0);
                        animation.resume();
                    }

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: You and your bull go to each antique shop in town.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                2 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: And then... you wreck the place.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                3 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Just destroy everything.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                4 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: How come I can hear you right now?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                5 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle); 
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: I gave you that earpiece, remember?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                6 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: Why's it so dark outside?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                7 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle); 
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Nevermind that! You're almost at the first shop.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                8 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle); 
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Remember, the best offensive is to be offensive!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelOneIntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = Some(Vec3::new(17.168844, 1.5, -0.0074485363));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247));
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }

                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.level_one_bgm);
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Hello, welcome to 'First Plate'!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Your one-stop shop for all your baseball antiques!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                2 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: ... IS THAT A BULL!?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                3 => {
                    cutscene_state.target_camera_translation = Some(Vec3::new(-10.231776, 1.5, -0.55118924));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472));

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: I think they see us.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                4 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: That's ok, we have plenty of time!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                5 => {
                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: I'm calling the cops!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                6 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: They're calling the cops.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                7 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: You got two minutes kid, here's what you gotta do.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                8 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Use your action button to signal your bull to charge at you.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                9 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Then try to get it to knock everything down before time runs out!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                10 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Did that make sense?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                11 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: Yeah, Pa, I trained the bull. I know how bulls work.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                12 => {
                    textbox.queued_text = Some(TextBoxText {
                        text: "Pa: Alright, GO!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelOnePostCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    audio.stop_bgm();

                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: ...wh..why??".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelTwoIntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = Some(Vec3::new(17.168844, 1.5, -0.0074485363));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247));
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }

                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.level_two_bgm);
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Hello, welcome to 'West Spoon'.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: excuse me, is that a service bull?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                2 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    cutscene_state.target_camera_translation = Some(Vec3::new(-10.231776, 1.5, -0.55118924));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472));

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: ...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                3 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    cutscene_state.target_camera_translation = Some(Vec3::new(-10.231776, 1.5, -0.55118924));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472));

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: ...yes.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                4 => {
                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Oh, ok. Feel free to check out our new Appalachian Holiday collection.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                5 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: ...S..sure.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                6 => {
                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: and don't worry if your bull breaks anything, I'll fix it no problem!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                7 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: oh geez".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelTwoPostCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Thanks for stopping in! Have a nice day!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: I'm not sure how effective that was...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                2 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: It's ok, I think you did some damage. Head to the next shop!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelThreeIntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = Some(Vec3::new(4.868683, 1.422154, -0.04496868));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(0.018233472, 0.9996569, 0.018801434), 4.681377));

                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }

                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.level_three_bgm);

                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Howdy and welcome to 'Dishes and Fishes'!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: no, we're not a restaurant, we actually sell fish and novelty plates.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                2 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: we also have a prized red herring on display but it is not for sale.".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                3 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: please be careful around the fish bowls".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                4 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: ...or else!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                5 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    cutscene_state.target_camera_translation = Some(Vec3::new(-10.231776, 1.5, -0.55118924));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472));

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: ...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                6 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    cutscene_state.target_camera_translation = Some(Vec3::new(-10.231776, 1.5, -0.55118924));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472));

                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: what a strange place".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                7 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }

                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: what are you doing!? Get to work, kid!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelThreePostCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    audio.stop_bgm();

                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(17.168844, 1.5, -0.0074485363);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.03418233, -0.9987495, -0.03648749), 1.5735247);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    textbox.queued_text = Some(TextBoxText {
                        text: "Shopkeeper: Over here, officer!!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                1 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: Oh geez! The cops caught up to us!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                2 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: it's ok! You didn't do anything wrong!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                3 => {
                    for entity in &players {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.matador_idle.clone_weak()).repeat();
                        animation.set_speed(4.0);
                    }
                    for entity in &bulls {
                        let mut animation = animations.get_mut(entity).unwrap();
                        animation.play(game_assets.bull_idle.clone_weak()).repeat();
                        animation.set_speed(2.0);
                        animation.resume();
                    }
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: They say I'm responsible for the bull!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                4 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: WHAT!?".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                5 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: They're going to arrest me!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                6 => {
                    camera.translation = Vec3::new(-10.231776, 1.5, -0.55118924);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.05911547, 0.9965279, 0.0586311), 1.5660472);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: What a load of b".to_string(),
                        speed: text_speed,
                        auto: true,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelFourIntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    audio.stop_bgm();
                    cutscene_state.target_camera_translation = Some((Vec3::new(19.3, 1.5, 0.0)));
                    textbox.queued_text = Some(TextBoxText {
                        text: "Level Four Intro Cutscene!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelFourPostCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    audio.stop_bgm();
                    cutscene_state.target_camera_translation = Some((Vec3::new(19.3, 1.5, 0.0)));
                    textbox.queued_text = Some(TextBoxText {
                        text: "Level Four POST Cutscene!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelFiveIntroCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    audio.stop_bgm();
                    cutscene_state.target_camera_translation = Some((Vec3::new(19.3, 1.5, 0.0)));
                    textbox.queued_text = Some(TextBoxText {
                        text: "Level Five Intro Cutscene!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::InGame, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::LevelFivePostCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    camera.translation = Vec3::new(22.5, 1.5, 0.0);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.034182332, -0.9987495, -0.03648749), 1.5735247);
                    audio.stop_bgm();
                    cutscene_state.target_camera_translation = Some((Vec3::new(19.3, 1.5, 0.0)));
                    textbox.queued_text = Some(TextBoxText {
                        text: "Level Five POST Cutscene!".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::LoadWorld, &mut game_assets, &game_state);
                }
            }
        },
        game_script::GameScript::EndCutscene => {
            match cutscene_state.cutscene_index {
                0 => {
                    println!("Cutscene: 0 in Outro");


                    cutscene_state.target_camera_translation = Some(Vec3::new(14.423876, 1.6476835, 21.160599));
                    cutscene_state.target_camera_rotation = Some(Quat::from_axis_angle(Vec3::new(-0.076007806, -0.9942875, -0.07493414), 1.5622988));

                    camera.translation = Vec3::new(20.253891, 1.6476835, 21.739094);
                    camera.rotation = Quat::from_axis_angle(Vec3::new(-0.076007806, -0.9942875, -0.07493414), 1.5622988);
                    cutscene_state.camera_speed = 1.0; 
                    audio.stop_bgm();
                    audio.play_bgm(&game_assets.intro_bgm);
//                  cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
//                                                   CutsceneTexture::PaTalk);
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle, 
                                                     CutsceneTexture::MatTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "MAT: Whelp...".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Mat,
                    });
                },
                1 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: Yeah that was 100% on me, kid".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                2 => {
                    cutscene_texture_state.mat = vec!(CutsceneTexture::MatIdle);
                    cutscene_texture_state.pa = vec!(CutsceneTexture::PaIdle, 
                                                     CutsceneTexture::PaTalk);
                    textbox.queued_text = Some(TextBoxText {
                        text: "PA: who would have known".to_string(),
                        speed: text_speed,
                        auto: false,
                        speaking: DisplayCharacter::Pa,
                    });
                },
                _ => {
                    camera.translation = Vec3::new(game_camera::INGAME_CAMERA_X, 
                                                   game_camera::INGAME_CAMERA_Y, 
                                                   0.0);
                    camera.rotation = Quat::from_axis_angle(game_camera::INGAME_CAMERA_ROTATION_AXIS, 
                                                game_camera::INGAME_CAMERA_ROTATION_ANGLE);
                    cutscene_state.target_camera_translation = None;
                    cutscene_state.target_camera_rotation = None;
                    cutscene_state.cutscene_index = 0;
                    game_script_state.next();
                    cutscene_state.waiting_on_input = false;
                    assets_handler.load(AppState::TitleScreen, &mut game_assets, &game_state);
                }
            }
        },
        _ => {
            println!("uhh not a cutscene???");
        }
    }
}
