use crate::{
    assets::GameAssets, game_state, menus, AppState, ui::text_size, ingame
};
use bevy::prelude::*;

const STAR_SIZE: f32 = 60.0;
pub struct InGameUIPlugin;
impl Plugin for InGameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
           .add_system_set(
               SystemSet::on_update(AppState::InGame)
                   .with_system(update_ui)
           );
    }
}

#[derive(Component)]
struct Star(usize);

fn update_ui(
    game_state: Res<game_state::GameState>,
    game_assets: Res<GameAssets>,
    mut stars: Query<(&mut UiImage, &Star)>,
    mut time_indicators: Query<&mut Text, (With<TimeIndicator>, Without<ScoreIndicator>)>,
) {
    let current_score = game_state.score * 5.0;
    for (mut image, star) in stars.iter_mut() {
        let star_value = star.0 as f32;
        if star_value + 0.5 < current_score {
            image.0 = game_assets.star_full_texture.image.clone();
        } else if star_value < current_score {
            image.0 = game_assets.star_half_texture.image.clone();
        } else {
            image.0 = game_assets.star_empty_texture.image.clone();
        }
    }

    for mut text in time_indicators.iter_mut() {
        text.sections[0].value = format!("{:0>2}:{:0>2}", (game_state.current_time / 60.0) as usize, 
                                                  (game_state.current_time % 60.0) as usize);
    }
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut game_state: ResMut<game_state::GameState>,
    text_scaler: text_size::TextScaler,
) {
    println!("Setting up UI");
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(ingame::CleanupMarker)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(50.0), Val::Percent(15.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        margin: UiRect {
                            left: Val::Auto,
                            right: Val::Auto,
                            ..default()
                        },
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    for i in 0..5 {
                        parent 
                             .spawn(NodeBundle {
                                 style: Style {
                                     size: Size::new(Val::Percent(20.0), Val::Percent(33.0)),
                                     position_type: PositionType::Relative,
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
                                         size: Size::new(Val::Percent(STAR_SIZE), Val::Auto),
                                         ..Default::default()
                                     },
                                     image: game_assets.star_full_texture.image.clone().into(),
                                     ..Default::default()
                                 })
                                 .insert(Star(i));
                             });
                    }
                });
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(5.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        flex_direction: FlexDirection::Row,
                        margin: UiRect {
                            //top: Val::Percent(-10.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 0.6),
                        "Time: ",
                        Vec::<ingame::CleanupMarker>::new(), // just an empty vec since can't do <impl Trait>
                    );
                    add_title(
                        parent,
                        game_assets.font.clone(),
                        text_scaler.scale(menus::DEFAULT_FONT_SIZE * 0.6),
                        "00:00",
                        vec!(TimeIndicator), // just an empty vec since can't do <impl Trait>
                    );
                });
        });
}

#[derive(Component)]
struct ScoreIndicator;

#[derive(Component)]
struct TimeIndicator;

pub fn add_title(
    builder: &mut ChildBuilder<'_, '_, '_>,
    font: Handle<Font>,
    font_size: f32,
    title: &str,
    mut components: Vec<impl Component>,
) {
    let mut text_bundle = builder.spawn_bundle(TextBundle {
        style: Style {
            position_type: PositionType::Relative,
            margin: UiRect {
//              left: Val::Percent(2.0),
//              right: Val::Auto,
                ..Default::default()
            },
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        text: Text::from_section(
            title.to_string(),
            TextStyle {
                font,
                font_size,
                color: Color::WHITE,
            },
        ).with_alignment(
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                ..Default::default()
            }
        ),
        ..Default::default()
    });

    components.drain(..).for_each(|c| {
        text_bundle.insert(c);
    });
}
