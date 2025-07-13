use bevy::prelude::*;
pub mod mechanics;

use mechanics::{
    debug_scene_setup, spawn_player, insert_simple_resources, player_controls, move_player,
    insert_webbing_assets, web_spawner, web_updater
};

pub struct SpiderWellPlugin;
impl Plugin for SpiderWellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, debug_scene_setup);
        app.add_systems(Startup, insert_simple_resources);
        app.add_systems(Startup, insert_webbing_assets);
        app.add_systems(Startup, spawn_player.after(insert_webbing_assets));
        app.add_systems(PreUpdate, player_controls);
        app.add_systems(Update, move_player);
        app.add_systems(PreUpdate, web_spawner);
        app.add_systems(Update, web_updater);
    }
}