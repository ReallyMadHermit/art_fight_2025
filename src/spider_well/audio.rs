use bevy::prelude::*;
use crate::common::AudioSystem;
use crate::spider_well::mechanics::{CollisionEvent, CheckPointEvent};

const MUSIC: &[u8] = include_bytes!("sound_files/groovy_descent.wav");
const OOPS: &[u8] = include_bytes!("sound_files/spiderwell_oops.wav");
const CHECKPOINT: &[u8] = include_bytes!("sound_files/spiderwell_checkpoint.wav");

pub fn play_music(
    my_audio: Res<AudioSystem>
) {
    my_audio.play_music(MUSIC);
}

pub fn oops_audio_player(
    mut collision_event: EventReader<CollisionEvent>,
    audio_system: Res<AudioSystem>
) {
    for _ in collision_event.read() {
        audio_system.play_sound(OOPS);
    };
}

pub fn checkpoint_audio_player(
    mut checkpoint_event: EventReader<CheckPointEvent>,
    audio_system: Res<AudioSystem>
) {
    for _ in checkpoint_event.read() {
        audio_system.play_sound(CHECKPOINT);
    };
}