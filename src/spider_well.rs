use bevy::prelude::*;
pub mod mechanics;

use mechanics::{
    debug_scene_setup, spawn_player, insert_resources, player_controls, move_player
};

pub struct SpiderWellPlugin;
impl Plugin for SpiderWellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_scene_setup);
        app.add_systems(Startup, insert_resources);
        app.add_systems(Startup, spawn_player);
        app.add_systems(PreUpdate, player_controls);
        app.add_systems(Update, move_player);
    }
}