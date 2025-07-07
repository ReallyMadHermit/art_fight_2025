use bevy::prelude::*;

mod common;

mod dino_run_characters;
mod dino_run_mechanics;
mod dino_run_environment;
use dino_run_mechanics::DinoRunPlugin;


// MAIN
fn main() {
    println!("hello world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DinoRunPlugin)
        .run();
}