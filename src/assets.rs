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

#[derive(Default)]
pub struct GameAssets {
    pub font: Handle<Font>,
    pub matador: Handle<Gltf>,
    pub matador_run: Handle<AnimationClip>,
    pub matador_idle: Handle<AnimationClip>,
    pub plate: Handle<Gltf>,
    pub level_one: Handle<Gltf>,

    pub blip: Handle<AudioSource>,

    pub bevy_icon: asset_loading::GameTexture,
}
