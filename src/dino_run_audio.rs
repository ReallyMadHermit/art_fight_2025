use bevy::prelude::*;
use crate::dino_run_mechanics::{PlayerHurt, PlayerScores, PlayerJumps};
use crate::audio_system::MyAudio;

const SOUND_TRACK: &[u8] = include_bytes!("dino_run_crystal.wav");
const HURT: &[u8] = include_bytes!("hurt.wav");
const JUMP: &[u8] = include_bytes!("jump.wav");
const SCORE: &[u8] = include_bytes!("score.wav");

pub fn setup_audio(
    my_audio: Res<MyAudio>
) {
    my_audio.play_music(SOUND_TRACK);
}

pub fn jump_audio(
    mut event_reader: EventReader<PlayerJumps>,
    my_audio: Res<MyAudio>
) {
    for _ in event_reader.read(){
        my_audio.play_sound(JUMP);
    };
}

pub fn score_audio(
    mut event_reader: EventReader<PlayerScores>,
    my_audio: Res<MyAudio>
) {
    for _ in event_reader.read(){
        my_audio.play_sound(SCORE);
    };
}

pub fn hurt_audio(
    mut event_reader: EventReader<PlayerHurt>,
    my_audio: Res<MyAudio>
) {
    for _ in event_reader.read(){
        my_audio.play_sound(HURT);
    };
}