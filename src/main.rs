use bevy::prelude::*;

mod common;

mod dino_run_mechanics;

use dino_run_mechanics::DinoRunPlugin;

// MAIN
fn main() {
    println!("hello world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DinoRunPlugin)
        .run();
}