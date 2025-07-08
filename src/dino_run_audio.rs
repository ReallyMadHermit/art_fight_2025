use bevy::prelude::*;
use crate::dino_run_mechanics::{PlayerHurt, PlayerScores, PlayerJumps};

#[derive(Component)]
struct HurtAudioMarker;

#[derive(Component)]
struct ScoreAudioMarker;

#[derive(Component)]
struct JumpAudioMarker;

#[derive(Component)]
struct MusicAudioMarker;

#[derive(Resource)]
pub struct DinoSounds{
    jump: Handle<AudioSource>,
    hurt: Handle<AudioSource>,
    score: Handle<AudioSource>
}

pub fn setup_audio(
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("dino_sounds/dino_run_crystal.wav")),
        PlaybackSettings::LOOP
    ));
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