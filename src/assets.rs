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
    pub plate: Handle<Gltf>,
    pub broken_plate: Handle<Gltf>,
    pub broken_mug: Handle<Gltf>,
    pub level_one: Handle<Gltf>,
    pub intro_level: Handle<Gltf>,

    pub blip: Handle<AudioSource>,

    pub bevy_icon: asset_loading::GameTexture,
    pub cloud_texture: asset_loading::GameTexture,
    pub wrench_texture: asset_loading::GameTexture,
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
