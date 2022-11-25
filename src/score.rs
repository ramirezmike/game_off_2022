use bevy::prelude::*;
use crate::{
    AppState, groups, game_state,
};
use std::collections::HashMap;

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::InGame)
           .with_system(check_score)
        );

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
    *cooldown = 2.0;

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
            let distance = current.distance(*original);
            if distance > 1.0 {
                group_broken_count += 1;
                break;
            }
        }
    }

    if group_count > 0 {
        game_state.score = 1.0 - (group_broken_count as f32 / group_count as f32);
        println!("Score {} ( {} / {} )", game_state.score, group_broken_count, group_count);
    }
}















