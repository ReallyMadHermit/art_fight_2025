use bevy::prelude::*;
use rodio::OutputStream;

mod common;
mod audio_system;
use audio_system::MyAudio;

mod dino_run;
use dino_run::mechanics::DinoRunPlugin;

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