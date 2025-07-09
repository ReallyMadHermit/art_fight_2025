use bevy::prelude::*;
use crate::dino_run::mechanics::{PlayerHurt, PlayerScores, PlayerJumps};
use crate::common::AudioSystem;

const SOUND_TRACK: &[u8] = include_bytes!("sound_files/crystal.wav");
const HURT: &[u8] = include_bytes!("sound_files/hurt.wav");
const JUMP: &[u8] = include_bytes!("sound_files/jump.wav");
const SCORE: &[u8] = include_bytes!("sound_files/score.wav");

pub fn setup_audio(
    my_audio: Res<AudioSystem>
) {
    my_audio.play_music(SOUND_TRACK);
}

pub fn jump_audio(
    mut event_reader: EventReader<PlayerJumps>,
    audio_system: Res<AudioSystem>
) {
    for _ in event_reader.read(){
        audio_system.play_sound(JUMP);
    };
}

pub fn score_audio(
    mut event_reader: EventReader<PlayerScores>,
    audio_system: Res<AudioSystem>
) {
    for _ in event_reader.read(){
        audio_system.play_sound(SCORE);
    };
}

pub fn hurt_audio(
    mut event_reader: EventReader<PlayerHurt>,
    audio_system: Res<AudioSystem>
) {
    for _ in event_reader.read(){
        audio_system.play_sound(HURT);
    };
}