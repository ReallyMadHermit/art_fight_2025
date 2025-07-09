use bevy::prelude::*;
use rodio::OutputStream;

mod common;
mod audio_system;
use audio_system::MyAudio;

mod dino_run_characters;
mod dino_run_mechanics;
mod dino_run_environment;
mod dino_run_audio;
use dino_run_mechanics::DinoRunPlugin;


// MAIN
fn main() {
    println!("hello world!");
    let (_stream, handle) = OutputStream::try_default().unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(MyAudio::new(handle))
        .add_plugins(DinoRunPlugin)
        .run();
}