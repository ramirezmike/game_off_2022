use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioPlugin, AudioSource, AudioControl};
use std::marker::PhantomData;

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SoundChannel>()
            .add_audio_channel::<TalkChannel>()
            .add_plugin(AudioPlugin);
    }
}

#[derive(Resource)]
pub struct MusicChannel;
#[derive(Resource)]
pub struct SoundChannel;
#[derive(Resource)]
pub struct TalkChannel;

#[derive(SystemParam)]
pub struct GameAudio<'w, 's> {
    music_channel: Res<'w, AudioChannel<MusicChannel>>,
    sound_channel: Res<'w, AudioChannel<SoundChannel>>,
    talk_channel: Res<'w, AudioChannel<TalkChannel>>,

    #[system_param(ignore)]
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's> GameAudio<'w, 's> {
    pub fn set_volume(&mut self) {
        self.sound_channel.set_volume(0.2);
        self.talk_channel.set_volume(0.2);
        self.music_channel.set_volume(0.5);
    }
    pub fn play_bgm(&mut self, handle: &Handle<AudioSource>) {
        self.music_channel.stop();
        self.music_channel.set_volume(0.7);
        self.music_channel.play(handle.clone()).looped();

    }

    pub fn play_bgm_once(&mut self, handle: &Handle<AudioSource>) {
        self.music_channel.stop();
        self.music_channel.play(handle.clone());
    }

    pub fn stop_bgm(&mut self) {
        self.music_channel.stop();
    }

    pub fn play_sfx_repeat(&mut self, handle: &Handle<AudioSource>) {
        self.sound_channel.set_volume(0.4);
        self.sound_channel.play(handle.clone()).looped();
    }

    pub fn stop_sfx(&mut self) {
        self.sound_channel.stop();
    }

    pub fn play_sfx(&mut self, handle: &Handle<AudioSource>) {
        self.sound_channel.set_volume(0.2);
        self.sound_channel.play(handle.clone());
    }
    pub fn play_talk(&mut self, handle: &Handle<AudioSource>) {
        self.talk_channel.play(handle.clone());
    }
}
