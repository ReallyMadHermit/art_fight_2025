use bevy::prelude::*;

mod common;

mod dino_run;

use dino_run::DinoRunPlugin;

// MAIN
fn main() {
    println!("hello world!");
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DinoRunPlugin)
        .run();
}