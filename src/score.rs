use bevy::prelude::*;
use crate::{
    AppState, groups, game_state, asset_loading, game_script, assets, cutscene,
};
use std::collections::HashMap;

pub const BREAK_DISTANCE: f32 = 0.5;
pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::InGame)
           .with_system(track_round_time)
           .with_system(check_score)
        );

    }
}

fn track_round_time(
    mut game_state: ResMut<game_state::GameState>,
    mut cutscene_state: ResMut<cutscene::CutsceneState>,
    time: Res<Time>,
    mut game_script_state: ResMut<game_script::GameScriptState>,
    mut assets_handler: asset_loading::AssetsHandler,
    mut game_assets: ResMut<assets::GameAssets>,
) {
    game_state.current_time -= time.delta_seconds();
    
    if game_state.current_time < 0.0 || game_state.live_score <= 0.0 {
        cutscene_state.cutscene_index = 0;
        game_script_state.next();
        assets_handler.load(AppState::Cutscene, &mut game_assets, &game_state);
    }
}

fn check_score(
    group_members: Query<(&groups::GroupMember, &Transform, &GlobalTransform)>,
    mut cooldown: Local<f32>,
    mut game_state: ResMut<game_state::GameState>,
    time: Res<Time>,
) {
    *cooldown -= time.delta_seconds();
    *cooldown = cooldown.clamp(0.0, 10.0);
    
    if *cooldown > 0.0 {
        return;
    }
    *cooldown = 1.0;

    let grouped_translations = 
        group_members.iter()
                     .fold(HashMap::<usize, Vec::<(Vec3, Vec3, Vec3)>>::new(),
                     |mut acc, (group_member, transform, global_transform)| {
                         acc.entry(group_member.group_id)
                            .or_insert(vec!())
                            .push((transform.translation, group_member.original_transform.translation, global_transform.translation()));

                         acc
                     });

    let mut group_count = 0;
    let mut group_broken_count = 0;
    for (_, translations) in grouped_translations.iter() {
        group_count += 1; 

        for (current, original, _) in translations.iter() {
            let distance = (current.y - original.y).abs();
            if distance > BREAK_DISTANCE {
                group_broken_count += 1;
                break;
            }
        }
    }

    if group_count > 0 {
        game_state.score_check_count += 1;
        let current = 1.0 - (group_broken_count as f32 / group_count as f32);
        game_state.live_score = current;
        // add as a running average of the score
        game_state.score = (game_state.score * (game_state.score_check_count - 1) as f32 + current) 
                            / game_state.score_check_count as f32;
        game_state.live_score = current;
//        println!("Score {} {} ( {} / {} )", game_state.live_score, game_state.score, group_broken_count, group_count);
    }
}



