use bevy::prelude::*;

#[derive(Component)]
struct HurtAudioMarker;

#[derive(Component)]
struct ScoreAudioMarker;

#[derive(Component)]
struct JumpAudioMarker;

#[derive(Component)]
struct MusicAudioMarker;

pub fn setup_audio(
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("dino_sounds/dino_run_crystal.wav")),
        PlaybackSettings::LOOP
    ));
}