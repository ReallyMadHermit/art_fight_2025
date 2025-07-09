use bevy::prelude::*;
use crate::dino_run_mechanics::{PlayerHurt, PlayerScores, PlayerJumps};
use crate::audio_system::MyAudio;

const SOUND_TRACK: &[u8] = include_bytes!("dino_run_crystal.wav");
const HURT: &[u8] = include_bytes!("hurt.wav");
const JUMP: &[u8] = include_bytes!("jump.wav");
const SCORE: &[u8] = include_bytes!("score.wav");


#[derive(Resource)]
pub struct DinoSounds{
    jump: Handle<AudioSource>,
    hurt: Handle<AudioSource>,
    score: Handle<AudioSource>
}

pub fn setup_audio(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    my_audio: Res<MyAudio>
) {
    // commands.spawn((
    //     AudioPlayer::new(asset_server.load("dino_sounds/dino_run_crystal.wav")),
    //     PlaybackSettings::LOOP
    // ));
    my_audio.play_music(SOUND_TRACK);
    commands.insert_resource(
        DinoSounds {
            jump: asset_server.load("dino_sounds/jump.wav"),
            hurt: asset_server.load("dino_sounds/hurt.wav"),
            score: asset_server.load("dino_sounds/score.wav")
        }
    );
}

pub fn jump_audio(
    dino_sounds: Res<DinoSounds>,
    mut event_reader: EventReader<PlayerJumps>,
    mut commands: Commands
) {
    for _ in event_reader.read(){
        commands.spawn((
            AudioPlayer::new(dino_sounds.jump.clone()),
            PlaybackSettings::DESPAWN
        ));
    };
}

pub fn score_audio(
    dino_sounds: Res<DinoSounds>,
    mut event_reader: EventReader<PlayerScores>,
    mut commands: Commands
) {
    for _ in event_reader.read(){
        commands.spawn((
            AudioPlayer::new(dino_sounds.score.clone()),
            PlaybackSettings::DESPAWN
        ));
    };
}

pub fn hurt_audio(
    dino_sounds: Res<DinoSounds>,
    mut event_reader: EventReader<PlayerHurt>,
    mut commands: Commands
) {
    for _ in event_reader.read(){
        commands.spawn((
            AudioPlayer::new(dino_sounds.hurt.clone()),
            PlaybackSettings::DESPAWN
        ));
    };
}