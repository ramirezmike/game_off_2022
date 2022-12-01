use crate::asset_loading;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default());
    }
}

#[derive(Default, Resource)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub matador: Handle<Gltf>,
    pub matador_run: Handle<AnimationClip>,
    pub matador_idle: Handle<AnimationClip>,
    pub matador_dive: Handle<AnimationClip>,
    pub matador_pose: Handle<AnimationClip>,
    pub bull: Handle<Gltf>,
    pub bull_idle: Handle<AnimationClip>,
    pub bull_walk: Handle<AnimationClip>,
    pub bull_run: Handle<AnimationClip>,
    pub bull_charge: Handle<AnimationClip>,
    pub bull_collide: Handle<AnimationClip>,
    pub fishmonger: Handle<Gltf>,
    pub fishmonger_with_fish: Handle<Gltf>,
    pub plate: Handle<Gltf>,
    pub broken_plate: Handle<Gltf>,
    pub broken_mug: Handle<Gltf>,
    pub broken_fishbowl: Handle<Gltf>,
    pub pregame: Handle<Gltf>,
    pub level_one: Handle<Gltf>,
    pub level_two: Handle<Gltf>,
    pub level_three: Handle<Gltf>,
    pub level_four: Handle<Gltf>,
    pub level_five: Handle<Gltf>,
    pub intro_level: Handle<Gltf>,
    pub outro_level: Handle<Gltf>,

    pub blip: Handle<AudioSource>,
    pub crash_sfx: Handle<AudioSource>,
    pub break_sfx: Handle<AudioSource>,
    pub clop_sfx: Handle<AudioSource>,
    pub fix_sfx: Handle<AudioSource>,
    pub intro_bgm: Handle<AudioSource>,
    pub pregame_bgm: Handle<AudioSource>,
    pub intro_end_bgm: Handle<AudioSource>,
    pub level_one_bgm: Handle<AudioSource>,
    pub level_two_bgm: Handle<AudioSource>,
    pub level_three_bgm: Handle<AudioSource>,
    pub title_screen_bgm: Handle<AudioSource>,
    pub mat_speak: Handle<AudioSource>,
    pub pa_speak: Handle<AudioSource>,

    pub bevy_icon: asset_loading::GameTexture,
    pub cloud_texture: asset_loading::GameTexture,
    pub wrench_texture: asset_loading::GameTexture,
    pub fire_texture: asset_loading::GameTexture,
    pub star_full_texture: asset_loading::GameTexture,
    pub star_half_texture: asset_loading::GameTexture,
    pub star_empty_texture: asset_loading::GameTexture,
    pub title_screen_logo: asset_loading::GameTexture,

    pub mat_idle: asset_loading::GameTexture,
    pub mat_talk: asset_loading::GameTexture,
    pub pa_no_mouth: asset_loading::GameTexture,
    pub pa_mouth: asset_loading::GameTexture,
    pub pa_lookleft: asset_loading::GameTexture,

    pub dust: Handle<Mesh>,
}
